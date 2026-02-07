# Installation Guide

[English](installation.md) | [中文](../zh/installation.md)

This guide covers installing Nudge, setting up shell integration, and verifying that everything works.

## Quick Install

Linux/macOS:

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

The installer runs an interactive wizard that asks for your LLM endpoint and preferred trigger mode.

## What the Installer Does

1. Downloads the `nudge` binary to `~/.nudge/bin/` (or `%USERPROFILE%\.nudge\bin\` on Windows)
2. Adds `~/.nudge/bin` to your `PATH`
3. Writes default config to `~/.nudge/config/config.default.yaml`
4. Generates user config at `~/.nudge/config/config.yaml` from wizard answers
5. Copies shell integration script to `~/.nudge/shell/`
6. Adds a `source` line to your shell profile (`.bashrc`, `.zshrc`, or PowerShell `$PROFILE`)
7. Starts the daemon

## Installation Options

```bash
# Install a specific version
./scripts/install.sh --version 0.5.0

# Custom install prefix
./scripts/install.sh --prefix "$HOME/.local"

# Skip shell profile modification
./scripts/install.sh --skip-shell

# Use a locally built binary
./scripts/install.sh --local

# Uninstall
./scripts/install.sh --uninstall
```

## Build From Source

```bash
git clone https://github.com/Zhangtiande/nudge.git
cd nudge
cargo build --release
./scripts/install.sh --local
```

## Shell Integration Setup

The `nudge setup` command writes the integration script and hooks it into your shell profile.

```bash
nudge setup bash          # Set up Bash integration
nudge setup zsh           # Set up Zsh integration
nudge setup powershell    # Set up PowerShell integration
```

Use `--force` to overwrite existing integration files:

```bash
nudge setup zsh --force
```

After setup, restart your shell or source the profile:

```bash
source ~/.zshrc   # or ~/.bashrc
```

CMD does not have an automatic setup path. See [CMD guide](shells/cmd.md) for manual setup.

## Post-Install Verification

Run these commands in order to confirm a healthy installation:

```bash
# 1. Check daemon is running
nudge status
# Expected: "Running (pid: ...)"

# 2. Show configuration summary
nudge info
# Shows: config_dir, socket_path, trigger_mode, etc.

# 3. Verify integration script exists
nudge info --field integration_script
# Should print a path like ~/.nudge/shell/integration.zsh

# 4. Run health check (Zsh/Bash)
nudge doctor zsh    # or: nudge doctor bash
# Reports key bindings, hooks, and integration status

# 5. Try a completion
# Type a partial command, press Ctrl+E
```

If `nudge status` shows `Not running`, start the daemon:

```bash
nudge start
```

## File Layout

After installation, your `~/.nudge/` directory looks like:

```
~/.nudge/
├── bin/
│   └── nudge              # Binary
├── config/
│   ├── config.default.yaml  # Shipped defaults (do not edit)
│   └── config.yaml          # Your overrides
├── shell/
│   ├── integration.bash
│   ├── integration.zsh
│   ├── integration.ps1
│   └── integration.cmd
├── logs/                    # When file_enabled: true
└── nudge.sock               # Unix domain socket (runtime)
```

On Windows, replace `~/.nudge/` with `%USERPROFILE%\.nudge\` and the socket with a named pipe `\\.\pipe\nudge_{username}`.

## Upgrading

Re-run the installer. It preserves your `config.yaml` and updates everything else:

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

After upgrading, refresh shell integration to pick up new features:

```bash
nudge setup zsh --force
nudge restart
```

## Uninstalling

```bash
./scripts/install.sh --uninstall
```

Or manually:

1. Remove `~/.nudge/`
2. Remove the `source` line from your shell profile
3. Stop the daemon: `nudge stop`

## Boundaries

- CMD has no automatic profile setup through `nudge setup`
- Bash does not support true auto ghost-text mode
- The installer requires `curl` (Unix) or PowerShell 5.1+ (Windows)
