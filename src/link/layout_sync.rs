use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use smithay::utils::{Logical, Point};

use crate::link::protocol::{
    ColumnId, GlobalWorkspace, LayoutOp, LayoutOpKind, NodeId, Participant, TileId,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SequencedOp {
    pub seq: u64,
    pub op: LayoutOp,
}

#[derive(Debug, Default, Clone)]
pub struct OperationLog {
    ops: Vec<SequencedOp>,
}

impl OperationLog {
    pub fn push(&mut self, op: LayoutOp) {
        self.ops.push(SequencedOp { seq: op.seq, op });
    }

    pub fn last_seq(&self) -> u64 {
        self.ops.last().map(|it| it.seq).unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn has_gap_after(&self, expected_next_seq: u64) -> bool {
        self.ops
            .iter()
            .skip_while(|it| it.seq < expected_next_seq)
            .next()
            .is_some_and(|it| it.seq != expected_next_seq)
    }

    pub fn all(&self) -> &[SequencedOp] {
        &self.ops
    }
}

pub fn choose_leader<'a>(participants: impl Iterator<Item = &'a Participant>) -> Option<NodeId> {
    let mut ids: BTreeSet<NodeId> = BTreeSet::new();
    for participant in participants {
        ids.insert(participant.node_id);
    }
    ids.into_iter().next()
}

pub fn next_generation(current: u64) -> u64 {
    current.saturating_add(1)
}

#[derive(Debug, Clone, PartialEq)]
pub struct TileGeometry {
    pub tile_id: TileId,
    pub owner_node_id: NodeId,
    pub column_id: ColumnId,
    pub logical_x: f64,
    pub logical_y: f64,
    pub logical_width: f64,
    pub logical_height: f64,
}

pub fn apply_op(workspace: &mut GlobalWorkspace, op: &LayoutOp) {
    workspace.operation_seq = op.seq;
    workspace.generation = op.generation;
    match &op.kind {
        LayoutOpKind::InsertTile { tile, index } => {
            if !workspace.columns.contains(&tile.column_id) {
                workspace.columns.push(tile.column_id);
            }
            let column_tiles = ordered_column_tiles(workspace);
            let mut next = column_tiles
                .get(&tile.column_id)
                .cloned()
                .unwrap_or_default();
            let idx = (*index).min(next.len());
            next.insert(idx, tile.tile_id);
            workspace.tiles.insert(tile.tile_id, tile.clone());
            rewrite_column_membership(workspace, tile.column_id, &next);
        }
        LayoutOpKind::RemoveTile { tile_id } => {
            if let Some(tile) = workspace.tiles.remove(tile_id) {
                let mut next = ordered_column_tiles(workspace)
                    .get(&tile.column_id)
                    .cloned()
                    .unwrap_or_default();
                next.retain(|id| id != tile_id);
                rewrite_column_membership(workspace, tile.column_id, &next);
            }
            if workspace.focused_tile == Some(*tile_id) {
                workspace.focused_tile = None;
            }
        }
        LayoutOpKind::MoveTile {
            tile_id,
            column_id,
            index,
        } => {
            if let Some(tile) = workspace.tiles.get_mut(tile_id) {
                let old_column = tile.column_id;
                tile.column_id = *column_id;
                let ordered = ordered_column_tiles(workspace);
                let mut old = ordered.get(&old_column).cloned().unwrap_or_default();
                old.retain(|id| id != tile_id);
                rewrite_column_membership(workspace, old_column, &old);
                let mut new = ordered.get(column_id).cloned().unwrap_or_default();
                let idx = (*index).min(new.len());
                new.insert(idx, *tile_id);
                rewrite_column_membership(workspace, *column_id, &new);
                if !workspace.columns.contains(column_id) {
                    workspace.columns.push(*column_id);
                }
            }
        }
        LayoutOpKind::MoveColumn { column_id, index } => {
            workspace.columns.retain(|id| id != column_id);
            let idx = (*index).min(workspace.columns.len());
            workspace.columns.insert(idx, *column_id);
        }
        LayoutOpKind::ResizeColumn { .. } => {}
        LayoutOpKind::FocusTile { tile_id } => workspace.focused_tile = *tile_id,
        LayoutOpKind::FocusColumn { column_id } => {
            workspace.focused_tile = workspace
                .tiles
                .values()
                .find(|tile| tile.column_id == *column_id)
                .map(|tile| tile.tile_id);
        }
        LayoutOpKind::ScrollViewport {
            node_id,
            output_name,
            global_x,
        } => {
            if let Some(viewports) = workspace.per_node_viewports.get_mut(node_id) {
                if let Some(viewport) = viewports
                    .iter_mut()
                    .find(|it| it.output_name == *output_name)
                {
                    viewport.global_x = *global_x;
                }
            }
        }
        LayoutOpKind::SetViewport { viewport } => {
            let entry = workspace
                .per_node_viewports
                .entry(viewport.node_id)
                .or_default();
            if let Some(existing) = entry
                .iter_mut()
                .find(|it| it.output_name == viewport.output_name)
            {
                *existing = viewport.clone();
            } else {
                entry.push(viewport.clone());
            }
        }
        LayoutOpKind::ChangeTileState { tile_id, change } => {
            if let Some(tile) = workspace.tiles.get_mut(tile_id) {
                match change {
                    crate::link::protocol::TileStateChange::Fullscreen(value) => {
                        tile.fullscreen = *value
                    }
                    crate::link::protocol::TileStateChange::Floating(value) => {
                        tile.floating = *value
                    }
                    crate::link::protocol::TileStateChange::Maximized(value) => {
                        tile.maximized = *value
                    }
                }
            }
        }
        LayoutOpKind::ChangeTileMetadata { tile } => {
            workspace.tiles.insert(tile.tile_id, tile.clone());
        }
        LayoutOpKind::SetFullscreen { tile_id, value } => {
            if let Some(tile) = workspace.tiles.get_mut(tile_id) {
                tile.fullscreen = *value;
            }
        }
        LayoutOpKind::SetFloating { tile_id, value } => {
            if let Some(tile) = workspace.tiles.get_mut(tile_id) {
                tile.floating = *value;
            }
        }
        LayoutOpKind::EnterOverview
        | LayoutOpKind::LeaveOverview
        | LayoutOpKind::EnableLink
        | LayoutOpKind::DisableLink => {}
    }
}

pub fn tile_geometries(workspace: &GlobalWorkspace) -> Vec<TileGeometry> {
    let ordered = ordered_column_tiles(workspace);
    let mut rv = Vec::new();
    let mut x = 0_f64;

    for column_id in &workspace.columns {
        let Some(tile_ids) = ordered.get(column_id) else {
            continue;
        };

        let mut y = 0_f64;
        let mut column_width = 1_f64;
        let mut column_tiles = Vec::new();

        for tile_id in tile_ids {
            let Some(tile) = workspace.tiles.get(tile_id) else {
                continue;
            };
            let width = f64::max(1., tile.current_logical_size.0 as f64);
            let height = f64::max(1., tile.current_logical_size.1 as f64);
            column_width = column_width.max(width);
            column_tiles.push(TileGeometry {
                tile_id: tile.tile_id,
                owner_node_id: tile.owner_node_id,
                column_id: *column_id,
                logical_x: x,
                logical_y: y,
                logical_width: width,
                logical_height: height,
            });
            y += height;
        }

        for tile in &mut column_tiles {
            tile.logical_width = column_width;
        }
        rv.extend(column_tiles);
        x += column_width;
    }

    rv
}

pub fn hit_test_tile(
    workspace: &GlobalWorkspace,
    global_pos: Point<f64, Logical>,
) -> Option<TileGeometry> {
    tile_geometries(workspace).into_iter().find(|tile| {
        global_pos.x >= tile.logical_x
            && global_pos.y >= tile.logical_y
            && global_pos.x < tile.logical_x + tile.logical_width
            && global_pos.y < tile.logical_y + tile.logical_height
    })
}

pub fn ordered_column_tiles(workspace: &GlobalWorkspace) -> BTreeMap<ColumnId, Vec<TileId>> {
    let mut by_column: BTreeMap<ColumnId, Vec<TileId>> = BTreeMap::new();
    for column in &workspace.columns {
        by_column.entry(*column).or_default();
    }
    let mut tiles: Vec<_> = workspace.tiles.values().collect();
    tiles.sort_by_key(|tile| (tile.column_id, tile.column_tile_index, tile.tile_id));
    for tile in tiles {
        by_column
            .entry(tile.column_id)
            .or_default()
            .push(tile.tile_id);
    }
    by_column
}

fn rewrite_column_membership(
    workspace: &mut GlobalWorkspace,
    column_id: ColumnId,
    tile_ids: &[TileId],
) {
    let mut tiles: Vec<_> = tile_ids
        .iter()
        .filter_map(|id| workspace.tiles.get(id).cloned())
        .collect();
    for tile in &mut tiles {
        tile.column_id = column_id;
    }
    for tile in tiles {
        workspace.tiles.insert(tile.tile_id, tile);
    }
}
