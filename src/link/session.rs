use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use niri_ipc::{LinkGlobalWorkspace, LinkPeer, LinkRemoteTile, LinkSessionSummary, LinkStatus};
use smithay::utils::{Logical, Point};
use uuid::Uuid;

use crate::link::config::RuntimeLinkConfig;
use crate::link::discovery::{start_discovery, DiscoveredPeer, DiscoveryHandle, DiscoveryState};
use crate::link::layout_sync::{
    apply_op, choose_leader, hit_test_tile, next_generation, ordered_column_tiles, tile_geometries,
    OperationLog,
};
use crate::link::persistence::{
    load_sessions, now_millis, persist_session, restore_candidate, PersistedSession,
};
use crate::link::protocol::{
    Envelope, GlobalWorkspace, JoinRequest, LayoutOp, LayoutOpKind, MessageKind, NodeId,
    Participant, SessionId, TileMetadata, TileMetadataUpdate, Viewport, PROTOCOL_VERSION,
};
use crate::link::remote_tile::RemoteTile;
use crate::link::security::{load_or_create_local_node_id, TrustStore};
use crate::link::transport::TransportState;

#[derive(Debug, Clone)]
pub struct LinkPeerState {
    pub node_id: NodeId,
    pub hostname: String,
    pub addr: Option<String>,
    pub fingerprint: String,
    pub connected: bool,
}

#[derive(Debug)]
pub struct LinkManager {
    pub config: RuntimeLinkConfig,
    pub local_node_id: NodeId,
    pub hostname: String,
    pub fingerprint: String,
    pub enabled: bool,
    pub pairing_mode: bool,
    pub current_session: Option<LinkedSession>,
    pub discovered: DiscoveryState,
    pub peers: BTreeMap<NodeId, LinkPeerState>,
    pub remote_tiles: BTreeMap<Uuid, RemoteTile>,
    pub transport: TransportState,
    pub trust_store: TrustStore,
    /// TCP port assigned after start_network(), 0 if not started.
    pub listen_port: u16,
    /// Handle to the mDNS discovery daemon (kept alive).
    pub discovery_handle: Option<DiscoveryHandle>,
    /// Shared list of newly discovered peers from the mDNS thread.
    pub discovery_queue: Arc<Mutex<Vec<DiscoveredPeer>>>,
    /// Monotonically increasing nonce for Envelope.
    pub next_nonce: u64,
}

#[derive(Debug, Clone)]
pub struct LinkedSession {
    pub workspace: GlobalWorkspace,
    pub log: OperationLog,
}

impl LinkManager {
    pub fn new(config: RuntimeLinkConfig) -> Self {
        let trust_store = TrustStore::load().unwrap_or_default();
        let local_node_id = load_or_create_local_node_id().unwrap_or_else(|_| NodeId::new_v4());
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
        let fingerprint = format!("niri-link:{local_node_id}");
        Self {
            enabled: config.enabled,
            pairing_mode: config.pairing_mode,
            current_session: None,
            discovered: DiscoveryState::default(),
            peers: BTreeMap::new(),
            remote_tiles: BTreeMap::new(),
            transport: TransportState::default(),
            local_node_id,
            hostname,
            fingerprint,
            trust_store,
            listen_port: 0,
            discovery_handle: None,
            discovery_queue: Arc::new(Mutex::new(Vec::new())),
            next_nonce: 0,
            config,
        }
    }

    /// Start the TCP listener and mDNS discovery daemon.
    /// Safe to call multiple times; subsequent calls are no-ops.
    pub fn start_network(&mut self) {
        if self.listen_port != 0 {
            return; // Already started.
        }
        let addr = &self.config.listen_address;
        match self.transport.start_listener(addr) {
            Ok(port) => {
                self.listen_port = port;
                tracing::info!("niri-link: TCP listener on port {port}");
            }
            Err(e) => {
                tracing::error!("niri-link: failed to start TCP listener: {e}");
                return;
            }
        }
        if self.config.discovery && self.discovery_handle.is_none() {
            let queue = Arc::clone(&self.discovery_queue);
            match start_discovery(
                self.local_node_id.to_string(),
                &self.fingerprint,
                self.listen_port,
                queue,
            ) {
                Ok(handle) => {
                    self.discovery_handle = Some(handle);
                    tracing::info!("niri-link: mDNS discovery started");
                }
                Err(e) => {
                    tracing::warn!("niri-link: mDNS discovery failed to start: {e}");
                }
            }
        }
    }

