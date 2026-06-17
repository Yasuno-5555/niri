//! Hyprland-compatible dispatch command parser.
//!
//! Maps Hyprland-style dispatch commands to niri IPC Actions, providing
//! a compatibility layer for Hyprland users migrating to niri-liquid.
//!
//! Usage: `niri msg dispatch <command> [args...]`
//!
//! ## Supported commands
//!
//! | Hyprland Command               | niri Equivalent                     |
//! |--------------------------------|-------------------------------------|
//! | `exec <cmd...>`                | `spawn`                             |
//! | `execr <cmd>`                  | `spawn-sh`                          |
//! | `killactive`                   | `close-window`                      |
//! | `closewindow`                  | `close-window`                      |
//! | `fullscreen` / `togglefullscreen` | `fullscreen-window` / `toggle-windowed-fullscreen` |
//! | `togglefloating`               | `toggle-window-floating`            |
//! | `movefocus <dir>`              | various focus actions               |
//! | `swapwindow <dir>`             | various swap actions                |
//! | `focusmonitor <dir>`           | various focus-monitor actions       |
//! | `movetoworkspace <name>`       | `move-window-to-workspace`          |
//! | `movetoworkspacesilent <name>` | `move-window-to-workspace` (no focus) |
//! | `workspace <name>`             | `focus-workspace`                   |
//! | `togglespecialworkspace [name]`| `toggle-scratch-column`             |
//! | `setanimationprofile <name>`   | `set-animation-profile`             |
//! | `setmaterial <name>`           | `set-material`                      |
//! | `toggleactionpalette`          | `toggle-action-palette`             |
//! | `quit` / `exit`                | `quit`                              |

use anyhow::bail;
use niri_ipc::Action;

