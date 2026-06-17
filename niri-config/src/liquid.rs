//! niri-liquid configuration types.
//!
//! Phase 0: Safety / Debug Foundation
//! - Config versioning
//! - Safe mode
//! - Feature flags
//! - Capability system

use crate::utils::{Flag, MergeWith};

/// The top-level `niri-liquid` configuration block.
///
/// Controls feature flags and config schema versioning.
///
/// ```kdl
/// niri-liquid {
///     config-version 1
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct NiriLiquid {
    pub config_version: u32,
    pub disable_scripts: bool,
    pub disable_gestures: bool,
    pub disable_expensive_effects: bool,
    pub disable_liquid_materials: bool,
}

impl Default for NiriLiquid {
    fn default() -> Self {
        Self {
            config_version: 1,
            disable_scripts: false,
            disable_gestures: false,
            disable_expensive_effects: false,
            disable_liquid_materials: false,
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct NiriLiquidPart {
    #[knuffel(child, unwrap(argument))]
    pub config_version: Option<u32>,

    #[knuffel(child)]
    pub disable_scripts: Option<Flag>,

    #[knuffel(child)]
    pub disable_gestures: Option<Flag>,

    #[knuffel(child)]
    pub disable_expensive_effects: Option<Flag>,

    #[knuffel(child)]
    pub disable_liquid_materials: Option<Flag>,
}

impl MergeWith<NiriLiquidPart> for NiriLiquid {
    fn merge_with(&mut self, part: &NiriLiquidPart) {
        merge!(
            (self, part),
            disable_scripts,
            disable_gestures,
            disable_expensive_effects,
            disable_liquid_materials
        );
        if let Some(v) = part.config_version {
            self.config_version = v;
        }
    }
}

/// Safe mode configuration.
///
/// Safe mode provides an emergency escape hatch. It is triggered by an
/// always-registered keybind and disables all potentially problematic features.
///
/// ```kdl
/// safe-mode {
///     bind "Mod+Shift+Backspace"
///     material "safe-solid"
///     animation-profile "safe"
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SafeMode {
    pub enable: bool,
    pub bind: String,
    pub material: String,
    pub animation_profile: String,
    pub disable_scripts: bool,
    pub disable_gestures: bool,
    pub disable_expensive_effects: bool,
}

impl Default for SafeMode {
    fn default() -> Self {
        Self {
            enable: true,
            bind: "Mod+Shift+Backspace".to_string(),
            material: "safe-solid".to_string(),
            animation_profile: "safe".to_string(),
            disable_scripts: true,
            disable_gestures: true,
            disable_expensive_effects: true,
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct SafeModePart {
    #[knuffel(child)]
    pub enable: Option<Flag>,

    #[knuffel(child, unwrap(argument))]
    pub bind: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub material: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub animation_profile: Option<String>,

    #[knuffel(child)]
    pub disable_scripts: Option<Flag>,

    #[knuffel(child)]
    pub disable_gestures: Option<Flag>,

    #[knuffel(child)]
    pub disable_expensive_effects: Option<Flag>,
}

impl MergeWith<SafeModePart> for SafeMode {
    fn merge_with(&mut self, part: &SafeModePart) {
        merge!(
            (self, part),
            enable,
            disable_scripts,
            disable_gestures,
            disable_expensive_effects
        );
        merge_clone!((self, part), bind, material, animation_profile);
    }
}

/// Capabilities that the compositor exposes.
///
/// These are determined at runtime based on hardware support, config flags,
/// and build-time feature gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Full liquid material pipeline (blur + refraction + dispersion).
    LiquidMaterials,
    /// Chromatic dispersion / aberration effect.
    ChromaticDispersion,
    /// Gesture-edge swipe progress animations.
    GestureProgress,
    /// Special workspace (scratch column) support.
    SpecialWorkspaces,
    /// Tabbed column display mode.
    TabbedColumns,
    /// Rhai scripting engine.
    RhaiScripts,
    /// Debug wireframe material.
    DebugWireframe,
    /// Safe mode is enabled.
    SafeMode,
    /// Action palette overlay.
    ActionPalette,
    /// Mode HUD overlay.
    ModeHud,
    /// Window magnet soft-snapping.
    WindowMagnet,
    /// Column landmark pills.
    ColumnLandmarks,
}

impl Capability {
    /// Human-readable snake_case capability name for IPC.
    pub fn name(self) -> &'static str {
        match self {
            Capability::LiquidMaterials => "liquid_materials",
            Capability::ChromaticDispersion => "chromatic_dispersion",
            Capability::GestureProgress => "gesture_progress",
            Capability::SpecialWorkspaces => "special_workspaces",
            Capability::TabbedColumns => "tabbed_columns",
            Capability::RhaiScripts => "rhai_scripts",
            Capability::DebugWireframe => "debug_wireframe",
            Capability::SafeMode => "safe_mode",
            Capability::ActionPalette => "action_palette",
            Capability::ModeHud => "mode_hud",
            Capability::WindowMagnet => "window_magnet",
            Capability::ColumnLandmarks => "column_landmarks",
        }
    }
}

/// Category of an action for the ActionRegistry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionCategory {
    /// Window manipulation (close, fullscreen, float, resize).
    Window,
    /// Column manipulation (focus, move, consume/expel).
    Column,
    /// Workspace navigation and management.
    Workspace,
    /// Special workspace (scratch column) operations.
    SpecialWorkspace,
    /// Material switching and effects.
    Material,
    /// Animation profile management.
    Animation,
    /// Overlay UI (action palette, mode HUD, keybind viewer).
    Overlay,
    /// Script and automation.
    Script,
    /// Debug and developer tools.
    Debug,
    /// System-level (quit, power, screenshot).
    System,
}

/// Specification for an action argument.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionArgSpec {
    /// Argument name (e.g., "name", "direction", "index").
    pub name: String,
    /// Human-readable description of what this argument accepts.
    pub description: String,
    /// Whether the argument is required.
    pub required: bool,
    /// Example values.
    pub examples: Vec<String>,
}

/// Descriptor for a single action in the ActionRegistry.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionDescriptor {
    /// Unique identifier (matches the Action enum variant name in snake_case).
    pub id: String,
    /// Human-readable label for the Action Palette and docs.
    pub label: String,
    /// Category for grouping in the Action Palette.
    pub category: ActionCategory,
    /// Default key combination suggestion.
    pub default_bind: Option<String>,
    /// Arguments expected by this action.
    pub args: Vec<ActionArgSpec>,
    /// Whether this action was defined by niri-liquid (vs upstream niri).
    pub source: ActionSource,
    /// Required capability (None if always available).
    pub capability: Option<Capability>,
}

/// Where an action originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionSource {
    /// Upstream niri action.
    Niri,
    /// niri-liquid extension.
    NiriLiquid,
}