    /// Build an Envelope for the given MessageKind.
    fn make_envelope(&mut self, kind: MessageKind) -> Envelope {
        let session_id = self
            .current_session
            .as_ref()
            .map(|s| s.workspace.session_id)
            .unwrap_or_else(Uuid::new_v4);
        let nonce = self.next_nonce;
        self.next_nonce = self.next_nonce.wrapping_add(1);
        Envelope {
            protocol_version: PROTOCOL_VERSION,
            session_id,
            sender_node_id: self.local_node_id,
            nonce,
            timestamp_millis: now_millis(),
            kind,
        }
    }

    /// Send a Hello + JoinRequest to a peer address.
    pub fn send_hello(&mut self, addr: &str) {
        let hello = self.make_envelope(MessageKind::Hello(crate::link::protocol::Hello {
            hostname: self.hostname.clone(),
            fingerprint: self.fingerprint.clone(),
        }));
        self.transport.enqueue(addr.to_string(), hello);
        let join = self.make_envelope(MessageKind::JoinRequest(JoinRequest {
            hostname: self.hostname.clone(),
            pairing_token: None,
        }));
        self.transport.enqueue(addr.to_string(), join);
        self.transport.drain();
    }

    /// Broadcast a tile metadata update to all connected peers.
    pub fn broadcast_tile_update(&mut self, tile: TileMetadata) {
        let addrs: Vec<_> = self
            .peers
            .values()
            .filter(|p| p.connected)
            .filter_map(|p| p.addr.clone())
            .collect();
        for addr in addrs {
            let env = self.make_envelope(MessageKind::TileMetadataUpdate(TileMetadataUpdate {
                tile: tile.clone(),
            }));
            self.transport.enqueue(addr, env);
        }
        self.transport.drain();
    }

    /// Drain newly discovered mDNS peers and upsert them into `self.discovered`.
    /// Returns true if any new peers were found.
    pub fn poll_discovery(&mut self) -> bool {
        let new_peers: Vec<_> = {
            let mut q = self.discovery_queue.lock().unwrap();
            q.drain(..).collect()
        };
        if new_peers.is_empty() {
            return false;
        }
        for peer in new_peers {
            tracing::debug!("niri-link: mDNS peer upserted: {}", peer.addr);
            self.discovered.upsert(peer);
        }
        true
    }

