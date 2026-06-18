use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use niri_config::{Action, Config, CornerRadius, Key, ModKey, Modifiers};
use ordered_float::NotNan;
use pangocairo::cairo::{self, ImageSurface};
use pangocairo::pango::FontDescription;
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::gles::GlesTexture;
use smithay::input::keyboard::xkb::keysym_get_name;
use smithay::output::Output;
use smithay::reexports::gbm::Format as Fourcc;
use smithay::utils::{Point, Rectangle, Transform};

use crate::liquid::action_registry::ActionRegistry;
use crate::niri_render_elements;
use crate::render_helpers::background_effect::{
    BackgroundEffect, BackgroundEffectElement, RenderParams,
};
use crate::render_helpers::memory::MemoryBuffer;
use crate::render_helpers::primary_gpu_texture::PrimaryGpuTextureRenderElement;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::texture::{TextureBuffer, TextureRenderElement};
use crate::render_helpers::xray::XrayPos;
use crate::render_helpers::RenderCtx;
use crate::ui::hotkey_overlay::action_name;
use crate::utils::{output_size, to_physical_precise_round};

niri_render_elements! {
    ActionPaletteRenderElement => {
        Texture = PrimaryGpuTextureRenderElement,
        BackgroundEffect = BackgroundEffectElement,
    }
}

struct RenderedPalette {
    texture: TextureBuffer<GlesTexture>,
    background_effect: BackgroundEffect,
}

#[derive(Clone)]
struct ActionItem {
    display_name: String,
    action: Action,
    keybind_hint: Option<String>,
}

pub struct ActionPalette {
    is_open: bool,
    query: String,
    selected_idx: usize,
    all_actions: Vec<ActionItem>,
    filtered_actions: Vec<ActionItem>,
    buffers: RefCell<HashMap<NotNan<f64>, Option<RenderedPalette>>>,
    config: Rc<RefCell<Config>>,
    mod_key: ModKey,
    registry: ActionRegistry,
}

impl ActionPalette {
    pub fn new(config: Rc<RefCell<Config>>, mod_key: ModKey, registry: ActionRegistry) -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_idx: 0,
            all_actions: Vec::new(),
            filtered_actions: Vec::new(),
            buffers: RefCell::new(HashMap::new()),
            config,
            mod_key,
            registry,
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn show(&mut self) {
        if self.is_open {
            return;
        }
        self.is_open = true;
        self.query.clear();
        self.selected_idx = 0;

        let config_borrow = self.config.borrow();
        let registry = self.registry.clone();
        self.all_actions = populate_actions(&config_borrow, self.mod_key, &registry);
        drop(config_borrow);
        self.filter_actions();
        self.buffers.borrow_mut().clear();
    }

    pub fn hide(&mut self) {
        if !self.is_open {
            return;
        }
        self.is_open = false;
        self.buffers.borrow_mut().clear();
    }

    pub fn on_config_updated(&mut self, mod_key: ModKey) {
        self.mod_key = mod_key;
        self.buffers.borrow_mut().clear();
    }

    pub fn handle_char(&mut self, c: char) {
        self.query.push(c);
        self.selected_idx = 0;
        self.filter_actions();
        self.buffers.borrow_mut().clear();
    }

    pub fn handle_backspace(&mut self) {
        self.query.pop();
        self.selected_idx = 0;
        self.filter_actions();
        self.buffers.borrow_mut().clear();
    }

