use std::cell::RefCell;

use serde::{Deserialize, Serialize};
use smithay::backend::renderer::element::Kind;
use smithay::backend::renderer::gles::{GlesRenderer, GlesTexture};
use smithay::output::{self, Output};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point, Rectangle, Scale, Serial, Size, Transform};

use crate::layout::{
    ConfigureIntent, InteractiveResizeData, LayoutElement, LayoutElementRenderElement,
    LayoutElementRenderSnapshot, SizingMode,
};
use crate::link::protocol::{NodeId, StreamState, TileId, TileMetadata};
use crate::link::stream::FramePacket;
use crate::render_helpers::background_effect::BackgroundEffectElement;
use crate::render_helpers::offscreen::OffscreenData;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::snapshot::RenderSnapshot;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};
use crate::render_helpers::texture::TextureBuffer;
use crate::render_helpers::xray::XrayPos;
use crate::render_helpers::{BakedBuffer, RenderCtx};
use crate::window::ResolvedWindowRules;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RemoteTileOverlay {
    Placeholder,
    Stale,
    Disconnected,
    PrivacyBlocked,
}

#[derive(Debug)]
pub struct RemoteTile {
    metadata: TileMetadata,
    size: Size<i32, Logical>,
    rules: ResolvedWindowRules,
    latest_frame: RefCell<Option<FramePacket>>,
    placeholder: SolidColorBuffer,
}

impl RemoteTile {
    pub fn new(metadata: TileMetadata) -> Self {
        let size = Size::from((
            metadata.current_logical_size.0.max(1),
            metadata.current_logical_size.1.max(1),
        ));
        Self {
            metadata,
            size,
            rules: ResolvedWindowRules::default(),
            latest_frame: RefCell::new(None),
            placeholder: SolidColorBuffer::new(
                (size.w as f64, size.h as f64),
                [0.15, 0.15, 0.18, 1.0],
            ),
        }
    }

    pub fn tile_id(&self) -> TileId {
        self.metadata.tile_id
    }

    pub fn owner_node_id(&self) -> NodeId {
        self.metadata.owner_node_id
    }

    pub fn metadata(&self) -> &TileMetadata {
        &self.metadata
    }

    pub fn update_metadata(&mut self, metadata: TileMetadata) {
        self.size = Size::from((
            metadata.current_logical_size.0.max(1),
            metadata.current_logical_size.1.max(1),
        ));
        self.metadata = metadata;
        self.placeholder
            .resize((self.size.w as f64, self.size.h as f64));
    }

    pub fn update_frame(&self, frame: FramePacket) {
        *self.latest_frame.borrow_mut() = Some(frame);
    }

    pub fn overlay(&self) -> RemoteTileOverlay {
        match self.metadata.stream_state {
            StreamState::Disconnected => RemoteTileOverlay::Disconnected,
            StreamState::Stale => RemoteTileOverlay::Stale,
            StreamState::PrivacyBlocked => RemoteTileOverlay::PrivacyBlocked,
            StreamState::Pending | StreamState::Idle => RemoteTileOverlay::Placeholder,
            StreamState::Streaming => {
                if self.latest_frame.borrow().is_some() {
                    RemoteTileOverlay::Placeholder
                } else {
                    RemoteTileOverlay::Stale
                }
            }
        }
    }
}

pub enum LinkedElement<W> {
    Local(W),
    Remote(RemoteTile),
}

impl<W: LayoutElement> std::fmt::Debug for LinkedElement<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local(_) => f.write_str("LinkedElement::Local(..)"),
            Self::Remote(remote) => f
                .debug_tuple("LinkedElement::Remote")
                .field(remote)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkedElementId<I> {
    Local(I),
    Remote(TileId),
}

pub type RemoteTileSnapshot =
    RenderSnapshot<BakedBuffer<TextureBuffer<GlesTexture>>, BakedBuffer<SolidColorBuffer>>;

impl LayoutElement for RemoteTile {
    type Id = TileId;