/// A dispatch alias — a named sequence of dispatch commands.
///
/// ```kdl
/// dispatch-alias "zen" {
///     dispatch "setanimationprofile" "focus"
///     dispatch "setmaterial" "focus-flat"
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchAlias {
    pub name: String,
    pub commands: Vec<(String, Vec<String>)>,
}

/// Parsed form of a single dispatch command within an alias.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct DispatchAliasEntry {
    #[knuffel(argument)]
    pub command: String,
    #[knuffel(arguments)]
    pub args: Vec<String>,
}

/// Overlay system configuration.
///
/// ```kdl
/// overlays {
///     default-material "dashboard-glass"
///     animation "glass-sheet"
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Overlays {
    pub default_material: String,
    pub animation: String,
}

impl Default for Overlays {
    fn default() -> Self {
        Self {
            default_material: "dashboard-glass".to_string(),
            animation: "glass-sheet".to_string(),
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct OverlaysPart {
    #[knuffel(child, unwrap(argument))]
    pub default_material: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub animation: Option<String>,
}

impl MergeWith<OverlaysPart> for Overlays {
    fn merge_with(&mut self, part: &OverlaysPart) {
        merge_clone!((self, part), default_material, animation);
    }
}

/// Used for parsing dispatch-alias from KDL.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct DispatchAliasDoc {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(children(name = "dispatch"))]
    pub entries: Vec<DispatchAliasEntry>,
}

impl From<DispatchAliasDoc> for DispatchAlias {
    fn from(doc: DispatchAliasDoc) -> Self {
        Self {
            name: doc.name,
            commands: doc
                .entries
                .into_iter()
                .map(|e| (e.command, e.args))
                .collect(),
        }
    }
}

// ── RuleEngine V2 types ──────────────────────────────────────────────

/// Unified rule target type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleTarget {
    Window,
    Layer,
    Workspace,
    Column,
    SpecialWorkspace,
    Output,
}

impl RuleTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleTarget::Window => "window",
            RuleTarget::Layer => "layer",
            RuleTarget::Workspace => "workspace",
            RuleTarget::Column => "column",
            RuleTarget::SpecialWorkspace => "special-workspace",
            RuleTarget::Output => "output",
        }
    }
}