    pub fn handle_move_up(&mut self) {
        if self.filtered_actions.is_empty() {
            return;
        }
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        } else {
            self.selected_idx = self.filtered_actions.len() - 1;
        }
        self.buffers.borrow_mut().clear();
    }

    pub fn handle_move_down(&mut self) {
        if self.filtered_actions.is_empty() {
            return;
        }
        if self.selected_idx < self.filtered_actions.len() - 1 {
            self.selected_idx += 1;
        } else {
            self.selected_idx = 0;
        }
        self.buffers.borrow_mut().clear();
    }

    pub fn get_selected_action(&self) -> Option<Action> {
        if self.filtered_actions.is_empty() {
            None
        } else {
            Some(self.filtered_actions[self.selected_idx].action.clone())
        }
    }

    fn filter_actions(&mut self) {
        let config = self.config.borrow();
        let use_fuzzy = config.action_palette.fuzzy_search;
        let query = self.query.trim().to_lowercase();

        if query.is_empty() {
            self.filtered_actions = self.all_actions.clone();
        } else {
            self.filtered_actions = self
                .all_actions
                .iter()
                .filter(|item| {
                    let text = item.display_name.to_lowercase();
                    if use_fuzzy {
                        fuzzy_match(&text, &query)
                    } else {
                        text.contains(&query)
                    }
                })
                .cloned()
                .collect();
        }

        if self.selected_idx >= self.filtered_actions.len() && !self.filtered_actions.is_empty() {
            self.selected_idx = 0;
        }
    }

    pub fn render<R: NiriRenderer>(
        &self,
        mut ctx: RenderCtx<R>,
        output: &Output,
        push: &mut dyn FnMut(ActionPaletteRenderElement),
    ) -> Option<()> {
        if !self.is_open {
            return None;
        }

        let scale = output.current_scale().fractional_scale();
        let output_size = output_size(output);

        let mut buffers = self.buffers.borrow_mut();
        let key = NotNan::new(scale).unwrap();

        let has_cached = buffers.get(&key).is_some();
        if !has_cached {
            let buffer = match render_palette_box(
                scale,
                &self.query,
                &self.filtered_actions,
                self.selected_idx,
                &self.config.borrow(),
            ) {
                Ok(buf) => {
                    if let Ok(tex) =
                        TextureBuffer::from_memory_buffer(ctx.renderer.as_gles_renderer(), &buf)
                    {
                        Some(RenderedPalette {
                            texture: tex,
                            background_effect: BackgroundEffect::new(),
                        })
                    } else {
                        None
                    }
                }
                Err(err) => {
                    warn!("Failed to render ActionPalette texture: {:?}", err);
                    None
                }
            };
            buffers.insert(key.clone(), buffer);
        }

        let rendered = buffers.get_mut(&key).and_then(|x| x.as_mut())?;

        let size = rendered.texture.logical_size();
        let x = (output_size.w as f64 - size.w) / 2.0;
        let y = (output_size.h as f64 - size.h) / 2.0;
        let rect = Rectangle::new(Point::new(x, y), size);

        let config = self.config.borrow();
        let material = config.materials.iter().find(|m| {
            m.name
                == config
                    .action_palette
                    .material
                    .clone()
                    .unwrap_or_else(|| "dashboard-glass".to_string())
        });
        let mut effect = niri_config::BackgroundEffect::default();
        let mut blur_config = niri_config::Blur::default();

        if let Some(mat) = material {
            if mat.blur.is_some() {
                effect.blur = Some(true);
            }
            effect.saturation = mat.saturation.map(|s| s.0);
            effect.noise = mat.noise.map(|n| n.0);
            if mat.refraction.is_some() || mat.specular.is_some() || mat.dispersion.is_some() {
                effect.liquid = Some(true);
            }
            effect.refraction = mat.refraction.and_then(|r| r.strength.map(|s| s.0));
            effect.specular = mat.specular.and_then(|s| s.strength.map(|st| st.0));
            effect.chromatic_aberration = mat
                .dispersion
                .and_then(|dispersion| dispersion.strength.map(|s| s.0));
            effect.edge_highlight = mat
                .edge_highlight
                .as_ref()
                .and_then(|e| e.width.map(|w| w.0));
            effect.bloom = mat.bloom.map(|b| b.0);

            if let Some(blur) = &mat.blur {
                blur_config.passes = blur.passes.map(|p| p as u8).unwrap_or(0);
                blur_config.offset = blur.offset.map(|o| o.0).unwrap_or(0.0);
            }
        } else {
            // Default fallback
            effect.blur = Some(true);
            effect.saturation = Some(1.20);
            effect.noise = Some(0.010);
            effect.liquid = Some(true);
            effect.refraction = Some(0.012);
            effect.specular = Some(0.12);
            effect.edge_highlight = Some(0.06);

            blur_config.passes = 4;
            blur_config.offset = 5.0;
        }

        rendered.background_effect.update_config(blur_config);

        let corner_radius_phys: i32 = to_physical_precise_round(scale, 14.0);
        let corner_radius = CornerRadius::from(corner_radius_phys as f32);
        rendered
            .background_effect
            .update_render_elements(corner_radius, effect, false);

        let params = RenderParams {
            geometry: rect,
            subregion: None,
            clip: Some((rect, corner_radius)),
            scale,
        };

        let xray_pos = XrayPos::default();

        // Render glass background
        rendered
            .background_effect
            .render(ctx.r().as_gles(), None, params, xray_pos, &mut |elem| {
                push(ActionPaletteRenderElement::BackgroundEffect(elem))
            });

        // Render foreground texture
        let elem = TextureRenderElement::from_texture_buffer(
            rendered.texture.clone(),
            rect.loc,
            1.0,
            None,
            None,
            Kind::Unspecified,
        );
        let elem = PrimaryGpuTextureRenderElement(elem);
        push(ActionPaletteRenderElement::Texture(elem));

        Some(())
    }
}