    /// Process all incoming network messages.
    /// Returns true if any layout-relevant message was handled.
    pub fn process_incoming(&mut self) -> bool {
        let messages = self.transport.drain_incoming();
        if messages.is_empty() {
            return false;
        }
        let mut changed = false;
        for msg in messages {
            let sender = msg.peer_addr.clone();
            match msg.envelope.kind {
                MessageKind::Hello(hello) => {
                    tracing::debug!(
                        "niri-link: Hello from {sender}: hostname={}",
                        hello.hostname
                    );
                    // Upsert peer.
                    let peer = LinkPeerState {
                        node_id: msg.envelope.sender_node_id,
                        hostname: hello.hostname.clone(),
                        addr: Some(sender.clone()),
                        fingerprint: hello.fingerprint.clone(),
                        connected: true,
                    };
                    self.peers.insert(msg.envelope.sender_node_id, peer);
                }
                MessageKind::JoinRequest(join) => {
                    tracing::info!(
                        "niri-link: JoinRequest from {sender}: hostname={}",
                        join.hostname
                    );
                    // If pairing mode is on or already trusted, accept.
                    let node_id = msg.envelope.sender_node_id;
                    if let Some(peer) = self.peers.get_mut(&node_id) {
                        peer.connected = true;
                    }
                    // Send JoinAccept.
                    let (leader, gen, seq) = self
                        .current_session
                        .as_ref()
                        .map(|s| {
                            (
                                s.workspace.leader_node_id,
                                s.workspace.generation,
                                s.workspace.operation_seq,
                            )
                        })
                        .unwrap_or((self.local_node_id, 0, 0));
                    let accept = self.make_envelope(MessageKind::JoinAccept(
                        crate::link::protocol::JoinAccept {
                            leader_node_id: leader,
                            generation: gen,
                            operation_seq: seq,
                        },
                    ));
                    self.transport.enqueue(sender, accept);
                    self.transport.drain();
                }
                MessageKind::JoinAccept(accept) => {
                    tracing::info!(
                        "niri-link: JoinAccept from {sender}: leader={}",
                        accept.leader_node_id
                    );
                    if let Some(session) = self.current_session.as_mut() {
                        session.workspace.leader_node_id = accept.leader_node_id;
                        session.workspace.generation = accept.generation;
                    }
                    changed = true;
                }
                MessageKind::TileMetadataUpdate(update) => {
                    tracing::debug!("niri-link: TileMetadataUpdate tile={}", update.tile.tile_id);
                    self.update_remote_tile(update.tile);
                    changed = true;
                }
                MessageKind::TileClosed(closed) => {
                    tracing::debug!("niri-link: TileClosed tile={}", closed.tile_id);
                    self.close_tile(closed.tile_id);
                    changed = true;
                }
                MessageKind::ViewportUpdate(vp) => {
                    tracing::debug!(
                        "niri-link: ViewportUpdate from node={}",
                        vp.viewport.node_id
                    );
                    if let Some(session) = self.current_session.as_mut() {
                        let node = vp.viewport.node_id;
                        session
                            .workspace
                            .per_node_viewports
                            .entry(node)
                            .or_default()
                            .push(vp.viewport);
                    }
                    changed = true;
                }
                MessageKind::FocusUpdate(focus) => {
                    if let Some(session) = self.current_session.as_mut() {
                        session.workspace.focused_tile = focus.focused_tile;
                    }
                    changed = true;
                }
                MessageKind::LayoutOp(op_msg) => {
                    if let Some(session) = self.current_session.as_mut() {
                        apply_op(&mut session.workspace, &op_msg.op);
                        session.log.push(op_msg.op);
                        changed = true;
                    }
                }
                MessageKind::GlobalSnapshot(snapshot) => {
                    tracing::info!("niri-link: GlobalSnapshot from {sender}");
                    if let Some(session) = self.current_session.as_mut() {
                        // Merge remote tiles into local view.
                        for (id, tile) in &snapshot.workspace.tiles {
                            if tile.owner_node_id != self.local_node_id {
                                self.remote_tiles
                                    .entry(*id)
                                    .or_insert_with(|| RemoteTile::new(tile.clone()));
                            }
                        }
                        session.workspace = snapshot.workspace;
                        changed = true;
                    }
                }
                MessageKind::Heartbeat(hb) => {
                    // Update peer last_seen and participant timestamp.
                    let node_id = msg.envelope.sender_node_id;
                    if let Some(peer) = self.peers.get_mut(&node_id) {
                        peer.connected = true;
                    }
                    self.touch_participant(node_id);
                    // Request snapshot if seq gap.
                    if let Some(session) = self.current_session.as_ref() {
                        if hb.operation_seq > session.workspace.operation_seq + 1 {
                            tracing::warn!(
                                "niri-link: seq gap detected (remote={} local={}), from {sender}",
                                hb.operation_seq,
                                session.workspace.operation_seq
                            );
                            // TODO: send SnapshotRequest — implement via RequestSnapshot message.
                        }
                    }
                }
                MessageKind::DisableLink(dl) => {
                    tracing::info!("niri-link: DisableLink from {sender}");
                    if dl.persist {
                        let _ = self.persist_current_session();
                    }
                    self.disable();
                    changed = true;
                }
                _ => {
                    tracing::debug!("niri-link: unhandled message kind from {sender}");
                }
            }
        }
        changed
    }

