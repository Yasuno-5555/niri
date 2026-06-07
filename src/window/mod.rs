use std::cmp::{max, min};

use niri_config::utils::MergeWith as _;
use niri_config::window_rule::{Match, WindowRule};
use niri_config::{
    BackgroundEffect, BlockOutFrom, BorderRule, CornerRadius, FloatingPosition, PresetSize,
    ResolvedPopupsRules, ShadowRule, TabIndicatorRule,
};
use niri_ipc::ColumnDisplay;
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_toplevel;
use smithay::utils::{Logical, Size};
use smithay::wayland::compositor::with_states;
use smithay::wayland::shell::xdg::{
    SurfaceCachedState, ToplevelSurface, XdgToplevelSurfaceRoleAttributes,
};

use crate::utils::with_toplevel_role;

pub mod mapped;
pub use mapped::Mapped;

pub mod unmapped;
pub use unmapped::{InitialConfigureState, Unmapped};

/// Reference to a mapped or unmapped window.
#[derive(Debug, Clone, Copy)]
pub enum WindowRef<'a> {
    Unmapped(&'a Unmapped),
    Mapped(&'a Mapped),
}

/// Rules fully resolved for a window.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ResolvedWindowRules {
    /// Default width for this window.
    ///
    /// - `None`: unset (global default should be used).
    /// - `Some(None)`: set to empty (window picks its own width).
    /// - `Some(Some(width))`: set to a particular width.
    pub default_width: Option<Option<PresetSize>>,

    /// Default height for this window.
    ///
    /// - `None`: unset (global default should be used).
    /// - `Some(None)`: set to empty (window picks its own height).
    /// - `Some(Some(height))`: set to a particular height.
    pub default_height: Option<Option<PresetSize>>,

    /// Default column display for this window.
    pub default_column_display: Option<ColumnDisplay>,

    /// Default floating position for this window.
    pub default_floating_position: Option<FloatingPosition>,

    /// Output to open this window on.
    pub open_on_output: Option<String>,

    /// Workspace to open this window on.
    pub open_on_workspace: Option<String>,

    /// Whether the window should open full-width.
    pub open_maximized: Option<bool>,

    /// Whether the window should open maximized to edges (true maximized).
    pub open_maximized_to_edges: Option<bool>,

    /// Whether the window should open fullscreen.
    pub open_fullscreen: Option<bool>,

    /// Whether the window should open floating.
    pub open_floating: Option<bool>,

    /// Whether the window should open focused.
    pub open_focused: Option<bool>,

    /// Extra bound on the minimum window width.
    pub min_width: Option<u16>,
    /// Extra bound on the minimum window height.
    pub min_height: Option<u16>,
    /// Extra bound on the maximum window width.
    pub max_width: Option<u16>,
    /// Extra bound on the maximum window height.
    pub max_height: Option<u16>,

    /// Focus ring overrides.
    pub focus_ring: BorderRule,
    /// Window border overrides.
    pub border: BorderRule,
    /// Shadow overrides.
    pub shadow: ShadowRule,
    /// Tab indicator overrides.
    pub tab_indicator: TabIndicatorRule,

    /// Whether or not to draw the border with a solid background.
    ///
    /// `None` means using the SSD heuristic.
    pub draw_border_with_background: Option<bool>,

    /// Extra opacity to draw this window with.
    pub opacity: Option<f32>,

    /// Corner radius to assume this window has.
    pub geometry_corner_radius: Option<CornerRadius>,

    /// Whether to clip this window to its geometry, including the corner radius.
    pub clip_to_geometry: Option<bool>,

    /// Whether to bob this window up and down.
    pub baba_is_float: Option<bool>,

    /// Whether to block out this window from certain render targets.
    pub block_out_from: Option<BlockOutFrom>,

    /// Whether to enable VRR on this window's primary output if it is on-demand.
    pub variable_refresh_rate: Option<bool>,

    /// Multiplier for all scroll events sent to this window.
    pub scroll_factor: Option<f64>,

    /// Override whether to set the Tiled xdg-toplevel state on the window.
    pub tiled_state: Option<bool>,

    /// Background effect configuration.
    pub background_effect: BackgroundEffect,

    /// Rules for this window's popups.
    pub popups: ResolvedPopupsRules,

    /// The name of the active effect preset, if any.
    pub effect_preset: Option<String>,
}

