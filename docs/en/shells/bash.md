# Bash Guide

[English](bash.md) | [中文](../../zh/shells/bash.md)

Bash provides manual completion plus a multi-candidate popup selector — the best way to browse and compare suggestions.

## Modes

| Mode | Trigger | Description |
|---|---|---|
| `bash-inline` | `Ctrl+E` | Fast single-candidate apply |
| `bash-popup` | `Alt+/` | Multi-candidate selector with risk preview |

## Quick Start

```bash
# Single candidate — type and press Ctrl+E
git st<Ctrl+E>
# → git status

# Multi-candidate — type and press Alt+/
docker<Alt+/>
# → Opens popup with numbered candidates to choose from
```

## Popup Selector

The popup selector (`Alt+/`) is Bash's standout feature. It requests multiple candidates from the LLM and presents them in an interactive list.

### How it works

1. Press `Alt+/` → integration calls `nudge complete --format list`
2. Daemon returns multiple candidates with metadata
3. Integration launches a selector UI (fzf, sk, peco, or builtin)
4. You pick a candidate → it replaces your command line

### Row format

Each candidate row contains tab-separated fields:

```
risk<TAB>command<TAB>warning<TAB>why<TAB>diff
```

| Field | Description |
|---|---|
| `risk` | Risk level: `safe`, `moderate`, `dangerous` |
| `command` | The suggested command |
| `warning` | Safety warning (empty if safe) |
| `why` | Brief explanation of what the command does |
| `diff` | What changed compared to your original input |

### Selector backend

The `NUDGE_POPUP_BACKEND` env var controls which selector is used:

| Value | Description |
|---|---|
| `auto` (default) | Auto-detect: tries fzf → sk → peco → builtin |
| `fzf` | Use [fzf](https://github.com/junegunn/fzf) |
| `sk` | Use [skim](https://github.com/lotabout/skim) |
| `peco` | Use [peco](https://github.com/peco/peco) |
| `builtin` | Built-in numbered list (no external dependency) |

Install a selector for the best experience:

```bash
# macOS
brew install fzf

# Ubuntu/Debian
sudo apt install fzf

# Or use without installing anything (builtin fallback)
```

### Candidate generation

The daemon asks the LLM for multiple suggestions in a single request. If the LLM returns fewer candidates than expected, local history may fill remaining slots. The LLM-generated candidates always rank first.

### Risk confirmation

By default, high-risk candidates (e.g., `rm -rf`, destructive operations) require explicit confirmation before being applied.

```bash
# Disable confirmation for risky commands (not recommended)
export NUDGE_POPUP_CONFIRM_RISKY=0
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `NUDGE_POPUP_BACKEND` | `auto` | Selector backend: `auto`, `fzf`, `sk`, `peco`, `builtin` |
| `NUDGE_POPUP_SHOW_PREVIEW` | `1` | Show preview pane with command details |
| `NUDGE_POPUP_HEIGHT` | `70%` | Selector window height |
| `NUDGE_POPUP_CONFIRM_RISKY` | `1` | Require confirmation for dangerous commands |

## Using with fzf/sk/peco

### fzf (recommended)

fzf provides fuzzy search, preview pane, and mouse support:

```bash
export NUDGE_POPUP_BACKEND=fzf
export NUDGE_POPUP_SHOW_PREVIEW=1
```

In the popup, use arrow keys to navigate and Enter to select.

### sk (skim)

Skim is a Rust-based fzf alternative with similar features:

```bash
export NUDGE_POPUP_BACKEND=sk
```

### peco

Peco provides a simpler selection interface:

```bash
export NUDGE_POPUP_BACKEND=peco
```

### builtin

The builtin selector shows a numbered list. Type a number and press Enter:

```bash
export NUDGE_POPUP_BACKEND=builtin
```

```
 1) [safe]      git status
 2) [safe]      git stash
 3) [moderate]  git stash drop
Select [1-3]:
```

## Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `Ctrl+E` does nothing | Integration not sourced | `nudge setup bash --force && source ~/.bashrc` |
| `Alt+/` does nothing | Key binding not set | Check `bind -P \| grep nudge` |
| Popup shows empty list | Daemon not running | `nudge start` |
| fzf not found | Not installed | `brew install fzf` or `apt install fzf` |
| Builtin shows garbled text | Terminal encoding | Ensure UTF-8 locale |

## Boundaries

- No true auto ghost-text in Bash (Bash readline limitations)
- Popup UX depends on selector backend availability
- No error diagnosis in Bash (planned for future)
- Bash 4.0+ is recommended for full readline binding support
