# Cidre Issue Reporting Guide

This document explains how to report problems without mixing together Cidre, `niri-cidre`, upstream `niri`, Asahi platform issues, and local breakage.

That split matters. A vague "Cidre is broken" report is usually not actionable.

## Start With Scope

Before filing anything, identify which layer is actually failing.

Use this rough split:

- `Cidre` product issue
  - install flow
  - package profile expectations
  - documented support boundary
  - recovery guidance
- `niri-cidre` compositor issue
  - fork-specific desktop behavior
  - session integration
  - gesture behavior
  - power-aware or shell-adjacent behavior
- upstream `niri` issue
  - baseline compositor behavior that also reproduces outside Cidre
- Asahi / platform issue
  - boot chain
  - kernel/platform support
  - device-specific hardware behavior
- local configuration issue
  - custom config
  - broken local build
  - personal overrides
  - unsupported package mix

## Quick Triage Questions

Answer these before you file a report:

1. Does it reproduce with the packaged `/usr/bin/niri-cidre`?
2. Does it reproduce only with your local `~/Projects/niri/target/release/niri-cidre` build?
3. Does it reproduce only with `config.cidre.local.kdl` enabled?
4. Does it reproduce on the tested hardware class, or only on another model?
5. Did it start after a package update, config edit, or compositor rebuild?

If you cannot answer those, your first step is probably local triage, not issue filing.

## What To Include

Every useful report should include:

- hardware model
- whether you are on Asahi Linux with Arch Linux ARM / ALARM
- whether you are using packaged `niri-cidre` or a local build
- whether `cidre-session` is in use
- whether a `cidre.service` override is present
- whether the problem reproduces with `config.cidre.local.kdl` disabled
- exact failure symptoms
- exact reproduction steps

Useful command output:

```bash
uname -a
systemctl --user cat cidre.service
systemctl --user status cidre.service
/usr/bin/niri-cidre --version
~/Projects/niri/target/release/niri-cidre --version
pacman -Q niri-cidre cidre-config cidre-session 2>/dev/null
```

If the issue is session startup related, also include relevant logs:

```bash
journalctl --user -b -u cidre.service
journalctl --user -b | rg -i 'niri|cidre|greetd|portal|pipewire'
```

## File It In The Right Place

File or route the issue based on where it reproduces.

### Likely Cidre issue

Use this when the problem is about:

- install steps
- documented package set
- session wiring
- Cidre-specific config layering
- recovery instructions
- supported hardware claims

### Likely `niri-cidre` issue

Use this when the problem is about:

- fork-only compositor behavior
- Cidre-specific gestures
- scratch-column workflow
- power-aware rendering behavior
- fork-specific config options

### Likely upstream `niri` issue

Use upstream when:

- it reproduces with upstream-style config
- it reproduces with packaged `niri`
- the behavior is clearly not tied to the Cidre fork

Upstream project:

- https://github.com/niri-wm/niri

### Likely Asahi/platform issue

Use platform-specific channels when:

- the compositor is not the real root cause
- the issue is device support, suspend, firmware, audio platform, or boot related

## When Not To File Yet

Do not file a new public bug first if:

- your local build does not compile
- your `override.conf` points to a stale path
- your custom config fails validation
- you changed package sets in a way outside the documented Cidre baseline

First restore the supported baseline:

- disable `config.cidre.local.kdl`
- disable the `cidre.service` override
- validate `~/.config/niri/config.kdl`
- try the packaged compositor path

If it still reproduces, then file it.

## Support Boundary Reminder

Cidre v1.0 currently under-promises on purpose.

That means:

- Apple Silicon Mac support is conservative
- Asahi Fedora is out of scope
- local heavy customization reduces supportability fast

Reports outside the documented boundary can still be useful, but they should be labeled as best-effort rather than release-blocking.
