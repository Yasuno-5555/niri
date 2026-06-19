# Cidre v1.0 Package Plan

This document organizes the proposed `Cidre v1.0` package set and compares it against the current explicit package state from `pacman -Qe`.

Snapshot source:

- command: `pacman -Qe`
- host state date: 2026-06-20

This is a planning document, not yet a lockfile or install manifest.

## Goal

`Cidre v1.0` should define a coherent package baseline for an Apple Silicon Linux developer machine.

Scope note:

- target product: full Cidre environment
- target platform: Asahi ALARM / Arch Linux ARM on Apple Silicon Macs
- standard install profiles: `core + desktop + dev`
- optional profiles: `diagnostics + optional apps`

The package set should answer four needs:

- bootable and recoverable base system
- comfortable desktop session
- strong developer defaults
- optional diagnostics and optional apps without polluting the core story

## Packaging Responsibility

For `Cidre v1.0`, not every dependency needs the same distribution treatment.

Recommended split:

### Must be packaged by Cidre

These are part of the Cidre product surface and should not rely on "build it yourself every time" as the public story.

- `niri-cidre`
- `cidre-session`
- `cidre-config`
- `cidre-meta-core`
- `cidre-meta-desktop`
- `cidre-meta-dev`
- `cidre-meta-diagnostics`
- `cidre-meta-optional`

Reason:

- they define the Cidre desktop/runtime identity
- they define the supported install path
- they are part of the support boundary

### May rely on external packages

These do not need to be maintained as Cidre-owned source packages in `v1.0` as long as the install path is documented and stable enough.

- `ghostty` or `ghostty-git`
- `zed`
- `helix`
- `firefox`
- most other standard user applications and toolchain packages from repos or AUR

Reason:

- they are important to the experience, but they are not Cidre-specific software products
- Cidre can depend on them without owning their upstream packaging story

### Not required to block `v1.0`

If a Cidre-adjacent tool still requires source builds and is not part of the minimum support promise, it should not block the release.

Current example:

- `cidre-top`, if still unfinished or not yet packaged

Interpretation:

- do not pretend it is part of the required baseline until it has a real package story
- keep the minimum Cidre promise centered on the installable desktop environment

## Proposed Package Groups

## `core`

Required system base, boot chain, and Apple Silicon platform support.

```text
core
base
base-devel
sudo
linux-asahi
linux-firmware
m1n1
uboot-asahi
grub
archlinuxarm-keyring
asahi-alarm-keyring
asahi-meta
asahi-fwextract
asahi-audio
asahi-scripts
bankstown
speakersafetyd
rtkit
btrfs-progs
snapper
snap-pac
zram-generator
networkmanager
openssh
rsync
```

Status vs current `pacman -Qe`:

- Present: `base`, `base-devel`, `sudo`, `linux-asahi`, `linux-firmware`, `m1n1`, `uboot-asahi`, `grub`, `archlinuxarm-keyring`, `asahi-alarm-keyring`, `asahi-meta`, `asahi-fwextract`, `asahi-audio`, `asahi-scripts`, `bankstown`, `speakersafetyd`, `rtkit`, `btrfs-progs`, `snapper`, `snap-pac`, `zram-generator`, `networkmanager`, `openssh`, `rsync`
- Missing from current explicit set: `core`

Notes:

- `core` is not currently explicit on this machine. For a public `Cidre v1.0` definition, decide whether to list it as a required group or rely on `base` plus installer assumptions.
- `linux-firmware`, `m1n1`, `uboot-asahi`, and keyrings should be treated as explicit public baseline, not hidden installer detail.
- If `NetworkManager` is the official path, `dhcpcd` and `netctl` should not be in the standard profile.

## `desktop`

Required user session and daily desktop workflow.

```text
desktop
niri-cidre
greetd
greetd-tuigreet
foot
ghostty
fish
bash-completion
fuzzel
swaybg
swayidle
swaylock-effects
brightnessctl
pamixer
playerctl
bluez
bluez-utils
pipewire-alsa
pipewire-pulse
pipewire-jack
wl-clipboard
xdg-desktop-portal
xdg-desktop-portal-gnome
polkit-gnome
fcitx5
fcitx5-configtool
fcitx5-mozc
fcitx5-gtk
fcitx5-qt
noto-fonts
noto-fonts-cjk
noto-fonts-emoji
ttf-jetbrains-mono-nerd
firefox
thunar
evince
imv
mpv
```

Status vs current `pacman -Qe`:

