use crate::appearance::{BackgroundEffectRule, BlockOutFrom, CornerRadius, ShadowRule};
use crate::utils::RegexEq;
use crate::window_rule::PopupsRule;
use crate::FloatOrInt;

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct LayerRule {
    #[knuffel(children(name = "match"))]
    pub matches: Vec<Match>,
    #[knuffel(children(name = "exclude"))]
    pub excludes: Vec<Match>,

    #[knuffel(child, unwrap(argument))]
    pub opacity: Option<f32>,
    #[knuffel(child, unwrap(argument))]
    pub block_out_from: Option<BlockOutFrom>,
    #[knuffel(child, default)]
    pub shadow: ShadowRule,
    #[knuffel(child)]
    pub geometry_corner_radius: Option<CornerRadius>,
    #[knuffel(child, unwrap(argument))]
    pub place_within_backdrop: Option<bool>,
    #[knuffel(child, unwrap(argument))]
    pub baba_is_float: Option<bool>,
    #[knuffel(child, default)]
    pub background_effect: BackgroundEffectRule,
    #[knuffel(child, default)]
    pub popups: PopupsRule,
    #[knuffel(child, unwrap(argument))]
    pub effect_preset: Option<String>,
    #[knuffel(child)]
    pub animation_open: Option<LayerAnimationRule>,
    #[knuffel(child)]
    pub animation_close: Option<LayerAnimationRule>,
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct LayerAnimationRule {
    #[knuffel(child, unwrap(argument))]
    pub style: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub from_scale: Option<FloatOrInt<0, 1000>>,
    #[knuffel(child, unwrap(argument))]
    pub direction: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub duration_ms: Option<u32>,
    #[knuffel(child, unwrap(argument))]
    pub curve: Option<String>,
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct Match {
    #[knuffel(property, str)]
    pub namespace: Option<RegexEq>,
    #[knuffel(property)]
    pub at_startup: Option<bool>,
    #[knuffel(property, str)]
    pub layer: Option<niri_ipc::Layer>,
}
