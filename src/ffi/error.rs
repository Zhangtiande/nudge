//! FFI error handling utilities
//!
//! This module provides utilities for safe error handling across the FFI boundary.

use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Mutex;

/// Thread-local storage for the last error message
/// Using a Mutex to ensure thread safety
pub struct ErrorState {
    last_error: Mutex<Option<CString>>,
}

impl ErrorState {
    pub const fn new() -> Self {
        Self {
            last_error: Mutex::new(None),
        }
    }

    /// Set the last error message
    pub fn set(&self, message: &str) {
        if let Ok(mut guard) = self.last_error.lock() {
            *guard = CString::new(message).ok();
        }
    }

    /// Get a pointer to the last error message
    /// Returns null if no error is set
    pub fn get(&self) -> *const c_char {
        if let Ok(guard) = self.last_error.lock() {
            match guard.as_ref() {
                Some(cstr) => cstr.as_ptr(),
                None => std::ptr::null(),
            }
        } else {
            std::ptr::null()
        }
    }

    /// Clear the last error
    pub fn clear(&self) {
        if let Ok(mut guard) = self.last_error.lock() {
            *guard = None;
        }
    }
}

impl Default for ErrorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Global error state for FFI functions
pub static GLOBAL_ERROR: ErrorState = ErrorState::new();

/// Set the global error message
pub fn set_error(message: &str) {
    GLOBAL_ERROR.set(message);
}

/// Get the global error message pointer
pub fn get_error() -> *const c_char {
    GLOBAL_ERROR.get()
}

/// Clear the global error
pub fn clear_error() {
    GLOBAL_ERROR.clear();
}
