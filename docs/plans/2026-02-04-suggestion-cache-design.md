# Suggestion Cache & Key Design (v1)

Date: 2026-02-04

## Summary
Implement a daemon-side suggestion cache to meet sub-20ms completion requirements (e.g., PowerShell 7+). The cache uses a deterministic `SuggestionKey` computed in the daemon from normalized request fields, with LRU + TTL eviction, plus negative caching and stale-while-revalidate to reduce LLM calls under rapid typing. The cache stores full `CompletionResponse` objects and returns cache metadata to the shell.

## Goals
- Provide fast cache hits under strict latency budgets.
- Ensure cross-shell consistency of cache keys.
- Avoid storing sensitive raw input in the cache key or value.
- Support stale-while-revalidate and negative caching for stability.

## Non-goals
- Persistent on-disk cache.
- Advanced per-project policies in v1 (left for v2).
- Multi-process shared cache.

## Inputs From Shell (IPC)
`CompletionRequest` is extended with optional fields to preserve backward compatibility:
- `git_root: Option<PathBuf>`
- `git_state: Option<String>`  // formatted as `repo_id|branch|dirty|staged`
- `shell_mode: Option<String>` // e.g., `zsh-auto`, `zsh-inline`, `ps-inline`, `bash-popup`
- `time_bucket: Option<u64>`   // only provided for auto mode

Fallbacks in daemon when missing:
- `git_root = None`
- `git_state = None`
- `shell_mode = detect_from_session_id()` or `"unknown"`
- `time_bucket = None`

## SuggestionKey v1
Key is constructed **in daemon**, based on sanitized and normalized inputs. No raw prefix is stored.

### Prefix handling
- `prefix = buffer[0..cursor_pos]`
- Apply Sanitizer before hashing.
- UTF-8 safe truncation to `N` bytes (default `N=80`, configurable).
- `prefix_norm` (whitespace compression) is **disabled by default**.

### Path handling
- `cwd_hash` uses `git_root` if present; otherwise `cwd`.
- Try `canonicalize()`; on failure, use original path.
- On Windows, normalize to lowercase before hashing.

### Git handling
- `git_hash` is the hash of `git_state` string (`repo_id|branch|dirty|staged`).
- If `git_state` is missing, use literal `"nogit"`.

### Hash algorithm
- `blake3`, output hex of the first 16 bytes (128-bit).

### Key format
```
sk:v1:{prefix_hash}:{cwd_hash}:{git_hash}:{shell_mode}:{time_bucket}
```
- `time_bucket` is **only present for auto mode**; if absent, use literal `"0"`.

## Cache Structure
`SuggestionCache` stores:
- `key -> CacheEntry`
- `CacheEntry`:
  - `response: CompletionResponse`
  - `created_at_ms`
  - `last_access_ms`
  - `ttl_ms`
  - `negative: bool`
  - `refreshing: bool`

Default config:
- Capacity: `1024`
- TTL:
  - `auto` mode: `3s`
  - `manual/ps-inline/bash-popup`: `15s`
  - `negative`: `2s`
- Stale threshold: `80%` of TTL

Eviction: LRU with TTL enforcement.

## Request Flow
1. Daemon builds `SuggestionKey`.
2. Lookup in cache:
   - **Fresh hit**: return cached response immediately; set `cache_hit=true`, `cache_age_ms`.
   - **Stale (age >= 0.8 * ttl)**: return cached response; trigger async refresh if not already `refreshing`.
   - **Expired**: treat as miss.
3. Cache miss: run full context + LLM pipeline; store response with TTL.
4. Negative cache: if LLM returns empty suggestions or error, store with `negative=true`, TTL=2s.

## Response Metadata
Extend `CompletionResponse` with optional fields:
- `cache_hit: Option<bool>`
- `cache_age_ms: Option<u64>`

These are omitted for older clients and remain backward compatible.

## Privacy & Security
- Do not store raw `prefix` in key or cache value.
- Sanitizer runs before hashing the prefix.
- Short TTLs limit exposure window.

## Error Handling
- If normalization fails (canonicalize, missing fields), degrade gracefully and proceed.
- Stale refresh is best-effort and never blocks request responses.
- Refresh failures do not overwrite existing cached entries.

## Observability
Debug logs (only at debug level):
- `cache_hit` / `cache_miss`
- `cache_age_ms`
- `cache_state` = `fresh | stale | negative`
- `refresh_triggered` (bool)

## Testing
Unit tests:
- `SuggestionKey`:
  - UTF-8 safe truncation
  - `time_bucket` inclusion only in auto mode
  - `git_root` preferred over `cwd`
  - Windows lowercase normalization (cfg)
- `SuggestionCache`:
  - LRU eviction order
  - TTL expiry
  - negative TTL
  - stale threshold refresh trigger (mockable time source)

Integration tests:
- Cache hit returns `cache_hit=true` and skips LLM call (mock LLM).
- Miss followed by hit for same key.

## Future Work
- `policy_hash` and per-project cache policy.
- Optional session-local L1 cache to reduce lock contention.
- Configurable `prefix_norm` whitespace compression toggle.
