//! niri-liquid runtime modules.
//!
//! Phase 1: ActionRegistry
//! Phase 2: Dispatcher Core
//! Phase 3: StateBus
//! Phase 4: OverlayManager

pub mod action_registry;
pub mod dispatcher;
pub mod overlay;
pub mod performance_budget;
pub mod rule_engine;
pub mod script_engine;
pub mod state_bus;
