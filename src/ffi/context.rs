//! NudgeContext - FFI context with embedded Tokio runtime
//!
//! This module provides the NudgeContext struct which holds the configuration
//! and Tokio runtime needed for FFI completion calls.

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;

use super::auto_mode::AutoModeState;
use crate::config::Config;

/// Context for FFI operations
///
/// This struct holds all state needed for FFI completion calls:
/// - Configuration loaded from file
/// - Tokio runtime for async operations
/// - Cache for completion results (keyed by hash of input)
/// - Last error message for error retrieval
/// - Auto mode state for background completion
pub struct NudgeContext {
    /// Loaded configuration
    pub config: Config,
    /// Tokio runtime for async operations
    pub runtime: Runtime,
    /// Simple cache for recent completions (hash -> suggestion)
    pub cache: Arc<Mutex<HashMap<u64, String>>>,
    /// Last error message (for nudge_get_error)
    pub last_error: Arc<Mutex<Option<String>>>,
    /// Auto mode state
    pub auto_mode: AutoModeState,
    /// Auto mode delay in milliseconds
    pub auto_delay_ms: AtomicU32,
}

impl NudgeContext {
    /// Create a new NudgeContext with the given configuration
    pub fn new(config: Config) -> Result<Self, String> {
        // Create a multi-threaded Tokio runtime
        let runtime =
            Runtime::new().map_err(|e| format!("Failed to create Tokio runtime: {}", e))?;

        // Get auto delay from config (convert u64 to u32, clamping if necessary)
        let auto_delay_ms = config.trigger.auto_delay_ms.min(u32::MAX as u64) as u32;

        Ok(Self {
            config,
            runtime,
            cache: Arc::new(Mutex::new(HashMap::new())),
            last_error: Arc::new(Mutex::new(None)),
            auto_mode: AutoModeState::new(),
            auto_delay_ms: AtomicU32::new(auto_delay_ms),
        })
    }

    /// Set the last error message
    pub fn set_error(&self, message: &str) {
        if let Ok(mut guard) = self.last_error.lock() {
            *guard = Some(message.to_string());
        }
    }

    /// Get the last error message
    pub fn get_error(&self) -> Option<String> {
        if let Ok(guard) = self.last_error.lock() {
            guard.clone()
        } else {
            None
        }
    }

    /// Clear the last error
    pub fn clear_error(&self) {
        if let Ok(mut guard) = self.last_error.lock() {
            *guard = None;
        }
    }

    /// Get a cached completion result
    pub fn get_cached(&self, hash: u64) -> Option<String> {
        if let Ok(guard) = self.cache.lock() {
            guard.get(&hash).cloned()
        } else {
            None
        }
    }

    /// Store a completion result in the cache
    pub fn set_cached(&self, hash: u64, suggestion: String) {
        if let Ok(mut guard) = self.cache.lock() {
            // Limit cache size to prevent unbounded growth
            const MAX_CACHE_SIZE: usize = 100;
            if guard.len() >= MAX_CACHE_SIZE {
                // Simple eviction: clear the cache when full
                guard.clear();
            }
            guard.insert(hash, suggestion);
        }
    }
}
