//! ActionRegistry — centralized action catalog.
//!
//! All compositor actions are registered here with metadata (category, label,
//! arguments, capabilities) so they can be uniformly accessed by:
//! - Keybind system
//! - IPC (`niri msg actions`)
//! - Action Palette
//! - Dispatch engine
//! - Documentation generator
//! - Future Rhai scripting layer

use niri_config::{ActionArgSpec, ActionCategory, ActionDescriptor, ActionSource, Capability};

/// The central action registry.
#[derive(Debug, Clone)]
pub struct ActionRegistry {
    actions: Vec<ActionDescriptor>,
}

impl ActionRegistry {
    /// Build the full registry from the Action enum and current config binds.
    pub fn build(config: &niri_config::Config) -> Self {
        let mut actions = Vec::new();

        // ── System ────────────────────────────────────────────────────
        actions.push(descriptor(
            "quit",
            "Quit niri",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[(
                "skip-confirmation",
                "Skip the exit confirmation dialog",
                false,
                &["true", "false"],
            )],
        ));
        actions.push(descriptor(
            "suspend",
            "Suspend the system",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "power-off-monitors",
            "Power off all monitors",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "power-on-monitors",
            "Power on all monitors",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "spawn",
            "Spawn a command",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[(
                "command",
                "Command and arguments to spawn",
                true,
                &["firefox", "ghostty"],
            )],
        ));
        actions.push(descriptor(
            "spawn-sh",
            "Spawn a command via shell",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[(
                "command",
                "Shell command to run",
                true,
                &["firefox --new-window"],
            )],
        ));
        actions.push(descriptor(
            "do-screen-transition",
            "Play a screen transition animation",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[(
                "delay-ms",
                "Delay before transition in milliseconds",
                false,
                &["100", "500"],
            )],
        ));

        // ── Window ────────────────────────────────────────────────────
        actions.push(descriptor(
            "close-window",
            "Close the focused window",
            ActionCategory::Window,
            ActionSource::Niri,
            Some("Mod+Q"),
            &[],
        ));
        actions.push(descriptor(
            "fullscreen-window",
            "Toggle fullscreen on the focused window",
            ActionCategory::Window,
            ActionSource::Niri,
            Some("Mod+F"),
            &[],
        ));
        actions.push(descriptor(
            "toggle-windowed-fullscreen",
            "Toggle fake fullscreen on the focused window",
            ActionCategory::Window,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "toggle-window-floating",
            "Toggle floating state of the focused window",
            ActionCategory::Window,
            ActionSource::Niri,
            Some("Mod+Shift+Space"),
            &[],
        ));
        actions.push(descriptor(
            "toggle-keyboard-shortcuts-inhibit",
            "Toggle keyboard shortcuts inhibitor",
            ActionCategory::Window,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "set-window-width",
            "Set the focused window width",
            ActionCategory::Window,
            ActionSource::Niri,
            None,
            &[(
                "change",
                "Size change (e.g., set 800, adjust +50)",
                true,
                &["set 800", "adjust +50", "set-pct 50%"],
            )],
        ));
        actions.push(descriptor(
            "set-window-height",
            "Set the focused window height",
            ActionCategory::Window,
            ActionSource::Niri,
            None,
            &[(
                "change",
                "Size change (e.g., set 600, adjust -20)",
                true,
                &["set 600", "adjust -20"],
            )],
        ));

        // ── Column ────────────────────────────────────────────────────
        actions.push(descriptor(
            "focus-column-left",
            "Focus the column to the left",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+H"),
            &[],
        ));
        actions.push(descriptor(
            "focus-column-right",
            "Focus the column to the right",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+L"),
            &[],
        ));
        actions.push(descriptor(
            "focus-window-up",
            "Focus the window above",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+K"),
            &[],
        ));
        actions.push(descriptor(
            "focus-window-down",
            "Focus the window below",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+J"),
            &[],
        ));
        actions.push(descriptor(
            "focus-window-or-monitor-up",
            "Focus window above or monitor above",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-window-or-monitor-down",
            "Focus window below or monitor below",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-column-or-monitor-left",
            "Focus column left or monitor left",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-column-or-monitor-right",
            "Focus column right or monitor right",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-window-previous",
            "Focus the previous window (MRU)",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-column-first",
            "Focus the first column",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-column-last",
            "Focus the last column",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "move-column-left",
            "Move the focused column left",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+Ctrl+H"),
            &[],
        ));
        actions.push(descriptor(
            "move-column-right",
            "Move the focused column right",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+Ctrl+L"),
            &[],
        ));
        actions.push(descriptor(
            "move-window-up",
            "Move the focused window up",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+Ctrl+K"),
            &[],
        ));
        actions.push(descriptor(
            "move-window-down",
            "Move the focused window down",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+Ctrl+J"),
            &[],
        ));
        actions.push(descriptor(
            "consume-or-expel-window-left",
            "Consume window into column or expel left",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "consume-or-expel-window-right",
            "Consume window into column or expel right",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "consume-window-into-column",
            "Consume the window below into this column",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "expel-window-from-column",
            "Expel the focused window from its column",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "swap-window-left",
            "Swap the focused window to the left",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "swap-window-right",
            "Swap the focused window to the right",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "toggle-column-tabbed-display",
            "Toggle the column between normal and tabbed display",
            ActionCategory::Column,
            ActionSource::Niri,
            Some("Mod+W"),
            &[],
        ));
        actions.push(descriptor(
            "set-column-display",
            "Set the column display mode",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[(
                "mode",
                "Display mode: normal or tabbed",
                true,
                &["normal", "tabbed"],
            )],
        ));
        actions.push(descriptor(
            "center-column",
            "Center the focused column",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "center-window",
            "Center the focused window",
            ActionCategory::Column,
            ActionSource::Niri,
            None,
            &[],
        ));

        // ── Workspace ─────────────────────────────────────────────────
        actions.push(descriptor(
            "focus-workspace-down",
            "Focus the workspace below",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-workspace-up",
            "Focus the workspace above",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-workspace-previous",
            "Focus the previous workspace",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-workspace",
            "Focus a specific workspace",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[(
                "reference",
                "Workspace name or index",
                true,
                &["dev", "3", "browser"],
            )],
        ));
        actions.push(descriptor(
            "move-window-to-workspace-down",
            "Move window to workspace below",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[(
                "focus",
                "Whether to follow the window",
                false,
                &["true", "false"],
            )],
        ));
        actions.push(descriptor(
            "move-window-to-workspace-up",
            "Move window to workspace above",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[(
                "focus",
                "Whether to follow the window",
                false,
                &["true", "false"],
            )],
        ));
        actions.push(descriptor(
            "move-window-to-workspace",
            "Move window to a specific workspace",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[("reference", "Workspace name or index", true, &["dev", "3"])],
        ));
        actions.push(descriptor(
            "move-column-to-workspace-down",
            "Move column to workspace below",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[(
                "focus",
                "Whether to follow the column",
                false,
                &["true", "false"],
            )],
        ));
        actions.push(descriptor(
            "move-column-to-workspace-up",
            "Move column to workspace above",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[(
                "focus",
                "Whether to follow the column",
                false,
                &["true", "false"],
            )],
        ));
        actions.push(descriptor(
            "move-workspace-down",
            "Move the current workspace down",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "move-workspace-up",
            "Move the current workspace up",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "set-workspace-name",
            "Set the name of the current workspace",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[(
                "name",
                "New workspace name",
                true,
                &["dev", "browser", "chat"],
            )],
        ));

        // ── Monitor ───────────────────────────────────────────────────
        actions.push(descriptor(
            "focus-monitor-left",
            "Focus the monitor to the left",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-monitor-right",
            "Focus the monitor to the right",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-monitor-up",
            "Focus the monitor above",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-monitor-down",
            "Focus the monitor below",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "focus-monitor-previous",
            "Focus the previous monitor",
            ActionCategory::Workspace,
            ActionSource::Niri,
            None,
            &[],
        ));

        // ── Screenshot ────────────────────────────────────────────────
        actions.push(descriptor(
            "screenshot",
            "Open the screenshot UI",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[(
                "show-pointer",
                "Whether to show pointer",
                false,
                &["true", "false"],
            )],
        ));
        actions.push(descriptor(
            "screenshot-screen",
            "Screenshot the focused screen",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "screenshot-window",
            "Screenshot the focused window",
            ActionCategory::System,
            ActionSource::Niri,
            None,
            &[],
        ));

        // ── Debug ─────────────────────────────────────────────────────
        actions.push(descriptor(
            "toggle-debug-tint",
            "Toggle debug tint overlay",
            ActionCategory::Debug,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "debug-toggle-opaque-regions",
            "Toggle debug opaque regions display",
            ActionCategory::Debug,
            ActionSource::Niri,
            None,
            &[],
        ));
        actions.push(descriptor(
            "debug-toggle-damage",
            "Toggle debug damage display",
            ActionCategory::Debug,
            ActionSource::Niri,
            None,
            &[],
        ));

        // ── niri-liquid: Overlay ──────────────────────────────────────
        actions.push(descriptor(
            "toggle-action-palette",
            "Toggle the action palette overlay",
            ActionCategory::Overlay,
            ActionSource::NiriLiquid,
            Some("Mod+P"),
            &[],
        ));

        // ── niri-liquid: Material ─────────────────────────────────────
        actions.push(descriptor(
            "set-material",
            "Set the active material",
            ActionCategory::Material,
            ActionSource::NiriLiquid,
            None,
            &[(
                "material",
                "Material name",
                true,
                &[
                    "obsidian-glass",
                    "frosted-ceramic",
                    "hologram-film",
                    "acrylic-smoke",
                    "neon-wet",
                    "paper-mist",
                ],
            )],
        ));
        let has_liquid = config.niri_liquid.disable_liquid_materials == false;
        if has_liquid {
            actions.last_mut().unwrap().capability = Some(Capability::LiquidMaterials);
        }

        // ── niri-liquid: Animation ────────────────────────────────────
        actions.push(descriptor(
            "set-animation-profile",
            "Set the active animation profile",
            ActionCategory::Animation,
            ActionSource::NiriLiquid,
            None,
            &[(
                "profile",
                "Animation profile name",
                true,
                &["default", "slow", "fast", "focus", "battery"],
            )],
        ));

        // ── niri-liquid: Special Workspace ────────────────────────────
        actions.push(descriptor(
            "toggle-scratch-column",
            "Toggle a scratch column / special workspace",
            ActionCategory::SpecialWorkspace,
            ActionSource::NiriLiquid,
            None,
            &[(
                "name",
                "Scratch column name",
                true,
                &["terminal", "music", "chat"],
            )],
        ));

        // ── niri-liquid: Safe Mode ────────────────────────────────────
        actions.push(descriptor(
            "toggle-safe-mode",
            "Toggle safe mode (emergency escape hatch)",
            ActionCategory::Debug,
            ActionSource::NiriLiquid,
            Some("Mod+Shift+Backspace"),
            &[],
        ));

        // Sort by category then label for consistent display.
        actions.sort_by(|a, b| {
            category_order(&a.category)
                .cmp(&category_order(&b.category))
                .then_with(|| a.label.cmp(&b.label))
        });

        Self { actions }
    }