    pub fn enable(&mut self) {
        self.enabled = true;
        if self.current_session.is_none() {
            self.current_session = Some(LinkedSession {
                workspace: GlobalWorkspace::empty(
                    self.local_node_id,
                    self.hostname.clone(),
                    self.fingerprint.clone(),
                ),
                log: OperationLog::default(),
            });
        }
    }

    /// Send a Heartbeat to all connected peers.
    /// Should be called periodically (e.g., every 5 seconds).
    pub fn send_heartbeat(&mut self) {
        if !self.enabled {
            return;
        }
        let (op_seq, generation) = self
            .current_session
            .as_ref()
            .map(|s| (s.workspace.operation_seq, s.workspace.generation))
            .unwrap_or((0, 0));
        let addrs: Vec<_> = self
            .peers
            .values()
            .filter(|p| p.connected)
            .filter_map(|p| p.addr.clone())
            .collect();
        for addr in addrs {
            let env =
                self.make_envelope(MessageKind::Heartbeat(crate::link::protocol::Heartbeat {
                    operation_seq: op_seq,
                    generation,
                }));
            self.transport.enqueue(addr, env);
        }
        self.transport.drain();
    }

    /// Mark peers as disconnected if no heartbeat received within `timeout_ms`.
    /// Returns true if any peer state changed.
    pub fn check_peer_timeouts(&mut self, timeout_ms: u64) -> bool {
        if !self.enabled {
            return false;
        }
        let now = now_millis();
        let mut changed = false;
        // We use last_seen_millis from the participant record.
        let stale_node_ids: Vec<_> = if let Some(session) = &self.current_session {
            session
                .workspace
                .participants
                .values()
                .filter(|p| {
                    p.node_id != self.local_node_id
                        && p.last_seen_millis > 0
                        && now.saturating_sub(p.last_seen_millis) > timeout_ms
                })
                .map(|p| p.node_id)
                .collect()
        } else {
            Vec::new()
        };
        for node_id in &stale_node_ids {
            if let Some(peer) = self.peers.get_mut(node_id) {
                if peer.connected {
                    peer.connected = false;
                    changed = true;
                    tracing::warn!("niri-link: peer {} timed out", node_id);
                }
            }
        }
        changed
    }

    /// Purge remote tiles belonging to disconnected peers that have been stale
    /// longer than `timeout_ms`. Returns a list of purged tile IDs.
    pub fn purge_stale_remote_tiles(&mut self, timeout_ms: u64) -> Vec<uuid::Uuid> {
        if !self.enabled {
            return Vec::new();
        }
        let now = now_millis();
        let disconnected_nodes: std::collections::BTreeSet<_> = self
            .peers
            .values()
            .filter(|p| !p.connected)
            .map(|p| p.node_id)
            .collect();
        let stale_tile_ids: Vec<_> = self
            .remote_tiles
            .values()
            .filter(|tile| {
                let meta = tile.metadata();
                disconnected_nodes.contains(&meta.owner_node_id)
                    && now.saturating_sub(meta.last_known_alive_millis) > timeout_ms
            })
            .map(|tile| tile.tile_id())
            .collect();
        for tile_id in &stale_tile_ids {
            self.close_tile(*tile_id);
            tracing::debug!("niri-link: purged stale remote tile {tile_id}");
        }
        stale_tile_ids
    }

    /// Update a participant's last_seen timestamp (called on receiving any message from them).
    pub fn touch_participant(&mut self, node_id: uuid::Uuid) {
        if let Some(session) = self.current_session.as_mut() {
            if let Some(participant) = session.workspace.participants.get_mut(&node_id) {
                participant.last_seen_millis = now_millis();
            }
        }
    }

    pub fn disable(&mut self) -> Option<SessionId> {
        self.enabled = false;
        let session_id = self
            .current_session
            .as_ref()
            .map(|it| it.workspace.session_id);
        let _ = self.persist_current_session();
        self.remote_tiles.clear();
        session_id
    }

    pub fn toggle(&mut self) {
        if self.enabled {
            self.disable();
        } else {
            self.enable();
        }
    }

