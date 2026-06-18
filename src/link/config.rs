use niri_config::Link as LinkConfig;

#[derive(Debug, Clone)]
pub struct RuntimeLinkConfig {
    pub enabled: bool,
    pub listen_address: String,
    pub discovery: bool,
    pub pairing_mode: bool,
    pub trusted_nodes: Vec<String>,
    pub transport_prefer: String,
    pub transport_fallback: String,
    pub preferred_codec: String,
    pub max_fps: u16,
    pub idle_fps: u16,
    pub max_bitrate_mbps: u16,
    pub stream_only_visible: bool,
    pub stale_frame_timeout_ms: u32,
    pub restore_last_session: bool,
    pub unlink_policy: String,
    pub remote_tile_placeholder: bool,
    pub animate_link_transition: bool,
    pub forward_keyboard: bool,
    pub forward_pointer: bool,
    pub forward_scroll: bool,
    pub forward_touch: bool,
    pub forward_tablet: bool,
    pub remote_focus_follows_pointer: bool,
}

impl From<&LinkConfig> for RuntimeLinkConfig {
    fn from(value: &LinkConfig) -> Self {
        Self {
            enabled: value.enable,
            listen_address: value.listen_address.clone(),
            discovery: value.discovery,
            pairing_mode: value.pairing_mode,
            trusted_nodes: value.trusted_nodes.clone(),
            transport_prefer: value.transport.prefer.clone(),
            transport_fallback: value.transport.fallback.clone(),
            preferred_codec: value.streaming.preferred_codec.clone(),
            max_fps: value.streaming.max_fps,
            idle_fps: value.streaming.idle_fps,
            max_bitrate_mbps: value.streaming.max_bitrate_mbps,
            stream_only_visible: value.streaming.stream_only_visible,
            stale_frame_timeout_ms: value.streaming.stale_frame_timeout_ms,
            restore_last_session: value.layout.restore_last_session,
            unlink_policy: value.layout.unlink_policy.clone(),
            remote_tile_placeholder: value.layout.remote_tile_placeholder,
            animate_link_transition: value.layout.animate_link_transition,
            forward_keyboard: value.input.forward_keyboard,
            forward_pointer: value.input.forward_pointer,
            forward_scroll: value.input.forward_scroll,
            forward_touch: value.input.forward_touch,
            forward_tablet: value.input.forward_tablet,
            remote_focus_follows_pointer: value.input.remote_focus_follows_pointer,
        }
    }
}
