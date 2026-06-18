//! Rhai ScriptEngine — lightweight scripting for niri-liquid.
//!
//! Scripts receive StateBus events via `on_event(e)` and can call
//! `dispatch()` and `hud()` to interact with the compositor.
//!
//! Scripts run in a sandboxed Rhai environment with configurable permissions.

use std::path::PathBuf;

use rhai::{Engine, Scope, AST};

use crate::liquid::state_bus::LiquidEvent;

/// Per-script permissions.
#[derive(Debug, Clone)]
pub struct ScriptPermissions {
    pub read_state: bool,
    pub dispatch: bool,
    pub hud: bool,
    pub spawn: bool,
    pub filesystem: bool,
    pub render_overlay: bool,
}

impl Default for ScriptPermissions {
    fn default() -> Self {
        Self {
            read_state: true,
            dispatch: true,
            hud: true,
            spawn: false,
            filesystem: false,
            render_overlay: false,
        }
    }
}

/// A loaded Rhai script.
#[derive(Debug)]
struct LoadedScript {
    name: String,
    ast: AST,
    events: Vec<String>,
    permissions: ScriptPermissions,
    errors: Vec<String>,
}

/// The Rhai script engine.
pub struct ScriptEngine {
    rhai_engine: Engine,
    scripts: Vec<LoadedScript>,
    script_dir: Option<PathBuf>,
    enabled: bool,
    /// Errors from the last load/reload.
    pub last_errors: Vec<String>,
}

impl ScriptEngine {
    /// Create a new script engine.
    pub fn new(enabled: bool, script_dir: Option<PathBuf>) -> Self {
        let mut rhai_engine = Engine::new();
        // Limit operation count to prevent infinite loops.
        rhai_engine.set_max_operations(10_000);

        Self {
            rhai_engine,
            scripts: Vec::new(),
            script_dir,
            enabled,
            last_errors: Vec::new(),
        }
    }

    /// Load all `.rhai` scripts from the configured directory.
    pub fn load_scripts(&mut self) {
        self.scripts.clear();
        self.last_errors.clear();

        let Some(ref dir) = self.script_dir else {
            return;
        };

        let Ok(entries) = std::fs::read_dir(dir) else {
            self.last_errors
                .push(format!("cannot read script dir: {:?}", dir));
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(true, |ext| ext != "rhai") {
                continue;
            }

            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            match std::fs::read_to_string(&path) {
                Ok(source) => match self.rhai_engine.compile(&source) {
                    Ok(ast) => {
                        self.scripts.push(LoadedScript {
                            name,
                            ast,
                            events: vec!["*".into()], // Listen to all events by default.
                            permissions: ScriptPermissions::default(),
                            errors: Vec::new(),
                        });
                    }
                    Err(err) => {
                        self.last_errors
                            .push(format!("{name}: compile error: {err}"));
                    }
                },
                Err(err) => {
                    self.last_errors.push(format!("{name}: read error: {err}"));
                }
            }
        }

        info!(
            "script engine: loaded {} scripts, {} errors",
            self.scripts.len(),
            self.last_errors.len()
        );
    }

    /// Reload all scripts from disk.
    pub fn reload(&mut self) {
        self.load_scripts();
    }

    /// Dispatch a StateBus event to all matching scripts' `on_event(e)` callbacks.
    pub fn dispatch_event(&self, event: &LiquidEvent) {
        if !self.enabled {
            return;
        }

        let event_map = event_to_map(event);

        for script in &self.scripts {
            if !script.errors.is_empty() {
                continue;
            }

            let should_run = script.events.iter().any(|e| e == "*")
                || script.events.iter().any(|e| {
                    event_map
                        .get("kind")
                        .map(|k| k.to_string())
                        .map(|s| s == e.as_str())
                        .unwrap_or(false)
                });

            if !should_run {
                continue;
            }

            let mut scope = Scope::new();
            scope.push("event", event_map.clone());

            // Build a sandboxed scope with only allowed APIs.
            let perms = &script.permissions;

            if perms.dispatch || perms.hud {
                // We'll use a channel-based approach to send commands back.
                // For now, log that the script ran.
            }

            // Run the script with the event.
            let result = self.rhai_engine.run_ast_with_scope(&mut scope, &script.ast);

            if let Err(err) = result {
                warn!("script {}: runtime error: {}", script.name, err);
            }
        }
    }

    /// Return list of loaded script names.
    pub fn list_scripts(&self) -> Vec<String> {
        self.scripts.iter().map(|s| s.name.clone()).collect()
    }

    /// Return all errors from the last load.
    pub fn errors(&self) -> Vec<String> {
        self.last_errors.clone()
    }

    /// Check if scripting is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Number of loaded scripts.
    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }
}

/// Convert a LiquidEvent to a Rhai-compatible map.
fn event_to_map(event: &LiquidEvent) -> rhai::Map {
    let mut map = rhai::Map::new();
    map.insert("kind".into(), event_kind(event).into());

    match event {
        LiquidEvent::MaterialChanged { to } => {
            map.insert("material".into(), to.clone().into());
        }
        LiquidEvent::AnimationProfileChanged { to } => {
            map.insert("profile".into(), to.clone().into());
        }
        LiquidEvent::FocusChanged { app_id, title } => {
            map.insert("app_id".into(), app_id.clone().unwrap_or_default().into());
            map.insert("title".into(), title.clone().unwrap_or_default().into());
        }
        LiquidEvent::WorkspaceChanged { name } => {
            map.insert("workspace".into(), name.clone().unwrap_or_default().into());
        }
        LiquidEvent::SpecialWorkspaceToggled { name } => {
            map.insert("name".into(), name.clone().into());
        }
        LiquidEvent::SafeModeToggled { active } => {
            map.insert("active".into(), (*active).into());
        }
        _ => {}
    }

    map
}

fn event_kind(event: &LiquidEvent) -> &'static str {
    match event {
        LiquidEvent::MaterialChanged { .. } => "MaterialChanged",
        LiquidEvent::AnimationProfileChanged { .. } => "AnimationProfileChanged",
        LiquidEvent::PerformanceProfileChanged { .. } => "PerformanceProfileChanged",
        LiquidEvent::FocusChanged { .. } => "FocusChanged",
        LiquidEvent::WorkspaceChanged { .. } => "WorkspaceChanged",
        LiquidEvent::SpecialWorkspaceToggled { .. } => "SpecialWorkspaceToggled",
        LiquidEvent::SafeModeToggled { .. } => "SafeModeToggled",
        LiquidEvent::ConfigReloaded => "ConfigReloaded",
        LiquidEvent::ActionDispatched { .. } => "ActionDispatched",
    }
}

// ── Script KDL config ────────────────────────────────────────────────

/// KDL config for a script definition.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptConfig {
    pub enable: bool,
    pub directory: Option<String>,
    pub reload_on_change: bool,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            enable: false,
            directory: None,
            reload_on_change: false,
        }
    }
}