- Present or equivalent on current machine: `niri` (current package name; planned public component is `niri-cidre`), `greetd`, `greetd-tuigreet`, `foot`, `ghostty-git`, `fish`, `bash-completion`, `fuzzel`, `swaybg`, `swayidle`, `swaylock-effects`, `brightnessctl`, `pamixer`, `playerctl`, `bluez`, `bluez-utils`, `pipewire-alsa`, `pipewire-pulse`, `pipewire-jack`, `wl-clipboard`, `xdg-desktop-portal`, `xdg-desktop-portal-gnome`, `polkit-gnome`, `fcitx5`, `fcitx5-configtool`, `fcitx5-mozc`, `fcitx5-gtk`, `fcitx5-qt`, `noto-fonts`, `noto-fonts-cjk`, `noto-fonts-emoji`, `ttf-jetbrains-mono-nerd`, `firefox`, `thunar`, `evince`, `imv`, `mpv`
- Missing from current explicit set: `desktop`

Notes:

- `desktop` is a conceptual group here, not a currently installed explicit package.
- `greetd` and `greetd-tuigreet` should be treated as required login-stack ownership of `cidre-meta-desktop`, not as optional extras.
- `cidre-meta-desktop` should install the Cidre session path, not just the compositor runtime.
- `polkit-gnome` is promoted into the standard desktop profile.
- `quickshell` and `gtk4-layer-shell` should be treated as standard if Cidre ships its own shell UI; otherwise they remain optional.
- `xdg-desktop-portal-wlr` should be treated as optional unless Cidre depends on it for a documented workflow.
- `iwd` is only standard if intentionally used as the `NetworkManager` backend.

Minimum expected responsibility of `cidre-meta-desktop`:

```text
cidre-config
cidre-session
niri-cidre
greetd
greetd-tuigreet
Cidre wayland session entry
cidre.service
cidre-shutdown.target
desktop runtime packages
```

## `dev`

Required developer tooling.

```text
dev
git
paru
bat
eza
fd
ripgrep
fzf
zoxide
starship
direnv
jq
just
helix
zed
clang
llvm
lld
mold
cmake
meson
ninja
rustup
zig
python
python-pip
wget
which
tree
```

Status vs current `pacman -Qe`:

- Present or equivalent on current machine: `git`, `paru`, `bat`, `eza`, `fd`, `ripgrep`, `fzf`, `zoxide`, `starship`, `direnv`, `jq`, `just`, `helix`, `zed`, `clang`, `llvm`, `lld`, `mold`, `cmake`, `meson`, `ninja`, `rustup`, `zig`, `python-pip`, `wget`, `which`, `tree`
- Needs explicit confirmation in package naming/base profile: `python`
- Missing from current explicit set: `dev`

Notes:

- `dev` is again a conceptual group, not a currently explicit package.
- If Cidre v1.0 is meant to feel complete on first boot, this is a strong default set.
- `Zed` and `Helix` should be documented as the standard GUI/terminal editor pair, not as random preferences.

## `diagnostics optional`

Optional debugging and performance tools.

```text
diagnostics optional
bpftrace
drm-info
dtc
iotop
libinput-tools
mesa-utils
perf
powertop
sysstat
trace-cmd
usbutils
strace
lsof
```

Status vs current `pacman -Qe`:

- Present: `bpftrace`, `drm-info`, `dtc`, `iotop`, `libinput-tools`, `mesa-utils`, `perf`, `powertop`, `sysstat`, `trace-cmd`, `usbutils`, `strace`, `lsof`

Notes:

- This entire set is already explicitly installed on the current machine.
- This supports the idea that diagnostics should be an official optional profile rather than ad hoc personal additions.

## `optional apps`

Nice-to-have applications and secondary tooling.

```text
optional apps
flatpak
github-cli
pandoc-bin
python-numpy
python-scipy
python-matplotlib
python-pyqtgraph
tailscale
file-roller
loupe
```

Status vs current `pacman -Qe`:

- Present: `flatpak`, `github-cli`, `pandoc-bin`, `python-numpy`, `python-scipy`, `python-matplotlib`, `python-pyqtgraph`, `tailscale`, `file-roller`, `loupe`

Notes:

- This set also already matches the current machine well.
- `flatpak` is optional in the draft, but decide whether the public story should treat it as standard.

## Current Explicit Packages Not In The Draft

The current machine has explicit packages that were not listed in the proposed v1.0 grouping.

### Platform / system

