use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use niri_config::{Config, CornerRadius};
use ordered_float::NotNan;
use pangocairo::cairo::{self, ImageSurface};
use pangocairo::pango::{Alignment, FontDescription};
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::gles::GlesTexture;
use smithay::output::Output;
use smithay::reexports::gbm::Format as Fourcc;
use smithay::utils::{Point, Rectangle, Transform};

use crate::animation::{Animation, Clock};
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
use crate::utils::{output_size, to_physical_precise_round};

niri_render_elements! {
    ModeHudRenderElement => {
        Texture = PrimaryGpuTextureRenderElement,
        BackgroundEffect = BackgroundEffectElement,
    }
}

struct RenderedHud {
    texture: TextureBuffer<GlesTexture>,
    background_effect: BackgroundEffect,
}

pub struct ModeHud {
    state: State,
    text: String,
    buffers: RefCell<HashMap<NotNan<f64>, Option<RenderedHud>>>,
    clock: Clock,
    config: Rc<RefCell<Config>>,
}

enum State {
    Hidden,
    Showing(Animation),
    Shown(Instant),
    Hiding(Animation),
}

impl ModeHud {
    pub fn new(clock: Clock, config: Rc<RefCell<Config>>) -> Self {
        Self {
            state: State::Hidden,
            text: String::new(),
            buffers: RefCell::new(HashMap::new()),
            clock,
            config,
        }
    }

    fn animation(&self, from: f64, to: f64) -> Animation {
        let c = self.config.borrow();
        Animation::new(
            self.clock.clone(),
            from,
            to,
            0.,
            c.animations.config_notification_open_close.0, /* Reuse config notification
                                                            * animation settings */
        )
    }

    pub fn trigger(&mut self, text: String) {
        let config = self.config.borrow();
        if !config.mode_hud.enable {
            return;
        }

        self.text = text;
        self.buffers.borrow_mut().clear();
        self.state = State::Showing(self.animation(0., 1.));
    }

    pub fn hide(&mut self) {
        if matches!(self.state, State::Hidden) {
            return;
        }

        let current_val = match &self.state {
            State::Hidden => 0.,
            State::Showing(anim) | State::Hiding(anim) => anim.value(),
            State::Shown(_) => 1.,
        };

        self.state = State::Hiding(self.animation(current_val, 0.));
    }

    pub fn advance_animations(&mut self) {
        match &mut self.state {
            State::Hidden => (),
            State::Showing(anim) => {
                if anim.is_done() {
                    let duration = {
                        let config = self.config.borrow();
                        Duration::from_millis(config.mode_hud.duration_ms)
                    };
                    self.state = State::Shown(Instant::now() + duration);
                }
            }
            State::Shown(deadline) => {
                if Instant::now() >= *deadline {
                    self.hide();
                }
            }
            State::Hiding(anim) => {
                if anim.is_clamped_done() {
                    self.state = State::Hidden;
                }
            }
        }
    }

    pub fn are_animations_ongoing(&self) -> bool {
        !matches!(self.state, State::Hidden)
    }

    pub fn render<R: NiriRenderer>(
        &self,
        mut ctx: RenderCtx<R>,
        output: &Output,
        push: &mut dyn FnMut(ModeHudRenderElement),
    ) -> Option<()> {
        if matches!(self.state, State::Hidden) {
            return None;
        }

        let scale = output.current_scale().fractional_scale();
        let output_size = output_size(output);

        let mut buffers = self.buffers.borrow_mut();
        let key = NotNan::new(scale).unwrap();

        let has_cached = buffers.get(&key).is_some();
        if !has_cached {
            let buffer = match render_hud_box(scale, &self.text) {
                Ok(buf) => {
                    if let Ok(tex) =
                        TextureBuffer::from_memory_buffer(ctx.renderer.as_gles_renderer(), &buf)
                    {
                        Some(RenderedHud {
                            texture: tex,
                            background_effect: BackgroundEffect::new(),
                        })
                    } else {
                        None
                    }
                }
                Err(err) => {
                    warn!("Failed to render ModeHUD texture: {:?}", err);
                    None
                }
            };
            buffers.insert(key.clone(), buffer);
        }

        let rendered = buffers.get_mut(&key).and_then(|x| x.as_mut())?;

        let size = rendered.texture.logical_size();
        let config = self.config.borrow();

        let margin = 24.0;
        let x = match config.mode_hud.position.as_str() {
            "top-left" | "bottom-left" => margin,
            "top-right" | "bottom-right" => output_size.w as f64 - size.w - margin,
            _ => (output_size.w as f64 - size.w) / 2.0, // default center
        };
        let y = match config.mode_hud.position.as_str() {
            "bottom-left" | "bottom-right" | "bottom-center" => {
                output_size.h as f64 - size.h - margin
            }
            "center" => (output_size.h as f64 - size.h) / 2.0,
            _ => margin, // default top
        };
        let rect = Rectangle::new(Point::new(x, y), size);

        // Resolve material configurations
        let material = config
            .materials
            .iter()
            .find(|m| m.name == config.mode_hud.material);
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
            effect.refraction = Some(0.010);
            effect.specular = Some(0.08);
            effect.edge_highlight = Some(0.05);

            blur_config.passes = 4;
            blur_config.offset = 5.0;
        }