impl<'a> WindowRef<'a> {
    pub fn toplevel(self) -> &'a ToplevelSurface {
        match self {
            WindowRef::Unmapped(unmapped) => unmapped.toplevel(),
            WindowRef::Mapped(mapped) => mapped.toplevel(),
        }
    }

    pub fn is_focused(self) -> bool {
        match self {
            WindowRef::Unmapped(_) => false,
            WindowRef::Mapped(mapped) => mapped.is_focused(),
        }
    }

    pub fn is_urgent(self) -> bool {
        match self {
            WindowRef::Unmapped(_) => false,
            WindowRef::Mapped(mapped) => mapped.is_urgent(),
        }
    }

    pub fn is_active_in_column(self) -> bool {
        match self {
            WindowRef::Unmapped(_) => true,
            WindowRef::Mapped(mapped) => mapped.is_active_in_column(),
        }
    }

    pub fn is_floating(self) -> bool {
        match self {
            // FIXME: This means you cannot set initial configure rules based on is-floating. I'm
            // not sure there's a good way to support it, since this matcher makes a cycle with the
            // open-floating rule.
            //
            // That said, I don't think there are a lot of useful initial configure properties you
            // may want to set through an is-floating matcher? Like, if you're configuring a
            // specific window to open as floating, you can also set those properties in that same
            // window rule, rather than relying on a different is-floating rule.
            WindowRef::Unmapped(_) => false,
            WindowRef::Mapped(mapped) => mapped.is_floating(),
        }
    }

    pub fn is_window_cast_target(self) -> bool {
        match self {
            WindowRef::Unmapped(_) => false,
            WindowRef::Mapped(mapped) => mapped.is_window_cast_target(),
        }
    }
}