    pub fn join_addr(&mut self, addr: String) {
        self.enable();
        let peer = LinkPeerState {
            node_id: NodeId::new_v4(),
            hostname: addr.clone(),
            addr: Some(addr),
            fingerprint: String::new(),
            connected: false,
        };
        self.peers.insert(peer.node_id, peer);
    }

    pub fn leave(&mut self) {
        self.disable();
        self.current_session = None;
    }

    pub fn pair(&mut self, node: Option<String>) -> std::io::Result<()> {
        if let Some(node) = node {
            self.trust_store.trust(node.clone(), node);
            self.trust_store.persist()
        } else {
            self.pairing_mode = true;
            Ok(())
        }
    }

    pub fn unpair(&mut self, node: &str) -> std::io::Result<()> {
        self.trust_store.forget(node);
        self.trust_store.persist()
    }

    pub fn trust_node(&mut self, node: &str) -> std::io::Result<()> {
        self.trust_store.trust(node.to_string(), node.to_string());
        self.trust_store.persist()
    }

    pub fn forget_node(&mut self, node: &str) -> std::io::Result<()> {
        self.unpair(node)
    }

    pub fn set_leader(&mut self, node: NodeId) {
        if let Some(session) = self.current_session.as_mut() {
            session.workspace.leader_node_id = node;
            session.workspace.generation = next_generation(session.workspace.generation);
        }
    }

