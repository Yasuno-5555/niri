use crate::appearance::{Color, WorkspaceShadow, WorkspaceShadowPart, DEFAULT_BACKDROP_COLOR};
use crate::utils::{Flag, MergeWith};
use crate::FloatOrInt;

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq)]
pub struct SpawnAtStartup {
    #[knuffel(arguments)]
    pub command: Vec<String>,
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq)]
pub struct SpawnShAtStartup {
    #[knuffel(argument)]
    pub command: String,
}

#[derive(Debug, PartialEq)]
pub struct Cursor {
    pub xcursor_theme: String,
    pub xcursor_size: u8,
    pub hide_when_typing: bool,
    pub hide_after_inactive_ms: Option<u32>,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            xcursor_theme: String::from("default"),
            xcursor_size: 24,
            hide_when_typing: false,
            hide_after_inactive_ms: None,
        }
    }
}

#[derive(knuffel::Decode, Debug, PartialEq)]
pub struct CursorPart {
    #[knuffel(child, unwrap(argument))]
    pub xcursor_theme: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub xcursor_size: Option<u8>,
    #[knuffel(child)]
    pub hide_when_typing: Option<Flag>,
    #[knuffel(child, unwrap(argument))]
    pub hide_after_inactive_ms: Option<u32>,
}