impl ResolvedWindowRules {
    pub fn compute(
        rules: &[WindowRule],
        window: WindowRef,
        is_at_startup: bool,
        presets: &[niri_config::EffectPreset],
        materials: &[niri_config::Material],
    ) -> Self {
        let _span = tracy_client::span!("ResolvedWindowRules::compute");

        let mut resolved = ResolvedWindowRules::default();

        with_toplevel_role(window.toplevel(), |role| {
            // Ensure server_pending like in Smithay's with_pending_state().
            if role.server_pending.is_none() {
                role.server_pending = Some(role.current_server_state().clone());
            }

            let mut open_on_output = None;
            let mut open_on_workspace = None;

            for rule in rules {
                let matches = |m: &Match| {
                    if let Some(at_startup) = m.at_startup {
                        if at_startup != is_at_startup {
                            return false;
                        }
                    }

                    window_matches(window, role, m)
                };

                if !(rule.matches.is_empty() || rule.matches.iter().any(matches)) {
                    continue;
                }

                if rule.excludes.iter().any(matches) {
                    continue;
                }

                if let Some(preset_name) = &rule.effect_preset {
                    resolved.effect_preset = Some(preset_name.clone());
                    if let Some(preset) = presets.iter().find(|p| &p.name == preset_name) {
                        if let Some(material_name) = &preset.material {
                            if let Some(mat) = get_material(materials, material_name) {
                                if mat.blur.is_some() {
                                    resolved.background_effect.blur = Some(true);
                                }
                                if let Some(sat) = mat.saturation {
                                    resolved.background_effect.saturation = Some(sat.0);
                                }
                                if let Some(noise) = mat.noise {
                                    resolved.background_effect.noise = Some(noise.0);
                                }
                                if mat.refraction.is_some()
                                    || mat.specular.is_some()
                                    || mat.dispersion.is_some()
                                {
                                    resolved.background_effect.liquid = Some(true);
                                }
                                if let Some(refraction) = mat.refraction {
                                    resolved.background_effect.refraction =
                                        Some(refraction.strength.map(|s| s.0).unwrap_or(0.0));
                                }
                                if let Some(spec) = mat.specular {
                                    resolved.background_effect.specular =
                                        Some(spec.strength.map(|s| s.0).unwrap_or(0.0));
                                }
                                if let Some(edge) = &mat.edge_highlight {
                                    resolved.background_effect.edge_highlight =
                                        Some(edge.width.map(|w| w.0).unwrap_or(0.0));
                                }
                                if let Some(bloom) = mat.bloom {
                                    resolved.background_effect.bloom = Some(bloom.0);
                                }
                                if let Some(dispersion) = mat.dispersion {
                                    if let Some(strength) = dispersion.strength {
                                        resolved.background_effect.chromatic_aberration =
                                            Some(strength.0);
                                    }
                                }
                            }
                        }
                        if let Some(radius) = preset.corner_radius {
                            resolved.geometry_corner_radius = Some(radius);
                        }
                    }
                }

                if let Some(x) = rule.default_column_width {
                    resolved.default_width = Some(x.0);
                }

                if let Some(x) = rule.default_window_height {
                    resolved.default_height = Some(x.0);
                }

                if let Some(x) = rule.default_column_display {
                    resolved.default_column_display = Some(x);
                }

                if let Some(x) = rule.default_floating_position {
                    resolved.default_floating_position = Some(x);
                }

                if let Some(x) = rule.open_on_output.as_deref() {
                    open_on_output = Some(x);
                }

                if let Some(x) = rule.open_on_workspace.as_deref() {
                    open_on_workspace = Some(x);
                }

                if let Some(x) = rule.open_maximized {
                    resolved.open_maximized = Some(x);
                }

                if let Some(x) = rule.open_maximized_to_edges {
                    resolved.open_maximized_to_edges = Some(x);
                }

                if let Some(x) = rule.open_fullscreen {
                    resolved.open_fullscreen = Some(x);
                }

                if let Some(x) = rule.open_floating {
                    resolved.open_floating = Some(x);
                }

                if let Some(x) = rule.open_focused {
                    resolved.open_focused = Some(x);
                }

                if let Some(x) = rule.min_width {
                    resolved.min_width = Some(x);
                }
                if let Some(x) = rule.min_height {
                    resolved.min_height = Some(x);
                }
                if let Some(x) = rule.max_width {
                    resolved.max_width = Some(x);
                }
                if let Some(x) = rule.max_height {
                    resolved.max_height = Some(x);
                }

                resolved.focus_ring.merge_with(&rule.focus_ring);
                resolved.border.merge_with(&rule.border);
                resolved.shadow.merge_with(&rule.shadow);
                resolved.tab_indicator.merge_with(&rule.tab_indicator);

                if let Some(x) = rule.draw_border_with_background {
                    resolved.draw_border_with_background = Some(x);
                }
                if let Some(x) = rule.opacity {
                    resolved.opacity = Some(x);
                }
                if let Some(x) = rule.geometry_corner_radius {
                    resolved.geometry_corner_radius = Some(x);
                }
                if let Some(x) = rule.clip_to_geometry {
                    resolved.clip_to_geometry = Some(x);
                }
                if let Some(x) = rule.baba_is_float {
                    resolved.baba_is_float = Some(x);
                }
                if let Some(x) = rule.block_out_from {
                    resolved.block_out_from = Some(x);
                }
                if let Some(x) = rule.variable_refresh_rate {
                    resolved.variable_refresh_rate = Some(x);
                }
                if let Some(x) = rule.scroll_factor {
                    resolved.scroll_factor = Some(x.0);
                }
                if let Some(x) = rule.tiled_state {
                    resolved.tiled_state = Some(x);
                }

                resolved
                    .background_effect
                    .merge_with(&rule.background_effect);

                resolved.popups.merge_with(&rule.popups);
            }

            resolved.open_on_output = open_on_output.map(|x| x.to_owned());
            resolved.open_on_workspace = open_on_workspace.map(|x| x.to_owned());
        });

        resolved
    }

