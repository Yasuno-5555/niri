# Cidre Recovery Guide

This document describes the practical recovery path for `Cidre v1.0`.

Cidre is a developer environment, not a sealed appliance. Recovery is part of the product, not an afterthought.

The recovery model assumes:

- Apple Silicon Mac
- Asahi Linux on Arch Linux ARM / ALARM
- `greetd` login flow
- `niri-cidre` in use, with optional local build overrides during development

## Recovery Principles

When something breaks, recover in this order:

1. get a shell
2. stop the graphical session if needed
3. remove the last risky override
4. validate config
5. fall back to packaged components
6. roll back system state only if necessary

Do not start with random reinstalls. That is just panic with more I/O.

## Fast Triage

Ask these first:

- Did the system fail before login, or after login?
- Is the issue config-only, session-only, or system-wide?
- Can you still reach a TTY?
- Did this start after a config edit, package update, or local compositor rebuild?

## 1. Get To A TTY

If the graphical session is broken, switch to a text console first.

Typical path:

- use a spare VT if one is available
- log in as your normal user
- escalate with `sudo` only when needed

If a session loop is making this difficult, stop `greetd`.

```bash
sudo systemctl stop greetd
```

To keep it from starting again during repair:

```bash
sudo systemctl disable greetd
```

Re-enable later when the session is healthy:

```bash
sudo systemctl enable greetd
```

## 2. Fall Back To A Minimal Shell Session

Cidre assumes `foot + bash` as the emergency fallback.

If your normal desktop stack is unhealthy:

- use TTY + `bash`
- avoid relying on `fish`, shell integrations, portals, or custom session startup

This is why fallback tools should stay boring.

## 3. Recover From Broken `niri` Or `niri-cidre` Session Overrides

The standard Cidre session uses the packaged `cidre.service` and reads `~/.config/niri/config.cidre.kdl` by default.

Typical override path:

```text
~/.config/systemd/user/cidre.service.d/override.conf
```

If a local build override is broken, disable the override first.

Example:

```bash
mv ~/.config/systemd/user/cidre.service.d/override.conf \
   ~/.config/systemd/user/cidre.service.d/override.conf.disabled
systemctl --user daemon-reload
```

This returns session startup to the packaged `cidre.service` behavior.

To inspect what is currently active:

```bash
systemctl --user cat cidre.service
```

If the override exists but you want to edit instead of disabling:

```bash
mkdir -p ~/.config/systemd/user/cidre.service.d
$EDITOR ~/.config/systemd/user/cidre.service.d/override.conf
systemctl --user daemon-reload
```

## 4. Recover From Broken Config

Cidre currently uses a layered config structure:

- `~/.config/niri/config.kdl`
- `~/.config/niri/config.cidre.kdl`
- `~/.config/niri/config.cidre.local.kdl`

Recovery rule:

- if the base config is broken, repair `config.kdl`
- if only Cidre-specific behavior is broken, disable `config.cidre.local.kdl` first

Fast fallback:

```bash
mv ~/.config/niri/config.cidre.local.kdl \
   ~/.config/niri/config.cidre.local.kdl.disabled
```

If needed, also replace the Cidre entrypoint with a minimal include-only file:

```kdl
include "./config.kdl"
```

Validate configs before restarting the session:

```bash
/usr/bin/niri-cidre validate -c ~/.config/niri/config.kdl
~/Projects/niri/target/release/niri-cidre validate -c ~/.config/niri/config.cidre.kdl
```

If the local build is unavailable or suspect, only validate the upstream-safe base first.

## 5. Recover From A Bad Local Build

If the compositor binary itself is bad:

1. disable the user override
2. fall back to the packaged `/usr/bin/niri-cidre`
3. rebuild the local tree from a shell

Rebuild example:

```bash
cd ~/Projects/niri
cargo build --release
```

Version check:

```bash
~/Projects/niri/target/release/niri-cidre --version
/usr/bin/niri-cidre --version
```

If the local build keeps failing, stop daily-driving it and return to the packaged binary until the issue is understood.

