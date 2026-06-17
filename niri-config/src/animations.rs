use knuffel::errors::DecodeError;
use knuffel::Decode as _;

use crate::utils::{expect_only_children, parse_arg_node, MergeWith};
use crate::FloatOrInt;

#[derive(Debug, Clone, PartialEq)]
pub struct Animations {
    pub off: bool,
    pub slowdown: f64,
    pub workspace_switch: WorkspaceSwitchAnim,
    pub window_open: WindowOpenAnim,
    pub window_close: WindowCloseAnim,
    pub horizontal_view_movement: HorizontalViewMovementAnim,
    pub window_movement: WindowMovementAnim,
    pub window_resize: WindowResizeAnim,
    pub config_notification_open_close: ConfigNotificationOpenCloseAnim,
    pub exit_confirmation_open_close: ExitConfirmationOpenCloseAnim,
    pub screenshot_ui_open: ScreenshotUiOpenAnim,
    pub overview_open_close: OverviewOpenCloseAnim,
    pub recent_windows_close: RecentWindowsCloseAnim,
}

impl Default for Animations {
    fn default() -> Self {
        Self {
            off: false,
            slowdown: 1.,
            workspace_switch: Default::default(),
            horizontal_view_movement: Default::default(),
            window_movement: Default::default(),
            window_open: Default::default(),
            window_close: Default::default(),
            window_resize: Default::default(),
            config_notification_open_close: Default::default(),
            exit_confirmation_open_close: Default::default(),
            screenshot_ui_open: Default::default(),
            overview_open_close: Default::default(),
            recent_windows_close: Default::default(),
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct AnimationsPart {
    #[knuffel(child)]
    pub off: bool,
    #[knuffel(child)]
    pub on: bool,
    #[knuffel(child, unwrap(argument))]
    pub slowdown: Option<FloatOrInt<0, { i32::MAX }>>,
    #[knuffel(child)]
    pub workspace_switch: Option<WorkspaceSwitchAnim>,
    #[knuffel(child)]
    pub window_open: Option<WindowOpenAnim>,
    #[knuffel(child)]
    pub window_close: Option<WindowCloseAnim>,
    #[knuffel(child)]
    pub horizontal_view_movement: Option<HorizontalViewMovementAnim>,
    #[knuffel(child)]
    pub window_movement: Option<WindowMovementAnim>,
    #[knuffel(child)]
    pub window_resize: Option<WindowResizeAnim>,
    #[knuffel(child)]
    pub config_notification_open_close: Option<ConfigNotificationOpenCloseAnim>,
    #[knuffel(child)]
    pub exit_confirmation_open_close: Option<ExitConfirmationOpenCloseAnim>,
    #[knuffel(child)]
    pub screenshot_ui_open: Option<ScreenshotUiOpenAnim>,
    #[knuffel(child)]
    pub overview_open_close: Option<OverviewOpenCloseAnim>,
    #[knuffel(child)]
    pub recent_windows_close: Option<RecentWindowsCloseAnim>,
}

impl MergeWith<AnimationsPart> for Animations {
    fn merge_with(&mut self, part: &AnimationsPart) {
        self.off |= part.off;
        if part.on {
            self.off = false;
        }

        merge!((self, part), slowdown);

        // Animation properties are fairly tied together, except maybe `off`. So let's just save
        // ourselves the work and not merge within individual animations.
        merge_clone!(
            (self, part),
            workspace_switch,
            window_open,
            window_close,
            horizontal_view_movement,
            window_movement,
            window_resize,
            config_notification_open_close,
            exit_confirmation_open_close,
            screenshot_ui_open,
            overview_open_close,
            recent_windows_close,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Animation {
    pub off: bool,
    pub kind: Kind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Easing(EasingParams),
    Spring(SpringParams),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EasingParams {
    pub duration_ms: u32,
    pub curve: Curve,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Curve {
    Linear,
    EaseOutQuad,
    EaseOutCubic,
    EaseOutExpo,
    CubicBezier(f64, f64, f64, f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringParams {
    pub damping_ratio: f64,
    pub stiffness: u32,
    pub epsilon: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkspaceSwitchAnim(pub Animation);

impl Default for WorkspaceSwitchAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 1.,
                stiffness: 1000,
                epsilon: 0.0001,
            }),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowOpenAnim {
    pub anim: Animation,
    pub custom_shader: Option<String>,
}

impl Default for WindowOpenAnim {
    fn default() -> Self {
        Self {
            anim: Animation {
                off: false,
                kind: Kind::Easing(EasingParams {
                    duration_ms: 150,
                    curve: Curve::EaseOutExpo,
                }),
            },
            custom_shader: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowCloseAnim {
    pub anim: Animation,
    pub custom_shader: Option<String>,
}

impl Default for WindowCloseAnim {
    fn default() -> Self {
        Self {
            anim: Animation {
                off: false,
                kind: Kind::Easing(EasingParams {
                    duration_ms: 150,
                    curve: Curve::EaseOutQuad,
                }),
            },
            custom_shader: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizontalViewMovementAnim(pub Animation);

impl Default for HorizontalViewMovementAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 1.,
                stiffness: 800,
                epsilon: 0.0001,
            }),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowMovementAnim(pub Animation);

impl Default for WindowMovementAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 1.,
                stiffness: 800,
                epsilon: 0.0001,
            }),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowResizeAnim {
    pub anim: Animation,
    pub custom_shader: Option<String>,
}

impl Default for WindowResizeAnim {
    fn default() -> Self {
        Self {
            anim: Animation {
                off: false,
                kind: Kind::Spring(SpringParams {
                    damping_ratio: 1.,
                    stiffness: 800,
                    epsilon: 0.0001,
                }),
            },
            custom_shader: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigNotificationOpenCloseAnim(pub Animation);

impl Default for ConfigNotificationOpenCloseAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 0.6,
                stiffness: 1000,
                epsilon: 0.001,
            }),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExitConfirmationOpenCloseAnim(pub Animation);

impl Default for ExitConfirmationOpenCloseAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 0.6,
                stiffness: 500,
                epsilon: 0.01,
            }),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenshotUiOpenAnim(pub Animation);

impl Default for ScreenshotUiOpenAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Easing(EasingParams {
                duration_ms: 200,
                curve: Curve::EaseOutQuad,
            }),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverviewOpenCloseAnim(pub Animation);

impl Default for OverviewOpenCloseAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 1.,
                stiffness: 800,
                epsilon: 0.0001,
            }),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecentWindowsCloseAnim(pub Animation);

impl Default for RecentWindowsCloseAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 1.,
                stiffness: 800,
                epsilon: 0.001,
            }),
        })
    }
}

impl<S> knuffel::Decode<S> for WorkspaceSwitchAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().0;
        Ok(Self(Animation::decode_node(node, ctx, default, |_, _| {
            Ok(false)
        })?))
    }
}

