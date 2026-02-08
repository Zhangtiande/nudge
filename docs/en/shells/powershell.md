# PowerShell Guide

[English](powershell.md) | [中文](../../zh/shells/powershell.md)

PowerShell provides manual completion (`Ctrl+E`), a multi-candidate popup selector, failed-command diagnosis, and experimental auto mode via PSReadLine predictor (PowerShell 7.2+).

## Modes

| Mode | Trigger | Description |
|---|---|---|
| `ps-inline` | `Ctrl+E` | Fast single-candidate apply |
| `ps-popup` | `Alt+/` (default) | Hotkey-triggered multi-candidate selector |

## Quick Start

```powershell
# Set up integration
nudge setup powershell --force
nudge restart

# Open a new PowerShell window, then:
# Single candidate
git st<Ctrl+E>
# → git status

# Multi-candidate popup
docker<Alt+/>
# → Opens selector (fzf/sk/peco/ConsoleGrid/GridView/builtin)
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

On load, you should see something like:

- `Nudge loaded (manual mode (Ctrl+E) + popup (Alt+/)).`
- `Nudge loaded (auto mode + popup (Alt+/) + error diagnosis).`

## Popup Selector

`ps-popup` is manual by design: no auto trigger, only key-triggered candidate selection.

### How it works

1. Press popup hotkey (`Alt+/` by default)
2. Integration calls `nudge complete --format list --shell-mode ps-popup`
3. Daemon returns candidates in tab-separated rows
4. Integration opens a selector backend and applies the selected command

### Row format

```
risk<TAB>command<TAB>warning<TAB>why<TAB>diff
```

| Field | Description |
|---|---|
| `risk` | Risk level: `low`, `high` |
| `command` | Suggested command |
| `warning` | Safety warning for risky command |
| `why` | Short explanation |
| `diff` | Delta against your input |

### Selector backend

Configure with `NUDGE_PS_POPUP_BACKEND` (or shared fallback `NUDGE_POPUP_BACKEND`):

| Value | Description |
|---|---|
| `auto` (default) | Auto-detect: `fzf` → `sk` → `peco` → `Out-ConsoleGridView` → `Out-GridView` → builtin |
| `fzf` | [fzf](https://github.com/junegunn/fzf) |
| `sk` | [skim](https://github.com/lotabout/skim) |
| `peco` | [peco](https://github.com/peco/peco) |
| `consolegrid` | `Out-ConsoleGridView` |
| `gridview` | `Out-GridView` |
| `builtin` | Numbered list fallback |

### Risk confirmation

High-risk suggestions require confirmation by default.

```powershell
# Disable confirmation (not recommended)
$env:NUDGE_PS_POPUP_CONFIRM_RISKY = "0"
```

## Popup Environment Variables

PowerShell-specific vars (with shared fallback where available):

| Variable | Default | Description |
|---|---|---|
| `NUDGE_PS_POPUP_ENABLED` | `1` | Enable popup key binding |
| `NUDGE_PS_POPUP_KEY` | `Alt+/` | Popup key chord |
| `NUDGE_PS_POPUP_BACKEND` | `auto` | Selector backend |
| `NUDGE_PS_POPUP_HEIGHT` | `70%` | Selector height (`fzf`/`sk`) |
| `NUDGE_PS_POPUP_CONFIRM_RISKY` | `1` | Confirm before applying high-risk candidate |

If your terminal maps `Option+/` to `÷` on macOS, set:

```powershell
$env:NUDGE_PS_POPUP_KEY = "Ctrl+/"
# If Ctrl+/ is not recognized in your host, try "Ctrl+_".
```

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

Same as Zsh: interactive commands (vim, ssh, etc.) are excluded from diagnosis. Check and extend:

```powershell
nudge info --field interactive_commands
```

### Tab acceptance

When a diagnosis suggestion is available, `Tab` inserts it. Otherwise, `Tab` falls back to default PowerShell tab completion.

## Key Bindings

| Key | Action |
|---|---|
| `Ctrl+E` | Manual single-candidate completion |
| `Alt+/` (default) | Popup multi-candidate selector |
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
| `Ctrl+E` does nothing | Key handler not registered | `Get-PSReadLineKeyHandler \| Where-Object Key -eq 'Ctrl+e'` |
| Popup key does nothing | Popup key disabled or chord conflict | Check `NUDGE_PS_POPUP_ENABLED`, `NUDGE_PS_POPUP_KEY`, and `Get-PSReadLineKeyHandler` |
| Popup shows empty list | Daemon unavailable or no candidates | `nudge status`, then retry with more context in buffer |
| Diagnosis not appearing | `diagnosis.enabled` is false | Set to `true` and restart |
| Auto mode warning on load | PS version < 7.2 or PSReadLine < 2.2.0 | Upgrade or use manual mode |
| Daemon not starting | Permission issue | Check `nudge status` and Windows Event Log |

## Boundaries

- Full auto ghost-text mode requires PowerShell 7.2+ and PSReadLine 2.2.0+
- Popup UX depends on available selector backend
- Behavior depends on PowerShell host capabilities (ISE vs Console vs Windows Terminal)
- PSReadLine predictor does not support the overlay display modes available in Zsh
