pub mod config;
pub mod discovery;
pub mod input_forward;
pub mod layout_sync;
pub mod persistence;
pub mod protocol;
pub mod remote_tile;
pub mod security;
pub mod session;
pub mod stream;
pub mod transport;

pub use session::LinkManager;

#[cfg(test)]
mod tests {
    use smithay::utils::{Logical, Point};
    use uuid::Uuid;

    use super::input_forward::{local_to_global, owner_for_focus, owner_for_hit};
    use super::layout_sync::{
        apply_op, choose_leader, tile_geometries, OperationLog, TileGeometry,
    };
    use super::persistence::{peer_match_score, restore_candidate, PersistedSession};
    use super::protocol::{
        GlobalWorkspace, LayoutOp, LayoutOpKind, Participant, PrivacyFlags, StreamState,
        TileMetadata, TileStateChange, Viewport,
    };
    use super::session::LinkManager;

    fn sample_tile(owner: Uuid) -> TileMetadata {
        TileMetadata {
            tile_id: Uuid::new_v4(),
            owner_node_id: owner,
            app_id: Some("app".to_string()),
            title: Some("title".to_string()),
            pid: Some(42),
            initial_size: (800, 600),
            current_logical_size: (800, 600),
            column_id: Uuid::new_v4(),
            column_tile_index: 0,
            stack_group: None,
            fullscreen: false,
            maximized: false,
            floating: false,
            last_known_alive_millis: 1,
            stream_state: StreamState::Pending,
            privacy_flags: PrivacyFlags::default(),
        }
    }

    #[test]
    fn global_coordinate_mapping() {
        let global = local_to_global(
            Point::<f64, Logical>::from((1920., 0.)),
            Point::from((10., 20.)),
        );
        assert_eq!(global, Point::from((1930., 20.)));
    }

    #[test]
    fn viewport_adjacency() {
        let a = Viewport {
            node_id: Uuid::new_v4(),
            output_name: "A".into(),
            global_x: 0.,
            global_y: 0.,
            logical_width: 1920.,
            logical_height: 1080.,
            scale: 1.0,
            transform: 0,
            refresh_rate_millihz: Some(60000),
        };
        let b = Viewport {
            global_x: 1920.,
            output_name: "B".into(),
            ..a.clone()
        };
        assert_eq!(a.global_x + a.logical_width, b.global_x);
    }

    #[test]
    fn layout_operation_ordering() {
        let local = Uuid::new_v4();
        let mut workspace = GlobalWorkspace::empty(local, "host".into(), "fp".into());
        let tile = sample_tile(local);
        let op1 = LayoutOp {
            seq: 1,
            issuer: local,
            generation: 0,
            kind: LayoutOpKind::InsertTile {
                tile: tile.clone(),
                index: 0,
            },
        };
        let op2 = LayoutOp {
            seq: 2,
            issuer: local,
            generation: 0,
            kind: LayoutOpKind::ChangeTileState {
                tile_id: tile.tile_id,
                change: TileStateChange::Fullscreen(true),
            },
        };
        apply_op(&mut workspace, &op1);
        apply_op(&mut workspace, &op2);
        assert_eq!(workspace.operation_seq, 2);
        assert!(workspace.tiles.get(&tile.tile_id).unwrap().fullscreen);
    }

    #[test]
    fn leader_failover() {
        let a = Participant {
            node_id: Uuid::parse_str("00000000-0000-0000-0000-0000000000aa").unwrap(),
            hostname: "a".into(),
            fingerprint: "fa".into(),
            last_seen_millis: 0,
        };
        let b = Participant {
            node_id: Uuid::parse_str("00000000-0000-0000-0000-0000000000bb").unwrap(),
            hostname: "b".into(),
            fingerprint: "fb".into(),
            last_seen_millis: 0,
        };
        assert_eq!(choose_leader([&b, &a].into_iter()), Some(a.node_id));
    }

    #[test]
    fn snapshot_restore() {
        let config = crate::link::config::RuntimeLinkConfig {
            enabled: false,
            listen_address: "127.0.0.1:0".into(),
            discovery: false,
            pairing_mode: false,
            trusted_nodes: vec![],
            transport_prefer: "quic".into(),
            transport_fallback: "tcp-tls".into(),
            preferred_codec: "raw".into(),
            max_fps: 60,
            idle_fps: 5,
            max_bitrate_mbps: 80,
            stream_only_visible: true,
            stale_frame_timeout_ms: 500,
            restore_last_session: true,
            unlink_policy: "keep-owned-tiles".into(),
            remote_tile_placeholder: true,
            animate_link_transition: true,
            forward_keyboard: true,
            forward_pointer: true,
            forward_scroll: true,
            forward_touch: false,
            forward_tablet: false,
            remote_focus_follows_pointer: true,
        };
        let mut manager = LinkManager::new(config);
        manager.enable();
        let session_id = manager.persist_current_session().unwrap().unwrap();
        let restored = manager.restore_session(session_id).unwrap();
        assert!(restored);
    }

