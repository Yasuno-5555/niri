use smithay::utils::{Logical, Point, Size};

use crate::binds::{Action, WorkspaceReference};
use crate::utils::MergeWith;
use crate::FloatOrInt;

#[derive(knuffel::Decode)]
struct GestureEdgeActionDoc {
    #[knuffel(children)]
    actions: Vec<Action>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Gestures {
    pub dnd_edge_view_scroll: DndEdgeViewScroll,
    pub dnd_edge_workspace_switch: DndEdgeWorkspaceSwitch,
    pub hot_corners: HotCorners,
    pub gesture_edges: Vec<GestureEdge>,
}

#[derive(knuffel::Decode, Debug, Default, Clone, PartialEq)]
pub struct GesturesPart {
    #[knuffel(child)]
    pub dnd_edge_view_scroll: Option<DndEdgeViewScrollPart>,
    #[knuffel(child)]
    pub dnd_edge_workspace_switch: Option<DndEdgeWorkspaceSwitchPart>,
    #[knuffel(child)]
    pub hot_corners: Option<HotCorners>,
    #[knuffel(children(name = "gesture-edge"))]
    pub gesture_edges: Vec<GestureEdge>,
}

impl MergeWith<GesturesPart> for Gestures {
    fn merge_with(&mut self, part: &GesturesPart) {
        merge!(
            (self, part),
            dnd_edge_view_scroll,
            dnd_edge_workspace_switch,
        );
        merge_clone!((self, part), hot_corners);
        self.gesture_edges
            .extend(part.gesture_edges.iter().cloned());
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct GestureEdge {
    #[knuffel(argument)]
    pub edge: String,
    #[knuffel(child, unwrap(argument))]
    pub fingers: Option<u8>,
    #[knuffel(child, unwrap(argument))]
    pub action: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub reveal_ratio: Option<FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub interactive: Option<bool>,
    #[knuffel(child)]
    pub progress_map: Option<GestureProgressMap>,
}

impl GestureEdge {
    pub fn fingers(&self) -> u8 {
        self.fingers.unwrap_or(3)
    }

    pub fn parsed_action(&self) -> Option<Action> {
        let action = self.action.as_deref()?.trim();
        if let Ok(doc) = knuffel::parse::<GestureEdgeActionDoc>("gesture-edge-action.kdl", action) {
            if let Some(action) = doc.actions.into_iter().next() {
                return Some(action);
            }
        }

        if let Some(name) = action.strip_prefix("toggle-scratch-column ") {
            return Some(Action::ToggleScratchColumn(name.trim().to_string()));
        }
        if let Some(profile) = action.strip_prefix("set-animation-profile ") {
            return Some(Action::SetAnimationProfile(profile.trim().to_string()));
        }
        if let Some(material) = action.strip_prefix("set-material ") {
            return Some(Action::SetMaterial(material.trim().to_string()));
        }
        if let Some(reference) = action.strip_prefix("focus-workspace ") {
            let reference = reference.trim();
            let reference = if let Ok(index) = reference.parse::<u8>() {
                WorkspaceReference::Index(index)
            } else {
                WorkspaceReference::Name(reference.to_string())
            };
            return Some(Action::FocusWorkspace(reference));
        }

        None
    }

    pub fn reveal_ratio(&self) -> f64 {
        self.reveal_ratio.map(|x| x.0).unwrap_or(0.68)
    }

    pub fn interactive(&self) -> bool {
        self.interactive.unwrap_or(false)
    }

    pub fn matches_start_region(
        &self,
        local_pos: Point<f64, Logical>,
        output_size: Size<i32, Logical>,
    ) -> bool {
        let width = output_size.w as f64;
        let height = output_size.h as f64;

        match self.edge.as_str() {
            "bottom" => local_pos.y >= height * 0.9,
            "top" => local_pos.y <= height * 0.1,
            "left" => local_pos.x <= width * 0.1,
            "right" => local_pos.x >= width * 0.9,
            _ => false,
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct GestureProgressMap {
    #[knuffel(child, unwrap(argument))]
    pub target: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub translate_x: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub translate_y: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub opacity: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub blur: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub refraction: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub specular: Option<String>,
    #[knuffel(child, unwrap(argument))]
    pub chromatic_aberration: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DndEdgeViewScroll {
    pub trigger_width: f64,
    pub delay_ms: u16,
    pub max_speed: f64,
}

impl Default for DndEdgeViewScroll {
    fn default() -> Self {
        Self {
            trigger_width: 30., // Taken from GTK 4.
            delay_ms: 100,
            max_speed: 1500.,
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct DndEdgeViewScrollPart {
    #[knuffel(child, unwrap(argument))]
    pub trigger_width: Option<FloatOrInt<0, 65535>>,
    #[knuffel(child, unwrap(argument))]
    pub delay_ms: Option<u16>,
    #[knuffel(child, unwrap(argument))]
    pub max_speed: Option<FloatOrInt<0, 1_000_000>>,
}

impl MergeWith<DndEdgeViewScrollPart> for DndEdgeViewScroll {
    fn merge_with(&mut self, part: &DndEdgeViewScrollPart) {
        merge!((self, part), trigger_width, max_speed);
        merge_clone!((self, part), delay_ms);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DndEdgeWorkspaceSwitch {
    pub trigger_height: f64,
    pub delay_ms: u16,
    pub max_speed: f64,
}

impl Default for DndEdgeWorkspaceSwitch {
    fn default() -> Self {
        Self {
            trigger_height: 50.,
            delay_ms: 100,
            max_speed: 1500.,
        }
    }
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct DndEdgeWorkspaceSwitchPart {
    #[knuffel(child, unwrap(argument))]
    pub trigger_height: Option<FloatOrInt<0, 65535>>,
    #[knuffel(child, unwrap(argument))]
    pub delay_ms: Option<u16>,
    #[knuffel(child, unwrap(argument))]
    pub max_speed: Option<FloatOrInt<0, 1_000_000>>,
}

impl MergeWith<DndEdgeWorkspaceSwitchPart> for DndEdgeWorkspaceSwitch {
    fn merge_with(&mut self, part: &DndEdgeWorkspaceSwitchPart) {
        merge!((self, part), trigger_height, max_speed);
        merge_clone!((self, part), delay_ms);
    }
}

#[derive(knuffel::Decode, Debug, Default, Clone, Copy, PartialEq)]
pub struct HotCorners {
    #[knuffel(child)]
    pub off: bool,
    #[knuffel(child)]
    pub top_left: bool,
    #[knuffel(child)]
    pub top_right: bool,
    #[knuffel(child)]
    pub bottom_left: bool,
    #[knuffel(child)]
    pub bottom_right: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binds::WorkspaceReference;

    #[test]
    fn parse_gesture_edge_actions_via_action_decoder() {
        let gesture = GestureEdge {
            edge: String::from("bottom"),
            fingers: None,
            action: Some(String::from("toggle-overview")),
            reveal_ratio: None,
            interactive: None,
            progress_map: None,
        };
        assert_eq!(gesture.parsed_action(), Some(Action::ToggleOverview));

        let gesture = GestureEdge {
            edge: String::from("bottom"),
            fingers: None,
            action: Some(String::from("focus-workspace dev")),
            reveal_ratio: None,
            interactive: None,
            progress_map: None,
        };
        assert_eq!(
            gesture.parsed_action(),
            Some(Action::FocusWorkspace(WorkspaceReference::Name(
                String::from("dev")
            )))
        );

        let gesture = GestureEdge {
            edge: String::from("bottom"),
            fingers: None,
            action: Some(String::from("move-column-to-index 3")),
            reveal_ratio: None,
            interactive: None,
            progress_map: None,
        };
        assert_eq!(gesture.parsed_action(), Some(Action::MoveColumnToIndex(3)));
    }
}