impl<S> knuffel::Decode<S> for HorizontalViewMovementAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().0;
        Ok(Self(Animation::decode_node(node, ctx, default, |_, _| {
            Ok(false)
        })?))
    }
}

impl<S> knuffel::Decode<S> for WindowMovementAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().0;
        Ok(Self(Animation::decode_node(node, ctx, default, |_, _| {
            Ok(false)
        })?))
    }
}

impl<S> knuffel::Decode<S> for WindowOpenAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().anim;
        let mut custom_shader = None;
        let anim = Animation::decode_node(node, ctx, default, |child, ctx| {
            if &**child.node_name == "custom-shader" {
                custom_shader = parse_arg_node("custom-shader", child, ctx)?;
                Ok(true)
            } else {
                Ok(false)
            }
        })?;

        Ok(Self {
            anim,
            custom_shader,
        })
    }
}

impl<S> knuffel::Decode<S> for WindowCloseAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().anim;
        let mut custom_shader = None;
        let anim = Animation::decode_node(node, ctx, default, |child, ctx| {
            if &**child.node_name == "custom-shader" {
                custom_shader = parse_arg_node("custom-shader", child, ctx)?;
                Ok(true)
            } else {
                Ok(false)
            }
        })?;

        Ok(Self {
            anim,
            custom_shader,
        })
    }
}