## 6. Recover `greetd`

If you cannot reach the session selector or login path cleanly:

Check service status:

```bash
sudo systemctl status greetd
```

Restart after config repairs:

```bash
sudo systemctl restart greetd
```

If `greetd` itself is the problem, do not debug the full desktop stack first. Get login working, then layer the rest back on.

## 7. Recover Networking

Cidre v1.0 assumes `NetworkManager` is the standard networking path.

Check status:

```bash
sudo systemctl status NetworkManager
nmcli general status
```

Restart:

```bash
sudo systemctl restart NetworkManager
```

If networking broke after profile confusion with `iwd`, `dhcpcd`, or `netctl`, simplify first:

- keep `NetworkManager`
- disable conflicting legacy services
- test wired or known-good Wi-Fi paths before restoring extras

## 8. Recover Audio

Cidre assumes the Asahi audio path and PipeWire stack.

Check user services:

```bash
systemctl --user status pipewire pipewire-pulse wireplumber
```

Restart:

```bash
systemctl --user restart pipewire pipewire-pulse wireplumber
```

Useful checks:

```bash
wpctl status
pactl list short sinks
pactl list short sources
```

If the issue is only policy/config related, check:

- sample rate overrides
- custom PipeWire config snippets
- whether the raw speaker path is accidentally being selected

If the issue is deeper:

- confirm `asahi-audio`
- confirm `bankstown`
- confirm `speakersafetyd`
- confirm `rtkit`

## 9. Recover Input / Keyboard / Brightness

If session-level key handling regressed:

- confirm the session actually started the expected compositor
- validate `binds` in `config.kdl`
- test raw input from a TTY if possible

Brightness helpers are only as good as the commands behind them. If bindings exist but no effect happens, test the tools directly.

Examples:

```bash
brightnessctl get
brightnessctl set 10%+
```

For keyboard backlight helpers, test the real helper script or tool directly before blaming the compositor bind layer.

## 10. Recover From A Bad Package Upgrade

If a system update broke the environment and config repair is not enough, use snapshots.

Check available `snapper` snapshots:

```bash
sudo snapper list
```

If you know the last good snapshot, roll back using your normal `snapper` workflow.

Because exact rollback commands vary by layout and local policy, do not improvise on a damaged root filesystem. Use the procedure you actually trust on your machine.

If you rely on `snap-pac`, confirm that snapshots were created around the transaction you want to unwind.

## 11. Recover Boot Chain Problems

If the system fails before the normal Linux userspace recovery path is available, this is no longer a compositor problem.

At that point you are in platform recovery territory:

- Asahi boot chain
- bootloader config
- kernel package state
- root filesystem state

Relevant components in the Cidre baseline:

- `m1n1`
- `uboot-asahi`
- `grub`
- `linux-asahi`
- `linux-firmware`

For v1.0, this guide deliberately does not pretend boot-chain recovery is "simple". If boot breaks, prioritize a known-good rollback path and documented Asahi recovery procedures over ad hoc experimentation.

## 12. Minimum Recovery Kit

Cidre should be considered incomplete if you do not have these available:

- TTY login
- `bash`
- `foot`
- `sudo`
- `systemctl`
- `NetworkManager`
- `snapper`
- one known-good editor (`nano`, `vi`, or `helix`)

## 13. Recovery Checklist Before Re-Enabling `greetd`

Before returning to the graphical path, confirm:

- config validates
- the active binary is the one you intended
- session override is sane
- networking is up
- audio stack is not obviously broken
- there is still a shell-based fallback path

Then:

```bash
sudo systemctl enable greetd
sudo systemctl restart greetd
```

## 14. What To Document When Recovery Was Needed

When something breaks, record:

- what changed
- what failed
- what fixed it
- whether the problem was config, package, service, or hardware-specific

That information should feed back into:

- `KNOWN_LIMITATIONS.md`
- install notes
- hardware support notes
- package profile changes

Good recovery docs are just bug reports that survived.