/// Parse a Hyprland-compatible dispatch command into an niri IPC Action.
pub fn parse_dispatch(args: &[String]) -> anyhow::Result<Action> {
    if args.is_empty() {
        bail!("dispatch requires a command (e.g. `niri msg dispatch exec firefox`)");
    }

    let cmd = &args[0];
    let cmd_args = &args[1..];

    match cmd.as_str() {
        // ── Application Launchers ──────────────────────────────────────
        "exec" => {
            if cmd_args.is_empty() {
                bail!("dispatch exec requires a command to run");
            }
            Ok(Action::Spawn {
                command: cmd_args.to_vec(),
            })
        }
        "execr" => {
            if cmd_args.is_empty() {
                bail!("dispatch execr requires a command to run");
            }
            Ok(Action::SpawnSh {
                command: cmd_args.join(" "),
            })
        }

        // ── Window Management ─────────────────────────────────────────
        "killactive" | "closewindow" => Ok(Action::CloseWindow { id: None }),

        "fullscreen" => Ok(Action::FullscreenWindow { id: None }),

        "togglefullscreen" => Ok(Action::ToggleWindowedFullscreen { id: None }),

        "togglefloating" => Ok(Action::ToggleWindowFloating { id: None }),

        // ── Focus / Navigation ────────────────────────────────────────
        "movefocus" => parse_movefocus(cmd_args),
        "swapwindow" => parse_swapwindow(cmd_args),
        "focusmonitor" => parse_focusmonitor(cmd_args),
        "focuswindow" => parse_focuswindow(cmd_args),

        // ── Workspace ─────────────────────────────────────────────────
        "workspace" => {
            if cmd_args.is_empty() {
                bail!("dispatch workspace requires a workspace name or index");
            }
            let ref_ = parse_workspace_reference(&cmd_args[0]);
            Ok(Action::FocusWorkspace { reference: ref_ })
        }
        "movetoworkspace" => {
            if cmd_args.is_empty() {
                bail!("dispatch movetoworkspace requires a workspace name or index");
            }
            let ref_ = parse_workspace_reference(&cmd_args[0]);
            Ok(Action::MoveWindowToWorkspace {
                window_id: None,
                reference: ref_,
                focus: true,
            })
        }
        "movetoworkspacesilent" => {
            if cmd_args.is_empty() {
                bail!("dispatch movetoworkspacesilent requires a workspace name or index");
            }
            let ref_ = parse_workspace_reference(&cmd_args[0]);
            Ok(Action::MoveWindowToWorkspace {
                window_id: None,
                reference: ref_,
                focus: false,
            })
        }
        "movewindow" => parse_movewindow(cmd_args),

        // ── Window Sizing ─────────────────────────────────────────────
        "setwindowwidth" => {
            if cmd_args.is_empty() {
                bail!("dispatch setwindowwidth requires a size");
            }
            Ok(Action::SetWindowWidth {
                id: None,
                change: parse_size_change(&cmd_args[0])?,
            })
        }
        "setwindowheight" => {
            if cmd_args.is_empty() {
                bail!("dispatch setwindowheight requires a size");
            }
            Ok(Action::SetWindowHeight {
                id: None,
                change: parse_size_change(&cmd_args[0])?,
            })
        }

        // ── Liquid-specific ───────────────────────────────────────────
        "togglespecialworkspace" | "togglescratchcolumn" => {
            let name = if cmd_args.is_empty() {
                // Use first scratch column name, or "default"
                "default".to_string()
            } else {
                cmd_args[0].clone()
            };
            Ok(Action::ToggleScratchColumn { name })
        }
        "setanimationprofile" => {
            if cmd_args.is_empty() {
                bail!("dispatch setanimationprofile requires a profile name");
            }
            Ok(Action::SetAnimationProfile {
                profile: cmd_args[0].clone(),
            })
        }
        "setmaterial" => {
            if cmd_args.is_empty() {
                bail!("dispatch setmaterial requires a material name");
            }
            Ok(Action::SetMaterial {
                material: cmd_args[0].clone(),
            })
        }
        "toggleactionpalette" => Ok(Action::ToggleActionPalette {}),
        "togglesafemode" | "safemode" => Ok(Action::ToggleSafeMode {}),

        // ── Column / Layout ───────────────────────────────────────────
        "togglecolumntabbed" => Ok(Action::ToggleColumnTabbedDisplay {}),

        "focuscolumnleft" => Ok(Action::FocusColumnLeft {}),
        "focuscolumnright" => Ok(Action::FocusColumnRight {}),
        "focuscolumnfirst" => Ok(Action::FocusColumnFirst {}),
        "focuscolumnlast" => Ok(Action::FocusColumnLast {}),
        "movecolumnleft" => Ok(Action::MoveColumnLeft {}),
        "movecolumnright" => Ok(Action::MoveColumnRight {}),
        "movecolumntofirst" => Ok(Action::MoveColumnToFirst {}),
        "movecolumntolast" => Ok(Action::MoveColumnToLast {}),
        "centercolumn" => Ok(Action::CenterColumn {}),
        "centerwindow" => Ok(Action::CenterWindow { id: None }),

        // ── Compositor ────────────────────────────────────────────────
        "quit" | "exit" => Ok(Action::Quit {
            skip_confirmation: false,
        }),
        "quitforce" => Ok(Action::Quit {
            skip_confirmation: true,
        }),
        "poweroffmonitors" => Ok(Action::PowerOffMonitors {}),
        "poweronmonitors" => Ok(Action::PowerOnMonitors {}),

        // ── Screenshot ────────────────────────────────────────────────
        "screenshot" => Ok(Action::Screenshot {
            show_pointer: true,
            path: None,
        }),
        "screenshotscreen" => Ok(Action::ScreenshotScreen {
            write_to_disk: true,
            show_pointer: true,
            path: None,
        }),
        "screenshotwindow" => Ok(Action::ScreenshotWindow {
            id: None,
            write_to_disk: true,
            show_pointer: false,
            path: None,
        }),

        _ => bail!(
            "unknown dispatch command: `{cmd}`\n\
             Try one of: exec, execr, killactive, closewindow, fullscreen, togglefloating, \
             movefocus, swapwindow, focusmonitor, workspace, movetoworkspace, \
             togglespecialworkspace, setanimationprofile, setmaterial, togglecolumntabbed, \
             centercolumn, centerwindow, movecolumnleft, movecolumnright, quit, exit"
        ),
    }
}

// ── Sub-parsers ──────────────────────────────────────────────────────────

fn parse_movefocus(args: &[String]) -> anyhow::Result<Action> {
    if args.is_empty() {
        bail!("dispatch movefocus requires a direction (l, r, u, d, left, right, up, down)");
    }
    match args[0].as_str() {
        "l" | "left" => Ok(Action::FocusColumnLeft {}),
        "r" | "right" => Ok(Action::FocusColumnRight {}),
        "u" | "up" => Ok(Action::FocusWindowUp {}),
        "d" | "down" => Ok(Action::FocusWindowDown {}),
        "first" => Ok(Action::FocusColumnFirst {}),
        "last" => Ok(Action::FocusColumnLast {}),
        dir => bail!(
            "unknown movefocus direction: `{dir}`. \
             Use l/left, r/right, u/up, d/down, first, or last"
        ),
    }
}