impl<S> knuffel::Decode<S> for WindowResizeAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().anim;
        let mut custom_shader = None;
        let anim = Animation::decode_node(node, ctx, default, |child, ctx| {
            if &**child.node_name == "custom-shader" {
                custom_shader = parse_arg_node("custom-shader", child, ctx)?;
                Ok(true)
            } else {
                Ok(false)
            }
        })?;

        Ok(Self {
            anim,
            custom_shader,
        })
    }
}

impl<S> knuffel::Decode<S> for ConfigNotificationOpenCloseAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().0;
        Ok(Self(Animation::decode_node(node, ctx, default, |_, _| {
            Ok(false)
        })?))
    }
}

impl<S> knuffel::Decode<S> for ExitConfirmationOpenCloseAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().0;
        Ok(Self(Animation::decode_node(node, ctx, default, |_, _| {
            Ok(false)
        })?))
    }
}

impl<S> knuffel::Decode<S> for ScreenshotUiOpenAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().0;
        Ok(Self(Animation::decode_node(node, ctx, default, |_, _| {
            Ok(false)
        })?))
    }
}

impl<S> knuffel::Decode<S> for OverviewOpenCloseAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().0;
        Ok(Self(Animation::decode_node(node, ctx, default, |_, _| {
            Ok(false)
        })?))
    }
}

impl<S> knuffel::Decode<S> for RecentWindowsCloseAnim
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let default = Self::default().0;
        Ok(Self(Animation::decode_node(node, ctx, default, |_, _| {
            Ok(false)
        })?))
    }
}

impl Animation {
    pub fn new_off() -> Self {
        Self {
            off: true,
            kind: Kind::Easing(EasingParams {
                duration_ms: 0,
                curve: Curve::Linear,
            }),
        }
    }

    fn decode_node<S: knuffel::traits::ErrorSpan>(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
        default: Self,
        process_children: impl FnMut(
            &knuffel::ast::SpannedNode<S>,
            &mut knuffel::decode::Context<S>,
        ) -> Result<bool, DecodeError<S>>,
    ) -> Result<Self, DecodeError<S>> {
        expect_only_children(node, ctx);
        Self::decode_children_impl(node, ctx, default, process_children)
    }

    /// Like `decode_node` but does not error on node arguments.
    /// Used for `animation-preset` nodes which have a name argument.
    pub(crate) fn decode_node_allow_args<S: knuffel::traits::ErrorSpan>(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
        default: Self,
        process_children: impl FnMut(
            &knuffel::ast::SpannedNode<S>,
            &mut knuffel::decode::Context<S>,
        ) -> Result<bool, DecodeError<S>>,
    ) -> Result<Self, DecodeError<S>> {
        if let Some(type_name) = &node.type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
        for (name, _) in &node.properties {
            ctx.emit_error(DecodeError::unexpected(
                name,
                "property",
                format!("unexpected property `{}`", name.escape_default()),
            ));
        }
        // Skip argument check — animation-preset has a name argument.
        Self::decode_children_impl(node, ctx, default, process_children)
    }

