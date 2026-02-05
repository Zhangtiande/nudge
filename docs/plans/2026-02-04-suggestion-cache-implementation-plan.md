# Suggestion Cache Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a daemon-side suggestion cache with deterministic `SuggestionKey` to return completions within strict latency budgets, including LRU+TTL, negative caching, and stale-while-revalidate.

**Architecture:** Extend the IPC request with optional cache-related fields, build `SuggestionKey` in the daemon (sanitized, hashed), then check a shared `SuggestionCache` before context+LLM. Cache hits return immediately with metadata; stale hits trigger async refresh without blocking.

**Tech Stack:** Rust (tokio, serde), `lru` crate, `blake3` crate, shell scripts (bash/zsh/powershell).

---

### Task 1: Extend IPC + Config for Cache Settings

**Files:**
- Modify: `src/protocol.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- Modify: `src/client/mod.rs`
- Modify: `src/config.rs`
- Modify: `docs/configuration.md`
- Modify: `tests/completion_test.rs`

**Step 1: Write failing tests for new IPC fields**

Add to `tests/completion_test.rs`:

```rust
#[test]
fn test_completion_request_deserialize_with_cache_fields() {
    let json = serde_json::json!({
        "session_id": "zsh-123",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "buffer": "git st",
        "cursor_pos": 6,
        "cwd": "/tmp",
        "last_exit_code": 0,
        "git_root": "/tmp",
        "git_state": "repo|main|0|0",
        "shell_mode": "zsh-inline",
        "time_bucket": 12345
    });

    let req: crate::protocol::CompletionRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.git_state.as_deref(), Some("repo|main|0|0"));
    assert_eq!(req.shell_mode.as_deref(), Some("zsh-inline"));
    assert_eq!(req.time_bucket, Some(12345));
}

