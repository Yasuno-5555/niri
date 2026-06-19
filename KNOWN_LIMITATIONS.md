# Cidre Known Limitations

This document defines the current limits of `Cidre v1.0`.

It exists to make the support boundary explicit before users discover it the hard way.

## Scope Reminder

Cidre v1.0 is:

- an Apple Silicon Mac developer environment
- built for Asahi Linux on Arch Linux ARM / ALARM
- centered on `pacman`, `paru`, `AUR`, and `niri-cidre`

Cidre v1.0 is not:

- a macOS replacement
- a generic Linux distribution for all hardware
- a compositor-only project

## Platform Limitations

### Asahi Fedora Is Not Supported

Cidre v1.0 does not currently target Asahi Fedora.

Reason:

- package assumptions are Arch Linux ARM / ALARM specific
- the current package story depends on `pacman`, `paru`, and AUR workflows

This does not mean Fedora is bad. It means it is outside the product boundary.

### Apple Silicon Coverage Is Still Narrow

Cidre is aimed at Apple Silicon Macs, but the practical support promise is narrower than "all Apple Silicon Macs work equally well".

Current reality:

- strongest confidence is on the development hardware class
- nearby M1 MacBook systems may work well
- broader hardware coverage is not yet frozen

See [SUPPORTED_HARDWARE.md](./SUPPORTED_HARDWARE.md).

### External USB-C Display Support Is Not Yet Reliable

On the primary development machine class, external display behavior should currently be treated as constrained by upstream Asahi display support rather than guaranteed by Cidre.

Current observed status on the tested `Apple MacBook Air (M1, 2020)`:

- internal display path works for the standard Cidre session
- a direct USB-C to HDMI external display test did not come up under Asahi ALARM
- the same cable/display path was confirmed separately on another Arch Linux machine, which narrows the failure toward the Asahi-side display stack rather than the monitor or HDMI cable alone

Interpretation:

- Cidre should not currently promise reliable external USB-C display support on the tested M1 Air baseline
- failures in this area may be upstream Asahi limitations, adapter/path compatibility issues, or other platform-level display pipeline problems rather than `niri-cidre` configuration mistakes

Until this changes, external display support should be documented as a known limitation, not as a guaranteed part of the v1.0 desktop story.

### Cidre Depends On Upstream Asahi Progress

Some hardware behavior is ultimately gated by upstream Asahi support rather than by Cidre alone.

Examples:

- boot chain behavior
- suspend/resume quirks
- display issues
- model-specific audio behavior
- firmware/platform integration edge cases

Cidre can improve the environment around these, but cannot wish them into existence.

## Install and Packaging Limitations

### Install Flow Is Still Manual

Cidre v1.0 does not yet ship a finished `cidre-installer`.

Current state:

- install flow is manual
- package groups are documented, not yet productized as final meta-packages
- some session naming and packaging details still need cleanup

See [INSTALL.md](./INSTALL.md) and [docs/cidre-v1-package-plan.md](./docs/cidre-v1-package-plan.md).

### Package Profiles Are Defined, But Not Yet Finalized As Public Artifacts

The intended package model is:

- `core`
- `desktop`
- `dev`
- `diagnostics`
- `optional apps`

But:

- these are still planning-level groupings
- `cidre-meta-*` packages are not yet fully realized release artifacts

### Session Naming Is Still In Transition

Today, some pieces still use upstream `niri` names:

- session files
- service names
- binary names

The intended product structure is `Cidre` with `niri-cidre` as a component, but the implementation is not fully renamed end-to-end yet.

## Desktop and UX Limitations

### `niri-cidre` May Diverge From Upstream `niri`

Cidre uses `niri-cidre` as its standard desktop component.

That means:

- some behavior may differ from upstream `niri`
- some config examples from upstream may not map perfectly to Cidre workflows
- some bugs will be Cidre-specific and not reproducible upstream

This is expected. A fork with product goals is allowed to be a fork.

### Upstream Compatibility Is Preserved Where Practical, Not Absolute

Cidre currently prefers this config model:

- `config.kdl` stays upstream-safe
- `config.cidre.kdl` is the Cidre entrypoint
- `config.cidre.local.kdl` carries fork-only behavior

This reduces merge pain, but does not eliminate it.

If you heavily customize:

- expect to maintain local config
- expect some drift from upstream examples
- expect fork-only features to require fork-only docs

### Some Cidre UX Pieces Are Still Under Construction

Depending on the exact local setup, Cidre may include components that are still evolving:

- Quickshell-based shell UI
- power-aware desktop policy
- gesture-linked shell animations
- scratch-column workflows

These are core to the project direction, but not all of them should be assumed fully frozen yet.

## Workflow Limitations

### Cidre Is Not For Pure GUI-Only Use

Cidre assumes comfort with:

- terminal usage
- package management
- editing config files
- session recovery from a TTY

If you want a system where everything is managed through GUI settings panels, Cidre is the wrong product.

### Cidre Is Not For Apple Ecosystem Lock-In Workflows

Cidre is a poor fit if your primary workflow depends on:

- iCloud
- AirDrop
- Adobe Creative Cloud
- Logic Pro
- Final Cut Pro

This is not ideology. It is product boundary.

### Cidre Is Not A "Mac Theme For Linux"

Cidre is not trying to imitate macOS visually and stop there.

The project is about:

- development workflow
- defaults
- system integration
- recovery
- input and power behavior

If you only want "Linux that looks like macOS", Cidre is solving a different problem.

## Operational Limitations

### Recovery Is Part Of Normal Operation

Cidre v1.0 expects that advanced users may need recovery steps.

This means:

- TTY recovery is in scope
- `greetd` recovery is in scope
- config rollback is in scope
- snapshot rollback is in scope

If that is unacceptable, Cidre is not the right daily-driver target yet.

See [RECOVERY.md](./RECOVERY.md).

### Local Builds Are Part Of The Current Development Reality

At the moment, the practical workflow may involve:

- local compositor builds
- user systemd overrides
- local config layering

This is acceptable for the project phase, but it is also a limitation of current polish.

### Hardware-Specific Tuning Means Support Debt

Some of Cidre's value comes from model-aware tuning:

- touchpad feel
- audio path handling
- power behavior
- animation policy

This is good for quality on supported hardware, but it also means broad claims should be treated skeptically until validated.

## Documentation Limitations

### Not All Public Docs Are Frozen Yet

Core docs now exist, but the overall public documentation surface is still being shaped.

What exists:

- `README.md`
- `INSTALL.md`
- `SUPPORTED_HARDWARE.md`
- `RECOVERY.md`
- `docs/cidre-v1-scope.md`
- `docs/cidre-v1-package-plan.md`
- `docs/niri-cidre-config.md`

What still needs continued hardening:

- install polish
- release notes
- issue reporting guidance
- clearer cross-repo/component boundaries

## Honest Summary

The current honest summary of Cidre v1.0 is:

- the product direction is clear
- the target platform is clear
- the package baseline is mostly clear
- the recovery model is clear
- the installation and release mechanics are still maturing
- the support matrix is still conservative by design

That is a much healthier position than pretending the project is broader or more polished than it is.