    fn decode_children_impl<S: knuffel::traits::ErrorSpan>(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
        default: Self,
        mut process_children: impl FnMut(
            &knuffel::ast::SpannedNode<S>,
            &mut knuffel::decode::Context<S>,
        ) -> Result<bool, DecodeError<S>>,
    ) -> Result<Self, DecodeError<S>> {
        #[derive(Default, PartialEq)]
        struct OptionalEasingParams {
            duration_ms: Option<u32>,
            curve: Option<Curve>,
        }

        let mut off = false;
        let mut easing_params = OptionalEasingParams::default();
        let mut spring_params = None;

        for child in node.children() {
            match &**child.node_name {
                "off" => {
                    knuffel::decode::check_flag_node(child, ctx);
                    if off {
                        ctx.emit_error(DecodeError::unexpected(
                            &child.node_name,
                            "node",
                            "duplicate node `off`, single node expected",
                        ));
                    } else {
                        off = true;
                    }
                }
                "spring" => {
                    if easing_params != OptionalEasingParams::default() {
                        ctx.emit_error(DecodeError::unexpected(
                            child,
                            "node",
                            "cannot set both spring and easing parameters at once",
                        ));
                    }
                    if spring_params.is_some() {
                        ctx.emit_error(DecodeError::unexpected(
                            &child.node_name,
                            "node",
                            "duplicate node `spring`, single node expected",
                        ));
                    }

                    spring_params = Some(SpringParams::decode_node(child, ctx)?);
                }
                "duration-ms" => {
                    if spring_params.is_some() {
                        ctx.emit_error(DecodeError::unexpected(
                            child,
                            "node",
                            "cannot set both spring and easing parameters at once",
                        ));
                    }
                    if easing_params.duration_ms.is_some() {
                        ctx.emit_error(DecodeError::unexpected(
                            &child.node_name,
                            "node",
                            "duplicate node `duration-ms`, single node expected",
                        ));
                    }

                    easing_params.duration_ms = Some(parse_arg_node("duration-ms", child, ctx)?);
                }
                "curve" => {
                    if spring_params.is_some() {
                        ctx.emit_error(DecodeError::unexpected(
                            child,
                            "node",
                            "cannot set both spring and easing parameters at once",
                        ));
                    }
                    if easing_params.curve.is_some() {
                        ctx.emit_error(DecodeError::unexpected(
                            &child.node_name,
                            "node",
                            "duplicate node `curve`, single node expected",
                        ));
                    }

                    let mut iter_args = child.arguments.iter();
                    let val = iter_args.next().ok_or_else(|| {
                        DecodeError::missing(child, "additional argument `curve` is required")
                    })?;
                    let animation_curve_string: String =
                        knuffel::traits::DecodeScalar::decode(val, ctx)?;

                    let animation_curve = match animation_curve_string.as_str() {
                        "linear" => Some(Curve::Linear),
                        "ease-out-quad" => Some(Curve::EaseOutQuad),
                        "ease-out-cubic" => Some(Curve::EaseOutCubic),
                        "ease-out-expo" => Some(Curve::EaseOutExpo),
                        "cubic-bezier" => {
                            let val = iter_args.next().ok_or_else(|| {
                                DecodeError::missing(
                                    child,
                                    "missing x1 coordinate for cubic Bézier curve control point",
                                )
                            })?;
                            // the X axis represents time frame so it cannot be negative
                            // or larger than 1
                            let x1: FloatOrInt<0, 1> =
                                knuffel::traits::DecodeScalar::decode(val, ctx)?;
                            let val = iter_args.next().ok_or_else(|| {
                                DecodeError::missing(
                                    child,
                                    "missing y1 coordinate for cubic Bézier curve control point",
                                )
                            })?;
                            let y1: FloatOrInt<{ i32::MIN }, { i32::MAX }> =
                                knuffel::traits::DecodeScalar::decode(val, ctx)?;
                            let val = iter_args.next().ok_or_else(|| {
                                DecodeError::missing(
                                    child,
                                    "missing x2 coordinate for cubic Bézier curve control point",
                                )
                            })?;
                            let x2: FloatOrInt<0, 1> =
                                knuffel::traits::DecodeScalar::decode(val, ctx)?;
                            let val = iter_args.next().ok_or_else(|| {
                                DecodeError::missing(
                                    child,
                                    "missing y2 coordinate for cubic Bézier curve control point",
                                )
                            })?;
                            let y2: FloatOrInt<{ i32::MIN }, { i32::MAX }> =
                                knuffel::traits::DecodeScalar::decode(val, ctx)?;

                            Some(Curve::CubicBezier(x1.0, y1.0, x2.0, y2.0))
                        }
                        unexpected_curve => {
                            ctx.emit_error(DecodeError::unexpected(
                                &val.literal,
                                "argument",
                                format!(
                                    "unexpected animation curve `{unexpected_curve}`. \
                                    Niri only supports five animation curves: \
                                    `ease-out-quad`, `ease-out-cubic`, `ease-out-expo`, `linear` and `cubic-bezier`."
                                ),
                            ));

                            None
                        }
                    };

                    if let Some(val) = iter_args.next() {
                        ctx.emit_error(DecodeError::unexpected(
                            &val.literal,
                            "argument",
                            "unexpected argument",
                        ));
                    }
                    for name in child.properties.keys() {
                        ctx.emit_error(DecodeError::unexpected(
                            name,
                            "property",
                            format!("unexpected property `{}`", name.escape_default()),
                        ));
                    }
                    for child in child.children() {
                        ctx.emit_error(DecodeError::unexpected(
                            child,
                            "node",
                            format!("unexpected node `{}`", child.node_name.escape_default()),
                        ));
                    }

                    easing_params.curve = animation_curve;
                }
                name_str => {
                    if !process_children(child, ctx)? {
                        ctx.emit_error(DecodeError::unexpected(
                            child,
                            "node",
                            format!("unexpected node `{}`", name_str.escape_default()),
                        ));
                    }
                }
            }
        }

        let kind = if let Some(spring_params) = spring_params {
            // Configured spring.
            Kind::Spring(spring_params)
        } else if easing_params == OptionalEasingParams::default() {
            // Did not configure anything.
            default.kind
        } else {
            // Configured easing.
            let default = if let Kind::Easing(easing) = default.kind {
                easing
            } else {
                // Generic fallback values for when the default animation is spring, but the user
                // configured an easing animation.
                EasingParams {
                    duration_ms: 250,
                    curve: Curve::EaseOutCubic,
                }
            };

            Kind::Easing(EasingParams {
                duration_ms: easing_params.duration_ms.unwrap_or(default.duration_ms),
                curve: easing_params.curve.unwrap_or(default.curve),
            })
        };

        Ok(Self { off, kind })
    }
}