    /// Return all registered actions.
    pub fn all(&self) -> &[ActionDescriptor] {
        &self.actions
    }

    /// Find an action by its id.
    pub fn find(&self, id: &str) -> Option<&ActionDescriptor> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// Return actions filtered by category.
    pub fn by_category(&self, category: ActionCategory) -> Vec<&ActionDescriptor> {
        self.actions
            .iter()
            .filter(|a| a.category == category)
            .collect()
    }

    /// Return actions matching a search query (fuzzy match on id and label).
    pub fn search(&self, query: &str) -> Vec<&ActionDescriptor> {
        let q = query.to_lowercase();
        self.actions
            .iter()
            .filter(|a| a.id.to_lowercase().contains(&q) || a.label.to_lowercase().contains(&q))
            .collect()
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn descriptor(
    id: &str,
    label: &str,
    category: ActionCategory,
    source: ActionSource,
    default_bind: Option<&str>,
    args: &[(&str, &str, bool, &[&str])],
) -> ActionDescriptor {
    ActionDescriptor {
        id: id.to_string(),
        label: label.to_string(),
        category,
        default_bind: default_bind.map(String::from),
        args: args
            .iter()
            .map(|(name, desc, required, examples)| ActionArgSpec {
                name: name.to_string(),
                description: desc.to_string(),
                required: *required,
                examples: examples.iter().map(|s| s.to_string()).collect(),
            })
            .collect(),
        source,
        capability: None,
    }
}

fn category_order(cat: &ActionCategory) -> u8 {
    match cat {
        ActionCategory::Window => 1,
        ActionCategory::Column => 2,
        ActionCategory::Workspace => 3,
        ActionCategory::SpecialWorkspace => 4,
        ActionCategory::Overlay => 5,
        ActionCategory::Material => 6,
        ActionCategory::Animation => 7,
        ActionCategory::System => 8,
        ActionCategory::Debug => 9,
        ActionCategory::Script => 10,
    }
}

#[cfg(test)]
mod tests {
    use niri_config::Config;

    use super::*;

    #[test]
    fn registry_has_all_categories() {
        let config = Config::default();
        let registry = ActionRegistry::build(&config);
        let actions = registry.all();

        // Core actions should exist.
        assert!(actions.iter().any(|a| a.id == "quit"));
        assert!(actions.iter().any(|a| a.id == "close-window"));
        assert!(actions.iter().any(|a| a.id == "focus-column-left"));
        assert!(actions.iter().any(|a| a.id == "focus-workspace-down"));

        // Liquid actions should exist.
        assert!(actions.iter().any(|a| a.id == "toggle-action-palette"));
        assert!(actions.iter().any(|a| a.id == "set-material"));
        assert!(actions.iter().any(|a| a.id == "toggle-scratch-column"));
        assert!(actions.iter().any(|a| a.id == "toggle-safe-mode"));
    }

    #[test]
    fn find_by_id() {
        let config = Config::default();
        let registry = ActionRegistry::build(&config);
        let action = registry.find("quit").unwrap();
        assert_eq!(action.label, "Quit niri");
        assert_eq!(action.category, ActionCategory::System);
    }

    #[test]
    fn search_finds_matches() {
        let config = Config::default();
        let registry = ActionRegistry::build(&config);
        let results = registry.search("focus");
        assert!(!results.is_empty());
        assert!(results.iter().all(|a| {
            a.id.to_lowercase().contains("focus") || a.label.to_lowercase().contains("focus")
        }));
    }

    #[test]
    fn by_category_filters() {
        let config = Config::default();
        let registry = ActionRegistry::build(&config);
        let windows = registry.by_category(ActionCategory::Window);
        assert!(windows.iter().any(|a| a.id == "close-window"));
        assert!(windows.iter().any(|a| a.id == "toggle-window-floating"));
    }

    #[test]
    fn liquid_actions_have_niri_liquid_source() {
        let config = Config::default();
        let registry = ActionRegistry::build(&config);
        let action = registry.find("toggle-action-palette").unwrap();
        assert_eq!(action.source, ActionSource::NiriLiquid);

        let action = registry.find("set-material").unwrap();
        assert_eq!(action.source, ActionSource::NiriLiquid);
    }
}