    fn id(&self) -> &Self::Id {
        &self.metadata.tile_id
    }

    fn size(&self) -> Size<i32, Logical> {
        self.size
    }

    fn buf_loc(&self) -> Point<i32, Logical> {
        Point::from((0, 0))
    }

    fn is_in_input_region(&self, point: Point<f64, Logical>) -> bool {
        point.x >= 0.
            && point.y >= 0.
            && point.x < self.size.w as f64
            && point.y < self.size.h as f64
    }

    fn render_normal<R: NiriRenderer>(
        &self,
        _ctx: RenderCtx<R>,
        location: Point<f64, Logical>,
        _scale: Scale<f64>,
        alpha: f32,
        push: &mut dyn FnMut(LayoutElementRenderElement<R>),
    ) {
        let element = SolidColorRenderElement::from_buffer(
            &self.placeholder,
            location,
            alpha,
            Kind::Unspecified,
        );
        push(LayoutElementRenderElement::SolidColor(element));
    }

    fn render_background_effect(
        &self,
        _ctx: RenderCtx<GlesRenderer>,
        _geometry: Rectangle<f64, Logical>,
        _scale: f64,
        _clip_to_geometry: bool,
        _surface_anim_scale: Scale<f64>,
        _radius: niri_config::CornerRadius,
        _xray_pos: XrayPos,
        _push: &mut dyn FnMut(BackgroundEffectElement),
    ) {
    }

    fn request_size(
        &mut self,
        size: Size<i32, Logical>,
        _mode: SizingMode,
        _animate: bool,
        _transaction: Option<crate::utils::transaction::Transaction>,
    ) {
        self.size = Size::from((size.w.max(1), size.h.max(1)));
    }

    fn min_size(&self) -> Size<i32, Logical> {
        Size::from((1, 1))
    }

    fn max_size(&self) -> Size<i32, Logical> {
        self.size
    }

    fn is_wl_surface(&self, _wl_surface: &WlSurface) -> bool {
        false
    }

    fn has_ssd(&self) -> bool {
        false
    }

    fn set_preferred_scale_transform(&self, _scale: output::Scale, _transform: Transform) {}

    fn output_enter(&self, _output: &Output) {}

    fn output_leave(&self, _output: &Output) {}

    fn set_offscreen_data(&self, _data: Option<OffscreenData>) {}

    fn set_activated(&mut self, _active: bool) {}

    fn set_active_in_column(&mut self, _active: bool) {}

    fn set_floating(&mut self, floating: bool) {
        self.metadata.floating = floating;
    }

    fn set_bounds(&self, _bounds: Size<i32, Logical>) {}

    fn is_ignoring_opacity_window_rule(&self) -> bool {
        false
    }

    fn is_urgent(&self) -> bool {
        false
    }

    fn configure_intent(&self) -> ConfigureIntent {
        ConfigureIntent::NotNeeded
    }

    fn send_pending_configure(&mut self) {}

    fn sizing_mode(&self) -> SizingMode {
        if self.metadata.fullscreen {
            SizingMode::Fullscreen
        } else if self.metadata.maximized {
            SizingMode::Maximized
        } else {
            SizingMode::Normal
        }
    }

    fn pending_sizing_mode(&self) -> SizingMode {
        self.sizing_mode()
    }

    fn requested_size(&self) -> Option<Size<i32, Logical>> {
        Some(self.size)
    }

    fn is_child_of(&self, _parent: &Self) -> bool {
        false
    }

    fn rules(&self) -> &ResolvedWindowRules {
        &self.rules
    }

    fn refresh(&self) {}

    fn take_animation_snapshot(&mut self) -> Option<LayoutElementRenderSnapshot> {
        None
    }

    fn set_interactive_resize(&mut self, _data: Option<InteractiveResizeData>) {}

    fn cancel_interactive_resize(&mut self) {}

    fn interactive_resize_data(&self) -> Option<InteractiveResizeData> {
        None
    }

    fn on_commit(&mut self, _serial: Serial) {}
}