/// Actions that a rule can apply.
#[derive(Debug, Clone, PartialEq)]
pub enum RuleAction {
    SetMaterial(String),
    SetAnimationProfile(String),
    SetWorkspace(String),
    SetEffectPreset(String),
    SetFloating(bool),
    SetOpacity(f32),
    SetColumnDisplay(String),
}

/// A unified rule combining a matcher with prioritized actions.
#[derive(Debug, Clone, PartialEq)]
pub struct UnifiedRule {
    pub id: String,
    pub target: RuleTarget,
    pub priority: i32,
    pub app_id_filter: Option<String>,
    pub title_filter: Option<String>,
    pub actions: Vec<RuleAction>,
}

/// Simple match spec for unified rules (avoids RegexEq parsing complexity).
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct RuleMatchDoc {
    #[knuffel(child, unwrap(argument))]
    pub app_id: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub title: Option<String>,
}

/// KDL parsed form of a unified rule.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct UnifiedRuleDoc {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub target: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub priority: Option<i32>,
    #[knuffel(children(name = "match"))]
    pub matchers: Vec<RuleMatchDoc>,
    #[knuffel(child)]
    pub apply: Option<UnifiedRuleApplyDoc>,
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct UnifiedRuleApplyDoc {
    #[knuffel(child, unwrap(argument))]
    pub workspace: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub material: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub animation_profile: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub effect_preset: Option<String>,
    #[knuffel(child)]
    pub floating: Option<crate::utils::Flag>,
    #[knuffel(child, unwrap(argument))]
    pub opacity: Option<crate::FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub column_display: Option<String>,
}

// ── MaterialGraph types ──────────────────────────────────────────────

/// Node in a material node graph.
#[derive(Debug, Clone, PartialEq)]
pub enum MaterialNode {
    BackdropBlur { passes: u8, offset: f64 },
    Tint { color: String, alpha: f64 },
    Saturation { amount: f64 },
    Noise { amount: f64 },
    Refraction { strength: f64 },
    Dispersion { strength: f64 },
    RimLight { strength: f64 },
    DebugWireframe,
}

/// Material graph composed of nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialGraph {
    pub name: String,
    pub nodes: Vec<MaterialNode>,
}

/// KDL parsed form of a material-graph node.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct MaterialGraphDoc {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(children(name = "node"))]
    pub nodes: Vec<MaterialNodeDoc>,
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct MaterialNodeDoc {
    #[knuffel(argument)]
    pub node_type: String,
    #[knuffel(child, unwrap(argument))]
    pub passes: Option<u8>,
    #[knuffel(child, unwrap(argument))]
    pub offset: Option<f64>,
    #[knuffel(child, unwrap(argument))]
    pub color: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub alpha: Option<f64>,
    #[knuffel(child, unwrap(argument))]
    pub amount: Option<f64>,
    #[knuffel(child, unwrap(argument))]
    pub strength: Option<f64>,
}

impl MaterialNodeDoc {
    pub fn into_node(self) -> Option<MaterialNode> {
        match self.node_type.as_str() {
            "backdrop-blur" => Some(MaterialNode::BackdropBlur {
                passes: self.passes.unwrap_or(4),
                offset: self.offset.unwrap_or(5.0),
            }),
            "tint" => Some(MaterialNode::Tint {
                color: self.color.unwrap_or_else(|| "#000000".into()),
                alpha: self.alpha.unwrap_or(0.3),
            }),
            "saturation" => Some(MaterialNode::Saturation {
                amount: self.amount.unwrap_or(1.2),
            }),
            "noise" => Some(MaterialNode::Noise {
                amount: self.amount.unwrap_or(0.01),
            }),
            "refraction" => Some(MaterialNode::Refraction {
                strength: self.strength.unwrap_or(0.012),
            }),
            "dispersion" => Some(MaterialNode::Dispersion {
                strength: self.strength.unwrap_or(0.016),
            }),
            "rim-light" => Some(MaterialNode::RimLight {
                strength: self.strength.unwrap_or(0.14),
            }),
            "debug-wireframe" => Some(MaterialNode::DebugWireframe),
            _ => None,
        }
    }
}

