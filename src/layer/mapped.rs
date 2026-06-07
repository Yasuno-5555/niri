use niri_config::utils::MergeWith as _;
use niri_config::{Config, LayerRule};
use smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement;
use smithay::backend::renderer::element::Kind;
use smithay::desktop::{LayerSurface, PopupKind, PopupManager};
use smithay::utils::{Logical, Point, Rectangle, Scale, Size};
use smithay::wayland::compositor::{remove_pre_commit_hook, HookId};
use smithay::wayland::shell::wlr_layer::{ExclusiveZone, Layer};

use super::ResolvedLayerRules;
use crate::animation::{Animation, Clock};
use crate::layout::shadow::Shadow;
use crate::niri_render_elements;
use crate::render_helpers::background_effect::BackgroundEffectElement;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::shadow::ShadowRenderElement;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};
use crate::render_helpers::surface::push_elements_from_surface_tree;
use crate::render_helpers::xray::XrayPos;
use crate::render_helpers::{background_effect, RenderCtx};
use crate::utils::{baba_is_float_offset, round_logical_in_physical};

#[derive(Debug)]
pub struct MappedLayer {
    /// The surface itself.
    surface: LayerSurface,

    /// Pre-commit hook that we have on all mapped layer surfaces.
    pre_commit_hook: HookId,

    /// Up-to-date rules.
    rules: ResolvedLayerRules,

    /// Whether to recompute layer rules on the next commit.
    ///
    /// Set in the pre-commit hook when the layer changes; consumed in the commit handler.
    recompute_rules_on_commit: bool,

    /// Buffer to draw instead of the surface when it should be blocked out.
    block_out_buffer: SolidColorBuffer,

    /// The shadow around the surface.
    shadow: Shadow,

    /// The blur config, passed for background effect rendering.
    blur_config: niri_config::Blur,

    /// The view size for the layer surface's output.
    view_size: Size<f64, Logical>,

    /// Scale of the output the layer surface is on (and rounds its sizes to).
    scale: f64,

    /// Clock for driving animations.
    clock: Clock,

    /// Config used for geometry interpolation.
    move_animation_config: niri_config::Animation,

    /// The last arranged layer geometry location.
    last_arranged_location: Option<Point<f64, Logical>>,

    /// Ongoing X-axis geometry interpolation.
    move_x_animation: Option<MoveAnimation>,

    /// Ongoing Y-axis geometry interpolation.
    move_y_animation: Option<MoveAnimation>,

    resolved_animation_open: Option<niri_config::layer_rule::LayerAnimationRule>,
    open_animation: Option<Animation>,
}

#[derive(Debug)]
struct MoveAnimation {
    anim: Animation,
    from: f64,
}

niri_render_elements! {
    LayerSurfaceRenderElement<R> => {
        Wayland = WaylandSurfaceRenderElement<R>,
        SolidColor = SolidColorRenderElement,
        Shadow = ShadowRenderElement,
        BackgroundEffect = BackgroundEffectElement,
    }
}

