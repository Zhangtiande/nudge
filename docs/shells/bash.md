# Bash Guide

Bash uses manual trigger mode with optional popup selector for ranked candidates.

## Modes Used

- `bash-inline`: `Ctrl+E` fast single-candidate path
- `bash-popup`: `Alt+/` candidate selector

## Quick Use

- Type partial command, press `Ctrl+E` for immediate apply
- Press `Alt+/` to open popup list and choose candidate

## Popup Behavior

- Candidates are requested with `--format list`
- Row format: `risk<TAB>command<TAB>warning<TAB>why<TAB>diff`
- Daemon asks LLM for multiple popup candidates; local history fills gaps only when needed
- High-risk candidates require confirmation by default

## Useful Env Vars

- `NUDGE_POPUP_BACKEND=auto|fzf|sk|peco|builtin`
- `NUDGE_POPUP_SHOW_PREVIEW=0|1`
- `NUDGE_POPUP_HEIGHT=70%`
- `NUDGE_POPUP_CONFIRM_RISKY=0|1`

## Boundaries

- No true auto ghost-text in Bash
- Popup UX depends on selector backend availability
