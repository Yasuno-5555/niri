# Cidre v1.0 Release Checklist

This document is the go/no-go checklist for shipping the first public `Cidre v1.0` release.

It is intentionally narrower than the broader release plan. The question here is simple:

> Can a supported Apple Silicon Mac boot, log in, recover, and be used as a Cidre developer machine without undocumented traps?

## Release Identity

- [x] Product name is `Cidre`
- [x] Desktop/compositor component is `niri-cidre`
- [x] Target platform is Asahi Linux on Arch Linux ARM / ALARM
- [x] Public runtime direction is `pacman + paru`, `Ghostty + fish`, `Zed + Helix`
- [x] Binary naming policy is frozen
- [ ] Companion repo/package split is frozen

## Naming Freeze

- [x] The compositor binary is installed as `niri-cidre`
- [x] The Cidre login session is exposed as `Cidre`
- [x] `cidre-session` starts `niri-cidre` by default
- [x] Documentation treats `Cidre` as the full environment and `niri-cidre` as a component
- [ ] Package metadata no longer presents upstream `niri` as the primary project
- [x] Compatibility aliases are optional rather than standard

## Naming Implementation

- [x] Cargo package name is `niri-cidre`
- [x] Release binary is `target/release/niri-cidre`
- [x] Public installed binary is `/usr/bin/niri-cidre`
- [x] Internal library naming remains upstream-compatible where useful
- [x] `cidre-session` starts `niri-cidre` by default
- [x] Standard session, service, and desktop files use Cidre naming
- [x] Upstream-compatible `niri-*` assets are not installed by default
- [x] Remaining `niri-*` resources are documented as compatibility assets
- [x] `README.md` explains `Cidre -> cidre-session -> niri-cidre`
- [x] Issue reporting commands use `niri-cidre --version`

## Documentation Gate

- [x] `README.md` explains what Cidre is
- [x] `INSTALL.md` documents the current install path
- [x] `SUPPORTED_HARDWARE.md` defines support tiers
- [x] `RECOVERY.md` documents break-glass recovery
- [x] `CONFIGURATION.md` points to the Cidre config model
- [x] `KNOWN_LIMITATIONS.md` defines non-goals and unsupported cases
- [x] `docs/niri-cidre-config.md` documents fork-specific config structure
- [x] `README.md` is fully free of misleading upstream-first wording
- [x] Issue/reporting guidance is frozen

## Session And Config Gate

- [x] `cidre-session` exists as the public session entrypoint
- [x] `cidre.service` exists as the public user unit name
- [x] Cidre config layering is documented
- [x] `greetd + greetd-tuigreet` is documented as the standard login stack owned by the desktop profile
- [x] Shipped config examples validate cleanly
- [x] Local-build override path is documented end-to-end
- [ ] Safe-mode entry is documented and verified
- [x] Compatibility behavior with `niri-session` is clearly bounded

## Packaging Gate

- [x] Package metadata includes Cidre session artifacts
- [x] `.desktop` entry exposes `Cidre Desktop`
- [ ] Package descriptions consistently describe Cidre rather than upstream niri
- [ ] Runtime dependency recommendations are aligned with the Cidre defaults
- [ ] Release artifact list is frozen
- [ ] Arch/ALARM-first install path is the documented primary path

## Arch Packaging

- [x] `niri-cidre` has an Arch PKGBUILD
- [x] `niri-cidre` installs `/usr/bin/niri-cidre`
- [x] `niri-cidre` does not install `cidre-session` or `cidre-config`
- [x] `cidre-session` depends on `niri-cidre`
- [x] `cidre-meta-desktop` depends on `niri-cidre`, `cidre-session`, and `cidre-config`
- [x] `.SRCINFO` is generated for all Arch package drafts
- [x] `namcap PKGBUILD` passes for all Arch package drafts
- [x] `niri-cidre`, `cidre-session`, and `cidre-config` package drafts build successfully with `makepkg`
- [x] `cidre-meta-core`, `cidre-meta-desktop`, `cidre-meta-dev`, `cidre-meta-diagnostics`, and `cidre-meta-optional` package drafts build successfully
- [x] Documentation no longer describes `cidre-session` or `cidre-config` as temporarily bundled with `niri-cidre`

## Hardware Validation Gate

- [x] At least one supported Apple Silicon Mac is explicitly documented
- [x] Fresh login tested on the primary supported machine
- [ ] Audio output/input sanity checked
- [ ] Wi-Fi and Bluetooth sanity checked
- [ ] Suspend/resume behavior characterized
- [x] External display behavior characterized
- [x] Touchpad workflow sanity checked
- [ ] Recovery path tested after a broken config

## Developer Experience Gate

- [x] `Ghostty` launches correctly from the default session
- [x] `fish` is the expected interactive shell
- [x] `Zed` and `Helix` launch cleanly
- [ ] Clipboard, portal, and screenshot basics work
- [ ] IME path is sane for `fcitx5` users
- [ ] Default fonts render correctly for Latin, CJK, and emoji

## Release Operations Gate

- [x] Version tag strategy is decided
- [x] Release notes are updated from the draft
- [ ] Screenshots/media reflect the current product identity
- [x] Known limitations are reviewed one last time
- [ ] Install steps have been dry-run from the docs
- [ ] Recovery steps have been dry-run from the docs

## Minimum Ship Criteria

Do not call it `v1.0` unless all of the following are true:

- the Cidre product scope is still accurate
- the documented install path is the path you actually support
- the documented recovery path works on supported hardware
- the standard `core + desktop + dev` story is coherent
- the session and config naming do not mislead users about what is supported