impl MappedLayer {
    pub fn new(
        surface: LayerSurface,
        pre_commit_hook: HookId,
        rules: ResolvedLayerRules,
        view_size: Size<f64, Logical>,
        scale: f64,
        clock: Clock,
        config: &Config,
    ) -> Self {
        let mut shadow_config = config.layout.shadow;
        // Shadows for layer surfaces need to be explicitly enabled.
        shadow_config.on = false;
        shadow_config.merge_with(&rules.shadow);

        let resolved_animation_open = rules.animation_open.clone().or_else(|| {
            config
                .resolved_animation_profile()
                .and_then(|profile| profile.layer_open.as_ref())
                .map(|preset_name| {
                    let (style, from_scale, direction, duration_ms, curve) =
                        match preset_name.as_str() {
                            "scale-fade" => (
                                Some("scale-fade".to_owned()),
                                Some(niri_config::FloatOrInt(0.95)),
                                None,
                                Some(140),
                                Some("ease-out-expo".to_owned()),
                            ),
                            "fade-slide" => (
                                Some("fade-slide".to_owned()),
                                None,
                                Some("down".to_owned()),
                                Some(100),
                                None,
                            ),
                            "slide-glass" => (
                                Some("slide-glass".to_owned()),
                                None,
                                Some("down".to_owned()),
                                Some(150),
                                None,
                            ),
                            "elastic-glass" => (
                                Some("scale-fade".to_owned()),
                                Some(niri_config::FloatOrInt(0.90)),
                                None,
                                Some(200),
                                None,
                            ),
                            "glass-slide" => (
                                Some("fade-slide".to_owned()),
                                None,
                                Some("bottom".to_owned()),
                                Some(160),
                                None,
                            ),
                            _ => (Some("fade".to_owned()), None, None, Some(120), None),
                        };
                    niri_config::layer_rule::LayerAnimationRule {
                        style,
                        from_scale,
                        direction,
                        duration_ms,
                        curve,
                    }
                })
        });

        let mut open_animation = None;
        if let Some(rule) = &resolved_animation_open {
            let kind = if let Some(curve_str) = &rule.curve {
                let curve = match curve_str.as_str() {
                    "linear" => niri_config::animations::Curve::Linear,
                    "ease-out-quad" => niri_config::animations::Curve::EaseOutQuad,
                    "ease-out-cubic" => niri_config::animations::Curve::EaseOutCubic,
                    "ease-out-expo" => niri_config::animations::Curve::EaseOutExpo,
                    _ => niri_config::animations::Curve::EaseOutCubic,
                };
                niri_config::animations::Kind::Easing(niri_config::animations::EasingParams {
                    duration_ms: rule.duration_ms.unwrap_or(150),
                    curve,
                })
            } else {
                niri_config::animations::Kind::Spring(niri_config::animations::SpringParams {
                    damping_ratio: 0.75,
                    stiffness: 900,
                    epsilon: 0.0001,
                })
            };
            let anim_config = niri_config::animations::Animation { off: false, kind };
            open_animation = Some(Animation::new(clock.clone(), 0.0, 1.0, 0.0, anim_config));
        }

        Self {
            surface,
            pre_commit_hook,
            rules,
            recompute_rules_on_commit: false,
            block_out_buffer: SolidColorBuffer::new((0., 0.), [0., 0., 0., 1.]),
            view_size,
            scale,
            shadow: Shadow::new(shadow_config),
            blur_config: config.blur,
            clock,
            move_animation_config: config.animations.window_movement.0,
            last_arranged_location: None,
            move_x_animation: None,
            move_y_animation: None,
            resolved_animation_open,
            open_animation,
        }
    }

    pub fn update_config(&mut self, config: &Config) {
        let mut shadow_config = config.layout.shadow;
        // Shadows for layer surfaces need to be explicitly enabled.
        shadow_config.on = false;
        shadow_config.merge_with(&self.rules.shadow);
        self.shadow.update_config(shadow_config);

        self.blur_config = config.blur;
        self.move_animation_config = config.animations.window_movement.0;

        self.resolved_animation_open = self.rules.animation_open.clone().or_else(|| {
            config
                .resolved_animation_profile()
                .and_then(|profile| profile.layer_open.as_ref())
                .map(|preset_name| {
                    let (style, from_scale, direction, duration_ms, curve) =
                        match preset_name.as_str() {
                            "scale-fade" => (
                                Some("scale-fade".to_owned()),
                                Some(niri_config::FloatOrInt(0.95)),
                                None,
                                Some(140),
                                Some("ease-out-expo".to_owned()),
                            ),
                            "fade-slide" => (
                                Some("fade-slide".to_owned()),
                                None,
                                Some("down".to_owned()),
                                Some(100),
                                None,
                            ),
                            "slide-glass" => (
                                Some("slide-glass".to_owned()),
                                None,
                                Some("down".to_owned()),
                                Some(150),
                                None,
                            ),
                            "elastic-glass" => (
                                Some("scale-fade".to_owned()),
                                Some(niri_config::FloatOrInt(0.90)),
                                None,
                                Some(200),
                                None,
                            ),
                            "glass-slide" => (
                                Some("fade-slide".to_owned()),
                                None,
                                Some("bottom".to_owned()),
                                Some(160),
                                None,
                            ),
                            _ => (Some("fade".to_owned()), None, None, Some(120), None),
                        };
                    niri_config::layer_rule::LayerAnimationRule {
                        style,
                        from_scale,
                        direction,
                        duration_ms,
                        curve,
                    }
                })
        });
    }

