# niri-cidre Configuration Notes

This document summarizes the config writing style and fork-specific options for the local `niri-cidre` fork in this repository.

It is not a replacement for upstream niri docs. Treat it as:

- upstream niri config docs for the base syntax and standard sections
- this document for fork-only sections and a safe file layout for daily use

## Recommended File Layout

Keep upstream-compatible config separate from fork-only config.

Example:

```kdl
// ~/.config/niri/config.kdl
// Upstream-safe base config.
input {
    touchpad {
        tap
        dwt
        natural-scroll
        accel-speed 0.2
        accel-profile "adaptive"
    }
}
```

```kdl
// ~/.config/niri/config.cidre.kdl
include "./config.kdl"
include optional=true "./config.cidre.local.kdl"
```

```kdl
// ~/.config/niri/config.cidre.local.kdl
niri-liquid {
    config-version 1
}
```

Recommended usage:

- `config.kdl`: settings that should still validate in upstream `niri` and in packaged `/usr/bin/niri-cidre`
- `config.cidre.kdl`: entrypoint for the local build
- `config.cidre.local.kdl`: fork-only sections, experiments, recovery toggles

This works well with `niri --config ...` and lets you keep a stable upstream fallback.

## Top-Level Fork-Specific Nodes

The fork currently accepts these extra top-level nodes in addition to upstream niri sections:

- `niri-liquid`
- `safe-mode`
- `action-palette`
- `mode-hud`
- `overlays`
- `dispatch-alias`
- `rule`
- `material-graph`
- `animation-graph`
- `script`
- `performance-budget`

It also extends some upstream sections with additional values:

- `gestures { gesture-edge ... }` accepts fork actions such as `toggle-scratch-column`, `set-animation-profile`, and `set-material`
- `window-rule` and `layer-rule` `background-effect {}` accept extra liquid-related properties
- `material`
- `effect-preset`
- `animation-profile`
- `scratch-column`

## Boolean Style

KDL flags in this config are usually written as bare child nodes, not `true/false` pairs.

Examples:

```kdl
safe-mode {
    enable
    disable-scripts
    disable-gestures
}
```

Some options do take explicit values:

```kdl
background-effect {
    liquid true
    blur true
    refraction 0.03
}
```

## Practical Baseline for niri-cidre

This is a good starting point for the local fork:

```kdl
niri-liquid {
    config-version 1
}

safe-mode {
    enable
    bind "Mod+Shift+Backspace"
    material "safe-solid"
    animation-profile "safe"
    disable-scripts
    disable-gestures
    disable-expensive-effects
}

action-palette {
    enable
    bind "Mod+P"
    material "dashboard-glass"
    fuzzy-search
    show-keybinds
    show-current-state
}

mode-hud {
    enable
    position "top-center"
    duration-ms 900
    material "hud-glass"
    show {
        animation-profile
        material
        performance-profile
        scratch-state
    }
}

overlays {
    default-material "dashboard-glass"
    animation "glass-sheet"
}

performance-budget {
    target-fps 60
    max-blur-passes 3
    disable-dispersion-on-battery
}

binds {
    Mod+Shift+Backspace { toggle-safe-mode; }
}
```

## Section Reference

### `niri-liquid`

Top-level fork feature flags and config schema versioning.

```kdl
niri-liquid {
    config-version 1
    // disable-scripts
    // disable-gestures
    // disable-expensive-effects
    // disable-liquid-materials
}
```

Options:

- `config-version <u32>`
- `disable-scripts`
- `disable-gestures`
- `disable-expensive-effects`
- `disable-liquid-materials`

Use this as the root capability toggle block for the fork.

### `safe-mode`

Emergency fallback mode. Intended for recovery from bad scripts, broken effects, or unstable gesture experiments.

```kdl
safe-mode {
    enable
    bind "Mod+Shift+Backspace"
    material "safe-solid"
    animation-profile "safe"
    disable-scripts
    disable-gestures
    disable-expensive-effects
}
```

Options:

- `enable`
- `bind "<keycombo>"`
- `material "<material-name>"`
- `animation-profile "<profile-name>"`
- `disable-scripts`
- `disable-gestures`
- `disable-expensive-effects`

Notes:

- The practical runtime bind is still added through normal `binds {}`.
- Keep this enabled in any daily-driver config.

### `action-palette`

Fork overlay for discoverable actions.

```kdl
action-palette {
    enable
    bind "Mod+P"
    material "dashboard-glass"
    fuzzy-search
    show-keybinds
    show-current-state
}
```

Options:

- `enable`
- `bind "<keycombo>"`
- `material "<material-name>"`
- `fuzzy-search`
- `show-keybinds`
- `show-current-state`

When enabled, the parser also installs the bind automatically if `bind` is set.

### `mode-hud`

Fork HUD for current compositor state.

```kdl
mode-hud {
    enable
    position "top-center"
    duration-ms 900
    material "hud-glass"
    show {
        animation-profile
        material
        performance-profile
        scratch-state
    }
}
```

Options:

- `enable`
- `position "<string>"`
- `duration-ms <u64>`
- `material "<material-name>"`
- `show { ... }`

`show {}` flags:

- `animation-profile`
- `material`
- `performance-profile`
- `scratch-state`

### `overlays`

Defaults for fork UI overlays.

```kdl
overlays {
    default-material "dashboard-glass"
    animation "glass-sheet"
}
```

Options:

- `default-material "<material-name>"`
- `animation "<animation-name>"`

### `performance-budget`

Quality fallback controls for liquid effects.

```kdl
performance-budget {
    target-fps 60
    max-blur-passes 3
    disable-dispersion-on-battery
    // downgrade-material-on-frame-drop
}
```

Options:

- `target-fps <u32>`
- `max-blur-passes <u8>`
- `disable-dispersion-on-battery`
- `downgrade-material-on-frame-drop`

Current default quality ladder in code:

- `hologram-film`
- `liquid-mocha`
- `acrylic-smoke`
- `safe-solid`

### `script`

Fork script engine configuration.

```kdl
script {
    enable
    directory "~/.config/niri-liquid/scripts"
}
```

Options:

- `enable`
- `directory "<path>"`

Recommendation:

- leave this off by default on a daily-driver session until the scripts are stable
- safe mode should disable it

### `dispatch-alias`

Named command bundles for repeated dispatch sequences.

```kdl
dispatch-alias "battery-quiet" {
    dispatch "setanimationprofile" "battery"
    dispatch "setmaterial" "safe-solid"
}
```

This is useful for binding one conceptual mode change to several fork actions.

### `rule`

Unified fork rule layer. This is separate from upstream `window-rule` and `layer-rule`.

```kdl
rule "ghostty-focus" {
    target "window"
    priority 50
    match app-id="ghostty"
    apply {
        material "obsidian-glass"
        animation-profile "focus"
        opacity 0.96
    }
}
```

Options:

- `target "window" | "layer" | "workspace" | "column" | "special-workspace" | "output"`
- `priority <i32>`
- `match ...`
- `apply { ... }`

Currently implemented `apply` fields:

- `workspace "<name>"`
- `material "<name>"`
- `animation-profile "<name>"`
- `effect-preset "<name>"`
- `floating`
- `opacity <0..1>`
- `column-display "<string>"`

Recommendation:

- use this only after the base session is stable
- prefer upstream `window-rule` and `layer-rule` where possible

## Extended Existing Sections

### `gestures { gesture-edge ... }`

The fork extends gesture-edge actions.

```kdl
gestures {
    gesture-edge "left" {
        fingers 3
        action "toggle-scratch-column terminal"
        reveal-ratio 0.85
        interactive true
        progress-map {
            target "view"
            translate-x "0..24"
            opacity "0.85..1.0"
            blur "0.0..1.0"
            refraction "0.0..0.03"
            specular "0.0..0.12"
            chromatic-aberration "0.0..0.02"
        }
    }
}
```

Supported fork action strings parsed here:

- `toggle-scratch-column <name>`
- `set-animation-profile <name>`
- `set-material <name>`
- `focus-workspace <index-or-name>`

Useful fields:

- `fingers <u8>`
- `action "<string>"`
- `reveal-ratio <0..1>`
- `interactive true|false`
- `progress-map { ... }`

### `scratch-column`

Scratch columns are first-class multipart config entries.

```kdl
scratch-column "terminal" {
    width 0.50
    position "bottom"
    animation "popin"
    monitor "eDP-1"
    close-on-focus-loss
}
```

Options:

- `width <number>`
- `position "<string>"`
- `animation "<string>"`
- `monitor "<output-name>"`
- `close-on-focus-loss`

### `background-effect` in `window-rule` and `layer-rule`

The fork adds liquid-specific properties on top of upstream blur/xray behavior.

```kdl
window-rule {
    match app-id="ghostty"
    background-effect {
        blur true
        xray true
        liquid true
        refraction 0.03
        edge-highlight 0.12
        specular 0.10
        chromatic-aberration 0.02
        foreground-liquid true
        foreground-refraction 0.02
        foreground-chromatic-aberration 0.01
        bloom 0.08
    }
}
```

