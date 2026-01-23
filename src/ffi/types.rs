//! FFI types for C interoperability
//!
//! This module defines C-compatible types used by the FFI interface.

use std::os::raw::{c_char, c_int, c_void};

/// Opaque handle to a NudgeContext
pub type NudgeContextHandle = *mut c_void;

/// Callback function type for async completion results
///
/// # Arguments
/// * `suggestion` - The completed command suggestion (null-terminated C string)
/// * `warning` - Warning message if command is dangerous (null-terminated C string, may be null)
/// * `error` - Error message if completion failed (null-terminated C string, may be null)
/// * `user_data` - User-provided data pointer passed to nudge_complete
///
/// # Safety
/// The callback is invoked from the Tokio runtime thread. The strings are valid
/// only for the duration of the callback. Copy them if you need to retain them.
pub type CompletionCallback = extern "C" fn(
    suggestion: *const c_char,
    warning: *const c_char,
    error: *const c_char,
    user_data: *mut c_void,
);

/// Error codes returned by FFI functions
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NudgeError {
    /// Operation completed successfully
    Success = 0,
    /// A null pointer was passed where a valid pointer was expected
    NullPointer = -1,
    /// A string parameter contained invalid UTF-8
    InvalidUtf8 = -2,
    /// Failed to load configuration
    ConfigLoadFailed = -3,
    /// Runtime error during completion
    RuntimeError = -4,
    /// Context has already been freed
    ContextFreed = -5,
    /// Failed to create Tokio runtime
    RuntimeCreateFailed = -6,
}

impl NudgeError {
    /// Convert to C-compatible integer
    pub fn to_c_int(self) -> c_int {
        self as c_int
    }
}

impl From<NudgeError> for c_int {
    fn from(err: NudgeError) -> Self {
        err as c_int
    }
}