    pub fn apply_min_size(&self, min_size: Size<i32, Logical>) -> Size<i32, Logical> {
        let mut size = min_size;

        if let Some(x) = self.min_width {
            size.w = max(size.w, i32::from(x));
        }
        if let Some(x) = self.min_height {
            size.h = max(size.h, i32::from(x));
        }

        size
    }

    pub fn apply_max_size(&self, max_size: Size<i32, Logical>) -> Size<i32, Logical> {
        let mut size = max_size;

        if let Some(x) = self.max_width {
            if size.w == 0 {
                size.w = i32::from(x);
            } else if x > 0 {
                size.w = min(size.w, i32::from(x));
            }
        }
        if let Some(x) = self.max_height {
            if size.h == 0 {
                size.h = i32::from(x);
            } else if x > 0 {
                size.h = min(size.h, i32::from(x));
            }
        }

        size
    }

    pub fn apply_min_max_size(
        &self,
        min_size: Size<i32, Logical>,
        max_size: Size<i32, Logical>,
    ) -> (Size<i32, Logical>, Size<i32, Logical>) {
        let min_size = self.apply_min_size(min_size);
        let max_size = self.apply_max_size(max_size);
        (min_size, max_size)
    }

    pub fn compute_open_floating(&self, toplevel: &ToplevelSurface) -> bool {
        if let Some(res) = self.open_floating {
            return res;
        }

        // Windows with a parent (usually dialogs) open as floating by default.
        if toplevel.parent().is_some() {
            return true;
        }

        let (min_size, max_size) = with_states(toplevel.wl_surface(), |state| {
            let mut guard = state.cached_state.get::<SurfaceCachedState>();
            let current = guard.current();
            (current.min_size, current.max_size)
        });
        let (min_size, max_size) = self.apply_min_max_size(min_size, max_size);

        // We open fixed-height windows as floating.
        min_size.h > 0 && min_size.h == max_size.h
    }
}

fn window_matches(window: WindowRef, role: &XdgToplevelSurfaceRoleAttributes, m: &Match) -> bool {
    // Must be ensured by the caller.
    let server_pending = role.server_pending.as_ref().unwrap();

    if let Some(is_focused) = m.is_focused {
        if window.is_focused() != is_focused {
            return false;
        }
    }

    if let Some(is_urgent) = m.is_urgent {
        if window.is_urgent() != is_urgent {
            return false;
        }
    }

    if let Some(is_active) = m.is_active {
        // Our "is-active" definition corresponds to the window having a pending Activated state.
        let pending_activated = server_pending
            .states
            .contains(xdg_toplevel::State::Activated);
        if is_active != pending_activated {
            return false;
        }
    }

    if let Some(app_id_re) = &m.app_id {
        let Some(app_id) = &role.app_id else {
            return false;
        };
        if !app_id_re.0.is_match(app_id) {
            return false;
        }
    }

    if let Some(title_re) = &m.title {
        let Some(title) = &role.title else {
            return false;
        };
        if !title_re.0.is_match(title) {
            return false;
        }
    }

    if let Some(is_active_in_column) = m.is_active_in_column {
        if window.is_active_in_column() != is_active_in_column {
            return false;
        }
    }

    if let Some(is_floating) = m.is_floating {
        if window.is_floating() != is_floating {
            return false;
        }
    }

    if let Some(is_window_cast_target) = m.is_window_cast_target {
        if window.is_window_cast_target() != is_window_cast_target {
            return false;
        }
    }

    true
}

