# Migration Guide

This guide helps existing Nudge users upgrade to version 0.3.0 with auto mode support.

## What's New in 0.3.0

- **Auto Mode**: Ghost text suggestions as you type (like GitHub Copilot)
- **FFI Layer**: Dynamic library for lower latency on Unix systems
- **PowerShell Predictor**: Native PSReadLine integration for Windows
- **Improved Shell Integration**: Simplified setup with `nudge setup`

## Upgrade Steps

### Step 1: Update Nudge Binary

**Using the installer (recommended):**

```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash

# Windows (PowerShell)
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

**Manual update:**

Download the latest release from [GitHub Releases](https://github.com/Zhangtiande/nudge/releases/latest) and replace your existing binary.

### Step 2: Update Shell Integration

The shell integration scripts have been updated. Re-run setup:

```bash
nudge setup
```

Then restart your shell or source your profile:

```bash
# Bash
source ~/.bashrc

# Zsh
source ~/.zshrc

# PowerShell
. $PROFILE
```

### Step 3: Enable Auto Mode (Optional)

Auto mode is disabled by default. To enable it, edit your config file:

**Location:**
- Linux: `~/.config/nudge/config.yaml`
- macOS: `~/Library/Application Support/nudge/config.yaml`
- Windows: `%APPDATA%\nudge\config\config.yaml`

**Add or modify:**

```yaml
trigger:
  mode: auto              # Change from "manual" to "auto"
  auto_delay_ms: 500      # Adjust debounce delay if needed
```

Then restart the daemon:

```bash
nudge restart
```

## Configuration Changes

### New Configuration Options

The following options are new in 0.3.0:

```yaml
trigger:
  mode: "manual"          # NEW: "manual" or "auto"
  hotkey: "\C-e"          # Existing: Ctrl+E hotkey
  auto_delay_ms: 500      # NEW: Debounce delay for auto mode
```

### Backward Compatibility

- All existing configuration files remain valid
- The `trigger` section is optional (defaults to manual mode)
- Old shell integration scripts continue to work but are deprecated

## Shell-Specific Notes

### Bash

Auto mode uses ANSI escape codes for inline preview. If your terminal doesn't support this, you may see garbled output. In that case, stick with manual mode or switch to Zsh.

### Zsh

Auto mode uses ZLE's `POSTDISPLAY` for native inline preview. This provides the best experience.

Key bindings in auto mode:
- `Tab`: Accept full suggestion
- `Right Arrow`: Accept next word

### PowerShell 7.2+

Auto mode uses PSReadLine's predictor API. The `NudgePredictor` module is automatically installed.

Requirements:
- PowerShell 7.2 or later
- PSReadLine 2.2.0 or later (bundled with PowerShell 7.2+)

Key bindings in auto mode:
- `Tab`: Accept full suggestion
- `Right Arrow`: Move cursor forward
- `Ctrl+Right Arrow`: Accept next word

### PowerShell 5.1

Auto mode is not supported. Only manual mode (Ctrl+E) is available.

### CMD

Auto mode is not supported. Only manual mode (Ctrl+E) is available.

## Troubleshooting

### Auto mode not working

1. Check if auto mode is enabled:
   ```bash
   nudge info --field trigger_mode
   ```

2. Verify the daemon is running:
   ```bash
   nudge status
   ```

3. Check shell integration is loaded:
   - Bash/Zsh: Look for "Nudge loaded" message on shell startup
   - PowerShell: Run `Get-Module NudgePredictor`

### Suggestions not appearing

1. Increase the debounce delay if suggestions are being cancelled:
   ```yaml
   trigger:
     auto_delay_ms: 800  # Increase from default 500ms
   ```

2. Check LLM endpoint is accessible:
   ```bash
   nudge complete --buffer "git sta" --cursor 7 --cwd . --session test
   ```

### Performance issues

If auto mode feels slow:

1. Use a local LLM (Ollama) instead of remote APIs
2. Increase debounce delay to reduce API calls
3. Consider using manual mode for better control

## Rolling Back

If you encounter issues, you can roll back to manual mode:

```yaml
trigger:
  mode: manual
```

Or temporarily disable auto mode with an environment variable:

```bash
export NUDGE_TRIGGER_MODE=manual
```

## Getting Help

- [Auto Mode Documentation](auto-mode.md)
- [Troubleshooting Guide](troubleshooting.md)
- [GitHub Issues](https://github.com/Zhangtiande/nudge/issues)