    pub fn update_shaders(&mut self) {
        self.shadow.update_shaders();
    }

    pub fn update_sizes(&mut self, view_size: Size<f64, Logical>, scale: f64) {
        self.view_size = view_size;
        self.scale = scale;
    }

    pub fn update_render_elements(&mut self, geometry: Rectangle<f64, Logical>) {
        self.update_geometry_animation(geometry.loc);

        // Round to physical pixels.
        let size = geometry
            .size
            .to_physical_precise_round(self.scale)
            .to_logical(self.scale);

        self.block_out_buffer.resize(size);

        let radius = self.rules.geometry_corner_radius.unwrap_or_default();
        // FIXME: is_active based on keyboard focus?
        self.shadow
            .update_render_elements(size, true, radius, self.scale, 1.);
    }

    pub fn are_animations_ongoing(&self) -> bool {
        self.rules.baba_is_float
            || self.move_x_animation.is_some()
            || self.move_y_animation.is_some()
            || self.open_animation.is_some()
    }

    pub fn surface(&self) -> &LayerSurface {
        &self.surface
    }

    pub fn rules(&self) -> &ResolvedLayerRules {
        &self.rules
    }

    /// Recomputes the resolved layer rules and returns whether they changed.
    pub fn recompute_layer_rules(
        &mut self,
        rules: &[LayerRule],
        is_at_startup: bool,
        presets: &[niri_config::EffectPreset],
        materials: &[niri_config::Material],
    ) -> bool {
        let new_rules =
            ResolvedLayerRules::compute(rules, &self.surface, is_at_startup, presets, materials);
        if new_rules == self.rules {
            return false;
        }

        self.rules = new_rules;
        true
    }

    pub fn set_recompute_rules_on_commit(&mut self) {
        self.recompute_rules_on_commit = true;
    }

    pub fn take_recompute_rules_on_commit(&mut self) -> bool {
        std::mem::take(&mut self.recompute_rules_on_commit)
    }

    pub fn place_within_backdrop(&self) -> bool {
        if !self.rules.place_within_backdrop {
            return false;
        }

        if self.surface.layer() != Layer::Background {
            return false;
        }

        let state = self.surface.cached_state();
        if state.exclusive_zone != ExclusiveZone::DontCare {
            return false;
        }

        true
    }

    pub fn bob_offset(&self) -> Point<f64, Logical> {
        if !self.rules.baba_is_float {
            return Point::from((0., 0.));
        }

        let y = baba_is_float_offset(self.clock.now(), self.view_size.h);
        let y = round_logical_in_physical(self.scale, y);
        Point::from((0., y))
    }

    pub fn advance_animations(&mut self) {
        if let Some(move_) = &self.move_x_animation {
            if move_.anim.is_done() {
                self.move_x_animation = None;
            }
        }
        if let Some(move_) = &self.move_y_animation {
            if move_.anim.is_done() {
                self.move_y_animation = None;
            }
        }
        if let Some(open) = &self.open_animation {
            if open.is_done() {
                self.open_animation = None;
            }
        }
    }

