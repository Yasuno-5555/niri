use crate::utils::{Flag, MergeWith};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub enable: bool,
    pub listen_address: String,
    pub discovery: bool,
    pub pairing_mode: bool,
    pub trusted_nodes: Vec<String>,
    pub transport: LinkTransport,
    pub streaming: LinkStreaming,
    pub layout: LinkLayout,
    pub input: LinkInput,
}

impl Default for Link {
    fn default() -> Self {
        Self {
            enable: false,
            listen_address: "127.0.0.1:0".to_string(),
            discovery: true,
            pairing_mode: false,
            trusted_nodes: Vec::new(),
            transport: LinkTransport::default(),
            streaming: LinkStreaming::default(),
            layout: LinkLayout::default(),
            input: LinkInput::default(),
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq, Default)]
pub struct LinkPart {
    #[knuffel(child)]
    pub enable: Option<Flag>,
    #[knuffel(child, unwrap(argument))]
    pub listen_address: Option<String>,
    #[knuffel(child)]
    pub discovery: Option<Flag>,
    #[knuffel(child)]
    pub pairing_mode: Option<Flag>,
    #[knuffel(children(name = "trusted-node"))]
    pub trusted_nodes: Vec<TrustedNodeValue>,
    #[knuffel(child)]
    pub transport: Option<LinkTransportPart>,
    #[knuffel(child)]
    pub streaming: Option<LinkStreamingPart>,
    #[knuffel(child)]
    pub layout: Option<LinkLayoutPart>,
    #[knuffel(child)]
    pub input: Option<LinkInputPart>,
}

impl MergeWith<LinkPart> for Link {
    fn merge_with(&mut self, part: &LinkPart) {
        merge!((self, part), enable, discovery, pairing_mode);
        merge_clone!((self, part), listen_address);
        self.trusted_nodes
            .extend(part.trusted_nodes.iter().map(|it| it.0.clone()));
        if let Some(transport) = &part.transport {
            self.transport.merge_with(transport);
        }
        if let Some(streaming) = &part.streaming {
            self.streaming.merge_with(streaming);
        }
        if let Some(layout) = &part.layout {
            self.layout.merge_with(layout);
        }
        if let Some(input) = &part.input {
            self.input.merge_with(input);
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq)]
pub struct TrustedNodeValue(#[knuffel(argument)] pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkTransport {
    pub prefer: String,
    pub fallback: String,
}

impl Default for LinkTransport {
    fn default() -> Self {
        Self {
            prefer: "quic".to_string(),
            fallback: "tcp-tls".to_string(),
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq, Default)]
pub struct LinkTransportPart {
    #[knuffel(child, unwrap(argument))]
    pub prefer: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub fallback: Option<String>,
}

impl MergeWith<LinkTransportPart> for LinkTransport {
    fn merge_with(&mut self, part: &LinkTransportPart) {
        merge_clone!((self, part), prefer, fallback);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkStreaming {
    pub preferred_codec: String,
    pub max_fps: u16,
    pub max_bitrate_mbps: u16,
    pub idle_fps: u16,
    pub stream_only_visible: bool,
    pub stale_frame_timeout_ms: u32,
}

impl Default for LinkStreaming {
    fn default() -> Self {
        Self {
            preferred_codec: "av1,h264,raw".to_string(),
            max_fps: 60,
            max_bitrate_mbps: 80,
            idle_fps: 5,
            stream_only_visible: true,
            stale_frame_timeout_ms: 500,
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq, Default)]
pub struct LinkStreamingPart {
    #[knuffel(child, unwrap(argument))]
    pub preferred_codec: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub max_fps: Option<u16>,
    #[knuffel(child, unwrap(argument))]
    pub max_bitrate_mbps: Option<u16>,
    #[knuffel(child, unwrap(argument))]
    pub idle_fps: Option<u16>,
    #[knuffel(child)]
    pub stream_only_visible: Option<Flag>,
    #[knuffel(child, unwrap(argument))]
    pub stale_frame_timeout_ms: Option<u32>,
}

impl MergeWith<LinkStreamingPart> for LinkStreaming {
    fn merge_with(&mut self, part: &LinkStreamingPart) {
        merge_clone!((self, part), preferred_codec);
        if let Some(max_fps) = part.max_fps {
            self.max_fps = max_fps;
        }
        if let Some(max_bitrate_mbps) = part.max_bitrate_mbps {
            self.max_bitrate_mbps = max_bitrate_mbps;
        }
        if let Some(idle_fps) = part.idle_fps {
            self.idle_fps = idle_fps;
        }
        if let Some(stale_frame_timeout_ms) = part.stale_frame_timeout_ms {
            self.stale_frame_timeout_ms = stale_frame_timeout_ms;
        }
        merge!((self, part), stream_only_visible);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkLayout {
    pub mode: String,
    pub restore_last_session: bool,
    pub unlink_policy: String,
    pub remote_tile_placeholder: bool,
    pub animate_link_transition: bool,
}

impl Default for LinkLayout {
    fn default() -> Self {
        Self {
            mode: "continuous-horizontal".to_string(),
            restore_last_session: true,
            unlink_policy: "keep-owned-tiles".to_string(),
            remote_tile_placeholder: true,
            animate_link_transition: true,
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq, Default)]
pub struct LinkLayoutPart {
    #[knuffel(child, unwrap(argument))]
    pub mode: Option<String>,
    #[knuffel(child)]
    pub restore_last_session: Option<Flag>,
    #[knuffel(child, unwrap(argument))]
    pub unlink_policy: Option<String>,
    #[knuffel(child)]
    pub remote_tile_placeholder: Option<Flag>,
    #[knuffel(child)]
    pub animate_link_transition: Option<Flag>,
}

impl MergeWith<LinkLayoutPart> for LinkLayout {
    fn merge_with(&mut self, part: &LinkLayoutPart) {
        merge_clone!((self, part), mode, unlink_policy);
        merge!(
            (self, part),
            restore_last_session,
            remote_tile_placeholder,
            animate_link_transition
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkInput {
    pub forward_keyboard: bool,
    pub forward_pointer: bool,
    pub forward_scroll: bool,
    pub forward_touch: bool,
    pub forward_tablet: bool,
    pub remote_focus_follows_pointer: bool,
}

impl Default for LinkInput {
    fn default() -> Self {
        Self {
            forward_keyboard: true,
            forward_pointer: true,
            forward_scroll: true,
            forward_touch: false,
            forward_tablet: false,
            remote_focus_follows_pointer: true,
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq, Default)]
pub struct LinkInputPart {
    #[knuffel(child)]
    pub forward_keyboard: Option<Flag>,
    #[knuffel(child)]
    pub forward_pointer: Option<Flag>,
    #[knuffel(child)]
    pub forward_scroll: Option<Flag>,
    #[knuffel(child)]
    pub forward_touch: Option<Flag>,
    #[knuffel(child)]
    pub forward_tablet: Option<Flag>,
    #[knuffel(child)]
    pub remote_focus_follows_pointer: Option<Flag>,
}

impl MergeWith<LinkInputPart> for LinkInput {
    fn merge_with(&mut self, part: &LinkInputPart) {
        merge!(
            (self, part),
            forward_keyboard,
            forward_pointer,
            forward_scroll,
            forward_touch,
            forward_tablet,
            remote_focus_follows_pointer
        );
    }
}