        rendered.background_effect.update_config(blur_config);

        let corner_radius_val: f32 = to_physical_precise_round(scale, 10.0);
        let corner_radius = CornerRadius::from(corner_radius_val);
        rendered
            .background_effect
            .update_render_elements(corner_radius, effect, false);

        let params = RenderParams {
            geometry: rect,
            subregion: None,
            clip: Some((rect, corner_radius)),
            scale,
        };

        let clamped_value = match &self.state {
            State::Hidden => return None,
            State::Showing(anim) | State::Hiding(anim) => anim.clamped_value() as f32,
            State::Shown(_) => 1.0,
        };

        let xray_pos = XrayPos::default();

        // Render glass background
        rendered
            .background_effect
            .render(ctx.r().as_gles(), None, params, xray_pos, &mut |elem| {
                push(ModeHudRenderElement::BackgroundEffect(elem))
            });

        // Render foreground texture
        let elem = TextureRenderElement::from_texture_buffer(
            rendered.texture.clone(),
            rect.loc,
            clamped_value,
            None,
            None,
            Kind::Unspecified,
        );
        let elem = PrimaryGpuTextureRenderElement(elem);
        push(ModeHudRenderElement::Texture(elem));

        Some(())
    }
}

fn render_hud_box(scale: f64, text: &str) -> anyhow::Result<MemoryBuffer> {
    let padding_x: i32 = to_physical_precise_round(scale, 20.0);
    let padding_y: i32 = to_physical_precise_round(scale, 12.0);

    let mut font = FontDescription::from_string("sans bold 12px");
    let font_size: i32 = to_physical_precise_round(scale, font.size());
    font.set_absolute_size(font_size as f64);

    let surface = ImageSurface::create(cairo::Format::ARgb32, 0, 0)?;
    let cr = cairo::Context::new(&surface)?;
    let layout = pangocairo::functions::create_layout(&cr);
    layout.context().set_round_glyph_positions(false);
    layout.set_font_description(Some(&font));
    layout.set_alignment(Alignment::Center);
    layout.set_text(text);

    let (mut width, mut height) = layout.pixel_size();
    width += padding_x * 2;
    height += padding_y * 2;

    let surface = ImageSurface::create(cairo::Format::ARgb32, width, height)?;
    let cr = cairo::Context::new(&surface)?;

    // Draw background overlay for legibility
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.24);
    cr.paint()?;

    cr.move_to(padding_x.into(), padding_y.into());
    let layout = pangocairo::functions::create_layout(&cr);
    layout.context().set_round_glyph_positions(false);
    layout.set_font_description(Some(&font));
    layout.set_alignment(Alignment::Center);
    layout.set_text(text);

    cr.set_source_rgb(1.0, 1.0, 1.0);
    pangocairo::functions::show_layout(&cr, &layout);

    // Subtle edge boundary
    cr.move_to(0., 0.);
    cr.line_to(width.into(), 0.);
    cr.line_to(width.into(), height.into());
    cr.line_to(0., height.into());
    cr.line_to(0., 0.);
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.16);
    cr.set_line_width(2.0 * scale);
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