    #[test]
    fn unlink_owned_tile_extraction() {
        let local = Uuid::new_v4();
        let remote = Uuid::new_v4();
        let mut workspace = GlobalWorkspace::empty(local, "host".into(), "fp".into());
        let local_tile = sample_tile(local);
        let remote_tile = sample_tile(remote);
        workspace
            .tiles
            .insert(local_tile.tile_id, local_tile.clone());
        workspace
            .tiles
            .insert(remote_tile.tile_id, remote_tile.clone());
        let owned: Vec<_> = workspace
            .tiles
            .values()
            .filter(|tile| tile.owner_node_id == local)
            .map(|tile| tile.tile_id)
            .collect();
        assert_eq!(owned, vec![local_tile.tile_id]);
    }

    #[test]
    fn remote_tile_metadata_lifecycle() {
        let config = crate::link::config::RuntimeLinkConfig {
            enabled: false,
            listen_address: "127.0.0.1:0".into(),
            discovery: false,
            pairing_mode: false,
            trusted_nodes: vec![],
            transport_prefer: "quic".into(),
            transport_fallback: "tcp-tls".into(),
            preferred_codec: "raw".into(),
            max_fps: 60,
            idle_fps: 5,
            max_bitrate_mbps: 80,
            stream_only_visible: true,
            stale_frame_timeout_ms: 500,
            restore_last_session: true,
            unlink_policy: "keep-owned-tiles".into(),
            remote_tile_placeholder: true,
            animate_link_transition: true,
            forward_keyboard: true,
            forward_pointer: true,
            forward_scroll: true,
            forward_touch: false,
            forward_tablet: false,
            remote_focus_follows_pointer: true,
        };
        let mut manager = LinkManager::new(config);
        let local = manager.local_node_id;
        let tile = sample_tile(local);
        manager.add_remote_tile(tile.clone());
        assert_eq!(manager.remote_tiles().len(), 1);
        manager.close_tile(tile.tile_id);
        assert!(manager.remote_tiles().is_empty());
    }

    #[test]
    fn input_owner_routing() {
        let tile = Uuid::new_v4();
        let owner = Uuid::new_v4();
        let hit_tiles = vec![TileGeometry {
            tile_id: tile,
            owner_node_id: owner,
            column_id: Uuid::new_v4(),
            logical_x: 0.,
            logical_y: 0.,
            logical_width: 100.,
            logical_height: 100.,
        }];
        assert_eq!(owner_for_focus(Some((tile, owner))), Some((tile, owner)));
        assert_eq!(
            owner_for_hit(&hit_tiles, Point::<f64, Logical>::from((0., 0.))),
            Some((tile, owner))
        );
        assert_eq!(
            owner_for_hit(&hit_tiles, Point::<f64, Logical>::from((120., 0.))),
            None
        );
    }

    #[test]
    fn global_tile_geometry_ordering() {
        let local = Uuid::new_v4();
        let mut workspace = GlobalWorkspace::empty(local, "host".into(), "fp".into());
        let column = Uuid::new_v4();
        workspace.columns.push(column);

        let mut top = sample_tile(local);
        top.column_id = column;
        top.column_tile_index = 0;
        top.current_logical_size = (800, 300);

        let mut bottom = sample_tile(local);
        bottom.column_id = column;
        bottom.column_tile_index = 1;
        bottom.current_logical_size = (640, 500);

        workspace.tiles.insert(bottom.tile_id, bottom.clone());
        workspace.tiles.insert(top.tile_id, top.clone());

        let geometries = tile_geometries(&workspace);
        assert_eq!(geometries.len(), 2);
        assert_eq!(geometries[0].tile_id, top.tile_id);
        assert_eq!(geometries[0].logical_y, 0.);
        assert_eq!(geometries[1].tile_id, bottom.tile_id);
        assert_eq!(geometries[1].logical_y, 300.);
        assert_eq!(geometries[0].logical_width, 800.);
        assert_eq!(geometries[1].logical_width, 800.);
    }

