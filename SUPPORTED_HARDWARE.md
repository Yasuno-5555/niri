# Cidre Supported Hardware

This document defines the current support stance for `Cidre v1.0`.

The goal is to be explicit about what is actually supported, what is only expected to work, and what is not yet part of the public promise.

## Scope

Cidre v1.0 targets:

- Apple Silicon Macs
- Asahi Linux on Arch Linux ARM / ALARM

Cidre v1.0 does not currently target:

- Asahi Fedora
- Intel Macs
- x86_64 PCs
- generic ARM laptops

## Support Tiers

Cidre uses four support tiers:

- `Tested`: used directly during Cidre development and considered part of the real support promise
- `Expected to work`: not validated as heavily, but reasonably close to tested hardware
- `Untested`: may work, but is not part of the public reliability promise yet
- `Unsupported`: out of v1.0 scope

## Tested

These are the systems that Cidre development is directly grounded in.

### Apple MacBook Air (M1, 2020)

Status: `Tested`

Known local development baseline:

- machine: `Apple MacBook Air (M1, 2020)`
- architecture: `aarch64`
- kernel family used in development: `linux-asahi`
- userland target: `Arch Linux ARM / ALARM`

This is currently the strongest support target for Cidre v1.0.

Areas directly exercised in development include:

- `niri-cidre` session startup
- Apple SPI trackpad tuning
- touchpad gesture telemetry
- power-aware compositor behavior
- Asahi audio stack integration

Known caveat on this tested baseline:

- external USB-C display output is not yet considered reliable enough to promise as part of the Cidre v1.0 support contract

If a user asks "what hardware should I use for Cidre right now?", this is the safest answer.

## Expected To Work

These systems are close enough to the tested baseline that support is a reasonable goal, but they should still be treated carefully until explicitly validated.

### Other M1 MacBook-class laptops

Status: `Expected to work`

Examples:

- M1 MacBook Pro models

Reasoning:

- similar Apple Silicon generation
- similar input expectations
- similar Asahi platform stack

Caveat:

- this is still weaker than the tested MacBook Air baseline
- model-specific quirks may exist in display, suspend, input, or thermal behavior

## Untested

These systems are within the broad product direction, but not yet part of the practical support promise.

### Other Apple Silicon Macs

Status: `Untested`

Examples:

- M1 desktop systems
- M2 generation systems
- newer Apple Silicon laptops and desktops

Interpretation:

- they are not rejected conceptually
- they are simply not validated enough for v1.0 claims

If you run Cidre on these systems, expect self-support and debugging work.

## Unsupported

These are outside the v1.0 target.

### Asahi Fedora

Status: `Unsupported for v1.0`

Reason:

- Cidre v1.0 is built around `pacman`, `paru`, `AUR`, and Arch Linux ARM package assumptions

### Intel Macs

Status: `Unsupported`

Reason:

- Cidre v1.0 is specifically aimed at Apple Silicon hardware

### Non-Mac ARM Linux systems

Status: `Unsupported`

Reason:

- the product is defined around Apple Silicon Mac hardware, Asahi platform assumptions, and MacBook-class input/power behavior

## Software Baseline Required For Support

To be considered within the intended support envelope, a system should roughly match these assumptions:

- Asahi Linux on Arch Linux ARM / ALARM
- `linux-asahi`
- Asahi boot chain and firmware userspace pieces
- Cidre package baseline (`core + desktop + dev`)
- `niri-cidre` as the standard desktop session

If you diverge from these, you may still be able to run Cidre, but support quality drops immediately.

## Areas Most Sensitive To Hardware Differences

Even within Apple Silicon Macs, these areas are the most likely to vary by model:

- display behavior
- suspend / resume stability
- keyboard backlight handling
- touchpad feel and gesture tuning
- speaker and audio safety path behavior
- thermals and power-saving behavior

Current example:

- on the tested M1 MacBook Air baseline, a direct USB-C to HDMI external display test did not come up under Asahi ALARM, so external display behavior should be treated cautiously even on the primary development machine

This is why the tested list should stay conservative.

## Known v1.0 Hardware Position

The current honest position for Cidre v1.0 is:

- strong confidence on the development machine class
- cautious optimism on nearby M1 MacBook hardware
- no broad claim yet for all Apple Silicon Macs

That is narrower than a marketing-friendly claim, but much safer.

## How This Should Evolve

This file should be updated when:

- a new model becomes part of regular development
- install and recovery are tested on additional hardware
- a model proves consistently problematic

Recommended future structure once coverage grows:

- `Tested and recommended`
- `Tested with caveats`
- `Expected to work`
- `Known problematic`
- `Unsupported`

## Release Rule

Do not expand the `Tested` section unless the hardware has actually been used through:

- install
- login/session startup
- normal development workflow
- suspend/resume or recovery checks

Cidre should under-promise here. Hardware support inflation creates support debt fast.
