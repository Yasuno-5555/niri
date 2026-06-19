# Cidre Install Guide

This document describes the current installation path for `Cidre v1.0` as it exists today.

Right now, Cidre should be treated as:

- a full Apple Silicon developer environment target
- built for Asahi Linux on Arch Linux ARM / ALARM
- moving toward a package-first install path built around `cidre-meta-*`

This is not yet a one-command `cidre-installer` flow.

## Scope

This install guide targets:

- Apple Silicon Macs
- Asahi ALARM / Arch Linux ARM
- users who are comfortable with TTY recovery and package management

This guide does not target:

- Asahi Fedora
- Intel Macs
- x86_64 PCs
- users expecting a macOS replacement

## Current Install Model

The current `Cidre v1.0` install model is:

1. start from a working Asahi ALARM / Arch Linux ARM system
2. install the Cidre package profiles
3. confirm the standard login stack and session path
4. validate config and recovery path before daily-driving
5. use local source builds only when doing Cidre development or testing unreleased changes

## Prerequisites

You should already have:

- a booting Apple Silicon Mac with Asahi Linux installed
- Arch Linux ARM / ALARM userland
- a working user account with `sudo`
- internet access
- basic TTY recovery familiarity

Recommended before continuing:

- working `snapper`
- working SSH access
- a known-good fallback shell

## Standard Package Profiles

The intended default package profile for Cidre v1.0 is:

- `core`
- `desktop`
- `dev`

Optional add-ons:

- `diagnostics`
- `optional apps`

See [docs/cidre-v1-package-plan.md](./docs/cidre-v1-package-plan.md) for the current package plan.

For the standard login path, also see [GREETD.md](./GREETD.md).

Planned package-facing install shape:

```bash
sudo pacman -S cidre-meta-core cidre-meta-desktop cidre-meta-dev
```

Or with an AUR-oriented workflow:

```bash
paru -S cidre-meta-core cidre-meta-desktop cidre-meta-dev
```

In that model, `cidre-meta-desktop` is expected to pull in:

- `niri-cidre`
- `cidre-session`
- `cidre-config`
- `greetd`
- `greetd-tuigreet`

This package-facing path is the intended public install story.

Manual source builds of `niri-cidre` should be treated as a development workflow, not as the primary user-facing release path.

## Packaging Responsibility

For `Cidre v1.0`, the intended package story is not "users build every important component from source by hand".

Cidre-owned packages should cover the Cidre-specific runtime surface:

- `niri-cidre`
- `cidre-session`
- `cidre-config`
- `cidre-meta-*`

External packages are acceptable for non-Cidre-specific applications and tools:

- `ghostty` or `ghostty-git`
- `zed`
- `helix`
- `firefox`
- other standard repo or AUR packages

That means `niri-cidre` should have a real package story for public release, while `ghostty` does not need to be packaged by the Cidre project itself.

If a Cidre-adjacent tool is still source-build-only and not part of the minimum support promise, it should not block `v1.0`.

## 1. Install Core Packages

Install or confirm the base platform packages:

```bash
sudo pacman -S --needed \
  base base-devel sudo \
  linux-asahi linux-firmware m1n1 uboot-asahi grub \
  archlinuxarm-keyring asahi-alarm-keyring \
  asahi-meta asahi-fwextract asahi-audio asahi-scripts \
  bankstown speakersafetyd rtkit \
  btrfs-progs snapper snap-pac zram-generator \
  networkmanager openssh rsync
```

After install, ensure these are in good shape:

- boot still works
- networking works through `NetworkManager`
- audio stack is still healthy
- snapshots are functioning if you rely on `snapper`

## 2. Install Desktop Packages

Install the current Cidre desktop baseline:

```bash
sudo pacman -S --needed \
  greetd greetd-tuigreet \
  foot fish bash-completion fuzzel swaybg swayidle swaylock-effects \
  brightnessctl pamixer playerctl \
  bluez bluez-utils \
  pipewire-alsa pipewire-pulse pipewire-jack \
  power-profiles-daemon \
  wl-clipboard xdg-desktop-portal xdg-desktop-portal-gnome polkit-gnome \
  xwayland-satellite \
  fcitx5 fcitx5-configtool fcitx5-mozc fcitx5-gtk fcitx5-qt \
  noto-fonts noto-fonts-cjk noto-fonts-emoji ttf-jetbrains-mono-nerd \
  firefox thunar evince imv mpv
```

This desktop profile is expected to own the default login experience.

That means `cidre-meta-desktop` should include:

- `greetd`
- `greetd-tuigreet`
- the Cidre session entry and session launcher

If Cidre Shell / Quickshell is part of your install target, also install:

```bash
sudo pacman -S --needed quickshell gtk4-layer-shell
```

Package choices that are still policy-sensitive:

- `gnome-keyring`
- `flatpak`
- `xdg-desktop-portal-wlr`
- `iwd`

Do not treat these as hard requirements unless your local Cidre profile explicitly uses them.

## 3. Install Developer Packages

Install the default developer toolset:

```bash
sudo pacman -S --needed \
  git bat eza fd ripgrep fzf zoxide starship direnv jq just \
  helix zed \
  clang llvm lld mold cmake meson ninja rustup zig \
  python python-pip \
  wget which tree
```