/// User-defined animation preset, parsed from `animation-preset` nodes in config.kdl.
///
/// ```kdl
/// animation-preset "my-pop" {
///     spring damping-ratio=0.55 stiffness=800 epsilon=0.0001
/// }
/// animation-preset "my-close" {
///     easing duration-ms=200 curve="ease-out-cubic"
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationPreset {
    pub name: String,
    pub animation: Animation,
}

impl<S> knuffel::Decode<S> for AnimationPreset
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        let mut iter = node.arguments.iter();
        let val = iter
            .next()
            .ok_or_else(|| DecodeError::missing(node, "argument `name` is required"))?;
        let name: String = knuffel::traits::DecodeScalar::decode(val, ctx)?;

        if let Some(extra) = iter.next() {
            ctx.emit_error(DecodeError::unexpected(
                &extra.literal,
                "argument",
                "unexpected argument",
            ));
        }

        let default_anim = Animation {
            off: false,
            kind: Kind::Spring(SpringParams {
                damping_ratio: 0.7,
                stiffness: 800,
                epsilon: 0.0001,
            }),
        };

        let mut easing_override: Option<EasingParams> = None;
        let animation =
            Animation::decode_node_allow_args(node, ctx, default_anim, |child, ctx| {
                if &**child.node_name == "easing" {
                    if easing_override.is_some() {
                        ctx.emit_error(DecodeError::unexpected(
                            &child.node_name,
                            "node",
                            "duplicate node `easing`, single node expected",
                        ));
                    } else {
                        easing_override = Some(EasingParams::decode_node(child, ctx)?);
                    }
                    Ok(true)
                } else {
                    Ok(false)
                }
            })?;

        // If an `easing` node was supplied, override the kind.
        let animation = if let Some(easing) = easing_override {
            Animation {
                off: animation.off,
                kind: Kind::Easing(easing),
            }
        } else {
            animation
        };

        Ok(Self { name, animation })
    }
}

