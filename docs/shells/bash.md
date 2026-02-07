# Bash Guide

## Current Behavior

- Integration script: `shell/integration.bash`
- Shell mode sent to daemon: `bash-popup`
- Primary trigger: `Ctrl+E` (apply first suggestion)
- Popup trigger: `Alt+/` (candidate selector)
- Popup backends: `fzf`, `sk`, `peco`, `builtin`

## Candidate and Explanation Model

- Bash popup requests `nudge complete --format list`.
- List rows are emitted as:
  - `risk<TAB>command<TAB>warning<TAB>why<TAB>diff`
- Current max candidates in daemon popup mode: `6`.
- `risk` is derived from safety warning presence.
- `why` and `diff` are currently generated client-side (heuristic), not produced by LLM.

## Known Gaps

- Prompt contract does not explicitly tell LLM that Bash may need ranked alternatives.
- `why` text quality is limited by local heuristics (`prefix completion` / `context rewrite`).
- No short, user-facing command explanation field from model output.

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
- `fzf`/`sk` preview is a good place to show model-provided `summary_short` once available.
