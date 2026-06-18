use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use smithay::utils::{Logical, Size};
use uuid::Uuid;

pub type NodeId = Uuid;
pub type SessionId = Uuid;
pub type TileId = Uuid;
pub type ColumnId = Uuid;

pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Envelope {
    pub protocol_version: u16,
    pub session_id: SessionId,
    pub sender_node_id: NodeId,
    pub nonce: u64,
    pub timestamp_millis: u64,
    pub kind: MessageKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageKind {
    Hello(Hello),
    CapabilityExchange(CapabilityExchange),
    JoinRequest(JoinRequest),
    JoinAccept(JoinAccept),
    JoinReject(JoinReject),
    PeerList(PeerList),
    LeaderElection(LeaderElection),
    Heartbeat(Heartbeat),
    GlobalSnapshot(GlobalSnapshot),
    LayoutOp(LayoutOpMessage),
    LayoutAck(LayoutAck),
    ViewportUpdate(ViewportUpdate),
    TileMetadataUpdate(TileMetadataUpdate),
    TileClosed(TileClosed),
    FocusUpdate(FocusUpdate),
    InputEvent(InputEventMessage),
    FrameRequest(FrameRequest),
    FrameBegin(FrameBegin),
    FrameChunk(FrameChunk),
    FrameEnd(FrameEnd),
    StreamControl(StreamControl),
    ClipboardOffer(ClipboardOffer),
    DisableLink(DisableLink),
    PersistSession(PersistSession),
    RestoreSession(RestoreSession),
    Error(LinkError),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Hello {
    pub hostname: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CapabilityExchange {
    pub transports: Vec<String>,
    pub codecs: Vec<String>,
    pub clipboard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JoinRequest {
    pub hostname: String,
    pub pairing_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JoinAccept {
    pub leader_node_id: NodeId,
    pub generation: u64,
    pub operation_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JoinReject {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PeerList {
    pub peers: Vec<PeerDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerDescriptor {
    pub node_id: NodeId,
    pub hostname: String,
    pub fingerprint: String,
    pub addr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderElection {
    pub generation: u64,
    pub leader_node_id: NodeId,
    pub alive_nodes: Vec<NodeId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Heartbeat {
    pub operation_seq: u64,
    pub generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalSnapshot {
    pub workspace: GlobalWorkspace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutOpMessage {
    pub op: LayoutOp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutAck {
    pub seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ViewportUpdate {
    pub viewport: Viewport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TileMetadataUpdate {
    pub tile: TileMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TileClosed {
    pub tile_id: TileId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FocusUpdate {
    pub focused_tile: Option<TileId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputEventMessage {
    pub event: crate::link::input_forward::ForwardedInputEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameRequest {
    pub tile_id: TileId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameBegin {
    pub tile_id: TileId,
    pub frame_id: u64,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameChunk {
    pub tile_id: TileId,
    pub frame_id: u64,
    pub offset: usize,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrameEnd {
    pub tile_id: TileId,
    pub frame_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamControl {
    pub tile_id: TileId,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClipboardOffer {
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisableLink {
    pub persist: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistSession {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreSession {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LinkError {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Participant {
    pub node_id: NodeId,
    pub hostname: String,
    pub fingerprint: String,
    pub last_seen_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Viewport {
    pub node_id: NodeId,
    pub output_name: String,
    pub global_x: f64,
    pub global_y: f64,
    pub logical_width: f64,
    pub logical_height: f64,
    pub scale: f64,
    pub transform: i32,
    pub refresh_rate_millihz: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StreamState {
    Pending,
    Streaming,
    Idle,
    Stale,
    Disconnected,
    PrivacyBlocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PrivacyFlags {
    pub private: bool,
    pub clipboard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TileMetadata {
    pub tile_id: TileId,
    pub owner_node_id: NodeId,
    pub app_id: Option<String>,
    pub title: Option<String>,
    pub pid: Option<i32>,
    pub initial_size: (i32, i32),
    pub current_logical_size: (i32, i32),
    pub column_id: ColumnId,
    pub column_tile_index: usize,
    pub stack_group: Option<String>,
    pub fullscreen: bool,
    pub maximized: bool,
    pub floating: bool,
    pub last_known_alive_millis: u64,
    pub stream_state: StreamState,
    pub privacy_flags: PrivacyFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalWorkspace {
    pub session_id: SessionId,
    pub generation: u64,
    pub operation_seq: u64,
    pub columns: Vec<ColumnId>,
    pub tiles: BTreeMap<TileId, TileMetadata>,
    pub focused_tile: Option<TileId>,
    pub per_node_viewports: BTreeMap<NodeId, Vec<Viewport>>,
    pub per_node_outputs: BTreeMap<NodeId, Vec<Viewport>>,
    pub participants: BTreeMap<NodeId, Participant>,
    pub leader_node_id: NodeId,
}

impl GlobalWorkspace {
    pub fn empty(local_node_id: NodeId, hostname: String, fingerprint: String) -> Self {
        let session_id = SessionId::new_v4();
        let mut participants = BTreeMap::new();
        participants.insert(
            local_node_id,
            Participant {
                node_id: local_node_id,
                hostname,
                fingerprint,
                last_seen_millis: 0,
            },
        );
        Self {
            session_id,
            generation: 0,
            operation_seq: 0,
            columns: Vec::new(),
            tiles: BTreeMap::new(),
            focused_tile: None,
            per_node_viewports: BTreeMap::new(),
            per_node_outputs: BTreeMap::new(),
            participants,
            leader_node_id: local_node_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TileStateChange {
    Fullscreen(bool),
    Floating(bool),
    Maximized(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayoutOpKind {
    InsertTile {
        tile: TileMetadata,
        index: usize,
    },
    RemoveTile {
        tile_id: TileId,
    },
    MoveTile {
        tile_id: TileId,
        column_id: ColumnId,
        index: usize,
    },
    MoveColumn {
        column_id: ColumnId,
        index: usize,
    },
    ResizeColumn {
        column_id: ColumnId,
        width: i32,
    },
    FocusTile {
        tile_id: Option<TileId>,
    },
    FocusColumn {
        column_id: ColumnId,
    },
    ScrollViewport {
        node_id: NodeId,
        output_name: String,
        global_x: f64,
    },
    SetViewport {
        viewport: Viewport,
    },
    ChangeTileState {
        tile_id: TileId,
        change: TileStateChange,
    },
    ChangeTileMetadata {
        tile: TileMetadata,
    },
    SetFullscreen {
        tile_id: TileId,
        value: bool,
    },
    SetFloating {
        tile_id: TileId,
        value: bool,
    },
    EnterOverview,
    LeaveOverview,
    EnableLink,
    DisableLink,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutOp {
    pub seq: u64,
    pub issuer: NodeId,
    pub generation: u64,
    pub kind: LayoutOpKind,
}

pub fn logical_size_tuple(size: Size<i32, Logical>) -> (i32, i32) {
    (size.w, size.h)
}