fn fuzzy_match(text: &str, query: &str) -> bool {
    let mut text_chars = text.chars();
    for q_char in query.chars() {
        loop {
            if let Some(t_char) = text_chars.next() {
                if t_char.to_lowercase().next() == q_char.to_lowercase().next() {
                    break;
                }
            } else {
                return false;
            }
        }
    }
    true
}

fn populate_actions(
    config: &Config,
    mod_key: ModKey,
    registry: &ActionRegistry,
) -> Vec<ActionItem> {
    let mut items = Vec::new();

    // First, add config-bound actions with labels from the registry.
    for bind in &config.binds.0 {
        // Try to get a label from the registry, fall back to the action name.
        let action_id = action_to_registry_id(&bind.action);
        let display_name = bind
            .hotkey_overlay_title
            .as_ref()
            .and_then(|title| title.clone())
            .or_else(|| action_id.and_then(|id| registry.find(id).map(|d| d.label.clone())))
            .unwrap_or_else(|| strip_markup(&action_name(&bind.action)));
        push_action_item(&mut items, bind.action.clone(), display_name);
    }

    // Add unbound actions from the registry.
    for desc in registry.all() {
        // Skip actions that are already listed via binds.
        if items
            .iter()
            .any(|item| action_to_registry_id(&item.action).is_some_and(|id| id == desc.id))
        {
            continue;
        }
        // Only include actions that have a corresponding Action variant.
        if let Some(action) = registry_id_to_action(&desc.id) {
            push_action_item(&mut items, action, desc.label.clone());
        }
    }

    // Add animation profiles (config-specific)
    for profile in &config.animation_profiles {
        let action = Action::SetAnimationProfile(profile.name.clone());
        push_action_item(
            &mut items,
            action,
            format!("Set Animation Profile: {}", profile.name),
        );
    }

    // Add materials (config-specific)
    for mat in &config.materials {
        let action = Action::SetMaterial(mat.name.clone());
        push_action_item(&mut items, action, format!("Set Material: {}", mat.name));
    }

    // Add scratch columns (config-specific)
    for scratch in &config.scratch_columns {
        let action = Action::ToggleScratchColumn(scratch.name.clone());
        push_action_item(
            &mut items,
            action,
            format!("Toggle Scratch Column: {}", scratch.name),
        );
    }

    // Populate keybind hints
    for item in &mut items {
        for bind in &config.binds.0 {
            if bind.action == item.action {
                item.keybind_hint = Some(format_bind_key(mod_key, &bind.key));
                break;
            }
        }
    }

    items
}

fn push_action_item(items: &mut Vec<ActionItem>, action: Action, display_name: String) {
    if items.iter().any(|item| item.action == action) {
        return;
    }

    items.push(ActionItem {
        display_name,
        action,
        keybind_hint: None,
    });
}