- `archlinuxarm-keyring`
- `asahi-alarm-keyring`
- `asahi-fwextract`
- `dhcpcd`
- `fwupd`
- `gnome-keyring`
- `grub`
- `iwd`
- `linux-firmware`
- `m1n1`
- `man-db`
- `man-pages`
- `net-tools`
- `netctl`
- `uboot-asahi`

### Desktop / session adjacent

- `gtk4-layer-shell`
- `papirus-icon-theme`
- `polkit-gnome`
- `quickshell`
- `swappy`
- `xdg-desktop-portal-wlr`

### Development / utility extras

- `7zip`
- `bash-completion`
- `blueprint-compiler`
- `btop`
- `ex-vi-compat`
- `fastfetch`
- `fcitx5-configtool`
- `ghostty-shell-integration-git`
- `ghostty-terminfo-git`
- `htop`
- `hyperfine`
- `nano`
- `oniguruma`
- `rsync`
- `tree`
- `unzip`
- `wget`
- `which`
- `zip`

## Recommended Interpretation for v1.0

Suggested shape:

- `core`: hard requirement
- `desktop`: hard requirement
- `dev`: default install profile
- `diagnostics`: documented optional profile, but recommended for Cidre contributors
- `optional apps`: add-on profile

Packages that likely deserve elevation into the official baseline:

- `quickshell`
- `polkit-gnome`
- `gtk4-layer-shell`
- `linux-firmware`
- `m1n1`
- `uboot-asahi`
- `fwupd`

Packages that need an explicit decision because they overlap with the main story:

- `iwd`
- `dhcpcd`
- `netctl`
- `xdg-desktop-portal-wlr`
- `gnome-keyring`
- `flatpak`

## Suggested Next Step

Turn this planning document into one of these:

1. `cidre-base` package group definition
2. install script manifest
3. `packages.txt` lock-style list for v1.0
4. "minimal" and "full" profiles

Recommended profiles:

- `cidre-minimal`
- `cidre-desktop`
- `cidre-dev`
- `cidre-diagnostics`
- `cidre-full`

Recommended v1.0 standard install:

- `cidre-meta-core`
- `cidre-meta-desktop`
- `cidre-meta-dev`

Draft Arch package skeletons now live under:

- `packages/arch/niri-cidre`
- `packages/arch/cidre-meta-core`
- `packages/arch/cidre-meta-desktop`
- `packages/arch/cidre-meta-dev`
- `packages/arch/cidre-meta-diagnostics`
- `packages/arch/cidre-meta-optional`
- `packages/arch/cidre-session`
- `packages/arch/cidre-config`

## Raw Current `pacman -Qe` Snapshot

Captured from the current machine:

```text
7zip
archlinuxarm-keyring
asahi-alarm-keyring
asahi-audio
asahi-fwextract
asahi-meta
asahi-scripts
bankstown
base
base-devel
bash-completion
bat
blueprint-compiler
bluez
bluez-utils
bpftrace
brightnessctl
btop
btrfs-progs
clang
cmake
dhcpcd
direnv
drm-info
dtc
evince
ex-vi-compat
eza
fastfetch
fcitx5
fcitx5-configtool
fcitx5-gtk
fcitx5-mozc
fcitx5-qt
fd
file-roller
firefox
fish
flatpak
foot
fuzzel
fwupd
fzf
ghostty-git
ghostty-shell-integration-git
ghostty-terminfo-git
git
github-cli
gnome-keyring
greetd
greetd-tuigreet
grub
gtk4-layer-shell
helix
htop
hyperfine
imv
iotop
iwd
jq
just
libinput-tools
linux-asahi
linux-firmware
lld
llvm
loupe
lsof
m1n1
man-db
man-pages
mesa-utils
meson
mold
mpv
nano
net-tools
netctl
networkmanager
ninja
niri
noto-fonts
noto-fonts-cjk
noto-fonts-emoji
oniguruma
openssh
pamixer
pandoc-bin
papirus-icon-theme
paru
perf
pipewire-alsa
pipewire-jack
pipewire-pulse
playerctl
polkit-gnome
powertop
python-matplotlib
python-numpy
python-pip
python-pyqtgraph
python-scipy
quickshell
ripgrep
rsync
rtkit
rustup
snap-pac
snapper
speakersafetyd
starship
strace
sudo
swappy
swaybg
swayidle
swaylock-effects
sysstat
tailscale
thunar
trace-cmd
tree
ttf-jetbrains-mono-nerd
uboot-asahi
unzip
usbutils
wget
which
wl-clipboard
xdg-desktop-portal
xdg-desktop-portal-gnome
xdg-desktop-portal-wlr
zed
zig
zip
zoxide
zram-generator
```