    fn render_offset(&self) -> Point<f64, Logical> {
        let mut offset = Point::from((0., 0.));

        if let Some(move_) = &self.move_x_animation {
            offset.x += move_.from * move_.anim.value();
        }
        if let Some(move_) = &self.move_y_animation {
            offset.y += move_.from * move_.anim.value();
        }

        offset
    }

    fn update_geometry_animation(&mut self, location: Point<f64, Logical>) {
        let Some(previous) = self.last_arranged_location.replace(location) else {
            return;
        };

        let delta = previous - location;

        if delta.x.abs() > f64::EPSILON {
            self.animate_move_x_from(delta.x);
        }
        if delta.y.abs() > f64::EPSILON {
            self.animate_move_y_from(delta.y);
        }
    }

    fn animate_move_x_from(&mut self, from: f64) {
        let current_offset = self.render_offset().x;

        let anim = self.move_x_animation.take().map(|move_| move_.anim);
        let anim = anim
            .map(|anim| anim.restarted(1., 0., 0.))
            .unwrap_or_else(|| {
                Animation::new(self.clock.clone(), 1., 0., 0., self.move_animation_config)
            });

        self.move_x_animation = Some(MoveAnimation {
            anim,
            from: from + current_offset,
        });
    }

    fn animate_move_y_from(&mut self, from: f64) {
        let current_offset = self.render_offset().y;

        let anim = self.move_y_animation.take().map(|move_| move_.anim);
        let anim = anim
            .map(|anim| anim.restarted(1., 0., 0.))
            .unwrap_or_else(|| {
                Animation::new(self.clock.clone(), 1., 0., 0., self.move_animation_config)
            });

        self.move_y_animation = Some(MoveAnimation {
            anim,
            from: from + current_offset,
        });
    }

    pub fn render_normal<R: NiriRenderer>(
        &self,
        mut ctx: RenderCtx<R>,
        ns: Option<usize>,
        location: Point<f64, Logical>,
        xray_pos: XrayPos,
        push: &mut dyn FnMut(LayerSurfaceRenderElement<R>),
    ) {
        let scale = Scale::from(self.scale);
        let mut alpha = self.rules.opacity.unwrap_or(1.).clamp(0., 1.);

        let mut open_scale = 1.0f64;
        let mut open_offset = Point::new(0.0, 0.0);

        if let Some(open) = &self.open_animation {
            let progress = open.value();
            if let Some(rule) = &self.resolved_animation_open {
                let style = rule.style.as_deref().unwrap_or("fade");

                if style.contains("fade") {
                    alpha *= progress as f32;
                }

                if style.contains("scale") {
                    let from = rule.from_scale.map(|s| s.0).unwrap_or(0.95);
                    open_scale = from + (1.0 - from) * progress;
                }

                if style.contains("slide") {
                    let dir = rule.direction.as_deref().unwrap_or("down");
                    let slide_dist = 50.0;
                    let offset_val = slide_dist * (1.0 - progress);
                    match dir {
                        "down" => open_offset.y += offset_val,
                        "up" => open_offset.y -= offset_val,
                        "left" => open_offset.x -= offset_val,
                        "right" => open_offset.x += offset_val,
                        "bottom" => open_offset.y += offset_val,
                        "top" => open_offset.y -= offset_val,
                        _ => {}
                    }
                }
            }
        }

        let render_offset = self.render_offset() + self.bob_offset() + open_offset;
        let mut location = location + render_offset;
        let xray_pos = xray_pos.offset(render_offset);

        if open_scale != 1.0 {
            let size = self.block_out_buffer.size();
            location.x += size.w * (1.0 - open_scale) / 2.0;
            location.y += size.h * (1.0 - open_scale) / 2.0;
        }

        let surface = self.surface.wl_surface();

        let should_block_out = ctx.target.should_block_out(self.rules.block_out_from);
        if should_block_out {
            // Round to physical pixels.
            let location = location.to_physical_precise_round(scale).to_logical(scale);

            // FIXME: take geometry-corner-radius into account.
            let elem = SolidColorRenderElement::from_buffer(
                &self.block_out_buffer,
                location,
                alpha,
                Kind::Unspecified,
            );
            push(elem.into());
        } else {
            // Layer surfaces don't have extra geometry like windows.
            let buf_pos = location;
            let element_scale = scale * Scale::from(open_scale);

            push_elements_from_surface_tree(
                ctx.renderer,
                surface,
                buf_pos.to_physical_precise_round(scale),
                element_scale,
                alpha,
                Kind::ScanoutCandidate,
                &mut |elem| push(elem.into()),
            );
        }

        let location = location.to_physical_precise_round(scale).to_logical(scale);
        self.shadow
            .render(ctx.renderer, location, &mut |elem| push(elem.into()));

        let geometry = Rectangle::new(location, self.block_out_buffer.size());
        let surface_off = Point::new(0., 0.); // No geometry on layer surfaces.
        let surface_anim_scale = Scale::from(open_scale);
        let radius = self.rules.geometry_corner_radius.unwrap_or_default();
        background_effect::render_for_tile(
            ctx.as_gles(),
            ns,
            geometry,
            self.scale,
            false,
            surface,
            surface_off,
            surface_anim_scale,
            self.blur_config,
            radius,
            self.rules.background_effect,
            should_block_out,
            xray_pos,
            &mut |elem| push(elem.into()),
        );
    }