/// Map an Action enum variant to its registry id string.
fn action_to_registry_id(action: &Action) -> Option<&'static str> {
    match action {
        Action::Quit(..) => Some("quit"),
        Action::Suspend => Some("suspend"),
        Action::PowerOffMonitors => Some("power-off-monitors"),
        Action::PowerOnMonitors => Some("power-on-monitors"),
        Action::Spawn(..) => Some("spawn"),
        Action::SpawnSh(..) => Some("spawn-sh"),
        Action::DoScreenTransition(..) => Some("do-screen-transition"),
        Action::CloseWindow => Some("close-window"),
        Action::FullscreenWindow => Some("fullscreen-window"),
        Action::ToggleWindowedFullscreen => Some("toggle-windowed-fullscreen"),
        Action::ToggleKeyboardShortcutsInhibit => Some("toggle-keyboard-shortcuts-inhibit"),
        Action::FocusColumnLeft => Some("focus-column-left"),
        Action::FocusColumnRight => Some("focus-column-right"),
        Action::FocusWindowUp => Some("focus-window-up"),
        Action::FocusWindowDown => Some("focus-window-down"),
        Action::FocusWindowOrMonitorUp => Some("focus-window-or-monitor-up"),
        Action::FocusWindowOrMonitorDown => Some("focus-window-or-monitor-down"),
        Action::FocusColumnOrMonitorLeft => Some("focus-column-or-monitor-left"),
        Action::FocusColumnOrMonitorRight => Some("focus-column-or-monitor-right"),
        Action::FocusWindowPrevious => Some("focus-window-previous"),
        Action::FocusColumnFirst => Some("focus-column-first"),
        Action::FocusColumnLast => Some("focus-column-last"),
        Action::MoveColumnLeft => Some("move-column-left"),
        Action::MoveColumnRight => Some("move-column-right"),
        Action::MoveWindowUp => Some("move-window-up"),
        Action::MoveWindowDown => Some("move-window-down"),
        Action::ConsumeOrExpelWindowLeft => Some("consume-or-expel-window-left"),
        Action::ConsumeOrExpelWindowRight => Some("consume-or-expel-window-right"),
        Action::ConsumeWindowIntoColumn => Some("consume-window-into-column"),
        Action::ExpelWindowFromColumn => Some("expel-window-from-column"),
        Action::SwapWindowLeft => Some("swap-window-left"),
        Action::SwapWindowRight => Some("swap-window-right"),
        Action::ToggleColumnTabbedDisplay => Some("toggle-column-tabbed-display"),
        Action::SetColumnDisplay(..) => Some("set-column-display"),
        Action::CenterColumn => Some("center-column"),
        Action::CenterWindow => Some("center-window"),
        Action::FocusWorkspaceDown => Some("focus-workspace-down"),
        Action::FocusWorkspaceUp => Some("focus-workspace-up"),
        Action::FocusWorkspacePrevious => Some("focus-workspace-previous"),
        Action::FocusWorkspace(..) => Some("focus-workspace"),
        Action::MoveWindowToWorkspaceDown(..) => Some("move-window-to-workspace-down"),
        Action::MoveWindowToWorkspaceUp(..) => Some("move-window-to-workspace-up"),
        Action::MoveWindowToWorkspace(..) => Some("move-window-to-workspace"),
        Action::MoveColumnToWorkspaceDown(..) => Some("move-column-to-workspace-down"),
        Action::MoveColumnToWorkspaceUp(..) => Some("move-column-to-workspace-up"),
        Action::MoveWorkspaceDown => Some("move-workspace-down"),
        Action::MoveWorkspaceUp => Some("move-workspace-up"),
        Action::SetWorkspaceName(..) => Some("set-workspace-name"),
        Action::FocusMonitorLeft => Some("focus-monitor-left"),
        Action::FocusMonitorRight => Some("focus-monitor-right"),
        Action::FocusMonitorUp => Some("focus-monitor-up"),
        Action::FocusMonitorDown => Some("focus-monitor-down"),
        Action::FocusMonitorPrevious => Some("focus-monitor-previous"),
        Action::Screenshot(..) => Some("screenshot"),
        Action::ScreenshotScreen(..) => Some("screenshot-screen"),
        Action::ScreenshotWindow(..) => Some("screenshot-window"),
        Action::ToggleDebugTint => Some("toggle-debug-tint"),
        Action::DebugToggleOpaqueRegions => Some("debug-toggle-opaque-regions"),
        Action::DebugToggleDamage => Some("debug-toggle-damage"),
        Action::ToggleActionPalette => Some("toggle-action-palette"),
        Action::SetAnimationProfile(..) => Some("set-animation-profile"),
        Action::ToggleScratchColumn(..) => Some("toggle-scratch-column"),
        Action::SetMaterial(..) => Some("set-material"),
        Action::ToggleSafeMode => Some("toggle-safe-mode"),
        Action::LinkEnable => Some("link-enable"),
        Action::LinkDisable => Some("link-disable"),
        Action::LinkToggle => Some("link-toggle"),
        Action::LinkJoin(..) => Some("link-join"),
        Action::LinkLeave => Some("link-leave"),
        Action::LinkPair(..) => Some("link-pair"),
        Action::LinkUnpair(..) => Some("link-unpair"),
        Action::LinkTrustNode(..) => Some("link-trust-node"),
        Action::LinkForgetNode(..) => Some("link-forget-node"),
        Action::LinkRestoreSession(..) => Some("link-restore-session"),
        Action::LinkSetLeader(..) => Some("link-set-leader"),
        Action::LinkStatus => Some("link-status"),
        _ => None,
    }
}