/// Parses `easing duration-ms=<u32> curve="<name>"` as a property-based node,
/// symmetric to the `spring` node. Used inside `animation-preset { ... }`.
///
/// Example:
/// ```kdl
/// animation-preset "liquid-melt" {
///     easing duration-ms=200 curve="ease-out-cubic"
/// }
/// ```
impl<S> knuffel::Decode<S> for EasingParams
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        if let Some(type_name) = &node.type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
        if let Some(val) = node.arguments.first() {
            ctx.emit_error(DecodeError::unexpected(
                &val.literal,
                "argument",
                "unexpected argument",
            ));
        }
        for child in node.children() {
            ctx.emit_error(DecodeError::unexpected(
                child,
                "node",
                format!("unexpected node `{}`", child.node_name.escape_default()),
            ));
        }

        let mut duration_ms: Option<u32> = None;
        let mut curve: Option<Curve> = None;

        for (name, val) in &node.properties {
            match &***name {
                "duration-ms" => {
                    let v: u32 = knuffel::traits::DecodeScalar::decode(val, ctx)?;
                    duration_ms = Some(v);
                }
                "curve" => {
                    let s: String = knuffel::traits::DecodeScalar::decode(val, ctx)?;
                    let parsed = match s.as_str() {
                        "linear" => Some(Curve::Linear),
                        "ease-out-quad" => Some(Curve::EaseOutQuad),
                        "ease-out-cubic" => Some(Curve::EaseOutCubic),
                        "ease-out-expo" => Some(Curve::EaseOutExpo),
                        unexpected => {
                            ctx.emit_error(DecodeError::unexpected(
                                name,
                                "property value",
                                format!(
                                    "unexpected animation curve `{unexpected}`. \
                                    Supported values: \
                                    `linear`, `ease-out-quad`, `ease-out-cubic`, `ease-out-expo`."
                                ),
                            ));
                            None
                        }
                    };
                    curve = parsed;
                }
                name_str => {
                    ctx.emit_error(DecodeError::unexpected(
                        name,
                        "property",
                        format!("unexpected property `{}`", name_str.escape_default()),
                    ));
                }
            }
        }

        let duration_ms = duration_ms
            .ok_or_else(|| DecodeError::missing(node, "property `duration-ms` is required"))?;
        let curve =
            curve.ok_or_else(|| DecodeError::missing(node, "property `curve` is required"))?;

        if duration_ms == 0 {
            ctx.emit_error(DecodeError::conversion(
                node,
                "duration-ms must be greater than 0",
            ));
        }

        Ok(EasingParams { duration_ms, curve })
    }
}