    pub fn render_popups<R: NiriRenderer>(
        &self,
        mut ctx: RenderCtx<R>,
        ns: Option<usize>,
        location: Point<f64, Logical>,
        xray_pos: XrayPos,
        push: &mut dyn FnMut(LayerSurfaceRenderElement<R>),
    ) {
        if ctx.target.should_block_out(self.rules.block_out_from) {
            return;
        }

        let scale = Scale::from(self.scale);
        let alpha = self.rules.opacity.unwrap_or(1.).clamp(0., 1.);

        let render_offset = self.render_offset() + self.bob_offset();
        let location = location + render_offset;
        let xray_pos = xray_pos.offset(render_offset);

        let surface = self.surface.wl_surface();
        for (popup, offset) in PopupManager::popups_for_surface(surface) {
            let popup_rules = match popup {
                PopupKind::Xdg(_) => self.rules.popups,
                // IME popups aren't affected by rules for regular popups.
                PopupKind::InputMethod(_) => niri_config::ResolvedPopupsRules::default(),
            };
            let alpha = alpha * popup_rules.opacity.unwrap_or(1.).clamp(0., 1.);

            let surface = popup.wl_surface();
            let popup_geo = popup.geometry();
            let surface_loc = location + (offset - popup_geo.loc).to_f64();

            push_elements_from_surface_tree(
                ctx.renderer,
                surface,
                surface_loc.to_physical_precise_round(scale),
                scale,
                alpha,
                Kind::ScanoutCandidate,
                &mut |elem| push(elem.into()),
            );

            let geometry = Rectangle::new(location + offset.to_f64(), popup_geo.size.to_f64());
            let surface_off = popup_geo.loc.upscale(-1).to_f64();
            let surface_anim_scale = Scale::from(1.);
            let mut effect = popup_rules.background_effect;
            // Default xray to false for pop-ups since they're always on top of something.
            if effect.xray.is_none() {
                effect.xray = Some(false);
            }
            let xray_pos = xray_pos.offset(offset.to_f64());
            background_effect::render_for_tile(
                ctx.as_gles(),
                ns,
                geometry,
                self.scale,
                false,
                surface,
                surface_off,
                surface_anim_scale,
                self.blur_config,
                popup_rules.geometry_corner_radius.unwrap_or_default(),
                effect,
                false,
                xray_pos,
                &mut |elem| push(elem.into()),
            );
        }
    }
}

impl Drop for MappedLayer {
    fn drop(&mut self) {
        remove_pre_commit_hook(self.surface.wl_surface(), &self.pre_commit_hook);
    }
}