// ── AnimationGraph types ─────────────────────────────────────────────

/// Animation curve definition.
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationCurve {
    Bezier {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    },
    Spring {
        stiffness: f64,
        damping: f64,
        mass: f64,
    },
}

/// Animation graph node connecting input to output.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationNode {
    pub name: String,
    pub curve: Option<AnimationCurve>,
    pub duration_ms: u64,
    pub transform: Option<String>,
    pub interactive_input: Option<String>,
    pub outputs: Vec<(String, String)>, // (property, range-spec)
}

/// KDL parsed form of an animation-graph.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct AnimationGraphDoc {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(children(name = "curve"))]
    pub curves: Vec<AnimationCurveDoc>,
    #[knuffel(children(name = "node"))]
    pub nodes: Vec<AnimationNodeDoc>,
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct AnimationCurveDoc {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub curve_type: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub values: Option<String>, // bezier: "0.25 0.10 0.25 1.00"
    #[knuffel(child, unwrap(argument))]
    pub stiffness: Option<f64>,
    #[knuffel(child, unwrap(argument))]
    pub damping: Option<f64>,
    #[knuffel(child, unwrap(argument))]
    pub mass: Option<f64>,
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct AnimationNodeDoc {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument))]
    pub curve: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub duration_ms: Option<u64>,
    #[knuffel(child, unwrap(argument))]
    pub transform: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub input: Option<String>,
    #[knuffel(child)]
    pub output: Option<AnimationOutputDoc>,
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct AnimationOutputDoc {
    #[knuffel(child, unwrap(argument))]
    pub opacity: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub translate_y: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub refraction: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub blur: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub specular: Option<String>,
}

// ── Performance Budget config ────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceBudget {
    pub target_fps: u32,
    pub max_blur_passes: u8,
    pub disable_dispersion_on_battery: bool,
    pub downgrade_material_on_frame_drop: bool,
    pub quality_levels: Vec<String>, // Ordered from highest to lowest quality
}