impl MergeWith<CursorPart> for Cursor {
    fn merge_with(&mut self, part: &CursorPart) {
        merge_clone!((self, part), xcursor_theme, xcursor_size);
        merge!((self, part), hide_when_typing);
        merge_clone_opt!((self, part), hide_after_inactive_ms);
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct ScreenshotPath(#[knuffel(argument)] pub Option<String>);

impl Default for ScreenshotPath {
    fn default() -> Self {
        Self(Some(String::from(
            "~/Pictures/Screenshots/Screenshot from %Y-%m-%d %H-%M-%S.png",
        )))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyOverlay {
    pub skip_at_startup: bool,
    pub hide_not_bound: bool,
}

#[derive(knuffel::Decode, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct HotkeyOverlayPart {
    #[knuffel(child)]
    pub skip_at_startup: Option<Flag>,
    #[knuffel(child)]
    pub hide_not_bound: Option<Flag>,
}

impl MergeWith<HotkeyOverlayPart> for HotkeyOverlay {
    fn merge_with(&mut self, part: &HotkeyOverlayPart) {
        merge!((self, part), skip_at_startup, hide_not_bound);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ConfigNotification {
    pub disable_failed: bool,
}

#[derive(knuffel::Decode, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ConfigNotificationPart {
    #[knuffel(child)]
    pub disable_failed: Option<Flag>,
}

impl MergeWith<ConfigNotificationPart> for ConfigNotification {
    fn merge_with(&mut self, part: &ConfigNotificationPart) {
        merge!((self, part), disable_failed);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Clipboard {
    pub disable_primary: bool,
}

#[derive(knuffel::Decode, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardPart {
    #[knuffel(child)]
    pub disable_primary: Option<Flag>,
}

impl MergeWith<ClipboardPart> for Clipboard {
    fn merge_with(&mut self, part: &ClipboardPart) {
        merge!((self, part), disable_primary);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Overview {
    pub zoom: f64,
    pub backdrop_color: Color,
    pub workspace_shadow: WorkspaceShadow,
}

impl Default for Overview {
    fn default() -> Self {
        Self {
            zoom: 0.5,
            backdrop_color: DEFAULT_BACKDROP_COLOR,
            workspace_shadow: WorkspaceShadow::default(),
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct OverviewPart {
    #[knuffel(child, unwrap(argument))]
    pub zoom: Option<FloatOrInt<0, 1>>,
    #[knuffel(child)]
    pub backdrop_color: Option<Color>,
    #[knuffel(child)]
    pub workspace_shadow: Option<WorkspaceShadowPart>,
}

impl MergeWith<OverviewPart> for Overview {
    fn merge_with(&mut self, part: &OverviewPart) {
        merge!((self, part), zoom, workspace_shadow);
        merge_clone!((self, part), backdrop_color);
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq, Eq)]
pub struct Environment(#[knuffel(children)] pub Vec<EnvironmentVariable>);

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentVariable {
    #[knuffel(node_name)]
    pub name: String,
    #[knuffel(argument)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XwaylandSatellite {
    pub off: bool,
    pub path: String,
}

impl Default for XwaylandSatellite {
    fn default() -> Self {
        Self {
            off: false,
            path: String::from("xwayland-satellite"),
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq, Eq)]
pub struct XwaylandSatellitePart {
    #[knuffel(child)]
    pub off: bool,
    #[knuffel(child)]
    pub on: bool,
    #[knuffel(child, unwrap(argument))]
    pub path: Option<String>,
}

impl MergeWith<XwaylandSatellitePart> for XwaylandSatellite {
    fn merge_with(&mut self, part: &XwaylandSatellitePart) {
        self.off |= part.off;
        if part.on {
            self.off = false;
        }

        merge_clone!((self, part), path);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionPalette {
    pub enable: bool,
    pub bind: Option<String>,
    pub material: Option<String>,
    pub fuzzy_search: bool,
    pub show_keybinds: bool,
    pub show_current_state: bool,
}

impl Default for ActionPalette {
    fn default() -> Self {
        Self {
            enable: false,
            bind: Some("Mod+P".to_string()),
            material: Some("dashboard-glass".to_string()),
            fuzzy_search: true,
            show_keybinds: true,
            show_current_state: true,
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct ActionPalettePart {
    #[knuffel(child)]
    pub enable: Option<Flag>,
    #[knuffel(child, unwrap(argument))]
    pub bind: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub material: Option<String>,
    #[knuffel(child)]
    pub fuzzy_search: Option<Flag>,
    #[knuffel(child)]
    pub show_keybinds: Option<Flag>,
    #[knuffel(child)]
    pub show_current_state: Option<Flag>,
}

impl MergeWith<ActionPalettePart> for ActionPalette {
    fn merge_with(&mut self, part: &ActionPalettePart) {
        merge!(
            (self, part),
            enable,
            fuzzy_search,
            show_keybinds,
            show_current_state
        );
        merge_clone_opt!((self, part), bind, material);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModeHud {
    pub enable: bool,
    pub position: String,
    pub duration_ms: u64,
    pub material: String,
    pub show: ModeHudShow,
}

impl Default for ModeHud {
    fn default() -> Self {
        Self {
            enable: false,
            position: "top-center".to_string(),
            duration_ms: 900,
            material: "hud-glass".to_string(),
            show: ModeHudShow::default(),
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct ModeHudPart {
    #[knuffel(child)]
    pub enable: Option<Flag>,
    #[knuffel(child, unwrap(argument))]
    pub position: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub duration_ms: Option<u64>,
    #[knuffel(child, unwrap(argument))]
    pub material: Option<String>,
    #[knuffel(child)]
    pub show: Option<ModeHudShowPart>,
}

impl MergeWith<ModeHudPart> for ModeHud {
    fn merge_with(&mut self, part: &ModeHudPart) {
        merge!((self, part), enable);
        merge_clone!((self, part), position, duration_ms, material);
        if let Some(show_part) = &part.show {
            self.show.merge_with(show_part);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeHudShow {
    pub animation_profile: bool,
    pub material: bool,
    pub performance_profile: bool,
    pub scratch_state: bool,
}

impl Default for ModeHudShow {
    fn default() -> Self {
        Self {
            animation_profile: true,
            material: true,
            performance_profile: true,
            scratch_state: true,
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ModeHudShowPart {
    #[knuffel(child)]
    pub animation_profile: Option<Flag>,
    #[knuffel(child)]
    pub material: Option<Flag>,
    #[knuffel(child)]
    pub performance_profile: Option<Flag>,
    #[knuffel(child)]
    pub scratch_state: Option<Flag>,
}

impl MergeWith<ModeHudShowPart> for ModeHudShow {
    fn merge_with(&mut self, part: &ModeHudShowPart) {
        merge!(
            (self, part),
            animation_profile,
            material,
            performance_profile,
            scratch_state
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Magnet {
    pub enable: bool,
    pub strength: f64,
    pub radius: f64,
    pub animation: String,
    pub haptic_visual: bool,
}

impl Default for Magnet {
    fn default() -> Self {
        Self {
            enable: false,
            strength: 0.65,
            radius: 32.0,
            animation: "soft-snap".to_string(),
            haptic_visual: false,
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct MagnetPart {
    #[knuffel(child)]
    pub enable: Option<Flag>,
    #[knuffel(child, unwrap(argument))]
    pub strength: Option<FloatOrInt<0, 100>>,
    #[knuffel(child, unwrap(argument))]
    pub radius: Option<FloatOrInt<0, 1000>>,
    #[knuffel(child, unwrap(argument))]
    pub animation: Option<String>,
    #[knuffel(child)]
    pub haptic_visual: Option<Flag>,
}

impl MergeWith<MagnetPart> for Magnet {
    fn merge_with(&mut self, part: &MagnetPart) {
        merge!((self, part), enable, haptic_visual);
        merge_clone!((self, part), animation);
        if let Some(s) = part.strength {
            self.strength = s.0;
        }
        if let Some(r) = part.radius {
            self.radius = r.0;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAnimationProfile {
    pub enable: bool,
    pub active: Option<String>,
    pub idle: Option<String>,
    pub on_battery: Option<String>,
    pub low_battery: Option<String>,
    pub power_saver: Option<String>,
    pub balanced: Option<String>,
    pub performance: Option<String>,
    pub settle_ms: u64,
}

impl Default for AdaptiveAnimationProfile {
    fn default() -> Self {
        Self {
            enable: false,
            active: None,
            idle: None,
            on_battery: None,
            low_battery: None,
            power_saver: None,
            balanced: None,
            performance: None,
            settle_ms: 900,
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct AdaptiveAnimationProfilePart {
    #[knuffel(child)]
    pub enable: Option<Flag>,
    #[knuffel(child, unwrap(argument))]
    pub active: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub idle: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub on_battery: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub low_battery: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub power_saver: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub balanced: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub performance: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub settle_ms: Option<u64>,
}

impl MergeWith<AdaptiveAnimationProfilePart> for AdaptiveAnimationProfile {
    fn merge_with(&mut self, part: &AdaptiveAnimationProfilePart) {
        merge!((self, part), enable);
        merge_clone_opt!(
            (self, part),
            active,
            idle,
            on_battery,
            low_battery,
            power_saver,
            balanced,
            performance
        );
        merge_clone!((self, part), settle_ms);
    }
}
