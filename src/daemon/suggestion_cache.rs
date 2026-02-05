//! Suggestion cache and key building.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

use crate::daemon::sanitizer;
use crate::protocol::CompletionRequest;
use crate::protocol::CompletionResponse;

pub struct SuggestionKey;

impl SuggestionKey {
    #[allow(dead_code)]
    pub fn build(
        req: &CompletionRequest,
        git_root: Option<&PathBuf>,
        git_state: Option<&str>,
        shell_mode: &str,
        time_bucket: Option<u64>,
        prefix_bytes: usize,
    ) -> String {
        Self::build_with_patterns(
            req,
            git_root,
            git_state,
            shell_mode,
            time_bucket,
            prefix_bytes,
            &[],
        )
    }

    pub fn build_with_patterns(
        req: &CompletionRequest,
        git_root: Option<&PathBuf>,
        git_state: Option<&str>,
        shell_mode: &str,
        time_bucket: Option<u64>,
        prefix_bytes: usize,
        custom_patterns: &[String],
    ) -> String {
        let cursor = req.cursor_pos.min(req.buffer.len());
        let prefix_raw = &req.buffer[..cursor];

        let (sanitized_prefix, _) = sanitizer::sanitize_string(prefix_raw, custom_patterns);
        let truncated = truncate_utf8(&sanitized_prefix, prefix_bytes);
        let prefix_hash = hash_hex_16(truncated.as_bytes());

        let path_for_hash = git_root.unwrap_or(&req.cwd);
        let cwd_hash = hash_hex_16(normalize_path(path_for_hash).as_bytes());

        let git_input = git_state.unwrap_or("nogit");
        let git_hash = hash_hex_16(git_input.as_bytes());

        let shell_mode_norm = shell_mode.to_lowercase();
        // time_bucket is intentionally ignored - see docs/plans/2026-02-05-auto-mode-widget-refactor.md
        // Auto mode has delay debounce, manual mode users don't repeat same prefix
        // So "flicker prevention" via time_bucket is unnecessary
        let _ = time_bucket;

        format!(
            "sk:v1:{}:{}:{}:{}",
            prefix_hash, cwd_hash, git_hash, shell_mode_norm
        )
    }
}

fn truncate_utf8(input: &str, max_bytes: usize) -> String {
    if max_bytes == 0 || input.is_empty() {
        return String::new();
    }
    if input.len() <= max_bytes {
        return input.to_string();
    }

    let mut end = max_bytes.min(input.len());
    while end > 0 && !input.is_char_boundary(end) {
        end -= 1;
    }
    input[..end].to_string()
}

fn normalize_path(path: &Path) -> String {
    let normalized = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut s = normalized.to_string_lossy().to_string();
    if cfg!(windows) {
        s = s.to_lowercase();
    }
    s
}

fn hash_hex_16(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes);
    let mut out = String::with_capacity(32);
    for b in digest.as_bytes().iter().take(16) {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

#[derive(Clone)]
pub struct CacheEntry {
    pub response: CompletionResponse,
    pub created_at_ms: u64,
    pub ttl_ms: u64,
    pub negative: bool,
    pub refreshing: bool,
}

pub struct CacheHit {
    pub response: CompletionResponse,
    pub age_ms: u64,
    #[allow(dead_code)]
    pub is_stale: bool,
    pub should_refresh: bool,
    #[allow(dead_code)]
    pub negative: bool,
}

pub struct SuggestionCache {
    capacity: usize,
    stale_ratio: f32,
    entries: HashMap<String, CacheEntry>,
    order: VecDeque<String>,
}

impl SuggestionCache {
    pub fn new(capacity: usize, stale_ratio: f32) -> Self {
        Self {
            capacity,
            stale_ratio,
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get(&mut self, key: &str, now_ms: u64) -> Option<CompletionResponse> {
        self.get_with_state(key, now_ms).map(|hit| hit.response)
    }

    pub fn get_with_state(&mut self, key: &str, now_ms: u64) -> Option<CacheHit> {
        let (age_ms, ttl_ms) = {
            let entry = self.entries.get(key)?;
            let age_ms = now_ms.saturating_sub(entry.created_at_ms);
            (age_ms, entry.ttl_ms)
        };

        if age_ms > ttl_ms {
            self.remove(key);
            return None;
        }

        let (response, is_stale, should_refresh, negative) = {
            let entry = self.entries.get_mut(key)?;
            let is_stale = (age_ms as f32) >= (entry.ttl_ms as f32 * self.stale_ratio);
            let should_refresh = is_stale && !entry.refreshing;
            if should_refresh {
                entry.refreshing = true;
            }
            (
                entry.response.clone(),
                is_stale,
                should_refresh,
                entry.negative,
            )
        };

        self.touch(key);

        Some(CacheHit {
            response,
            age_ms,
            is_stale,
            should_refresh,
            negative,
        })
    }

    pub fn insert(
        &mut self,
        key: String,
        response: CompletionResponse,
        now_ms: u64,
        ttl_ms: u64,
        negative: bool,
    ) {
        if self.capacity == 0 {
            return;
        }

        if self.entries.contains_key(&key) {
            self.remove(&key);
        }

        while self.entries.len() >= self.capacity {
            if let Some(old_key) = self.order.pop_front() {
                self.entries.remove(&old_key);
            } else {
                break;
            }
        }

        self.entries.insert(
            key.clone(),
            CacheEntry {
                response,
                created_at_ms: now_ms,
                ttl_ms,
                negative,
                refreshing: false,
            },
        );
        self.order.push_back(key);
    }

    fn touch(&mut self, key: &str) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.push_back(key.to_string());
        }
    }

    fn remove(&mut self, key: &str) {
        self.entries.remove(key);
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::CompletionRequest;
    use crate::protocol::CompletionResponse;
    use std::path::PathBuf;

    #[test]
    fn test_utf8_safe_truncate() {
        let input = "你好世界"; // 12 bytes
        let truncated = truncate_utf8(input, 5);
        assert!(truncated.is_char_boundary(truncated.len()));
        assert!(truncated.len() <= 5);
    }

    #[test]
    fn test_time_bucket_ignored() {
        // time_bucket is intentionally ignored in cache key
        // See docs/plans/2026-02-05-auto-mode-widget-refactor.md
        let req = CompletionRequest::new(
            "zsh-1".into(),
            "git st".into(),
            6,
            PathBuf::from("/tmp"),
            None,
        );
        let key_manual = SuggestionKey::build(&req, None, None, "zsh-inline", None, 80);
        let key_auto = SuggestionKey::build(&req, None, None, "zsh-auto", Some(123), 80);
        let key_auto_different_bucket =
            SuggestionKey::build(&req, None, None, "zsh-auto", Some(456), 80);

        // Key format: sk:v1:{prefix}:{cwd}:{git}:{mode} (4 hashes + mode, no time_bucket)
        // Should have exactly 5 colon-separated parts: sk, v1, prefix, cwd, git, mode
        assert_eq!(key_manual.matches(':').count(), 5);
        assert_eq!(key_auto.matches(':').count(), 5);

        // Same input with different time_bucket should produce same key
        assert_eq!(key_auto, key_auto_different_bucket);

        // Different shell_mode should produce different keys
        assert_ne!(key_manual, key_auto);
    }

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
}
