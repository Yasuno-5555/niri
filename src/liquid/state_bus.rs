//! StateBus — centralized compositor event notification.
//!
//! Events are published to a queue and drained each frame by
//! `advance_animations()`. This avoids borrow-checker issues with
//! callback-based subscriptions while keeping the event flow centralized.
//!
//! Future subscribers (IPC event stream, Rhai scripts, debug overlay)
//! will consume events from the same bus.

use std::collections::VecDeque;

/// Events emitted by the compositor when state changes.
#[derive(Debug, Clone, PartialEq)]
pub enum LiquidEvent {
    /// Active material changed.
    MaterialChanged { to: String },
    /// Animation profile changed.
    AnimationProfileChanged { to: String },
    /// Performance profile changed.
    PerformanceProfileChanged { to: String },
    /// Window focus changed.
    FocusChanged {
        app_id: Option<String>,
        title: Option<String>,
    },
    /// Active workspace changed.
    WorkspaceChanged { name: Option<String> },
    /// A special workspace (scratch column) was toggled.
    SpecialWorkspaceToggled { name: String },
    /// Safe mode was toggled.
    SafeModeToggled { active: bool },
    /// Config was reloaded from disk.
    ConfigReloaded,
    /// Keybind action executed with feedback.
    ActionDispatched { action_id: String, source: String },
}

impl LiquidEvent {
    /// Human-readable summary for a Mode HUD display line.
    pub fn hud_line(&self) -> Option<String> {
        match self {
            LiquidEvent::MaterialChanged { to } => Some(format!("MATERIAL · {to}")),
            LiquidEvent::AnimationProfileChanged { to } => Some(format!("PROFILE · {to}")),
            LiquidEvent::PerformanceProfileChanged { to } => Some(format!("PERF · {to}")),
            LiquidEvent::SpecialWorkspaceToggled { name } => Some(format!("SCRATCH · {name}")),
            LiquidEvent::SafeModeToggled { active } => {
                let state = if *active { "ON" } else { "OFF" };
                Some(format!("SAFE MODE · {state}"))
            }
            LiquidEvent::ActionDispatched { action_id, .. } => {
                // Map internal action ids to short display tokens.
                let token = short_token(action_id);
                Some(format!("ACTION · {token}"))
            }
            // Events that don't need HUD display.
            LiquidEvent::FocusChanged { .. } => None,
            LiquidEvent::WorkspaceChanged { .. } => None,
            LiquidEvent::ConfigReloaded => Some("CONFIG · reloaded".into()),
        }
    }
}

/// Central event bus for compositor state changes.
///
/// Events are queued and drained each frame in `advance_animations()`.
#[derive(Debug, Default)]
pub struct StateBus {
    queue: VecDeque<LiquidEvent>,
    max_queue: usize,
}

impl StateBus {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            max_queue: 64,
        }
    }

    /// Publish an event to the queue.
    pub fn publish(&mut self, event: LiquidEvent) {
        trace!("state-bus: {:?}", event);
        if self.queue.len() >= self.max_queue {
            self.queue.pop_front();
        }
        self.queue.push_back(event);
    }

    /// Drain all pending events into a Vec (oldest first).
    pub fn drain(&mut self) -> Vec<LiquidEvent> {
        self.queue.drain(..).collect()
    }

    /// Check if there are pending events.
    pub fn has_pending(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Return a clone of all queued events for IPC/debugging.
    pub fn snapshot(&self) -> Vec<LiquidEvent> {
        self.queue.iter().cloned().collect()
    }
}

/// Convert an action id to a short display token.
fn short_token(id: &str) -> &str {
    match id {
        "set-material" => "material",
        "set-animation-profile" => "anim",
        "toggle-scratch-column" => "scratch",
        "toggle-safe-mode" => "safe",
        "toggle-action-palette" => "palette",
        "close-window" => "close",
        "fullscreen-window" => "fullscreen",
        "toggle-window-floating" => "float",
        "quit" => "quit",
        _ => id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_and_drain() {
        let mut bus = StateBus::new();
        bus.publish(LiquidEvent::MaterialChanged {
            to: "obsidian-glass".into(),
        });
        bus.publish(LiquidEvent::AnimationProfileChanged { to: "focus".into() });

        let events = bus.drain();
        assert_eq!(events.len(), 2);
        assert!(bus.drain().is_empty());
    }

    #[test]
    fn hud_lines_for_display_events() {
        let event = LiquidEvent::MaterialChanged {
            to: "frosted-ceramic".into(),
        };
        assert_eq!(event.hud_line(), Some("MATERIAL · frosted-ceramic".into()));

        let event = LiquidEvent::SafeModeToggled { active: true };
        assert_eq!(event.hud_line(), Some("SAFE MODE · ON".into()));

        // Focus/workspace events are not for HUD.
        let event = LiquidEvent::FocusChanged {
            app_id: Some("ghostty".into()),
            title: None,
        };
        assert_eq!(event.hud_line(), None);
    }

    #[test]
    fn queue_respects_max_size() {
        let mut bus = StateBus::new();
        bus.max_queue = 4;
        for i in 0..6 {
            bus.publish(LiquidEvent::MaterialChanged {
                to: format!("mat-{i}"),
            });
        }
        let events = bus.drain();
        assert_eq!(events.len(), 4);
        // Oldest events were dropped.
        assert_eq!(events[0].hud_line(), Some("MATERIAL · mat-2".into()));
    }
}
