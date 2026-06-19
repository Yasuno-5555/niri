# Cidre v0.1.0 Release Notes

This document is the release text baseline for the first public Cidre tag.

## Cidre v0.1.0

The cider after Homebrew.

Cidre `v0.1.0` is the first public Cidre release tag.

It is an early public milestone for an opinionated Arch Linux ARM developer environment on Apple Silicon Macs.

It is built for Asahi Linux on Arch Linux ARM / ALARM and ships `niri-cidre` as its standard desktop/compositor component.

This is not presented as a fully hardened `v1.0.0` release. It is a real public snapshot of the current Cidre direction, documentation, packaging work, and tested machine baseline.

## What Cidre Is

Cidre is for people who:

- like Apple Silicon Mac hardware
- want Linux for development instead of macOS
- prefer `pacman`, `paru`, and AUR workflows
- want a coherent workstation instead of rebuilding a setup from scratch

Cidre is not trying to be:

- a macOS clone
- a generic Linux distro for all hardware
- a Fedora-first Asahi product

## Highlights

- fixed product scope for Apple Silicon developer use
- `niri-cidre` as the standard desktop component
- `cidre-session` and `cidre-config` split out as first-class companion packages
- documented install flow
- documented hardware support tiers
- documented recovery path
- documented package profile model
- documented fork-specific configuration structure

## Runtime Direction

Current Cidre direction:

- base: Asahi Linux on Arch Linux ARM / ALARM
- package management: `pacman + paru`
- desktop: `niri-cidre`
- terminal: `Ghostty`
- shell: `fish`
- GUI editor: `Zed`
- terminal editor: `Helix`
- fallback: `foot + bash`

## Package Profiles

Standard install profiles:

- `core`
- `desktop`
- `dev`

Optional profiles:

- `diagnostics`
- `optional apps`

Companion packages currently defined in-tree:

- `niri-cidre`
- `cidre-session`
- `cidre-config`
- `cidre-meta-core`
- `cidre-meta-desktop`
- `cidre-meta-dev`
- `cidre-meta-diagnostics`
- `cidre-meta-optional`

## Validation Status

Validated on the primary development machine so far:

- boot back into the standard Cidre environment succeeds
- `greetd -> cidre-session -> niri-cidre` login path works
- shipped `config.cidre.kdl` validates with `/usr/bin/niri-cidre`
- audio output works on the primary test machine
- Wi-Fi connectivity works on the primary test machine
- touchpad workflow is usable in the live Cidre session
- `Ghostty + fish` is functioning as the default interactive session path
- `Zed` and `Helix` launch cleanly
- clipboard basics work in the live session

Still pending before a confident `v1.0.0` claim:

- audio input sanity pass
- Bluetooth sanity pass
- suspend/resume characterization
- clipboard/portal/screenshot user flow pass
- recovery and safe-mode dry run

## Known Limits

- Asahi Fedora is not supported
- hardware support is still conservative
- install mechanics are still more manual than ideal
- some naming and packaging cleanup is still underway

See:

- [SUPPORTED_HARDWARE.md](../SUPPORTED_HARDWARE.md)
- [KNOWN_LIMITATIONS.md](../KNOWN_LIMITATIONS.md)

## Documentation

Release-critical docs:

- [README.md](../README.md)
- [INSTALL.md](../INSTALL.md)
- [SUPPORTED_HARDWARE.md](../SUPPORTED_HARDWARE.md)
- [RECOVERY.md](../RECOVERY.md)
- [CONFIGURATION.md](../CONFIGURATION.md)
- [KNOWN_LIMITATIONS.md](../KNOWN_LIMITATIONS.md)

## Release Position

`v0.1.0` should be read as:

- public and usable on the primary tested machine class
- honest about the current support boundary
- packaging-first in direction, but not yet a polished one-command install product
- clearly short of the final `v1.0.0` quality bar