fn parse_swapwindow(args: &[String]) -> anyhow::Result<Action> {
    if args.is_empty() {
        bail!("dispatch swapwindow requires a direction (l, r, left, right)");
    }
    match args[0].as_str() {
        "l" | "left" => Ok(Action::SwapWindowLeft {}),
        "r" | "right" => Ok(Action::SwapWindowRight {}),
        dir => bail!("unknown swapwindow direction: `{dir}`. Use l/left or r/right"),
    }
}

fn parse_focusmonitor(args: &[String]) -> anyhow::Result<Action> {
    if args.is_empty() {
        bail!(
            "dispatch focusmonitor requires a direction (l, r, u, d, left, right, up, down, prev, next)"
        );
    }
    match args[0].as_str() {
        "l" | "left" => Ok(Action::FocusMonitorLeft {}),
        "r" | "right" => Ok(Action::FocusMonitorRight {}),
        "u" | "up" => Ok(Action::FocusMonitorUp {}),
        "d" | "down" => Ok(Action::FocusMonitorDown {}),
        "prev" | "previous" => Ok(Action::FocusMonitorPrevious {}),
        "next" => Ok(Action::FocusMonitorNext {}),
        name => Ok(Action::FocusMonitor {
            output: name.to_string(),
        }),
    }
}

fn parse_focuswindow(args: &[String]) -> anyhow::Result<Action> {
    if args.is_empty() {
        bail!("dispatch focuswindow requires a direction (up, down, top, bottom, previous)");
    }
    match args[0].as_str() {
        "u" | "up" => Ok(Action::FocusWindowUp {}),
        "d" | "down" => Ok(Action::FocusWindowDown {}),
        "top" => Ok(Action::FocusWindowTop {}),
        "bottom" => Ok(Action::FocusWindowBottom {}),
        "prev" | "previous" => Ok(Action::FocusWindowPrevious {}),
        "first" => Ok(Action::FocusWindowDownOrTop {}),
        "last" => Ok(Action::FocusWindowUpOrBottom {}),
        dir => bail!(
            "unknown focuswindow direction: `{dir}`. Use u/up, d/down, top, bottom, prev/previous"
        ),
    }
}

fn parse_movewindow(args: &[String]) -> anyhow::Result<Action> {
    if args.is_empty() {
        bail!("dispatch movewindow requires a direction (u, d, up, down, l, r, left, right)");
    }
    match args[0].as_str() {
        "u" | "up" => Ok(Action::MoveWindowUp {}),
        "d" | "down" => Ok(Action::MoveWindowDown {}),
        "l" | "left" => Ok(Action::ConsumeOrExpelWindowLeft { id: None }),
        "r" | "right" => Ok(Action::ConsumeOrExpelWindowRight { id: None }),
        dir => bail!("unknown movewindow direction: `{dir}`. Use u/up, d/down, l/left, r/right"),
    }
}

fn parse_workspace_reference(s: &str) -> niri_ipc::WorkspaceReferenceArg {
    use niri_ipc::WorkspaceReferenceArg;
    if let Ok(idx) = s.parse::<u8>() {
        WorkspaceReferenceArg::Index(idx)
    } else {
        WorkspaceReferenceArg::Name(s.to_string())
    }
}

