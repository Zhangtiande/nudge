# PowerShell Guide

[English](powershell.md) | [中文](../../zh/shells/powershell.md)

PowerShell provides manual completion with `Ctrl+E`, error diagnosis for failed commands, and experimental auto mode via PSReadLine predictor (PowerShell 7.2+).

## Mode

| Mode | Trigger | Description |
|---|---|---|
| `ps-inline` | `Ctrl+E` | Manual single-candidate completion |

## Quick Start

```powershell
# Set up integration
nudge setup powershell --force
nudge restart

# Open a new PowerShell window, then:
# Type a partial command, press Ctrl+E
git st<Ctrl+E>
# → git status
```

## Setup

```powershell
nudge setup powershell --force
```

This adds a `source` line to your PowerShell `$PROFILE` that loads `integration.ps1` on every session.

Verify:

```powershell
nudge status
# Should show: Running (pid: ...)
```

On load, you should see: `Nudge loaded (manual mode).` or `Nudge loaded (manual mode + error diagnosis).`

## PSReadLine Predictor (Auto Mode)

PowerShell 7.2+ with PSReadLine 2.2.0+ supports an experimental auto mode using the `NudgePredictor` module.

### Requirements

- PowerShell 7.2 or later
- PSReadLine 2.2.0 or later

### How it works

1. `NudgePredictor` registers as a PSReadLine prediction source
2. As you type, PSReadLine requests predictions from all registered sources
3. Nudge returns LLM-powered suggestions alongside built-in history predictions
4. Suggestions appear as inline gray text (PSReadLine's `InlineView`)

### Enable

```yaml
# ~/.nudge/config/config.yaml
trigger:
  mode: auto
```

```powershell
nudge restart
# Reopen PowerShell
```

If auto mode requirements are not met, integration falls back to manual mode automatically with a warning.

## Error Diagnosis

When `diagnosis.enabled: true`, PowerShell integration captures failed commands and provides fix suggestions.

### How it works

1. A prompt hook runs after every command
2. It checks `$LASTEXITCODE` (external commands) and `$Error` (PowerShell exceptions)
3. On failure, it calls `nudge diagnose` with the error details
4. The diagnosis appears as yellow text with a suggested fix
5. Press `Tab` to accept the suggested fix into your command line

### Configuration

```yaml
diagnosis:
  enabled: true
```

### What gets captured

| Error source | How it's detected | What's sent |
|---|---|---|
| External command failure | `$LASTEXITCODE ≠ 0` | Command from history + exit code |
| PowerShell exception | `$Error` count increased | Exception message + script stack trace + category |

### Interactive command exclusions

Same as Zsh — interactive commands (vim, ssh, etc.) are excluded from diagnosis. Check and extend:

```powershell
nudge info --field interactive_commands
```

### Tab acceptance

When a diagnosis suggestion is available, `Tab` inserts it. Otherwise, `Tab` falls back to default PowerShell tab completion.

## Key Bindings

| Key | Action |
|---|---|
| `Ctrl+E` | Manual completion |
| `Tab` | Accept diagnosis suggestion (if available), else default tab completion |

## Profile Troubleshooting

### Profile not loading

Check that your profile exists and contains the Nudge source line:

```powershell
Test-Path $PROFILE
Get-Content $PROFILE | Select-String "nudge"
```

If missing, re-run setup:

```powershell
nudge setup powershell --force
```

### Multiple profiles

PowerShell has multiple profile paths (CurrentUserCurrentHost, CurrentUserAllHosts, etc.). Nudge writes to `$PROFILE` which is `CurrentUserCurrentHost`. If you use a different profile, copy the source line manually.

### Execution policy

If scripts are blocked, you may need:

```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## Windows Environment Notes

- IPC uses named pipes (`\\.\pipe\nudge_{username}`) instead of Unix sockets
- The daemon is started via `Start-Process` with `-WindowStyle Hidden`
- Config path: `%USERPROFILE%\.nudge\config\`
- Logs path: `%USERPROFILE%\.nudge\logs\`

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| "Nudge loaded" message missing | Profile not sourced | `nudge setup powershell --force` |
| `Ctrl+E` does nothing | Key handler not registered | Check `Get-PSReadLineKeyHandler \| Where-Object Key -eq 'Ctrl+e'` |
| Diagnosis not appearing | `diagnosis.enabled` is false | Set to `true` and restart |
| Auto mode warning on load | PS version < 7.2 or PSReadLine < 2.2.0 | Upgrade or use manual mode |
| Daemon not starting | Permission issue | Check `nudge status` and Windows Event Log |

## Boundaries

- Full auto ghost-text mode requires PowerShell 7.2+ and PSReadLine 2.2.0+
- No popup selector (Bash-only feature)
- Behavior depends on PowerShell host capabilities (ISE vs Console vs Windows Terminal)
- PSReadLine predictor does not support the overlay display modes available in Zsh
