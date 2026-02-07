# Installation Guide

This guide helps you get Nudge running quickly and verify that integration is healthy.

## Quick Install

Linux/macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

After install:

```bash
nudge status
nudge info
```

## What You Get

- Binary installed to your PATH
- Shell integration script copied to `~/.nudge/shell/`
- Profile hook added (`.bashrc`, `.zshrc`, or PowerShell profile)
- Daemon started

## Installation Options

```bash
# specific version
./scripts/install.sh --version 0.4.5

# custom prefix
./scripts/install.sh --prefix "$HOME/.local"

# skip profile integration
./scripts/install.sh --skip-shell

# use local build artifact
./scripts/install.sh --local

# uninstall
./scripts/install.sh --uninstall
```

## Build From Source

```bash
git clone https://github.com/Zhangtiande/nudge.git
cd nudge
cargo build --release
./scripts/install.sh --local
```

## Reconfigure Shell Integration

```bash
nudge setup zsh --force
nudge setup bash --force
nudge setup powershell --force
```

## Troubleshooting

```bash
nudge doctor zsh
nudge doctor bash
```

Common checks:

- `nudge status` should be `Running`
- `nudge info --field integration_script` should point to an existing file
- Restart shell after setup if key bindings are not active

## Boundaries

- CMD has no automatic profile setup through `nudge setup`
- Bash does not support true auto ghost-text mode
