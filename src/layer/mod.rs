use niri_config::layer_rule::{LayerAnimationRule, LayerRule, Match};
use niri_config::utils::MergeWith as _;
use niri_config::{BackgroundEffect, BlockOutFrom, CornerRadius, ResolvedPopupsRules, ShadowRule};
use smithay::desktop::LayerSurface;
use smithay::wayland::shell::wlr_layer::Layer;

pub mod mapped;
pub use mapped::MappedLayer;

/// Rules fully resolved for a layer-shell surface.
#[derive(Debug, Default, PartialEq)]
pub struct ResolvedLayerRules {
    /// Extra opacity to draw this layer surface with.
    pub opacity: Option<f32>,

    /// Whether to block out this layer surface from certain render targets.
    pub block_out_from: Option<BlockOutFrom>,

    /// Shadow overrides.
    pub shadow: ShadowRule,

    /// Corner radius to assume this layer surface has.
    pub geometry_corner_radius: Option<CornerRadius>,

    /// Whether to place this layer surface within the overview backdrop.
    pub place_within_backdrop: bool,

    /// Whether to bob this window up and down.
    pub baba_is_float: bool,

    /// Background effect configuration.
    pub background_effect: BackgroundEffect,

    /// Rules for this layer surface's popups.
    pub popups: ResolvedPopupsRules,

    pub animation_open: Option<LayerAnimationRule>,
    pub animation_close: Option<LayerAnimationRule>,
}

impl ResolvedLayerRules {
    pub fn compute(
        rules: &[LayerRule],
        surface: &LayerSurface,
        is_at_startup: bool,
        presets: &[niri_config::EffectPreset],
        materials: &[niri_config::Material],
    ) -> Self {
        let _span = tracy_client::span!("ResolvedLayerRules::compute");

        let mut resolved = ResolvedLayerRules::default();

        for rule in rules {
            let matches = |m: &Match| {
                if let Some(at_startup) = m.at_startup {
                    if at_startup != is_at_startup {
                        return false;
                    }
                }

                surface_matches(surface, m)
            };

            if !(rule.matches.is_empty() || rule.matches.iter().any(matches)) {
                continue;
            }

            if rule.excludes.iter().any(matches) {
                continue;
            }

            if let Some(preset_name) = &rule.effect_preset {
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

            if let Some(x) = rule.opacity {
                resolved.opacity = Some(x);
            }
            if let Some(x) = rule.block_out_from {
                resolved.block_out_from = Some(x);
            }
            if let Some(x) = rule.geometry_corner_radius {
                resolved.geometry_corner_radius = Some(x);
            }
            if let Some(x) = rule.place_within_backdrop {
                resolved.place_within_backdrop = x;
            }
            if let Some(x) = rule.baba_is_float {
                resolved.baba_is_float = x;
            }

            if let Some(x) = &rule.animation_open {
                resolved.animation_open = Some(x.clone());
            }
            if let Some(x) = &rule.animation_close {
                resolved.animation_close = Some(x.clone());
            }

            resolved.shadow.merge_with(&rule.shadow);

            resolved
                .background_effect
                .merge_with(&rule.background_effect);

            resolved.popups.merge_with(&rule.popups);
        }

        resolved
    }
}

fn surface_matches(surface: &LayerSurface, m: &Match) -> bool {
    if let Some(namespace_re) = &m.namespace {
        if !namespace_re.0.is_match(surface.namespace()) {
            return false;
        }
    }

    if let Some(layer) = m.layer {
        let surface_layer = match surface.layer() {
            Layer::Background => niri_ipc::Layer::Background,
            Layer::Bottom => niri_ipc::Layer::Bottom,
            Layer::Top => niri_ipc::Layer::Top,
            Layer::Overlay => niri_ipc::Layer::Overlay,
        };
        if layer != surface_layer {
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