fn get_material<'a>(
    materials: &'a [niri_config::Material],
    name: &str,
) -> Option<std::borrow::Cow<'a, niri_config::Material>> {
    if let Some(mat) = materials.iter().find(|m| m.name == name) {
        return Some(std::borrow::Cow::Borrowed(mat));
    }

    // Fallback to built-in presets
    let mut built_in = niri_config::Material {
        name: name.to_string(),
        blur: None,
        tint: None,
        saturation: None,
        noise: None,
        refraction: None,
        specular: None,
        edge_highlight: None,
        bloom: None,
        dispersion: None,
        debug: None,
    };

    match name {
        "frosted-ceramic" => {
            built_in.blur = Some(niri_config::BlurPart {
                off: false,
                on: true,
                ..Default::default()
            });
            built_in.tint = Some("rgba(255, 255, 255, 0.4)".to_string());
            built_in.refraction = Some(niri_config::MaterialRefraction {
                strength: Some(niri_config::FloatOrInt(0.015)),
                edge_strength: None,
                normal_noise: None,
            });
            Some(std::borrow::Cow::Owned(built_in))
        }
        "obsidian-glass" => {
            built_in.blur = Some(niri_config::BlurPart {
                off: false,
                on: true,
                ..Default::default()
            });
            built_in.tint = Some("rgba(15, 15, 15, 0.7)".to_string());
            built_in.specular = Some(niri_config::MaterialSpecular {
                strength: Some(niri_config::FloatOrInt(0.18)),
                angle: None,
                width: None,
            });
            built_in.refraction = Some(niri_config::MaterialRefraction {
                strength: Some(niri_config::FloatOrInt(0.01)),
                edge_strength: None,
                normal_noise: None,
            });
            Some(std::borrow::Cow::Owned(built_in))
        }
        "hologram-film" => {
            built_in.blur = Some(niri_config::BlurPart {
                off: false,
                on: true,
                ..Default::default()
            });
            built_in.tint = Some("rgba(200, 220, 255, 0.15)".to_string());
            built_in.dispersion = Some(niri_config::MaterialDispersion {
                strength: Some(niri_config::FloatOrInt(0.06)),
                red_offset: None,
                blue_offset: None,
            });
            Some(std::borrow::Cow::Owned(built_in))
        }
        "paper-mist" => {
            built_in.blur = Some(niri_config::BlurPart {
                off: false,
                on: true,
                ..Default::default()
            });
            built_in.noise = Some(niri_config::FloatOrInt(0.35));
            built_in.tint = Some("rgba(240, 240, 240, 0.3)".to_string());
            Some(std::borrow::Cow::Owned(built_in))
        }
        "acrylic-smoke" => {
            built_in.blur = Some(niri_config::BlurPart {
                off: false,
                on: true,
                ..Default::default()
            });
            built_in.noise = Some(niri_config::FloatOrInt(0.15));
            built_in.tint = Some("rgba(30, 30, 30, 0.6)".to_string());
            built_in.saturation = Some(niri_config::FloatOrInt(0.8));
            Some(std::borrow::Cow::Owned(built_in))
        }
        "neon-wet" => {
            built_in.blur = Some(niri_config::BlurPart {
                off: false,
                on: true,
                ..Default::default()
            });
            built_in.saturation = Some(niri_config::FloatOrInt(1.6));
            built_in.refraction = Some(niri_config::MaterialRefraction {
                strength: Some(niri_config::FloatOrInt(0.04)),
                edge_strength: None,
                normal_noise: None,
            });
            built_in.dispersion = Some(niri_config::MaterialDispersion {
                strength: Some(niri_config::FloatOrInt(0.08)),
                red_offset: None,
                blue_offset: None,
            });
            Some(std::borrow::Cow::Owned(built_in))
        }
        "debug-wireframe" => {
            built_in.debug = Some(niri_config::MaterialDebug {
                show_bounds: Some(true),
                show_damage: Some(true),
                show_layer: None,
                show_material_id: None,
                show_animation_state: None,
            });
            Some(std::borrow::Cow::Owned(built_in))
        }
        _ => None,
    }
}
