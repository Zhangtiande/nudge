# PowerShell Guide

PowerShell uses inline manual completion and supports failure diagnosis.

## Mode Used

- `ps-inline`

## Quick Use

- Press `Ctrl+E` for completion
- Use `nudge diagnose` flow through integration on failed commands

## Setup

```powershell
nudge setup powershell --force
nudge restart
```

## Boundaries

- No auto ghost-text mode
- Behavior depends on PowerShell profile loading and host capabilities