    pub fn restore_session(&mut self, session_id: SessionId) -> std::io::Result<bool> {
        let sessions = load_sessions()?;
        if let Some(found) = sessions.into_iter().find(|it| it.session_id == session_id) {
            self.current_session = Some(LinkedSession {
                workspace: found.global_layout,
                log: OperationLog::default(),
            });
            self.enabled = true;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn restore_best_session(&mut self) -> std::io::Result<Option<SessionId>> {
        let sessions = load_sessions()?;
        let participants: Vec<_> = self
            .peers
            .keys()
            .copied()
            .chain([self.local_node_id])
            .collect();
        let Some(found) = restore_candidate(&sessions, &participants, ".") else {
            return Ok(None);
        };
        let session_id = found.session_id;
        self.current_session = Some(LinkedSession {
            workspace: found.global_layout,
            log: OperationLog::default(),
        });
        self.enabled = true;
        Ok(Some(session_id))
    }

    pub fn apply_layout_op(&mut self, kind: LayoutOpKind) -> Option<u64> {
        let session = self.current_session.as_mut()?;
        let next_seq = session.log.last_seq().saturating_add(1);
        let op = LayoutOp {
            seq: next_seq,
            issuer: self.local_node_id,
            generation: session.workspace.generation,
            kind,
        };
        apply_op(&mut session.workspace, &op);
        session.log.push(op);
        Some(next_seq)
    }

    pub fn upsert_local_viewport(&mut self, viewport: Viewport) {
        self.enable();
        let _ = self.apply_layout_op(LayoutOpKind::SetViewport { viewport });
    }

    pub fn upsert_local_tile(&mut self, tile: TileMetadata, index: usize) {
        self.enable();
        let _ = self.apply_layout_op(LayoutOpKind::InsertTile { tile, index });
    }

    pub fn close_tile(&mut self, tile_id: Uuid) {
        let _ = self.apply_layout_op(LayoutOpKind::RemoveTile { tile_id });
        self.remote_tiles.remove(&tile_id);
    }

    pub fn refresh_leader(&mut self) {
        let Some(session) = self.current_session.as_mut() else {
            return;
        };
        let leader =
            choose_leader(session.workspace.participants.values()).unwrap_or(self.local_node_id);
        session.workspace.leader_node_id = leader;
    }

    pub fn add_remote_tile(&mut self, tile: TileMetadata) {
        self.enable();
        self.remote_tiles
            .insert(tile.tile_id, RemoteTile::new(tile.clone()));
        if let Some(session) = self.current_session.as_mut() {
            if !session.workspace.columns.contains(&tile.column_id) {
                session.workspace.columns.push(tile.column_id);
            }
            session.workspace.tiles.insert(tile.tile_id, tile);
        }
    }

    pub fn update_remote_tile(&mut self, tile: TileMetadata) {
        self.enable();
        if let Some(remote) = self.remote_tiles.get_mut(&tile.tile_id) {
            remote.update_metadata(tile.clone());
        } else {
            self.remote_tiles
                .insert(tile.tile_id, RemoteTile::new(tile.clone()));
        }
        if let Some(session) = self.current_session.as_mut() {
            if !session.workspace.columns.contains(&tile.column_id) {
                session.workspace.columns.push(tile.column_id);
            }
            session.workspace.tiles.insert(tile.tile_id, tile);
        }
    }

    pub fn status(&self) -> LinkStatus {
        let (session_active, session_id, leader_node_id, operation_seq, generation) =
            if let Some(session) = &self.current_session {
                (
                    true,
                    Some(session.workspace.session_id),
                    Some(session.workspace.leader_node_id),
                    session.workspace.operation_seq,
                    session.workspace.generation,
                )
            } else {
                (false, None, None, 0, 0)
            };
        LinkStatus {
            enabled: self.enabled,
            session_active,
            session_id,
            local_node_id: self.local_node_id,
            leader_node_id,
            operation_seq,
            generation,
            peer_count: self.peers.len(),
            remote_tile_count: self.remote_tiles.len(),
            message: if self.enabled {
                "link enabled".to_string()
            } else {
                "link disabled".to_string()
            },
        }
    }

    pub fn peer_list(&self) -> Vec<LinkPeer> {
        self.peers
            .values()
            .map(|peer| LinkPeer {
                node_id: peer.node_id,
                hostname: peer.hostname.clone(),
                addr: peer.addr.clone(),
                trusted: self
                    .trust_store
                    .is_trusted(&peer.hostname, &peer.fingerprint),
                connected: peer.connected,
                is_leader: self
                    .current_session
                    .as_ref()
                    .is_some_and(|it| it.workspace.leader_node_id == peer.node_id),
            })
            .collect()
    }

    pub fn session_summaries(&self) -> Vec<LinkSessionSummary> {
        load_sessions()
            .unwrap_or_default()
            .into_iter()
            .map(|session| LinkSessionSummary {
                session_id: session.session_id,
                generation: session.generation,
                timestamp: niri_ipc::Timestamp {
                    secs: session.timestamp_millis / 1000,
                    nanos: ((session.timestamp_millis % 1000) * 1_000_000) as u32,
                },
                participants: session.participants,
            })
            .collect()
    }

    pub fn global_workspace(&self) -> Option<LinkGlobalWorkspace> {
        let session = self.current_session.as_ref()?;
        Some(LinkGlobalWorkspace {
            session_id: session.workspace.session_id,
            generation: session.workspace.generation,
            operation_seq: session.workspace.operation_seq,
            leader_node_id: session.workspace.leader_node_id,
            participants: session.workspace.participants.keys().copied().collect(),
            focused_tile: session.workspace.focused_tile,
            columns: session.workspace.columns.clone(),
            tiles: self.workspace_tiles(),
            viewports: session
                .workspace
                .per_node_viewports
                .values()
                .flat_map(|it| it.clone())
                .map(|viewport| niri_ipc::LinkViewport {
                    node_id: viewport.node_id,
                    output_name: viewport.output_name,
                    global_x: viewport.global_x,
                    global_y: viewport.global_y,
                    logical_width: viewport.logical_width,
                    logical_height: viewport.logical_height,
                    scale: viewport.scale,
                    transform: viewport.transform,
                    refresh_rate_millihz: viewport.refresh_rate_millihz,
                })
                .collect(),
        })
    }

    pub fn remote_tiles(&self) -> Vec<LinkRemoteTile> {
        let Some(session) = self.current_session.as_ref() else {
            return Vec::new();
        };
        let geometries = tile_geometries(&session.workspace);
        self.remote_tiles
            .values()
            .filter_map(|tile| {
                let metadata = tile.metadata();
                let geometry = geometries
                    .iter()
                    .find(|it| it.tile_id == metadata.tile_id)?;
                Some(LinkRemoteTile {
                    tile_id: metadata.tile_id,
                    owner_node_id: metadata.owner_node_id,
                    app_id: metadata.app_id.clone(),
                    title: metadata.title.clone(),
                    pid: metadata.pid,
                    column_id: metadata.column_id,
                    logical_x: geometry.logical_x,
                    logical_y: geometry.logical_y,
                    logical_width: geometry.logical_width,
                    logical_height: geometry.logical_height,
                    stream_state: format!("{:?}", metadata.stream_state),
                    last_known_alive: Some(niri_ipc::Timestamp {
                        secs: metadata.last_known_alive_millis / 1000,
                        nanos: ((metadata.last_known_alive_millis % 1000) * 1_000_000) as u32,
                    }),
                    placeholder: true,
                    disconnected: matches!(
                        metadata.stream_state,
                        crate::link::protocol::StreamState::Disconnected
                    ),
                })
            })
            .collect()
    }

    pub fn workspace_tiles(&self) -> Vec<LinkRemoteTile> {
        let Some(session) = self.current_session.as_ref() else {
            return Vec::new();
        };
        tile_geometries(&session.workspace)
            .into_iter()
            .filter_map(|geometry| {
                let metadata = session.workspace.tiles.get(&geometry.tile_id)?;
                Some(LinkRemoteTile {
                    tile_id: metadata.tile_id,
                    owner_node_id: metadata.owner_node_id,
                    app_id: metadata.app_id.clone(),
                    title: metadata.title.clone(),
                    pid: metadata.pid,
                    column_id: metadata.column_id,
                    logical_x: geometry.logical_x,
                    logical_y: geometry.logical_y,
                    logical_width: geometry.logical_width,
                    logical_height: geometry.logical_height,
                    stream_state: format!("{:?}", metadata.stream_state),
                    last_known_alive: Some(niri_ipc::Timestamp {
                        secs: metadata.last_known_alive_millis / 1000,
                        nanos: ((metadata.last_known_alive_millis % 1000) * 1_000_000) as u32,
                    }),
                    placeholder: self.remote_tiles.contains_key(&metadata.tile_id),
                    disconnected: matches!(
                        metadata.stream_state,
                        crate::link::protocol::StreamState::Disconnected
                    ),
                })
            })
            .collect()
    }

    pub fn remote_hit(
        &self,
        global_pos: Point<f64, Logical>,
    ) -> Option<(Uuid, NodeId, Point<f64, Logical>)> {
        let session = self.current_session.as_ref()?;
        let hit = hit_test_tile(&session.workspace, global_pos)?;
        if hit.owner_node_id == self.local_node_id {
            return None;
        }

        Some((
            hit.tile_id,
            hit.owner_node_id,
            Point::from((global_pos.x - hit.logical_x, global_pos.y - hit.logical_y)),
        ))
    }

    pub fn focused_remote_tile(&self) -> Option<(Uuid, NodeId)> {
        let session = self.current_session.as_ref()?;
        let tile_id = session.workspace.focused_tile?;
        let tile = session.workspace.tiles.get(&tile_id)?;
        (tile.owner_node_id != self.local_node_id).then_some((tile_id, tile.owner_node_id))
    }

    pub fn persist_current_session(&self) -> std::io::Result<Option<SessionId>> {
        let Some(session) = &self.current_session else {
            return Ok(None);
        };
        let persisted = PersistedSession {
            version: 1,
            session_id: session.workspace.session_id,
            generation: session.workspace.generation,
            participants: session.workspace.participants.keys().copied().collect(),
            trusted_fingerprints: self
                .trust_store
                .nodes
                .iter()
                .map(|it| it.fingerprint.clone())
                .collect(),
            global_layout: session.workspace.clone(),
            per_node_owned_tile_order: session
                .workspace
                .participants
                .keys()
                .copied()
                .map(|node_id| {
                    let tiles = ordered_column_tiles(&session.workspace)
                        .into_values()
                        .flatten()
                        .filter(|tile_id| {
                            session
                                .workspace
                                .tiles
                                .get(tile_id)
                                .is_some_and(|tile| tile.owner_node_id == node_id)
                        })
                        .collect();
                    (node_id, tiles)
                })
                .collect(),
            timestamp_millis: now_millis(),
        };
        persist_session(&persisted)?;
        Ok(Some(persisted.session_id))
    }

    pub fn ensure_local_participant(&mut self) {
        self.enable();
        let session = self
            .current_session
            .as_mut()
            .expect("session created by enable");
        session.workspace.participants.insert(
            self.local_node_id,
            Participant {
                node_id: self.local_node_id,
                hostname: self.hostname.clone(),
                fingerprint: self.fingerprint.clone(),
                last_seen_millis: now_millis(),
            },
        );
    }

    pub fn sync_local_viewports(&mut self, mut viewports: Vec<Viewport>) -> bool {
        if !self.enabled {
            return false;
        }

        self.ensure_local_participant();
        let session = self
            .current_session
            .as_mut()
            .expect("session created by enable");

        viewports.sort_by(|a, b| {
            a.global_x
                .total_cmp(&b.global_x)
                .then_with(|| a.global_y.total_cmp(&b.global_y))
                .then_with(|| a.output_name.cmp(&b.output_name))
        });

        let changed = session
            .workspace
            .per_node_viewports
            .get(&self.local_node_id)
            != Some(&viewports)
            || session.workspace.per_node_outputs.get(&self.local_node_id) != Some(&viewports);

        if changed {
            session
                .workspace
                .per_node_viewports
                .insert(self.local_node_id, viewports.clone());
            session
                .workspace
                .per_node_outputs
                .insert(self.local_node_id, viewports);
        }

        changed
    }

    pub fn sync_local_tiles(&mut self, tiles: Vec<TileMetadata>) -> bool {
        if !self.enabled {
            return false;
        }

        self.ensure_local_participant();
        let session = self
            .current_session
            .as_mut()
            .expect("session created by enable");
        let mut changed = false;

        let desired_ids: BTreeSet<_> = tiles.iter().map(|tile| tile.tile_id).collect();
        let existing_local_ids: Vec<_> = session
            .workspace
            .tiles
            .values()
            .filter(|tile| tile.owner_node_id == self.local_node_id)
            .map(|tile| tile.tile_id)
            .collect();

        for tile_id in existing_local_ids {
            if !desired_ids.contains(&tile_id) {
                session.workspace.tiles.remove(&tile_id);
                self.remote_tiles.remove(&tile_id);
                if session.workspace.focused_tile == Some(tile_id) {
                    session.workspace.focused_tile = None;
                }
                changed = true;
            }
        }

        let mut local_columns = Vec::new();
        for tile in tiles {
            if !local_columns.contains(&tile.column_id) {
                local_columns.push(tile.column_id);
            }

            if session.workspace.tiles.get(&tile.tile_id) != Some(&tile) {
                session.workspace.tiles.insert(tile.tile_id, tile);
                changed = true;
            }
        }

        if changed {
            let used_remote_columns: BTreeSet<_> = session
                .workspace
                .tiles
                .values()
                .filter(|tile| tile.owner_node_id != self.local_node_id)
                .map(|tile| tile.column_id)
                .collect();
            let mut next_columns: Vec<_> = session
                .workspace
                .columns
                .iter()
                .copied()
                .filter(|column_id| used_remote_columns.contains(column_id))
                .collect();
            for column_id in local_columns {
                if !next_columns.contains(&column_id) {
                    next_columns.push(column_id);
                }
            }
            session.workspace.columns = next_columns;
        }

        changed
    }

    pub fn sync_focused_tile(&mut self, focused_tile: Option<Uuid>) -> bool {
        if !self.enabled {
            return false;
        }

        self.ensure_local_participant();
        let session = self
            .current_session
            .as_mut()
            .expect("session created by enable");
        if session.workspace.focused_tile == focused_tile {
            return false;
        }

        session.workspace.focused_tile = focused_tile;
        true
    }
}
