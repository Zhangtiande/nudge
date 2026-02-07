# CMD Guide

[English](cmd.md) | [中文](../../zh/shells/cmd.md)

CMD (Command Prompt) provides basic manual completion through doskey macros. It is the most limited integration path.

## Mode

| Mode | Trigger | Description |
|---|---|---|
| `cmd-inline` | `nudge-complete <command>` | Manual completion via doskey macro |

## Quick Start

```cmd
REM Load integration (or add to AutoRun registry key)
path\to\integration.cmd

REM Get a completion suggestion
nudge-complete git st
```

## Setup

CMD does not support `nudge setup`. You need to load the integration script manually.

### Option 1: Manual load per session

```cmd
%USERPROFILE%\.nudge\shell\integration.cmd
```

### Option 2: AutoRun registry (persistent)

Add the script to the CMD AutoRun key so it loads every time CMD starts:

```cmd
reg add "HKCU\Software\Microsoft\Command Processor" /v AutoRun /t REG_SZ /d "%USERPROFILE%\.nudge\shell\integration.cmd" /f
```

To remove:

```cmd
reg delete "HKCU\Software\Microsoft\Command Processor" /v AutoRun /f
```

## Available Commands

The integration script creates these doskey macros:

| Macro | Description |
|---|---|
| `nudge-complete <args>` | Get completion for the given command prefix |
| `nudge-suggest <args>` | Alias for `nudge-complete` |
| `nudge-start` | Start the daemon |
| `nudge-stop` | Stop the daemon |
| `nudge-status` | Check daemon status |

### Example

```cmd
nudge-complete docker run
REM Outputs: docker run -it --rm ubuntu:latest /bin/bash
```

## How It Differs From Other Shells

- **No hotkey binding**: CMD does not support readline-style key bindings. You must type the `nudge-complete` command explicitly.
- **No inline replacement**: The suggestion is printed as output. You need to copy-paste or retype it.
- **No auto mode**: No ghost text or live suggestions.
- **No diagnosis**: Error diagnosis is not available in CMD.

## Daemon Auto-Start

The integration script checks if the daemon is running on first load and starts it if needed:

```cmd
nudge status >nul 2>&1
if errorlevel 1 (
    start /b nudge start >nul 2>&1
)
```

A "Nudge loaded" message appears in interactive sessions (when `PROMPT` is defined).

## Known Limitations

- No true hotkey support — requires typing `nudge-complete` command
- No inline command replacement — output must be manually used
- No stderr capture or error diagnosis
- No popup selector or multi-candidate display
- Session tracking uses `%RANDOM%` which provides limited uniqueness
- Doskey macros are session-scoped (lost when CMD closes unless using AutoRun)

## Boundaries

- CMD integration is intentionally minimal
- For a better Windows experience, use PowerShell instead
- If you need auto mode on Windows, PowerShell 7.2+ is required
