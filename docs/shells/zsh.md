# Zsh Guide

Zsh is the most complete integration path (manual + auto + diagnosis).

## Modes Used

- `zsh-inline`: `Ctrl+E` fast single-candidate path
- `zsh-auto`: live suggestions/overlay while typing

## Quick Use

- Manual: type command -> `Ctrl+E`
- Auto: set `trigger.mode: auto` and restart daemon

## Auto/Overlay Notes

- `zsh_ghost_owner: nudge` -> Nudge controls ghost text
- `zsh_ghost_owner: autosuggestions` -> Nudge uses overlay, accept via `Ctrl+G`
- `zsh_overlay_backend: message|rprompt`
- `F1` toggles explanation details (`why/diff/risk`)

## Diagnosis

- Supported in Zsh when `diagnosis.enabled: true`
- Failed commands can show fix suggestions inline

## Boundaries

- Overlay density depends on terminal width
- Function key mapping (`F1`) may vary across terminal apps
