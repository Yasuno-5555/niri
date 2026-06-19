# Cidre v1.0 Release Plan

This document is the working checklist for preparing the first public `Cidre v1.0` release.

Scope is now fixed:

- `Cidre v1.0` is a full developer environment release
- target platform is Apple Silicon Macs on Asahi ALARM / Arch Linux ARM
- `niri-cidre` is the standard desktop/compositor component inside Cidre
- release quality means "daily-drivable on supported hardware with clear recovery paths"

See also: `docs/cidre-v1-scope.md`

## Release Goals

Cidre v1.0 should communicate one clear promise:

> Cidre turns Apple Silicon MacBook hardware into a Linux-based developer workstation.

That promise needs to hold across:

- branding
- documentation
- install flow
- runtime defaults
- recovery path
- release artifacts

## Release Tracks

### 1. Product Definition

`v1.0` scope is no longer open. What remains is documenting and enforcing it.

Checklist:

- [x] Freeze the v1.0 support target
- [x] Define supported Apple Silicon models
- [x] Define supported distro/base story
- [x] Define whether v1.0 is "compositor release" or "full environment release"
- [ ] Define required companion repos or dotfiles
- [x] Define non-goals and unsupported workflows

Fixed decisions:

- `Cidre v1.0` is a full developer environment release
- target base is Asahi ALARM / Arch Linux ARM
- `niri-cidre` is a component, not the product
- `greetd + greetd-tuigreet` is the default login stack owned by the desktop profile

Still to resolve:

- whether installation is manual, scripted, or image-based
- how companion repos are split (`cidre-meta`, `cidre-config`, `cidre-session`, installer)

### 2. Branding and Naming

The current repository still exposes upstream `niri` naming in many places.

Checklist:

- [x] Decide public naming: `Cidre`, `niri-cidre`, or both
- [ ] Audit `README.md` for upstream-first wording
- [x] Audit `Cargo.toml` metadata
- [x] Audit `.desktop`, service, and session names
- [ ] Audit docs titles and screenshots
- [x] Freeze the public runtime binary as `niri-cidre`

Public split:

- Product name: `Cidre`
- Compositor implementation name: `niri-cidre`
- Runtime binary name: `niri-cidre`
- Session launcher: `cidre-session`

### 3. Packaging and Install Story

Public releases need one install path that is actually supportable.

Checklist:

- [ ] Decide package distribution format
- [ ] Decide whether users install from source or packages
- [ ] Decide whether release artifacts are per-arch or source-only
- [x] Audit `package.metadata.generate-rpm`
- [x] Audit `package.metadata.deb`
- [x] Add Arch/ALARM-oriented installation documentation
- [x] Document required runtime dependencies
- [x] Document rollback path

Current mismatch to fix:

- Package metadata still describes upstream `niri`
- Release assets do not yet reflect a broader `Cidre` story

### 4. Session and Config Story

The release needs a stable answer for how the compositor is started and configured.

Checklist:

- [ ] Freeze the `cidre-session` strategy
- [ ] Freeze systemd user override behavior
- [x] Document the recommended config file layout
- [ ] Keep upstream-safe config separate from fork-only config
- [ ] Ensure `safe-mode` is always available
- [x] Document recovery commands

Current baseline:

- `config.kdl` stays upstream-compatible
- `config.cidre.kdl` is the Cidre entrypoint
- `config.cidre.local.kdl` carries fork-only config
- `greetd + greetd-tuigreet` is the standard login entry path
- official session shape should be `cidre-session -> cidre.service -> niri-cidre`

### 5. User-Facing Documentation

Public release quality requires docs that answer first-run questions fast.

Checklist:

- [x] Add "What is Cidre?" page
- [x] Add install guide
- [x] Add supported hardware page
- [x] Add recovery and safe mode page
- [x] Add configuration guide for `niri-cidre`
- [ ] Add migration guide from upstream niri or macOS expectations
- [x] Add known limitations page
- [x] Add issue reporting guide for Cidre-specific bugs

Documents already started:

- `README.md`
- `docs/niri-cidre-config.md`
- `docs/cidre-v1-scope.md`
- `docs/cidre-v1-package-plan.md`

Documents still missing:

- migration guide from upstream niri or macOS expectations

### 6. Quality Gates

Before calling it `v1.0`, the release should pass a fixed validation set.

Checklist:

- [ ] Clean build from a fresh checkout
- [ ] Config validation passes on shipped example config
- [x] Session starts successfully via `cidre-session`
- [ ] Safe mode entry works
- [ ] Scratch column workflow works
- [ ] Touchpad gesture telemetry works
- [ ] Power-profile IPC path works
- [ ] Basic screenshot flow works
- [ ] No obvious startup regressions on supported hardware

Suggested validation buckets:

- startup/session
- config/reload
- input/gestures
- power behavior
- recovery behavior
- packaging/install

### 7. Release Artifacts

Decide exactly what ships with `v1.0`.

Checklist:

- [ ] Source tarball or tagged git release
- [ ] release notes
- [ ] default config examples
- [ ] packaged session files
- [ ] recovery instructions
- [ ] screenshots or demo assets

Minimum acceptable public release payload:

- tagged source
- README with clear project positioning
- install instructions
- config docs
- known limitations
- release notes

### 8. Repository Hygiene

The repository should look intentional before public release.

Checklist:

- [ ] Audit stale upstream references
- [ ] Audit version strings
- [ ] Audit package descriptions
- [ ] Audit generated assets and screenshots
- [ ] Add Cidre-specific issue labels or templates if needed
- [ ] Decide branch/tag naming for v1.0

## Immediate Priorities

These should happen first because they unblock everything else.

1. Decide install/distribution story.
2. Finish the public-facing README and core docs.
3. Define the validation checklist for supported hardware.
4. Align packaging metadata with the actual release story.
5. Split or define companion deliverables (`cidre-meta`, `cidre-config`, `cidre-session`, installer).

## Recommended Next Deliverables

In order:

1. `docs/cidre-install.md`
2. `docs/cidre-supported-hardware.md`
3. `docs/cidre-recovery.md`
4. `docs/cidre-known-limitations.md`
5. `docs/cidre-v1-release-notes-draft.md`

## Notes on Current State

As of now:

- `README.md` has started moving toward Cidre positioning
- `docs/niri-cidre-config.md` exists and documents fork-specific config
- `docs/cidre-v1-scope.md` now freezes the public scope
- shipped config validates with `/usr/bin/niri-cidre`
- the standard Cidre login path now returns successfully through `greetd -> cidre-session -> niri-cidre` on the primary development machine
- package metadata and several project strings still reflect upstream `niri`
- the install/release mechanics are not yet frozen

This means the project has moved from "scope discovery" into "release shaping", but is not yet in a release-candidate phase.