fn parse_size_change(s: &str) -> anyhow::Result<niri_ipc::SizeChange> {
    use std::str::FromStr;
    niri_ipc::SizeChange::from_str(s).map_err(|e| {
        anyhow::anyhow!(
            "invalid size change: `{s}`: {e}. \
             Try e.g. `set 800`, `adjust +50`, `set-pct 50%`, `adjust-pct +10%`"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn debug_action(action: Action) -> String {
        format!("{action:?}")
    }

    #[test]
    fn dispatch_exec() {
        let action = parse_dispatch(&["exec".into(), "firefox".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::Spawn {
                command: vec!["firefox".to_string()]
            })
        );
    }

    #[test]
    fn dispatch_execr() {
        let action = parse_dispatch(&["execr".into(), "firefox --new-window".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::SpawnSh {
                command: "firefox --new-window".to_string()
            })
        );
    }

    #[test]
    fn dispatch_killactive() {
        let action = parse_dispatch(&["killactive".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::CloseWindow { id: None })
        );
    }

    #[test]
    fn dispatch_closewindow() {
        let action = parse_dispatch(&["closewindow".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::CloseWindow { id: None })
        );
    }

    #[test]
    fn dispatch_fullscreen() {
        let action = parse_dispatch(&["fullscreen".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::FullscreenWindow { id: None })
        );
    }

    #[test]
    fn dispatch_togglefloating() {
        let action = parse_dispatch(&["togglefloating".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::ToggleWindowFloating { id: None })
        );
    }

    #[test]
    fn dispatch_movefocus_left() {
        let action = parse_dispatch(&["movefocus".into(), "l".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::FocusColumnLeft {})
        );
    }

    #[test]
    fn dispatch_movefocus_right() {
        let action = parse_dispatch(&["movefocus".into(), "r".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::FocusColumnRight {})
        );
    }

    #[test]
    fn dispatch_movefocus_up() {
        let action = parse_dispatch(&["movefocus".into(), "u".into()]).unwrap();
        assert_eq!(debug_action(action), debug_action(Action::FocusWindowUp {}));
    }

    #[test]
    fn dispatch_movefocus_down() {
        let action = parse_dispatch(&["movefocus".into(), "d".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::FocusWindowDown {})
        );
    }

    #[test]
    fn dispatch_swapwindow() {
        let action = parse_dispatch(&["swapwindow".into(), "left".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::SwapWindowLeft {})
        );
    }

    #[test]
    fn dispatch_focusmonitor() {
        let action = parse_dispatch(&["focusmonitor".into(), "l".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::FocusMonitorLeft {})
        );
    }

    #[test]
    fn dispatch_workspace_by_name() {
        let action = parse_dispatch(&["workspace".into(), "dev".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::FocusWorkspace {
                reference: niri_ipc::WorkspaceReferenceArg::Name("dev".to_string())
            })
        );
    }

    #[test]
    fn dispatch_workspace_by_index() {
        let action = parse_dispatch(&["workspace".into(), "3".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::FocusWorkspace {
                reference: niri_ipc::WorkspaceReferenceArg::Index(3)
            })
        );
    }

    #[test]
    fn dispatch_movetoworkspace() {
        let action = parse_dispatch(&["movetoworkspace".into(), "dev".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::MoveWindowToWorkspace {
                window_id: None,
                reference: niri_ipc::WorkspaceReferenceArg::Name("dev".to_string()),
                focus: true
            })
        );
    }

    #[test]
    fn dispatch_movetoworkspacesilent() {
        let action = parse_dispatch(&["movetoworkspacesilent".into(), "dev".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::MoveWindowToWorkspace {
                window_id: None,
                reference: niri_ipc::WorkspaceReferenceArg::Name("dev".to_string()),
                focus: false
            })
        );
    }

    #[test]
    fn dispatch_togglespecialworkspace() {
        let action = parse_dispatch(&["togglespecialworkspace".into(), "terminal".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::ToggleScratchColumn {
                name: "terminal".to_string()
            })
        );
    }

    #[test]
    fn dispatch_togglespecialworkspace_default() {
        let action = parse_dispatch(&["togglespecialworkspace".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::ToggleScratchColumn {
                name: "default".to_string()
            })
        );
    }

    #[test]
    fn dispatch_setanimationprofile() {
        let action = parse_dispatch(&["setanimationprofile".into(), "slow".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::SetAnimationProfile {
                profile: "slow".to_string()
            })
        );
    }

    #[test]
    fn dispatch_setmaterial() {
        let action = parse_dispatch(&["setmaterial".into(), "frosted-ceramic".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::SetMaterial {
                material: "frosted-ceramic".to_string()
            })
        );
    }

    #[test]
    fn dispatch_quit() {
        let action = parse_dispatch(&["quit".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::Quit {
                skip_confirmation: false
            })
        );
    }

    #[test]
    fn dispatch_unknown() {
        let result = parse_dispatch(&["nonexistent".into()]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unknown dispatch command"));
    }

    #[test]
    fn dispatch_missing_args() {
        let result = parse_dispatch(&["exec".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn dispatch_movecolumn() {
        let action = parse_dispatch(&["movecolumnleft".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::MoveColumnLeft {})
        );
    }

    #[test]
    fn dispatch_focuscolumn() {
        let action = parse_dispatch(&["focuscolumnright".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::FocusColumnRight {})
        );
    }

    #[test]
    fn dispatch_center() {
        let action = parse_dispatch(&["centercolumn".into()]).unwrap();
        assert_eq!(debug_action(action), debug_action(Action::CenterColumn {}));

        let action = parse_dispatch(&["centerwindow".into()]).unwrap();
        assert_eq!(
            debug_action(action),
            debug_action(Action::CenterWindow { id: None })
        );
    }
}