Fork-specific additions:

- `liquid`
- `refraction`
- `edge-highlight`
- `specular`
- `chromatic-aberration`
- `foreground-liquid`
- `foreground-refraction`
- `foreground-chromatic-aberration`
- `bloom`

### `material`

The fork extends materials into a liquid pipeline description.

```kdl
material "liquid-mocha" {
    blur {
        passes 6
        offset 6.0
    }
    tint "rgba(30, 30, 46, 0.17)"
    saturation 1.8
    noise 0.01
    bloom 0.08

    refraction {
        strength 0.035
        edge-strength 0.06
        normal-noise 0.012
    }

    dispersion {
        strength 0.02
        red-offset 0.006
        blue-offset -0.006
    }

    specular {
        strength 0.15
        angle 45.0
        width 0.2
    }

    edge-highlight {
        color "rgba(255, 255, 255, 0.22)"
        width 1.0
    }
}
```

Sections and fields:

- `blur { passes, offset }`
- `tint`
- `saturation`
- `noise`
- `bloom`
- `refraction { strength, edge-strength, normal-noise }`
- `dispersion { strength, red-offset, blue-offset }`
- `specular { strength, angle, width }`
- `edge-highlight { color, width }`
- `debug { show-bounds, show-damage, show-layer, show-material-id, show-animation-state }`

### `effect-preset`

Shortcut wrapper for applying visual bundles.

```kdl
effect-preset "floating-glass" {
    material "liquid-mocha"
    corner-radius 14
    shadow "soft"
    border "mauve-sapphire"
}
```

### `animation-profile`

Profiles group named animation presets by usage site.

```kdl
animation-profile "battery" {
    window-open "fast-fade"
    window-close "fast-fade"
    layer-open "slide-glass"
    layer-close "fade-slide"
    overview-open "zoom-out"
    workspace "calm"
}
```

Fields:

- `window-open`
- `window-close`
- `layer-open`
- `layer-close`
- `overview-open`
- `workspace`

## Experimental Graph Sections

These are implemented in config parsing, but should be treated as experimental until the runtime side is clearly frozen.

### `material-graph`

```kdl
material-graph "glass-stack" {
    node "backdrop-blur" {
        passes 4
        offset 5.0
    }
    node "tint" {
        color "#1e1e2e"
        alpha 0.24
    }
    node "refraction" {
        strength 0.012
    }
}
```

Supported node types:

- `backdrop-blur`
- `tint`
- `saturation`
- `noise`
- `refraction`
- `dispersion`
- `rim-light`
- `debug-wireframe`

### `animation-graph`

```kdl
animation-graph "workspace-pull" {
    curve "spring-soft" {
        curve-type "spring"
        stiffness 380.0
        damping 32.0
        mass 1.0
    }

    node "pull" {
        curve "spring-soft"
        duration-ms 240
        transform "translate"
        input "gesture-progress"
        output {
            translate-y "0..18"
            opacity "0.92..1.0"
            blur "0.0..1.0"
            specular "0.0..0.12"
        }
    }
}
```

Treat this as an experimental authoring surface, not a stable public interface yet.

## Validation Workflow

Validate the upstream-safe base:

```bash
/usr/bin/niri-cidre validate -c ~/.config/niri/config.kdl
```

Validate the fork entrypoint:

```bash
~/Projects/niri/target/release/niri-cidre validate -c ~/.config/niri/config.cidre.kdl
```

Recommended workflow:

1. Keep `config.kdl` passing upstream validation.
2. Put fork-only syntax into `config.cidre.local.kdl`.
3. Validate the fork config with the local build before restarting the session.
4. Keep `safe-mode` enabled and bound at all times.

## Operational Advice

- Do not put fork-only nodes directly into `config.kdl` unless you intentionally give up upstream compatibility.
- Keep `script` disabled unless you are actively testing scripts.
- Introduce `material`, `rule`, `material-graph`, and `animation-graph` incrementally.
- Prefer a recovery-first workflow: validate, restart session, test, then expand.
- For a daily-driver session, `safe-mode` and `performance-budget` are more important than visual complexity.

## Pointers

- Upstream base config docs: `docs/wiki/Configuration:-Introduction.md`
- Include behavior: `docs/wiki/Configuration:-Include.md`
- Liquid fork parser/types: `niri-config/src/liquid.rs`
- Gesture extension parser: `niri-config/src/gestures.rs`
- Material/effect extensions: `niri-config/src/appearance.rs`
- Animation profile extensions: `niri-config/src/animations.rs`
