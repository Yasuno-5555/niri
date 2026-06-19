# Cidre Login Stack

This document defines the standard login stack for `Cidre v1.0`.

## Standard Choice

Cidre v1.0 standard login stack:

- `greetd`
- `greetd-tuigreet`

This is not an optional recommendation for the default desktop profile.

For the standard Cidre desktop experience, `cidre-meta-desktop` should own the login path and include:

- `niri-cidre`
- `cidre-session`
- `cidre-config`
- `greetd`
- `greetd-tuigreet`
- the Cidre wayland session entry
- `cidre.service`
- `cidre-shutdown.target`

Current draft package skeleton:

- `packages/arch/niri-cidre`
- `packages/arch/cidre-meta-desktop`
- `packages/arch/cidre-session`
- `packages/arch/cidre-config`

## Why This Is Standard

Cidre is trying to ship a coherent Apple Silicon Linux developer workstation, not just a compositor binary.

That means the login experience is part of the product surface.

If `tuigreet` is missing from the default desktop profile, users do not get the intended Cidre session selection and login flow by default.

## Package Ownership

Recommended responsibility split:

- `cidre-meta-core`
  - base system
  - Asahi platform stack
  - recovery and boot baseline
- `cidre-meta-desktop`
  - `niri-cidre`
  - `greetd`
  - `greetd-tuigreet`
  - session files
  - desktop runtime dependencies
- `cidre-meta-dev`
  - developer tooling

In other words:

> If a user installs `cidre-meta-desktop`, they should get a working Cidre login session path rather than just "some compositor-related packages".

## Minimum Runtime Expectation

The standard path should be:

```text
greetd
└─ tuigreet
   └─ Cidre
      └─ cidre-session
         └─ cidre.service
            └─ niri-cidre
```

## Documentation Rule

Whenever the default desktop profile is described, it should treat `greetd + greetd-tuigreet` as part of the baseline rather than an optional extra.
