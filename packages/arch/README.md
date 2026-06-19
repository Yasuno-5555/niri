# Cidre Arch Package Drafts

This directory contains draft Arch Linux ARM / ALARM package definitions for the planned Cidre meta packages.

Current package set:

- `niri-cidre`
- `cidre-meta-core`
- `cidre-meta-desktop`
- `cidre-meta-dev`
- `cidre-meta-diagnostics`
- `cidre-meta-optional`
- `cidre-session`
- `cidre-config`

These are packaging drafts for `Cidre v1.0` planning, not yet published repository packages.

Planned standard install:

- `cidre-meta-core`
- `cidre-meta-desktop`
- `cidre-meta-dev`

## Design Notes

- `niri-cidre` owns the compositor binary.
- `cidre-meta-core` owns the base system and Asahi platform baseline.
- `cidre-meta-desktop` owns the default login stack and desktop runtime.
- `cidre-meta-dev` owns the default developer toolchain.
- `cidre-meta-diagnostics` and `cidre-meta-optional` are additive profiles.
- `cidre-session` owns the public login entry and session launcher.
- `cidre-config` owns the default Cidre configuration assets.

Intended dependency direction:

- `cidre-session -> niri-cidre`
- `cidre-meta-desktop -> niri-cidre, cidre-session, cidre-config`
- `niri-cidre` does not depend on `cidre-session`

For the current public package model, see:

- [docs/cidre-v1-package-plan.md](../../docs/cidre-v1-package-plan.md)
- [GREETD.md](../../GREETD.md)