Install `paru` if it is not already available.

Example:

```bash
git clone https://aur.archlinux.org/paru.git
cd paru
makepkg -si
```

Install your preferred Ghostty package after that. On the current machine this is:

```bash
paru -S ghostty-git ghostty-shell-integration-git ghostty-terminfo-git
```

## 4. Enable Base Services

Enable the basic system services that Cidre expects:

```bash
sudo systemctl enable NetworkManager
sudo systemctl enable bluetooth
sudo systemctl enable greetd
```

The standard Cidre desktop profile assumes `greetd` is enabled and used as the normal login path.

If you are using additional services in your local Cidre build, enable them explicitly rather than assuming they come up by magic.

## 5. Development Fallback: Build `niri-cidre` From Source

Only use this path when:

- developing `niri-cidre`
- testing unreleased compositor changes
- temporarily working ahead of the package story

At the moment, this repository is the `niri-cidre` source tree.

Build it:

```bash
cd ~/Projects/niri
cargo build --release
```

Validate the build:

```bash
~/Projects/niri/target/release/niri-cidre --version
```

## 6. Development Fallback: Wire A Local Build Into The Session

This is for development-only local overrides.

Today there are two realistic approaches:

- copy the built binary into a system path
- keep the binary in the build tree and point the session to it

For current Cidre development, keep this as an explicit fallback rather than the default login path.

Current working model:

- `cidre-session` is the public session launcher
- `cidre.service` is the public user unit name
- the packaged `cidre.service` reads `~/.config/niri/config.cidre.kdl` by default
- a user override is only needed when testing an unreleased local build
- `niri-session` compatibility is optional and not part of the standard v1.0 install path

Example override:

```ini
[Service]
ExecStart=
ExecStart=/home/USER/Projects/niri/target/release/niri-cidre --session --config /home/USER/.config/niri/config.cidre.kdl
```

Install it at:

```text
~/.config/systemd/user/cidre.service.d/override.conf
```

Then reload user units:

```bash
systemctl --user daemon-reload
```

## 7. Install The Cidre Config Layout

Recommended config layering:

- `~/.config/niri/config.kdl`
- `~/.config/niri/config.cidre.kdl`
- `~/.config/niri/config.cidre.local.kdl`

Recommended structure:

```kdl
// ~/.config/niri/config.cidre.kdl
include "./config.kdl"
include optional=true "./config.cidre.local.kdl"
```

Validate before logging in:

```bash
/usr/bin/niri-cidre validate -c ~/.config/niri/config.kdl
~/Projects/niri/target/release/niri-cidre validate -c ~/.config/niri/config.cidre.kdl
```

See [docs/niri-cidre-config.md](./docs/niri-cidre-config.md) for the current fork-specific config notes.

## 8. Select The Session

Today the practical path is:

- use `greetd`
- launch `cidre-session`
- let user systemd start the packaged `cidre.service`
- use a temporary `NIRI_BIN=/path/to/local/build/niri-cidre cidre-session` override when testing an alternate local binary

For the public package-facing story, the standard expectation is:

- `cidre-meta-desktop` installs `niri-cidre`, `cidre-session`, and `cidre-config`
- `greetd + greetd-tuigreet` provides the normal login path
- the user chooses `Cidre` at login rather than manually wiring compositor binaries

## 9. First Boot Checklist

After the first login, confirm:

- compositor starts successfully
- `Ghostty` launches
- `fish` is active as expected
- audio works
- brightness keys work
- clipboard works
- portals work well enough for Firefox/screenshots
- lock/unlock works
- safe mode bind works if configured

Minimum config validation:

```bash
~/Projects/niri/target/release/niri-cidre validate -c ~/.config/niri/config.cidre.kdl
```

## 10. Update Workflow

For the current source-driven development workflow:

```bash
cd ~/Projects/niri
git pull --ff-only
cargo build --release
systemctl --user daemon-reload
```

Then restart the session or relogin.

If package profiles are also changing:

```bash
sudo pacman -Syu
```

## 11. Rollback / Uninstall

Current rollback path:

1. switch to a TTY
2. stop `greetd` if needed
3. remove or disable the user `cidre.service` override
4. fall back to the packaged `/usr/bin/niri-cidre` session or recovery shell
5. revert config if the issue is config-related
6. use `snapper` rollback if the system state itself is broken

Example:

```bash
mv ~/.config/systemd/user/cidre.service.d/override.conf ~/.config/systemd/user/cidre.service.d/override.conf.disabled
systemctl --user daemon-reload
```

Full recovery procedures belong in `RECOVERY.md`. Until that document is written, do not treat this install path as fully polished.

## Known Gaps In The Current Install Story

The following are not yet fully productized:

- no `cidre-installer` yet
- no finalized `cidre-meta-*` packages yet
- session naming still partially upstream
- package metadata still partially upstream
- supported hardware matrix is not yet frozen in a dedicated doc

## Release Direction

The intended v1.0 direction is:

- product name: `Cidre`
- desktop component: `niri-cidre`
- target platform: Asahi ALARM / Arch Linux ARM
- default install profiles: `core + desktop + dev`

That direction is frozen even if some install mechanics are still manual today.