impl<S> knuffel::Decode<S> for SpringParams
where
    S: knuffel::traits::ErrorSpan,
{
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        if let Some(type_name) = &node.type_name {
            ctx.emit_error(DecodeError::unexpected(
                type_name,
                "type name",
                "no type name expected for this node",
            ));
        }
        if let Some(val) = node.arguments.first() {
            ctx.emit_error(DecodeError::unexpected(
                &val.literal,
                "argument",
                "unexpected argument",
            ));
        }
        for child in node.children() {
            ctx.emit_error(DecodeError::unexpected(
                child,
                "node",
                format!("unexpected node `{}`", child.node_name.escape_default()),
            ));
        }

        let mut damping_ratio = None;
        let mut stiffness = None;
        let mut epsilon = None;
        for (name, val) in &node.properties {
            match &***name {
                "damping-ratio" => {
                    damping_ratio = Some(knuffel::traits::DecodeScalar::decode(val, ctx)?);
                }
                "stiffness" => {
                    stiffness = Some(knuffel::traits::DecodeScalar::decode(val, ctx)?);
                }
                "epsilon" => {
                    epsilon = Some(knuffel::traits::DecodeScalar::decode(val, ctx)?);
                }
                name_str => {
                    ctx.emit_error(DecodeError::unexpected(
                        name,
                        "property",
                        format!("unexpected property `{}`", name_str.escape_default()),
                    ));
                }
            }
        }
        let damping_ratio = damping_ratio
            .ok_or_else(|| DecodeError::missing(node, "property `damping-ratio` is required"))?;
        let stiffness = stiffness
            .ok_or_else(|| DecodeError::missing(node, "property `stiffness` is required"))?;
        let epsilon =
            epsilon.ok_or_else(|| DecodeError::missing(node, "property `epsilon` is required"))?;

        if !(0.1..=10.).contains(&damping_ratio) {
            ctx.emit_error(DecodeError::conversion(
                node,
                "damping-ratio must be between 0.1 and 10.0",
            ));
        }
        if stiffness < 1 {
            ctx.emit_error(DecodeError::conversion(node, "stiffness must be >= 1"));
        }
        if !(0.00001..=0.1).contains(&epsilon) {
            ctx.emit_error(DecodeError::conversion(
                node,
                "epsilon must be between 0.00001 and 0.1",
            ));
        }

        Ok(SpringParams {
            damping_ratio,
            stiffness,
            epsilon,
        })
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct AnimationProfile {
    #[knuffel(argument)]
    pub name: String,

    #[knuffel(child, unwrap(argument))]
    pub window_open: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub window_close: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub layer_open: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub layer_close: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub overview_open: Option<String>,

    #[knuffel(child, unwrap(argument))]
    pub workspace: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    // ── animation-preset easing パーサーテスト ────────────────────────

    #[test]
    fn animation_preset_easing_ease_out_cubic() {
        let config = Config::parse_mem(
            r#"
            animation-preset "liquid-melt" {
                easing duration-ms=200 curve="ease-out-cubic"
            }
            "#,
        )
        .unwrap();

        let preset = config
            .animation_presets
            .iter()
            .find(|p| p.name == "liquid-melt")
            .expect("preset not found");

        assert_eq!(
            preset.animation.kind,
            Kind::Easing(EasingParams {
                duration_ms: 200,
                curve: Curve::EaseOutCubic,
            }),
            "easing preset should parse correctly"
        );
        assert!(!preset.animation.off);
    }

    #[test]
    fn animation_preset_easing_ease_out_quad() {
        let config = Config::parse_mem(
            r#"
            animation-preset "shrinkfade" {
                easing duration-ms=150 curve="ease-out-quad"
            }
            "#,
        )
        .unwrap();

        let preset = config
            .animation_presets
            .iter()
            .find(|p| p.name == "shrinkfade")
            .expect("preset not found");

        assert_eq!(
            preset.animation.kind,
            Kind::Easing(EasingParams {
                duration_ms: 150,
                curve: Curve::EaseOutQuad,
            })
        );
    }

    #[test]
    fn animation_preset_easing_ease_out_expo() {
        let config = Config::parse_mem(
            r#"
            animation-preset "fast-open" {
                easing duration-ms=120 curve="ease-out-expo"
            }
            "#,
        )
        .unwrap();

        let preset = config
            .animation_presets
            .iter()
            .find(|p| p.name == "fast-open")
            .expect("preset not found");

        assert_eq!(
            preset.animation.kind,
            Kind::Easing(EasingParams {
                duration_ms: 120,
                curve: Curve::EaseOutExpo,
            })
        );
    }

    #[test]
    fn animation_preset_easing_linear() {
        let config = Config::parse_mem(
            r#"
            animation-preset "flat" {
                easing duration-ms=100 curve="linear"
            }
            "#,
        )
        .unwrap();

        let preset = config
            .animation_presets
            .iter()
            .find(|p| p.name == "flat")
            .expect("preset not found");

        assert_eq!(
            preset.animation.kind,
            Kind::Easing(EasingParams {
                duration_ms: 100,
                curve: Curve::Linear,
            })
        );
    }

    #[test]
    fn animation_preset_spring_still_works() {
        let config = Config::parse_mem(
            r#"
            animation-preset "liquid-pop" {
                spring damping-ratio=0.55 stiffness=800 epsilon=0.0001
            }
            "#,
        )
        .unwrap();

        let preset = config
            .animation_presets
            .iter()
            .find(|p| p.name == "liquid-pop")
            .expect("preset not found");

        assert_eq!(
            preset.animation.kind,
            Kind::Spring(SpringParams {
                damping_ratio: 0.55,
                stiffness: 800,
                epsilon: 0.0001,
            })
        );
    }

    #[test]
    fn animation_preset_easing_all_four_supported_curves() {
        // Verify all four documented curve values parse without error.
        let curves = ["linear", "ease-out-quad", "ease-out-cubic", "ease-out-expo"];
        for curve in &curves {
            let src = format!(
                "animation-preset \"test-{curve}\" {{\n    easing duration-ms=100 curve=\"{curve}\"\n}}"
            );
            let config = Config::parse_mem(&src)
                .unwrap_or_else(|e| panic!("parse failed for curve={curve}: {e:?}"));
            let preset = config
                .animation_presets
                .iter()
                .find(|p| p.name == format!("test-{curve}"))
                .expect("preset not found");
            assert!(
                matches!(preset.animation.kind, Kind::Easing(_)),
                "curve={curve} should produce Easing kind"
            );
        }
    }

    #[test]
    fn animation_preset_easing_invalid_curve_is_error() {
        // Unknown curve names should fail validation.
        let result = Config::parse_mem(
            r#"
            animation-preset "bad" {
                easing duration-ms=100 curve="nonexistent-curve"
            }
            "#,
        );
        assert!(result.is_err(), "invalid curve should produce parse error");
    }

    #[test]
    fn animation_preset_easing_off_flag() {
        let config = Config::parse_mem(
            r#"
            animation-preset "disabled" {
                off
                easing duration-ms=100 curve="linear"
            }
            "#,
        )
        .unwrap();

        let preset = config
            .animation_presets
            .iter()
            .find(|p| p.name == "disabled")
            .expect("preset not found");

        assert!(preset.animation.off, "off flag should be respected");
        assert_eq!(
            preset.animation.kind,
            Kind::Easing(EasingParams {
                duration_ms: 100,
                curve: Curve::Linear,
            })
        );
    }

    #[test]
    fn animations_kdl_file_all_presets() {
        // Smoke test: parse the full animations.kdl content from the user config.
        // This exercises the exact syntax that was previously failing.
        let kdl = r#"
            animation-preset "liquid-melt" {
                easing duration-ms=200 curve="ease-out-cubic"
            }
            animation-preset "shrinkfade" {
                easing duration-ms=150 curve="ease-out-quad"
            }
            animation-preset "fade-shrink" {
                easing duration-ms=180 curve="ease-out-quad"
            }
            animation-preset "minimal-slide" {
                easing duration-ms=120 curve="ease-out-quad"
            }
            animation-preset "liquid-pop" {
                spring damping-ratio=0.55 stiffness=800 epsilon=0.0001
            }
        "#;

        let config =
            Config::parse_mem(kdl).expect("animations.kdl content should parse without error");

        let names: Vec<_> = config
            .animation_presets
            .iter()
            .map(|p| p.name.as_str())
            .collect();

        assert!(names.contains(&"liquid-melt"));
        assert!(names.contains(&"shrinkfade"));
        assert!(names.contains(&"fade-shrink"));
        assert!(names.contains(&"minimal-slide"));
        assert!(names.contains(&"liquid-pop"));
    }
}
