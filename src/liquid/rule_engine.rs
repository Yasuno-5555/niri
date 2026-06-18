//! RuleEngine V2 — unified rule matching and application.
//!
//! Provides priority-based rule evaluation across all targets.

use niri_config::{RuleAction, RuleTarget, UnifiedRule};

/// The runtime rule engine.
#[derive(Debug, Clone)]
pub struct RuleEngine {
    rules: Vec<UnifiedRule>,
}

impl RuleEngine {
    pub fn new(config: &niri_config::Config) -> Self {
        let mut rules = config.unified_rules.clone();
        rules.sort_by_key(|r| -r.priority);
        Self { rules }
    }

    /// Find all rules matching a target. Rules are sorted by priority.
    pub fn find_matching(
        &self,
        target: RuleTarget,
        app_id: Option<&str>,
        title: Option<&str>,
    ) -> Vec<&UnifiedRule> {
        self.rules
            .iter()
            .filter(|rule| {
                rule.target == target && {
                    let app_match = rule
                        .app_id_filter
                        .as_ref()
                        .map(|f| app_id.is_some_and(|id| id.contains(f.as_str())))
                        .unwrap_or(true);
                    let title_match = rule
                        .title_filter
                        .as_ref()
                        .map(|f| title.is_some_and(|t| t.contains(f.as_str())))
                        .unwrap_or(true);
                    app_match && title_match
                }
            })
            .collect()
    }

    /// Apply actions from matching rules.
    pub fn apply(
        &self,
        target: RuleTarget,
        app_id: Option<&str>,
        title: Option<&str>,
    ) -> Vec<RuleAction> {
        self.find_matching(target, app_id, title)
            .iter()
            .flat_map(|rule| rule.actions.iter().cloned())
            .collect()
    }

    /// Trace rule matching for debugging.
    pub fn trace(
        &self,
        target: RuleTarget,
        app_id: Option<&str>,
        title: Option<&str>,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        for rule in &self.rules {
            if rule.target != target {
                continue;
            }
            let matched = {
                let app_match = rule
                    .app_id_filter
                    .as_ref()
                    .map(|f| app_id.is_some_and(|id| id.contains(f.as_str())))
                    .unwrap_or(true);
                let title_match = rule
                    .title_filter
                    .as_ref()
                    .map(|f| title.is_some_and(|t| t.contains(f.as_str())))
                    .unwrap_or(true);
                app_match && title_match
            };
            let status = if matched { "[match]" } else { "[skip]" };
            lines.push(format!(
                "{} rule={} target={} priority={}",
                status,
                rule.id,
                rule.target.as_str(),
                rule.priority
            ));
            if matched {
                for action in &rule.actions {
                    lines.push(format!("  [apply] {:?}", action));
                }
            }
        }
        lines
    }

    pub fn reload(&mut self, config: &niri_config::Config) {
        let mut rules = config.unified_rules.clone();
        rules.sort_by_key(|r| -r.priority);
        self.rules = rules;
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use niri_config::Config;

    use super::*;

    #[test]
    fn rule_engine_from_empty_config() {
        let config = Config::default();
        let engine = RuleEngine::new(&config);
        assert!(engine.is_empty());
    }

    #[test]
    fn rule_engine_with_rules() {
        let config = Config::parse_mem(
            r#"
            rule "terminal-to-special" {
                target "window"
                priority 100
                match { app-id "ghostty"; }
                apply {
                    workspace "special:terminal"
                    material "obsidian-glass"
                }
            }
            "#,
        )
        .unwrap();
        let engine = RuleEngine::new(&config);
        assert_eq!(engine.len(), 1);

        let matching = engine.find_matching(RuleTarget::Window, Some("ghostty"), None);
        assert_eq!(matching.len(), 1);
        assert_eq!(matching[0].id, "terminal-to-special");

        let not_matching = engine.find_matching(RuleTarget::Window, Some("firefox"), None);
        assert!(not_matching.is_empty());
    }

    #[test]
    fn rule_tracing() {
        let config = Config::parse_mem(
            r#"
            rule "terminal-glass" {
                target "window"
                priority 100
                match { app-id "ghostty"; }
                apply { material "obsidian-glass"; }
            }
            rule "all-windows" {
                target "window"
                priority 10
                apply { effect-preset "default-glass"; }
            }
            "#,
        )
        .unwrap();
        let engine = RuleEngine::new(&config);
        let trace = engine.trace(RuleTarget::Window, Some("ghostty"), None);

        let matched_terminal = trace
            .iter()
            .any(|l| l.contains("[match]") && l.contains("terminal-glass"));
        assert!(matched_terminal);

        let matched_all = trace
            .iter()
            .any(|l| l.contains("[match]") && l.contains("all-windows"));
        assert!(matched_all);
    }
}
