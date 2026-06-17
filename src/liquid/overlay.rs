//! OverlayManager — unified management of internal UI overlays.
//!
//! Handles priority, exclusive input focus, z-ordering, and
//! render pipeline integration for:
//! - Mode HUD
//! - Action Palette
//! - Future: Debug Overlay
//! - Future: Keybind Viewer

/// Priority levels for UI overlays.
/// Lower number = higher priority (renders on top, captures input first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OverlayPriority {
    /// Emergency / lock screen. Always on top, not dismissable.
    Critical = 0,
    /// Action Palette — keyboard-driven command search.
    ActionPalette = 10,
    /// Power menu / exit confirmation.
    PowerMenu = 20,
    /// Debug overlay showing frame stats, material info, etc.
    DebugOverlay = 30,
    /// Keybind viewer overlay.
    KeybindViewer = 40,
    /// Mode HUD — transient status notifications.
    ModeHud = 50,
}

/// Result of a key event handled by an overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKeyResult {
    /// Key was consumed by this overlay.
    Consumed,
    /// Key was not handled, pass to next overlay / compositor.
    Pass,
    /// Overlay requested dismissal.
    Dismiss,
}

/// Trait for all compositor UI overlays.
pub trait UiOverlay {
    /// Unique identifier for this overlay.
    fn id(&self) -> &'static str;

    /// Whether the overlay is currently visible.
    fn visible(&self) -> bool;

    /// Priority for z-ordering and input capture.
    fn priority(&self) -> OverlayPriority;

    /// Handle a key event. Return whether it was consumed.
    fn handle_key(&mut self, _key: &str) -> OverlayKeyResult {
        OverlayKeyResult::Pass
    }

    /// Whether this overlay wants to capture all keyboard input
    /// while visible (e.g. Action Palette).
    fn captures_input(&self) -> bool {
        false
    }
}

/// Manages all compositor UI overlays.
///
/// Tracks which overlays are visible, handles priority-based
/// input routing, and coordinates render order.
pub struct OverlayManager {
    /// Overlays sorted by priority (highest first).
    overlays: Vec<Box<dyn UiOverlay>>,
    /// Which overlay currently has input capture, if any.
    input_captured_by: Option<String>,
}

impl OverlayManager {
    pub fn new() -> Self {
        Self {
            overlays: Vec::new(),
            input_captured_by: None,
        }
    }

    /// Register an overlay. Overlays are kept sorted by priority.
    pub fn register(&mut self, overlay: Box<dyn UiOverlay>) {
        self.overlays.push(overlay);
        self.overlays.sort_by_key(|o| o.priority());
    }

    /// Check if any overlay captures keyboard input.
    pub fn is_input_captured(&self) -> bool {
        self.topmost_visible().is_some_and(|o| o.captures_input())
    }

    /// Route a key event to the appropriate overlay.
    /// Returns true if the key was consumed.
    pub fn route_key(&mut self, key: &str) -> bool {
        // Only route to the topmost visible overlay that captures input.
        if let Some(overlay) = self.topmost_visible_mut() {
            if overlay.captures_input() {
                match overlay.handle_key(key) {
                    OverlayKeyResult::Consumed => return true,
                    OverlayKeyResult::Dismiss => {
                        self.input_captured_by = None;
                        return true;
                    }
                    OverlayKeyResult::Pass => {}
                }
            }
        }
        false
    }

    /// Check if any overlay at or above the given priority is visible.
    pub fn is_blocking(&self, priority: OverlayPriority) -> bool {
        self.overlays
            .iter()
            .filter(|o| o.visible())
            .any(|o| o.priority() <= priority)
    }

    /// Notify the manager that an overlay was shown/hidden.
    pub fn notify_visibility_changed(&mut self, overlay_id: &str, visible: bool) {
        if visible {
            // If a higher-priority overlay that captures input becomes visible,
            // it takes input capture.
            if let Some(overlay) = self
                .overlays
                .iter()
                .find(|o| o.id() == overlay_id && o.captures_input())
            {
                self.input_captured_by = Some(overlay.id().to_string());
            }
        } else if self.input_captured_by.as_deref() == Some(overlay_id) {
            self.input_captured_by = None;
        }
    }

    /// Return the topmost visible overlay for rendering, if any.
    pub fn topmost_visible(&self) -> Option<&dyn UiOverlay> {
        self.overlays
            .iter()
            .filter(|o| o.visible())
            .map(|o| o.as_ref())
            .next()
    }

    fn topmost_visible_mut(&mut self) -> Option<&mut Box<dyn UiOverlay>> {
        self.overlays.iter_mut().filter(|o| o.visible()).next()
    }
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── Overlay wrappers for existing UI components ──────────────────────

/// Adapter: wraps ActionPalette as a UiOverlay.
pub struct ActionPaletteOverlay<'a> {
    pub palette: &'a crate::ui::action_palette::ActionPalette,
}

impl UiOverlay for ActionPaletteOverlay<'_> {
    fn id(&self) -> &'static str {
        "action-palette"
    }

    fn visible(&self) -> bool {
        self.palette.is_open()
    }

    fn priority(&self) -> OverlayPriority {
        OverlayPriority::ActionPalette
    }

    fn captures_input(&self) -> bool {
        true
    }
}

/// Adapter: wraps ModeHud as a UiOverlay.
pub struct ModeHudOverlay<'a> {
    pub hud: &'a crate::ui::mode_hud::ModeHud,
}

impl UiOverlay for ModeHudOverlay<'_> {
    fn id(&self) -> &'static str {
        "mode-hud"
    }

    fn visible(&self) -> bool {
        self.hud.are_animations_ongoing()
    }

    fn priority(&self) -> OverlayPriority {
        OverlayPriority::ModeHud
    }

    fn captures_input(&self) -> bool {
        false
    }
}
