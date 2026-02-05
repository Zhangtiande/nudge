//! Suggestion cache and key building.

use std::path::{Path, PathBuf};

use crate::daemon::sanitizer;
use crate::protocol::CompletionRequest;

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
        let cursor = req.cursor_pos.min(req.buffer.len());
        let prefix_raw = &req.buffer[..cursor];

        let (sanitized_prefix, _) = sanitizer::sanitize_string(prefix_raw, &[]);
        let truncated = truncate_utf8(&sanitized_prefix, prefix_bytes);
        let prefix_hash = hash_hex_16(truncated.as_bytes());

        let path_for_hash = git_root.unwrap_or(&req.cwd);
        let cwd_hash = hash_hex_16(normalize_path(path_for_hash).as_bytes());

        let git_input = git_state.unwrap_or("nogit");
        let git_hash = hash_hex_16(git_input.as_bytes());

        let shell_mode_norm = shell_mode.to_lowercase();
        let bucket = if shell_mode_norm.ends_with("-auto") {
            time_bucket.unwrap_or(0)
        } else {
            0
        };

        format!(
            "sk:v1:{}:{}:{}:{}:{}",
            prefix_hash, cwd_hash, git_hash, shell_mode_norm, bucket
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
    let h1 = xxhash64(bytes, 0);
    let h2 = xxhash64(bytes, 0x9e37_79b1_85eb_ca87);
    format!("{:016x}{:016x}", h1, h2)
}

fn xxhash64(input: &[u8], seed: u64) -> u64 {
    const PRIME1: u64 = 11400714785074694791;
    const PRIME2: u64 = 14029467366897019727;
    const PRIME3: u64 = 1609587929392839161;
    const PRIME4: u64 = 9650029242287828579;
    const PRIME5: u64 = 2870177450012600261;

    let len = input.len();
    let mut i = 0usize;
    let mut hash: u64;

    if len >= 32 {
        let mut v1 = seed.wrapping_add(PRIME1).wrapping_add(PRIME2);
        let mut v2 = seed.wrapping_add(PRIME2);
        let mut v3 = seed;
        let mut v4 = seed.wrapping_sub(PRIME1);

        while i + 32 <= len {
            v1 = round(v1, read_u64(&input[i..]));
            v2 = round(v2, read_u64(&input[i + 8..]));
            v3 = round(v3, read_u64(&input[i + 16..]));
            v4 = round(v4, read_u64(&input[i + 24..]));
            i += 32;
        }

        hash = v1.rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));

        hash = merge_round(hash, v1);
        hash = merge_round(hash, v2);
        hash = merge_round(hash, v3);
        hash = merge_round(hash, v4);
    } else {
        hash = seed.wrapping_add(PRIME5);
    }

    hash = hash.wrapping_add(len as u64);

    while i + 8 <= len {
        let k1 = round(0, read_u64(&input[i..]));
        hash ^= k1;
        hash = hash
            .rotate_left(27)
            .wrapping_mul(PRIME1)
            .wrapping_add(PRIME4);
        i += 8;
    }

    if i + 4 <= len {
        hash ^= (read_u32(&input[i..]) as u64).wrapping_mul(PRIME1);
        hash = hash
            .rotate_left(23)
            .wrapping_mul(PRIME2)
            .wrapping_add(PRIME3);
        i += 4;
    }

    while i < len {
        hash ^= (input[i] as u64).wrapping_mul(PRIME5);
        hash = hash.rotate_left(11).wrapping_mul(PRIME1);
        i += 1;
    }

    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME3);
    hash ^= hash >> 32;
    hash
}

fn round(acc: u64, input: u64) -> u64 {
    let mut acc = acc.wrapping_add(input.wrapping_mul(14029467366897019727));
    acc = acc.rotate_left(31);
    acc.wrapping_mul(11400714785074694791)
}

fn merge_round(acc: u64, val: u64) -> u64 {
    let mut acc = acc ^ round(0, val);
    acc = acc.wrapping_mul(11400714785074694791).wrapping_add(9650029242287828579);
    acc
}

fn read_u64(slice: &[u8]) -> u64 {
    u64::from_le_bytes(slice[..8].try_into().unwrap())
}

fn read_u32(slice: &[u8]) -> u32 {
    u32::from_le_bytes(slice[..4].try_into().unwrap())
}

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
