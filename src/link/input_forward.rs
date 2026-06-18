use serde::{Deserialize, Serialize};
use smithay::utils::{Logical, Point};
use uuid::Uuid;

use crate::link::layout_sync::TileGeometry;
use crate::link::protocol::{NodeId, TileId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForwardedInputEvent {
    PointerMotion {
        tile_id: TileId,
        local_pos: (f64, f64),
        global_pos: (f64, f64),
    },
    PointerButton {
        tile_id: TileId,
        button: u32,
        pressed: bool,
    },
    PointerScroll {
        tile_id: TileId,
        horizontal: f64,
        vertical: f64,
    },
    KeyboardKey {
        tile_id: TileId,
        keycode: u32,
        pressed: bool,
    },
    Modifiers {
        tile_id: TileId,
        depressed: u32,
        latched: u32,
        locked: u32,
        group: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoutedInput {
    pub owner_node_id: NodeId,
    pub tile_id: TileId,
    pub global_pos: Point<f64, Logical>,
}

pub fn local_to_global(
    output_origin: Point<f64, Logical>,
    local_pos: Point<f64, Logical>,
) -> Point<f64, Logical> {
    Point::from((output_origin.x + local_pos.x, output_origin.y + local_pos.y))
}

pub fn owner_for_focus(focused_tile: Option<(TileId, NodeId)>) -> Option<(TileId, NodeId)> {
    focused_tile
}

pub fn owner_for_hit(
    tiles: &[TileGeometry],
    global_pos: Point<f64, Logical>,
) -> Option<(TileId, NodeId)> {
    tiles
        .iter()
        .find(|tile| {
            global_pos.x >= tile.logical_x
                && global_pos.y >= tile.logical_y
                && global_pos.x < tile.logical_x + tile.logical_width
                && global_pos.y < tile.logical_y + tile.logical_height
        })
        .map(|tile| (tile.tile_id, tile.owner_node_id))
}

pub fn random_focus_marker() -> Uuid {
    Uuid::new_v4()
}
