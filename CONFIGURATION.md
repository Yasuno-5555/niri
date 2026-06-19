# Cidre Configuration Guide

This file is the public entry point for Cidre configuration guidance.

For the actual fork-specific config structure and options, read:

- [docs/niri-cidre-config.md](./docs/niri-cidre-config.md)

Recommended reading order:

1. [docs/cidre-v1-scope.md](./docs/cidre-v1-scope.md)
2. [INSTALL.md](./INSTALL.md)
3. [RECOVERY.md](./RECOVERY.md)
4. [docs/niri-cidre-config.md](./docs/niri-cidre-config.md)

Current Cidre config layering:

- `~/.config/niri/config.kdl`
- `~/.config/niri/config.cidre.kdl`
- `~/.config/niri/config.cidre.local.kdl`

Role split:

- `config.kdl`: upstream-compatible base
- `config.cidre.kdl`: Cidre entrypoint
- `config.cidre.local.kdl`: fork-only and local overrides

Validation workflow:

```bash
/usr/bin/niri-cidre validate -c ~/.config/niri/config.kdl
~/Projects/niri/target/release/niri-cidre validate -c ~/.config/niri/config.cidre.kdl
```

Recovery-first rule:

- keep the base config as clean as possible
- put fork-only behavior in Cidre-specific config layers
- do not make recovery depend on your fanciest local tweaks
