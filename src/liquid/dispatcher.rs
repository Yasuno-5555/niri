//! Dispatcher Core — unified action execution hub.
//!
//! All compositor operations go through the dispatcher, regardless of source.
//!
//! Types defined here; the dispatch methods live on `Niri` in `src/niri.rs`.

use niri_config::Action;

/// Identifies where a dispatch command originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchSource {
    /// Triggered by a keyboard shortcut.
    Keybind,
    /// Triggered via IPC.
    Ipc,
    /// Triggered from the Action Palette UI.
    ActionPalette,
    /// Triggered from a Rhai script.
    Script,
    /// Triggered from a compositor hook.
    Hook,
    /// Triggered during safe mode activation.
    SafeMode,
}

impl DispatchSource {
    pub fn name(self) -> &'static str {
        match self {
            DispatchSource::Keybind => "keybind",
            DispatchSource::Ipc => "ipc",
            DispatchSource::ActionPalette => "action-palette",
            DispatchSource::Script => "script",
            DispatchSource::Hook => "hook",
            DispatchSource::SafeMode => "safe-mode",
        }
    }
}

/// A command to be dispatched.
#[derive(Debug, Clone)]
pub struct DispatchCommand {
    /// The action to execute.
    pub action: Action,
    /// Where the command came from.
    pub source: DispatchSource,
    /// Optional label for logging/HUD.
    pub source_label: Option<String>,
}

/// Result of a dispatch operation.
#[derive(Debug, Clone)]
pub struct DispatchResult {
    /// Whether the command executed successfully.
    pub success: bool,
    /// Optional message for Mode HUD / logging.
    pub message: Option<String>,
    /// Any error that occurred.
    pub error: Option<String>,
}

impl DispatchResult {
    pub fn ok() -> Self {
        Self {
            success: true,
            message: None,
            error: None,
        }
    }

    pub fn ok_with(msg: impl Into<String>) -> Self {
        Self {
            success: true,
            message: Some(msg.into()),
            error: None,
        }
    }

    pub fn err(error: impl Into<String>) -> Self {
        Self {
            success: false,
            message: None,
            error: Some(error.into()),
        }
    }
}
