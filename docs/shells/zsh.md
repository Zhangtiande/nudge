# Zsh Guide

## Current Behavior

- Integration script: `shell/integration.zsh`
- Shell modes:
  - `zsh-inline` (manual completion)
  - `zsh-auto` (event-driven auto fetch)
- Auto mode has two rendering paths:
  - Ghost owner = `nudge`: ghost text (`POSTDISPLAY`) + optional overlay details
  - Ghost owner = `autosuggestions`: Nudge overlay only (accept with `Ctrl+G`)
- Overlay backend:
  - `message` (`zle -M`)
  - `rprompt` (`RPS1`/`RPROMPT`)

## Candidate and Explanation Model

- Zsh currently consumes a single primary suggestion in plain mode.
- `risk/why/diff` text shown in overlay is generated in shell integration logic.
- `F1` toggles expanded explanation details.

## Known Gaps

- No multi-candidate flow for Zsh today.
- Overlay line can become dense when autosuggestions + diagnosis + explanation details coexist.
- Prompt contract is still generic, not tuned for Zsh overlay width constraints.

## Recommended Overlay Densification Controls

1. Introduce explicit overlay verbosity levels:
   - `compact` (default): `risk + short diff + accept hint`
   - `standard`: add `why`
   - `full`: add warning + expanded suggestion metadata
2. Set backend-specific width budgets:
   - `message`: wider budget
   - `rprompt`: tighter budget + earlier truncation
3. Prioritize stable fields:
   - Keep risk badge and accept hint always visible.
   - Truncate details first, never controls.
4. Add rate limiting for message refresh:
   - Avoid noisy redraw on minor cursor activity.

## Prompt Strategy Direction

- For `zsh-auto` / `zsh-inline`, prefer one high-confidence command plus one short summary.
- Keep response concise to match overlay and ghost rendering budgets.