/// Map a registry id string back to an Action variant (for static actions).
fn registry_id_to_action(id: &str) -> Option<Action> {
    match id {
        "quit" => Some(Action::Quit(false)),
        "suspend" => Some(Action::Suspend),
        "power-off-monitors" => Some(Action::PowerOffMonitors),
        "power-on-monitors" => Some(Action::PowerOnMonitors),
        "close-window" => Some(Action::CloseWindow),
        "fullscreen-window" => Some(Action::FullscreenWindow),
        "toggle-windowed-fullscreen" => Some(Action::ToggleWindowedFullscreen),
        "center-column" => Some(Action::CenterColumn),
        "center-window" => Some(Action::CenterWindow),
        "focus-column-left" => Some(Action::FocusColumnLeft),
        "focus-column-right" => Some(Action::FocusColumnRight),
        "toggle-column-tabbed-display" => Some(Action::ToggleColumnTabbedDisplay),
        "toggle-action-palette" => Some(Action::ToggleActionPalette),
        "toggle-safe-mode" => Some(Action::ToggleSafeMode),
        "link-enable" => Some(Action::LinkEnable),
        "link-disable" => Some(Action::LinkDisable),
        "link-toggle" => Some(Action::LinkToggle),
        "link-leave" => Some(Action::LinkLeave),
        "link-status" => Some(Action::LinkStatus),
        _ => None,
    }
}

fn strip_markup(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_tag = false;

    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => (),
        }
    }

    out
}

fn format_bind_key(mod_key: ModKey, key: &Key) -> String {
    let mut name = String::new();
    let has_comp_mod = key.modifiers.contains(Modifiers::COMPOSITOR);

    if has_comp_mod {
        match mod_key {
            ModKey::Super => name.push_str("Super+"),
            ModKey::Alt => name.push_str("Alt+"),
            ModKey::Shift => name.push_str("Shift+"),
            ModKey::Ctrl => name.push_str("Ctrl+"),
            ModKey::IsoLevel3Shift => name.push_str("Mod5+"),
            ModKey::IsoLevel5Shift => name.push_str("Mod3+"),
        }
    }

    if key.modifiers.contains(Modifiers::SHIFT) && (!has_comp_mod || mod_key != ModKey::Shift) {
        name.push_str("Shift+");
    }
    if key.modifiers.contains(Modifiers::CTRL) && (!has_comp_mod || mod_key != ModKey::Ctrl) {
        name.push_str("Ctrl+");
    }
    if key.modifiers.contains(Modifiers::ALT) && (!has_comp_mod || mod_key != ModKey::Alt) {
        name.push_str("Alt+");
    }
    if key.modifiers.contains(Modifiers::SUPER) && (!has_comp_mod || mod_key != ModKey::Super) {
        name.push_str("Super+");
    }

    match &key.trigger {
        niri_config::Trigger::Keysym(keysym) => {
            let keysym_name = keysym_get_name(*keysym);
            name.push_str(&keysym_name);
        }
        _ => name.push_str("?"),
    }

    name
}