impl Default for PerformanceBudget {
    fn default() -> Self {
        Self {
            target_fps: 60,
            max_blur_passes: 3,
            disable_dispersion_on_battery: true,
            downgrade_material_on_frame_drop: false,
            quality_levels: vec![
                "hologram-film".into(),
                "liquid-mocha".into(),
                "acrylic-smoke".into(),
                "safe-solid".into(),
            ],
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct PerformanceBudgetPart {
    #[knuffel(child, unwrap(argument))]
    pub target_fps: Option<u32>,
    #[knuffel(child, unwrap(argument))]
    pub max_blur_passes: Option<u8>,
    #[knuffel(child)]
    pub disable_dispersion_on_battery: Option<crate::utils::Flag>,
    #[knuffel(child)]
    pub downgrade_material_on_frame_drop: Option<crate::utils::Flag>,
}

impl MergeWith<PerformanceBudgetPart> for PerformanceBudget {
    fn merge_with(&mut self, part: &PerformanceBudgetPart) {
        merge!(
            (self, part),
            disable_dispersion_on_battery,
            downgrade_material_on_frame_drop
        );
        merge_clone!((self, part), target_fps, max_blur_passes);
    }
}

// ── Script config ────────────────────────────────────────────────────

/// Script engine configuration.
///
/// ```kdl
/// script {
///     enable true
///     directory "~/.config/niri-liquid/scripts"
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptConfig {
    pub enable: bool,
    pub directory: Option<String>,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            enable: false,
            directory: None,
        }
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct ScriptConfigPart {
    #[knuffel(child)]
    pub enable: Option<crate::utils::Flag>,
    #[knuffel(child, unwrap(argument))]
    pub directory: Option<String>,
}

impl MergeWith<ScriptConfigPart> for ScriptConfig {
    fn merge_with(&mut self, part: &ScriptConfigPart) {
        merge!((self, part), enable);
        merge_clone_opt!((self, part), directory);
    }
}

// ── UnifiedRuleDoc impl (continued) ──────────────────────────────────

impl UnifiedRuleDoc {
    pub fn into_rule(self) -> UnifiedRule {
        let mut actions = Vec::new();
        if let Some(apply) = &self.apply {
            if let Some(ref ws) = apply.workspace {
                actions.push(RuleAction::SetWorkspace(ws.clone()));
            }
            if let Some(ref mat) = apply.material {
                actions.push(RuleAction::SetMaterial(mat.clone()));
            }
            if let Some(ref prof) = apply.animation_profile {
                actions.push(RuleAction::SetAnimationProfile(prof.clone()));
            }
            if let Some(ref preset) = apply.effect_preset {
                actions.push(RuleAction::SetEffectPreset(preset.clone()));
            }
            if let Some(ref flag) = apply.floating {
                actions.push(RuleAction::SetFloating(flag.0));
            }
            if let Some(ref opacity) = apply.opacity {
                actions.push(RuleAction::SetOpacity(opacity.0 as f32));
            }
            if let Some(ref display) = apply.column_display {
                actions.push(RuleAction::SetColumnDisplay(display.clone()));
            }
        }
        let target = self
            .target
            .and_then(|t| match t.as_str() {
                "window" => Some(RuleTarget::Window),
                "layer" => Some(RuleTarget::Layer),
                "workspace" => Some(RuleTarget::Workspace),
                "column" => Some(RuleTarget::Column),
                "special-workspace" => Some(RuleTarget::SpecialWorkspace),
                "output" => Some(RuleTarget::Output),
                _ => None,
            })
            .unwrap_or(RuleTarget::Window);

        // Extract simple filters from the first match block.
        let (app_id_filter, title_filter) = self
            .matchers
            .first()
            .map_or((None, None), |m| (m.app_id.clone(), m.title.clone()));

        UnifiedRule {
            id: self.name,
            target,
            priority: self.priority.unwrap_or(50),
            app_id_filter,
            title_filter,
            actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn parse_niri_liquid_default() {
        let config = Config::parse_mem("").unwrap();
        assert_eq!(config.niri_liquid.config_version, 1);
        assert!(!config.niri_liquid.disable_scripts);
    }

    #[test]
    fn parse_niri_liquid_with_flags() {
        let config = Config::parse_mem(
            r#"
            niri-liquid {
                config-version 2
                disable-scripts
                disable-gestures
            }
            "#,
        )
        .unwrap();
        assert_eq!(config.niri_liquid.config_version, 2);
        assert!(config.niri_liquid.disable_scripts);
        assert!(config.niri_liquid.disable_gestures);
    }

    #[test]
    fn parse_safe_mode() {
        let config = Config::parse_mem(
            r#"
            safe-mode {
                enable
                bind "Mod+Shift+Escape"
                material "safe-solid"
                animation-profile "safe"
                disable-scripts
            }
            "#,
        )
        .unwrap();
        assert!(config.safe_mode.enable);
        assert_eq!(config.safe_mode.bind, "Mod+Shift+Escape");
        assert_eq!(config.safe_mode.material, "safe-solid");
        assert_eq!(config.safe_mode.animation_profile, "safe");
        assert!(config.safe_mode.disable_scripts);
    }

    #[test]
    fn capability_names_are_unique() {
        use std::collections::HashSet;
        let names: Vec<_> = [
            Capability::LiquidMaterials,
            Capability::ChromaticDispersion,
            Capability::GestureProgress,
            Capability::SpecialWorkspaces,
            Capability::TabbedColumns,
            Capability::RhaiScripts,
            Capability::DebugWireframe,
            Capability::SafeMode,
            Capability::ActionPalette,
            Capability::ModeHud,
            Capability::WindowMagnet,
            Capability::ColumnLandmarks,
        ]
        .iter()
        .map(|c| c.name())
        .collect();
        let set: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), set.len(), "capability names must be unique");
    }
}