#[test]
fn test_completion_response_serializes_cache_meta() {
    let mut resp = crate::protocol::CompletionResponse::success(
        "req-1".to_string(),
        vec![crate::protocol::Suggestion::new("ls".to_string())],
        0,
    );
    resp.cache_hit = Some(true);
    resp.cache_age_ms = Some(12);
    let serialized = serde_json::to_string(&resp).unwrap();
    assert!(serialized.contains("cache_hit"));
    assert!(serialized.contains("cache_age_ms"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test completion_test::test_completion_request_deserialize_with_cache_fields completion_test::test_completion_response_serializes_cache_meta`
Expected: FAIL (unknown fields / missing struct fields)

**Step 3: Implement protocol changes**

Update `CompletionRequest` and `CompletionResponse` in `src/protocol.rs`:

```rust
pub struct CompletionRequest {
    // existing fields...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_root: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_bucket: Option<u64>,
}

pub struct CompletionResponse {
    // existing fields...
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_hit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_age_ms: Option<u64>,
}
```

Keep `CompletionRequest::new(...)` signature but set new fields to `None`.

**Step 4: Add cache config defaults**

In `src/config.rs` add:

```rust
pub struct CacheConfig {
    pub capacity: usize,
    pub prefix_bytes: usize,
    pub ttl_auto_ms: u64,
    pub ttl_manual_ms: u64,
    pub ttl_negative_ms: u64,
    pub stale_ratio: f32,
}
```

Add `cache: CacheConfig` to `Config` with defaults:
- `capacity=1024`
- `prefix_bytes=80`
- `ttl_auto_ms=3000`
- `ttl_manual_ms=15000`
- `ttl_negative_ms=2000`
- `stale_ratio=0.8`

Update `docs/configuration.md` to include a `cache:` section with those defaults.

**Step 5: Extend CLI for new optional fields**

In `src/cli.rs` add optional flags to `Command::Complete`:
- `--git-root <path>`
- `--git-state <string>`
- `--shell-mode <string>`
- `--time-bucket <u64>`

Wire through `src/main.rs` into `client::complete` and update `src/client/mod.rs` to set these fields on the request.

**Step 6: Run tests again**

Run: `cargo test completion_test::test_completion_request_deserialize_with_cache_fields completion_test::test_completion_response_serializes_cache_meta`
Expected: PASS

**Step 7: Commit**

```bash
git add src/protocol.rs src/config.rs src/cli.rs src/main.rs src/client/mod.rs docs/configuration.md tests/completion_test.rs
git commit -m "feat(protocol): add cache fields and config defaults"
```

---

### Task 2: Implement SuggestionKey Builder

**Files:**
- Create: `src/daemon/suggestion_cache.rs`
- Modify: `src/daemon/mod.rs`

**Step 1: Write failing tests for key building**

In `src/daemon/suggestion_cache.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::CompletionRequest;
    use std::path::PathBuf;

    #[test]
    fn test_utf8_safe_truncate() {
        let input = "你好世界"; // 12 bytes
        let truncated = truncate_utf8(input, 5);
        assert!(truncated.is_char_boundary(truncated.len()));
        assert!(truncated.as_bytes().len() <= 5);
    }

    #[test]
    fn test_time_bucket_only_for_auto() {
        let req = CompletionRequest::new(
            "zsh-1".into(),
            "git st".into(),
            6,
            PathBuf::from("/tmp"),
            None,
        );
        let key_manual = SuggestionKey::build(&req, None, None, "zsh-inline", None, 80);
        let key_auto = SuggestionKey::build(&req, None, None, "zsh-auto", Some(123), 80);
        assert!(key_manual.ends_with(":0"));
        assert!(key_auto.ends_with(":123"));
    }
}
```

**Step 2: Run tests to verify failure**

Run: `cargo test suggestion_cache::tests::test_utf8_safe_truncate suggestion_cache::tests::test_time_bucket_only_for_auto`
Expected: FAIL (missing functions/types)

**Step 3: Implement key builder**

Add helper functions and struct:

```rust
pub struct SuggestionKey;

impl SuggestionKey {
    pub fn build(
        req: &CompletionRequest,
        git_root: Option<&PathBuf>,
        git_state: Option<&str>,
        shell_mode: &str,
        time_bucket: Option<u64>,
        prefix_bytes: usize,
    ) -> String {
        // sanitize prefix, truncate, hash
        // normalize path, hash
        // hash git_state or "nogit"
        // append time_bucket only if auto
    }
}
```

Implement:
- `truncate_utf8()` helper
- `hash_hex_16()` helper using `blake3`
- path normalization with canonicalize + windows lowercase
- use sanitized prefix from `sanitizer::sanitize_string`

**Step 4: Run tests again**

Run: `cargo test suggestion_cache::tests::test_utf8_safe_truncate suggestion_cache::tests::test_time_bucket_only_for_auto`
Expected: PASS

**Step 5: Commit**

```bash
git add src/daemon/suggestion_cache.rs src/daemon/mod.rs
git commit -m "feat(cache): add suggestion key builder"
```

---

### Task 3: Implement SuggestionCache (LRU + TTL + Negative + Stale)

**Files:**
- Modify: `src/daemon/suggestion_cache.rs`
- Modify: `Cargo.toml`

**Step 1: Write failing tests for cache behavior**

Add tests in `src/daemon/suggestion_cache.rs`:

```rust
#[test]
fn test_cache_ttl_expiry() {
    let mut cache = SuggestionCache::new(2, 0.8);
    let response = CompletionResponse::success("req".into(), vec![], 0);
    cache.insert("k".into(), response, 1000, 10, false);
    assert!(cache.get("k", 1005).is_some());
    assert!(cache.get("k", 1011).is_none());
}

#[test]
fn test_cache_stale_threshold() {
    let mut cache = SuggestionCache::new(2, 0.8);
    let response = CompletionResponse::success("req".into(), vec![], 0);
    cache.insert("k".into(), response, 1000, 10, false);
    let hit = cache.get_with_state("k", 1008).unwrap();
    assert!(hit.is_stale);
    assert!(hit.should_refresh);
}
```

**Step 2: Run tests to verify failure**

Run: `cargo test suggestion_cache::tests::test_cache_ttl_expiry suggestion_cache::tests::test_cache_stale_threshold`
Expected: FAIL (missing cache implementation)

**Step 3: Implement cache structures**

Use `lru` crate and add to `Cargo.toml`:

```toml
lru = "0.12"
blake3 = "1"
```

Implement:
- `CacheEntry` with `response`, `created_at_ms`, `ttl_ms`, `negative`, `refreshing`
- `CacheHit` with `response`, `age_ms`, `is_stale`, `should_refresh`
- `SuggestionCache::get_with_state(key, now_ms)` that:
  - returns `None` if expired
  - returns `CacheHit` if fresh/stale
  - sets `refreshing=true` when stale and not already refreshing
- `SuggestionCache::insert(...)` with LRU eviction

**Step 4: Run tests again**

Run: `cargo test suggestion_cache::tests::test_cache_ttl_expiry suggestion_cache::tests::test_cache_stale_threshold`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/daemon/suggestion_cache.rs
git commit -m "feat(cache): add LRU TTL negative and stale logic"
```

---

### Task 4: Wire Cache Into Daemon + Shell Integrations

**Files:**
- Modify: `src/daemon/server.rs`
- Modify: `src/daemon/mod.rs`
- Modify: `shell/integration.zsh`
- Modify: `shell/integration.bash`
- Modify: `shell/integration.ps1`
- Modify: `shell/NudgePredictor/NudgePredictor.psm1`
- Modify: `shell/integration.cmd`

**Step 1: Refactor completion path for cache**

In `src/daemon/server.rs`:
- Create shared `SuggestionCache` in `run()` and pass to connection handler.
- Split completion logic into `compute_completion(...)` (context + LLM) and a cached wrapper.
- Build `SuggestionKey` from request + config.
- On cache hit: return cached response with `cache_hit=true` and `cache_age_ms`.
- On stale hit: return cached response and `tokio::spawn` a refresh (if `should_refresh`).
- On miss: compute, then insert into cache; store negative on empty/error.

**Step 2: Update shell integrations**

Add `--shell-mode` to all `nudge complete` calls:
- zsh manual: `zsh-inline`
- zsh auto: `zsh-auto` + `--time-bucket $((EPOCHSECONDS / 2))`
- bash: `bash-popup`
- PowerShell manual: `ps-inline`
- PowerShell predictor (auto): `ps-auto` + time bucket via `[DateTimeOffset]::UtcNow.ToUnixTimeSeconds() / 2`
- cmd: `cmd-inline`

**Step 3: Run full test suite**

Run: `cargo test`
Expected: PASS

**Step 4: Commit**

```bash
git add src/daemon/server.rs src/daemon/mod.rs shell/integration.zsh shell/integration.bash shell/integration.ps1 shell/NudgePredictor/NudgePredictor.psm1 shell/integration.cmd
git commit -m "feat(cache): integrate daemon cache and shell modes"
```

---

## Execution Options
Plan complete and saved to `docs/plans/2026-02-04-suggestion-cache-implementation-plan.md`.

Two execution options:

1. Subagent-Driven (this session) - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. Parallel Session (separate) - Open new session with executing-plans, batch execution with checkpoints

Which approach?