fn render_palette_box(
    scale: f64,
    query: &str,
    actions: &[ActionItem],
    selected_idx: usize,
    config: &Config,
) -> anyhow::Result<MemoryBuffer> {
    let padding: i32 = to_physical_precise_round(scale, 16.0);
    let border: i32 = to_physical_precise_round(scale, 4.0);

    let width: i32 = to_physical_precise_round(scale, 560.0);
    let height: i32 = to_physical_precise_round(scale, 360.0);

    let mut font = FontDescription::from_string("sans 13px");
    let font_size_phys: i32 = to_physical_precise_round(scale, font.size());
    font.set_absolute_size(font_size_phys as f64);

    let mut font_bold = FontDescription::from_string("sans bold 13px");
    let font_bold_size_phys: i32 = to_physical_precise_round(scale, font_bold.size());
    font_bold.set_absolute_size(font_bold_size_phys as f64);

    let surface = ImageSurface::create(cairo::Format::ARgb32, width, height)?;
    let cr = cairo::Context::new(&surface)?;

    // Paint semi-transparent dark overlay for high contrast
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.28);
    cr.paint()?;

    // Draw Search Input Prompt
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.move_to(padding.into(), padding.into());
    let prompt_layout = pangocairo::functions::create_layout(&cr);
    prompt_layout.set_font_description(Some(&font_bold));
    prompt_layout.set_text(&format!("> {}", query));
    pangocairo::functions::show_layout(&cr, &prompt_layout);

    let prompt_height = prompt_layout.pixel_size().1;
    let mut separator_y = padding + prompt_height + padding / 2;

    if config.action_palette.show_current_state {
        let status_line = format!("Profile: {}", config.resolved_animation_profile_label());

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.55);
        cr.move_to(padding.into(), (separator_y + 4).into());
        let status_layout = pangocairo::functions::create_layout(&cr);
        status_layout.set_font_description(Some(&font));
        status_layout.set_text(&status_line);
        pangocairo::functions::show_layout(&cr, &status_layout);

        separator_y += status_layout.pixel_size().1 + padding / 2;
    }

    // Separator line
    cr.move_to(0.0, separator_y.into());
    cr.line_to(width.into(), separator_y.into());
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.16);
    cr.set_line_width(1.0 * scale);
    cr.stroke()?;

    // Draw filtered action items
    let max_visible_items = 8;
    let mut current_y = separator_y + padding / 2;
    let item_height: i32 = to_physical_precise_round(scale, 28.0);

    for (idx, item) in actions.iter().take(max_visible_items).enumerate() {
        let is_selected = idx == selected_idx;

        if is_selected {
            // Draw highlight bar
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.10);
            cr.rectangle(
                padding.into(),
                current_y.into(),
                (width - padding * 2).into(),
                item_height.into(),
            );
            cr.fill()?;
        }

        // Draw item display name
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.move_to((padding + 8).into(), (current_y + 4).into());
        let item_layout = pangocairo::functions::create_layout(&cr);
        item_layout.set_font_description(Some(&font));
        let display_text = if is_selected {
            format!("▶  {}", item.display_name)
        } else {
            format!("    {}", item.display_name)
        };
        item_layout.set_text(&display_text);
        pangocairo::functions::show_layout(&cr, &item_layout);

        // Draw keybind hint if enabled and exists
        if config.action_palette.show_keybinds {
            if let Some(hint) = &item.keybind_hint {
                let hint_layout = pangocairo::functions::create_layout(&cr);
                hint_layout.set_font_description(Some(&font));
                hint_layout.set_text(hint);
                let hint_width = hint_layout.pixel_size().0;

                cr.set_source_rgba(1.0, 1.0, 1.0, 0.45);
                cr.move_to(
                    (width - padding - 8 - hint_width).into(),
                    (current_y + 4).into(),
                );
                pangocairo::functions::show_layout(&cr, &hint_layout);
            }
        }

        current_y += item_height;
    }

    if actions.is_empty() {
        cr.set_source_rgba(1.0, 1.0, 1.0, 0.4);
        cr.move_to((padding + 8).into(), (separator_y + padding).into());
        let empty_layout = pangocairo::functions::create_layout(&cr);
        empty_layout.set_font_description(Some(&font));
        empty_layout.set_text("No matching actions found");
        pangocairo::functions::show_layout(&cr, &empty_layout);
    }

    // Outer border stroke
    cr.move_to(0., 0.);
    cr.line_to(width.into(), 0.);
    cr.line_to(width.into(), height.into());
    cr.line_to(0., height.into());
    cr.line_to(0., 0.);
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.18);
    cr.set_line_width(border.into());
    cr.stroke()?;
    drop(cr);

    let data = surface.take_data().unwrap();
    let buffer = MemoryBuffer::new(
        data.to_vec(),
        Fourcc::Argb8888,
        (width, height),
        scale,
        Transform::Normal,
    );

    Ok(buffer)
}
