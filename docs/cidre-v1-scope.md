# Cidre v1.0 Scope

This document freezes the public scope of `Cidre v1.0`.

## Product Definition

`Cidre v1.0` is not a compositor-only release.

`Cidre v1.0` is a full developer environment for Apple Silicon Macs.

Canonical statement:

> Cidre v1.0 is an opinionated Arch Linux ARM developer environment for Apple Silicon Macs.  
> `niri-cidre` is the desktop/compositor component shipped as part of Cidre.

Japanese working statement:

> Cidre v1.0 は、Apple Silicon Mac を pacman/AUR ベースの Linux 開発機として使うための、Asahi ALARM / Arch Linux ARM 向け開発環境である。  
> `niri-cidre` は、その標準デスクトップ体験を担う構成要素である。

## Target Platform

Cidre v1.0 targets:

- Apple Silicon Macs
- Asahi Linux on Arch Linux ARM / ALARM

More explicit technical wording:

- built for Asahi ALARM / Arch Linux ARM
- not currently targeting Asahi Fedora

Recommended wording by context:

- marketing copy: `Apple Silicon Macs` / `Asahi Linux`
- README overview: `Asahi Linux on Arch Linux ARM / ALARM`
- install guide: `Asahi ALARM / Arch Linux ARM`
- package metadata: `Arch Linux ARM / aarch64`

## Product Structure

Recommended public structure:

```text
Cidre
├─ cidre-installer
├─ cidre-meta
├─ cidre-config
├─ cidre-session
└─ niri-cidre
```

Role split:

- `Cidre`: product name, distribution unit, public brand
- `niri-cidre`: compositor / desktop component
- `cidre-session`: official login/session wrapper
- `cidre-meta`: standard package bundle definitions
- `cidre-config`: shipped config and defaults
- `cidre-installer`: installation entrypoint

## Standard Runtime

Cidre v1.0 standard runtime:

- package manager: `pacman + paru`
- default desktop: `niri-cidre`
- default login stack: `greetd + greetd-tuigreet`
- main terminal: `Ghostty`
- main interactive shell: `fish`
- fallback terminal: `foot`
- fallback shell: `bash`
- GUI editor: `Zed`
- terminal editor: `Helix`

## Standard Package Profiles

Cidre v1.0 package profile model:

- required: `core`
- required: `desktop`
- required: `dev`
- optional: `diagnostics`
- optional: `optional apps`

Recommended meta-package naming:

- `cidre-meta-core`
- `cidre-meta-desktop`
- `cidre-meta-dev`
- `cidre-meta-diagnostics`
- `cidre-meta-optional`

Recommended standard install:

- `cidre-meta-core`
- `cidre-meta-desktop`
- `cidre-meta-dev`

`cidre-meta-desktop` should include the default login path, not only desktop runtime packages.

Minimum login/session ownership:

- `greetd`
- `greetd-tuigreet`
- Cidre wayland session entry
- `cidre-session`
- `cidre.service`
- `cidre-shutdown.target`

## Config and Session Policy

Cidre v1.0 should officially document this three-layer config structure:

- `config.kdl`: upstream-compatible base
- `config.cidre.kdl`: Cidre entrypoint
- `config.cidre.local.kdl`: fork-only and local overrides

Official session structure:

```text
cidre-session
└─ cidre.service
   └─ niri-cidre
```

## Non-Goals

Cidre v1.0 is not trying to be:

- a macOS clone
- a generic Linux desktop for everyone
- a compositor-only release
- a Fedora-first Asahi distribution
- a cosmetic Mac theme project

Also out of scope for the public promise:

- Adobe workflows
- Logic Pro / Final Cut Pro parity
- Apple ecosystem feature parity

## Required Documentation for v1.0

Cidre v1.0 should not ship publicly without these:

- `README.md`
- `INSTALL.md`
- `SUPPORTED_HARDWARE.md`
- `RECOVERY.md`
- `PACKAGE_PLAN.md`
- `CONFIGURATION.md`
- `KNOWN_LIMITATIONS.md`

## Naming Guidance

Recommended public naming split:

- brand: `Cidre`
- tagline: `The cider after Homebrew.`
- component name: `niri-cidre`

Recommended README opening:

```text
# Cidre

The cider after Homebrew.

Cidre is an opinionated Arch Linux ARM developer environment for Apple Silicon Macs.

It ships with niri-cidre, a Cidre-focused fork/configuration of niri, as its standard desktop experience.
```

## Release Gate

From this point onward, planning documents should assume:

- `Cidre v1.0` means the full environment
- `niri-cidre` is a component, not the product itself
- Asahi ALARM / Arch Linux ARM is the primary target
