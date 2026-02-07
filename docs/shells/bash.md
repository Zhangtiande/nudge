# Bash Guide

## Current Behavior

- Integration script: `shell/integration.bash`
- Shell modes sent to daemon:
  - `bash-inline` for `Ctrl+E` manual completion
  - `bash-popup` for `Alt+/` selector
- Primary trigger: `Ctrl+E` (fastest single-candidate path)
- Popup trigger: `Alt+/` (candidate selector)
- Popup backends: `fzf`, `sk`, `peco`, `builtin`

## Fast Path Guarantee

- `Ctrl+E` must stay available.
- `Ctrl+E` always uses `bash-inline` and returns one primary suggestion.
- This path is intentionally the lowest-latency baseline and should not depend on popup ranking.

## Candidate and Explanation Model

- Bash popup requests `nudge complete --format list`.
- List rows are emitted as:
  - `risk<TAB>command<TAB>warning<TAB>why<TAB>diff`
- Current max candidates in daemon popup mode: `6`.
- `risk` is derived from safety warning presence.
- `why` prefers model metadata (`reason_short`, then `summary_short`) with local heuristic fallback.
- `diff` is generated client-side for deterministic display.

## Known Gaps

- Candidate count is fixed at daemon constant today (not yet user-configurable).
- Popup candidates after the first still rely mainly on local ranking of history matches.

## Recommended Improvements

1. Add shell-mode-aware prompt strategy:
   - For `bash-popup`, request ranked candidates with compact rationale.
2. Add structured suggestion metadata in protocol:
   - Candidate fields: `text`, `summary_short`, `reason_short`, `risk_hint`.
3. Keep local fallback:
   - If metadata missing, preserve existing heuristic `why/diff` generation.
4. Add explicit candidate-count control:
   - Use shell-mode defaults but keep config override for upper bound.

## UX Notes

- High-risk candidates already require confirmation (`NUDGE_POPUP_CONFIRM_RISKY`).
- `fzf`/`sk` default to list-only popup (no bottom preview panel), so you can see more candidates at once.
- Set `NUDGE_POPUP_SHOW_PREVIEW=1` if you want the preview panel (`risk/why/diff/warn`) back.
- `NUDGE_POPUP_HEIGHT` controls selector height (default `70%`).
