# Zsh Flagship L3 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver a flagship Zsh experience with conflict-safe coexistence, richer partial accept controls, explanation UX, and diagnosability.

**Architecture:** Keep ghost ownership explicit (`nudge` vs `autosuggestions`), route slow suggestions through an overlay channel, and enforce response arbitration locally by generation ID. Replace sleep-based debounce with event-driven triggering (`PENDING`/`KEYS_QUEUED_COUNT`) to eliminate redraw jitter. Add a dedicated `doctor zsh` command to inspect runtime state and key binding health.

**Tech Stack:** Rust (clap/tokio), Zsh ZLE widgets/hooks, existing daemon IPC protocol.

### Task 1: Config and Runtime Surface

**Files:**
- Modify: `src/config.rs`
- Modify: `src/commands/info.rs`
- Modify: `config/config.default.yaml.template`
- Modify: `docs/configuration.md`

Steps:
1. Add `trigger.zsh_overlay_backend` enum (`message`/`rprompt`) with default.
2. Expose the field in `nudge info --field`.
3. Add tests for default and YAML parsing.
4. Update template/docs.

### Task 2: Event-Driven Fetch + Arbitration

**Files:**
- Modify: `shell/integration.zsh`
- Test: `tests/zsh_integration_test.rs`

Steps:
1. Remove sleep-based debounce path and timer fd handlers.
2. Use event-triggered fetch only when queue drains (`PENDING`/`KEYS_QUEUED_COUNT`).
3. Add request generation IDs in async pipeline and discard stale responses.
4. Keep existing cancellation semantics.

### Task 3: Enhanced Partial Accept

**Files:**
- Modify: `shell/integration.zsh`
- Test: `tests/zsh_integration_test.rs`
- Modify: `docs/auto-mode.md`

Steps:
1. Add accept-by-argument widget (Alt+Right).
2. Add accept-by-segment widget (Ctrl+Right).
3. Bind across keymaps and common terminal sequences.
4. Ensure fallback behavior when no suggestion exists.

### Task 4: Explanation Layer (Why/Risk/Diff + F1)

**Files:**
- Modify: `shell/integration.zsh`
- Modify: `docs/auto-mode.md`
- Modify: `README.md`
- Modify: `README_zh.md`

Steps:
1. Compute explanation summary from current buffer + suggestion + warning.
2. Render one-line explanation in overlay channel.
3. Add F1 toggle for expanded details.
4. Ensure no hard ownership conflict with autosuggestions.

### Task 5: Overlay Backend Switch

**Files:**
- Modify: `shell/integration.zsh`
- Modify: `docs/auto-mode.md`

Steps:
1. Support `message` backend via `zle -M`.
2. Support `rprompt` backend with save/restore semantics.
3. Prevent redundant redraw by message dedupe.

### Task 6: `nudge doctor zsh`

**Files:**
- Modify: `src/cli.rs`
- Create: `src/commands/doctor.rs`
- Modify: `src/commands/mod.rs`
- Modify: `src/main.rs`
- Modify: `docs/cli-reference.md`

Steps:
1. Add `doctor` subcommand with `zsh` target.
2. Report config + integration checks + keybinding snapshot.
3. Run a small completion benchmark and print p50/p95.
4. Include clear remediation hints for common conflicts.

### Task 7: Verification

**Files:**
- Test: `tests/zsh_integration_test.rs`

Steps:
1. Run targeted Zsh integration tests.
2. Run config parsing tests.
3. Run `zsh -n shell/integration.zsh`, `cargo fmt --check`, `cargo check`.
