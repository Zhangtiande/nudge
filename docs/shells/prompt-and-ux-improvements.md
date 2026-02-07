# Prompt and UX Improvements (Shell-Aware)

This document captures concrete improvements for prompt design and UI surfaces across shell modes.

## Problem Summary

1. `diff/why/risk` are displayed in Bash/Zsh, but prompts do not explicitly guide LLM output for these modules.
2. Shell modes have different capabilities (candidate count, display surface, latency tolerance), but prompt strategy is still largely shared.
3. Users need a short command description alongside suggestions.
4. Zsh `overlay + message` can feel crowded.
5. Cross-shell docs are mixed; behavior differences are hard to discover.

## Current State Snapshot

- Completion system prompt currently asks for only one completed command text.
- Bash popup supports multi-candidate ranking (`bash-popup` mode).
- Zsh (`zsh-inline`/`zsh-auto`) currently uses single-candidate completion.
- `why/diff` in list/overlay are mostly local heuristics, not model-native explanations.

## Direction: Shell-Mode Prompt Contracts

Use a mode-specific output contract instead of one global instruction style.

### A. Bash Popup Contract

- Goal: ranked alternatives for selection UIs.
- Suggested model output shape (internally parsed):
  - `candidates[]` (up to shell budget)
  - each candidate: `command`, `summary_short`, `reason_short`
- Display mapping:
  - `risk`: still from safety module (authoritative)
  - `why`: prefer `reason_short`, fallback to heuristic
  - `diff`: deterministic local computation

### B. Zsh Inline/Auto Contract

- Goal: one strong suggestion + tiny explanation.
- Suggested model output shape:
  - `command`
  - `summary_short` (single short sentence)
- Display mapping:
  - Overlay compact line uses `risk + short diff + key hint`
  - Expanded mode may include `summary_short`

### C. PowerShell/CMD Contract

- Goal: keep plain completion stable and cheap.
- Suggested shape:
  - `command`
  - optional `summary_short` for future non-intrusive hints

## Short Command Description

Add optional `summary_short` to suggestion metadata:

- Intended length: 8-20 words.
- Tone: direct action intent (no tutorial).
- Example:
  - command: `git restore --staged src/main.rs`
  - summary: `Unstage changes for src/main.rs only`

Fallback policy:

- If model does not provide a summary, no additional text is shown.
- Do not block completion on summary generation.

## Zsh Overlay Crowding Plan

1. Define verbosity levels in config:
   - `compact` (default), `standard`, `full`
2. Default `compact` content:
   - `[risk] diff:<trimmed> (F1 details, <accept-key> accept)`
3. Move long fields behind explicit toggle:
   - `warning`, `full command`, `reason_short`
4. Backend-aware truncation:
   - `message`: larger budget
   - `rprompt`: smaller budget + strict truncation
5. Redraw throttling:
   - skip overlay refresh if rendered payload is unchanged

## Documentation Split Plan

1. Keep this directory (`docs/shells/`) as the shell behavior entry point.
2. Maintain one focused page per shell capability profile.
3. Keep cross-shell docs limited to shared concepts only.

## Additional Optimizations

1. Introduce evaluation metrics by shell mode:
   - acceptance rate, manual edits after accept, high-risk decline rate, suggestion latency p50/p95.
2. Add prompt regression tests:
   - validate schema parse, fallback safety, and truncated UI formatting.
3. Add config-level feature flags:
   - independently control multi-candidate, explanation density, summary output.
4. Keep transport backward compatible:
   - legacy plain text behavior should keep working until all integrations migrate.

## Incremental Rollout (Atomic Changes)

1. Add mode-aware prompt templates (no protocol changes).
2. Add optional `summary_short` in protocol + parser.
3. Wire Bash popup to consume `reason_short`/`summary_short`.
4. Add Zsh overlay verbosity and truncation budgets.
5. Update docs and examples per shell.