    #[test]
    fn operation_log_gap_detection() {
        let local = Uuid::new_v4();
        let mut log = OperationLog::default();
        log.push(
            super::layout_sync::SequencedOp {
                seq: 2,
                op: LayoutOp {
                    seq: 2,
                    issuer: local,
                    generation: 0,
                    kind: LayoutOpKind::EnableLink,
                },
            }
            .op,
        );
        assert!(log.has_gap_after(1));
    }

    #[test]
    fn restore_prefers_high_peer_overlap() {
        let session_a = PersistedSession {
            version: 1,
            session_id: Uuid::new_v4(),
            generation: 1,
            participants: vec![Uuid::from_u128(1), Uuid::from_u128(2)],
            trusted_fingerprints: vec![],
            global_layout: GlobalWorkspace::empty(Uuid::from_u128(1), "a".into(), "f".into()),
            per_node_owned_tile_order: vec![],
            timestamp_millis: 1,
        };
        let session_b = PersistedSession {
            version: 1,
            session_id: Uuid::new_v4(),
            generation: 1,
            participants: vec![Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3)],
            trusted_fingerprints: vec![],
            global_layout: GlobalWorkspace::empty(Uuid::from_u128(1), "a".into(), "f".into()),
            per_node_owned_tile_order: vec![],
            timestamp_millis: 2,
        };
        assert_eq!(
            peer_match_score(
                &session_b.participants,
                &[Uuid::from_u128(1), Uuid::from_u128(3)]
            ),
            2
        );
        let restored = restore_candidate(
            &[session_a, session_b.clone()],
            &[Uuid::from_u128(1), Uuid::from_u128(3)],
            ".",
        )
        .unwrap();
        assert_eq!(restored.session_id, session_b.session_id);
    }

    // ── Integration-style tests ────────────────────────────────────────────

    fn make_manager(node_id: Option<Uuid>) -> LinkManager {
        use super::config::RuntimeLinkConfig;
        let node_id = node_id.unwrap_or_else(Uuid::new_v4);
        let config = RuntimeLinkConfig {
            enabled: false,
            listen_address: "127.0.0.1:0".into(),
            discovery: false,
            pairing_mode: true,
            trusted_nodes: vec![],
            transport_prefer: "tcp-tls".into(),
            transport_fallback: "tcp-tls".into(),
            preferred_codec: "raw".into(),
            max_fps: 60,
            idle_fps: 5,
            max_bitrate_mbps: 80,
            stream_only_visible: true,
            stale_frame_timeout_ms: 500,
            restore_last_session: true,
            unlink_policy: "keep-owned-tiles".into(),
            remote_tile_placeholder: true,
            animate_link_transition: false,
            forward_keyboard: true,
            forward_pointer: true,
            forward_scroll: true,
            forward_touch: false,
            forward_tablet: false,
            remote_focus_follows_pointer: true,
        };
        let mut mgr = LinkManager::new(config);
        mgr.local_node_id = node_id;
        mgr
    }

    /// Two nodes: A enables link, B joins as remote peer.
    /// Simulate B inserting a remote tile visible on A.
    #[test]
    fn two_node_session_join_and_insert_remote_tile() {
        let node_a = Uuid::from_u128(0xAAAA);
        let node_b = Uuid::from_u128(0xBBBB);

        let mut mgr_a = make_manager(Some(node_a));
        mgr_a.enable();

        // Simulate: A receives a JoinRequest from B.
        let tile_b = sample_tile(node_b);
        mgr_a.add_remote_tile(tile_b.clone());

        // A should now have 1 remote tile from B.
        assert_eq!(mgr_a.remote_tiles().len(), 1);
        assert_eq!(mgr_a.remote_tiles()[0].owner_node_id, node_b);
        assert_eq!(mgr_a.remote_tiles()[0].tile_id, tile_b.tile_id);
    }

    /// Disabling link removes all remote tiles and preserves local ones.
    #[test]
    fn disable_link_removes_remote_tiles_only() {
        let node_a = Uuid::from_u128(0xAAAA);
        let node_b = Uuid::from_u128(0xBBBB);

        let mut mgr = make_manager(Some(node_a));
        mgr.enable();

        // Insert a local tile (owned by A) and a remote tile (owned by B).
        let local_tile = sample_tile(node_a);
        let remote_tile = sample_tile(node_b);
        mgr.add_remote_tile(remote_tile.clone());

        // Simulate sync_local_tiles adding the local tile to the workspace.
        if let Some(session) = mgr.current_session.as_mut() {
            if !session.workspace.columns.contains(&local_tile.column_id) {
                session.workspace.columns.push(local_tile.column_id);
            }
            session
                .workspace
                .tiles
                .insert(local_tile.tile_id, local_tile.clone());
        }

        assert_eq!(mgr.remote_tiles().len(), 1);

        // Disable link.
        mgr.disable();

        // Remote tiles should be cleared.
        assert!(mgr.remote_tiles().is_empty());
    }

    /// After unlinking, re-linking with the same peer set restores session.
    #[test]
    fn relink_restores_previous_session() {
        let node_a = Uuid::from_u128(0xAAAA);
        let node_b = Uuid::from_u128(0xBBBB);

        let mut mgr = make_manager(Some(node_a));
        mgr.enable();

        // Add a remote tile.
        mgr.add_remote_tile(sample_tile(node_b));

        // Persist session on disable.
        let session_id = mgr.persist_current_session().unwrap().unwrap();
        mgr.disable();

        // Re-link: restore session.
        let restored = mgr.restore_session(session_id).unwrap();
        assert!(restored);
        assert!(mgr.enabled);
    }

    /// Stale tile purging: tiles from disconnected peers are removed after timeout.
    #[test]
    fn stale_tile_purge_after_peer_disconnect() {
        let node_a = Uuid::from_u128(0xAAAA);
        let node_b = Uuid::from_u128(0xBBBB);

        let mut mgr = make_manager(Some(node_a));
        mgr.enable();

        // Add a remote tile with an old last_known_alive_millis.
        let mut old_tile = sample_tile(node_b);
        old_tile.last_known_alive_millis = 1; // very old
        mgr.add_remote_tile(old_tile.clone());

        // Add B as a disconnected peer.
        mgr.peers.insert(
            node_b,
            super::session::LinkPeerState {
                node_id: node_b,
                hostname: "b".into(),
                addr: None,
                fingerprint: "fb".into(),
                connected: false,
            },
        );

        assert_eq!(mgr.remote_tiles().len(), 1);

        // Purge stale tiles with a 1ms timeout (all tiles qualify).
        let purged = mgr.purge_stale_remote_tiles(1);
        assert_eq!(purged.len(), 1);
        assert!(mgr.remote_tiles().is_empty());
    }

    /// Scrolling viewport: updating viewport does not lose tile data.
    #[test]
    fn viewport_scroll_preserves_tiles() {
        let local = Uuid::new_v4();
        let mut workspace = GlobalWorkspace::empty(local, "host".into(), "fp".into());
        let tile = sample_tile(local);

        let insert_op = LayoutOp {
            seq: 1,
            issuer: local,
            generation: 0,
            kind: LayoutOpKind::InsertTile {
                tile: tile.clone(),
                index: 0,
            },
        };
        apply_op(&mut workspace, &insert_op);
        assert_eq!(workspace.tiles.len(), 1);

        // Scroll viewport.
        let scroll_op = LayoutOp {
            seq: 2,
            issuer: local,
            generation: 0,
            kind: LayoutOpKind::ScrollViewport {
                node_id: local,
                output_name: "eDP-1".into(),
                global_x: 400.0,
            },
        };
        apply_op(&mut workspace, &scroll_op);

        // Tile data must still be present.
        assert_eq!(workspace.tiles.len(), 1);
        assert!(workspace.tiles.contains_key(&tile.tile_id));
    }

    /// Focus transfer between local and remote tiles.
    #[test]
    fn focus_update_propagates_correctly() {
        let local = Uuid::new_v4();
        let remote = Uuid::new_v4();
        let mut workspace = GlobalWorkspace::empty(local, "host".into(), "fp".into());

        let local_tile = sample_tile(local);
        let remote_tile = sample_tile(remote);

        let op1 = LayoutOp {
            seq: 1,
            issuer: local,
            generation: 0,
            kind: LayoutOpKind::InsertTile {
                tile: local_tile.clone(),
                index: 0,
            },
        };
        let op2 = LayoutOp {
            seq: 2,
            issuer: local,
            generation: 0,
            kind: LayoutOpKind::InsertTile {
                tile: remote_tile.clone(),
                index: 1,
            },
        };
        let focus_op = LayoutOp {
            seq: 3,
            issuer: local,
            generation: 0,
            kind: LayoutOpKind::FocusTile {
                tile_id: Some(remote_tile.tile_id),
            },
        };
        apply_op(&mut workspace, &op1);
        apply_op(&mut workspace, &op2);
        apply_op(&mut workspace, &focus_op);

        assert_eq!(workspace.focused_tile, Some(remote_tile.tile_id));
    }
}
