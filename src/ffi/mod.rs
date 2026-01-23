//! FFI module for C interoperability
//!
//! This module provides C-compatible functions that can be loaded by shell
//! integration scripts for lower-latency completion.
//!
//! # Safety
//!
//! All FFI functions use `panic::catch_unwind` to prevent panics from crossing
//! the FFI boundary. Null pointer checks are performed on all inputs.
//!
//! # Example Usage (C)
//!
//! ```c
//! #include "nudge.h"
//!
//! void on_completion(const char* suggestion, const char* warning,
//!                    const char* error, void* user_data) {
//!     if (error) {
//!         fprintf(stderr, "Error: %s\n", error);
//!     } else {
//!         printf("Suggestion: %s\n", suggestion);
//!         if (warning) {
//!             printf("Warning: %s\n", warning);
//!         }
//!     }
//! }
//!
//! int main() {
//!     NudgeContext ctx = nudge_init(NULL);
//!     if (!ctx) {
//!         fprintf(stderr, "Failed to init: %s\n", nudge_get_error(NULL));
//!         return 1;
//!     }
//!
//!     nudge_complete(ctx, "git sta", 7, "/home/user/project", "session1",
//!                    on_completion, NULL);
//!
//!     nudge_free(ctx);
//!     return 0;
//! }
//! ```

pub mod auto_mode;
pub mod completion;
pub mod context;
pub mod error;
pub mod types;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::panic;
use std::path::PathBuf;

use crate::config::Config;

pub use context::NudgeContext;
pub use types::{CompletionCallback, NudgeContextHandle, NudgeError};

/// Initialize a new NudgeContext
///
/// # Arguments
/// * `config_path` - Path to configuration file (null-terminated C string).
///   If NULL, uses default configuration path.
///
/// # Returns
/// * Opaque handle to NudgeContext on success
/// * NULL on failure (call `nudge_get_error` for details)
///
/// # Safety
/// * If `config_path` is not NULL, it must be a valid null-terminated C string
/// * The returned handle must be freed with `nudge_free` when no longer needed
#[no_mangle]
pub unsafe extern "C" fn nudge_init(config_path: *const c_char) -> NudgeContextHandle {
    let result = panic::catch_unwind(|| {
        error::clear_error();

        // Load configuration
        let config = if config_path.is_null() {
            // Use default configuration
            match Config::load() {
                Ok(c) => c,
                Err(e) => {
                    error::set_error(&format!("Failed to load config: {}", e));
                    return std::ptr::null_mut();
                }
            }
        } else {
            // Load from specified path
            // SAFETY: Caller guarantees config_path is valid if not null
            let path_str = match CStr::from_ptr(config_path).to_str() {
                Ok(s) => s,
                Err(_) => {
                    error::set_error("Invalid UTF-8 in config path");
                    return std::ptr::null_mut();
                }
            };

            match Config::load_from_path(&PathBuf::from(path_str)) {
                Ok(c) => c,
                Err(e) => {
                    error::set_error(&format!("Failed to load config from {}: {}", path_str, e));
                    return std::ptr::null_mut();
                }
            }
        };

        // Create context
        match NudgeContext::new(config) {
            Ok(ctx) => Box::into_raw(Box::new(ctx)) as NudgeContextHandle,
            Err(e) => {
                error::set_error(&e);
                std::ptr::null_mut()
            }
        }
    });

    match result {
        Ok(handle) => handle,
        Err(_) => {
            error::set_error("Panic during initialization");
            std::ptr::null_mut()
        }
    }
}

/// Request a completion
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
/// * `buffer` - Current command line buffer (null-terminated C string)
/// * `cursor` - Cursor position in buffer (0-indexed)
/// * `cwd` - Current working directory (null-terminated C string)
/// * `session_id` - Shell session identifier (null-terminated C string)
/// * `callback` - Function to call with completion result
/// * `user_data` - User data pointer passed to callback
///
/// # Returns
/// * 0 on success (callback will be invoked)
/// * Negative error code on failure
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init`
/// * All string parameters must be valid null-terminated UTF-8 strings
/// * `callback` must be a valid function pointer
#[no_mangle]
pub unsafe extern "C" fn nudge_complete(
    ctx: NudgeContextHandle,
    buffer: *const c_char,
    cursor: c_int,
    cwd: *const c_char,
    session_id: *const c_char,
    callback: CompletionCallback,
    user_data: *mut c_void,
) -> c_int {
    let result = panic::catch_unwind(|| {
        // Null pointer checks
        if ctx.is_null() {
            error::set_error("Context handle is null");
            return NudgeError::NullPointer.into();
        }
        if buffer.is_null() {
            error::set_error("Buffer is null");
            return NudgeError::NullPointer.into();
        }
        if cwd.is_null() {
            error::set_error("CWD is null");
            return NudgeError::NullPointer.into();
        }
        if session_id.is_null() {
            error::set_error("Session ID is null");
            return NudgeError::NullPointer.into();
        }

        // Convert C strings to Rust strings
        // SAFETY: Caller guarantees these are valid null-terminated strings
        let buffer_str = match CStr::from_ptr(buffer).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_error("Invalid UTF-8 in buffer");
                return NudgeError::InvalidUtf8.into();
            }
        };

        let cwd_str = match CStr::from_ptr(cwd).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_error("Invalid UTF-8 in cwd");
                return NudgeError::InvalidUtf8.into();
            }
        };

        let session_str = match CStr::from_ptr(session_id).to_str() {
            Ok(s) => s,
            Err(_) => {
                error::set_error("Invalid UTF-8 in session_id");
                return NudgeError::InvalidUtf8.into();
            }
        };

        // Get context reference
        // SAFETY: Caller guarantees ctx is a valid NudgeContext pointer
        let context = &*(ctx as *const NudgeContext);

        // Run completion in the Tokio runtime
        let result = context.runtime.block_on(async {
            completion::complete(
                buffer_str,
                cursor as usize,
                cwd_str,
                session_str,
                &context.config,
            )
            .await
        });

        // Prepare callback arguments
        let suggestion_cstr = CString::new(result.suggestion.as_str()).unwrap_or_default();
        let warning_cstr = result
            .warning
            .as_ref()
            .and_then(|w| CString::new(w.as_str()).ok());
        let error_cstr = result
            .error
            .as_ref()
            .and_then(|e| CString::new(e.as_str()).ok());

        // Invoke callback
        callback(
            suggestion_cstr.as_ptr(),
            warning_cstr
                .as_ref()
                .map(|c| c.as_ptr())
                .unwrap_or(std::ptr::null()),
            error_cstr
                .as_ref()
                .map(|c| c.as_ptr())
                .unwrap_or(std::ptr::null()),
            user_data,
        );

        NudgeError::Success.into()
    });

    match result {
        Ok(code) => code,
        Err(_) => {
            error::set_error("Panic during completion");
            NudgeError::RuntimeError.into()
        }
    }
}

/// Get the last error message
///
/// # Arguments
/// * `ctx` - NudgeContext handle (can be NULL to get global error)
///
/// # Returns
/// * Pointer to null-terminated error string
/// * NULL if no error is set
///
/// # Safety
/// * If `ctx` is not NULL, it must be a valid handle from `nudge_init`
/// * The returned string is valid until the next FFI call
/// * Do not free the returned pointer
#[no_mangle]
pub unsafe extern "C" fn nudge_get_error(ctx: NudgeContextHandle) -> *const c_char {
    let result = panic::catch_unwind(|| {
        if ctx.is_null() {
            // Return global error
            error::get_error()
        } else {
            // Return context-specific error
            // SAFETY: Caller guarantees ctx is a valid NudgeContext pointer
            let context = &*(ctx as *const NudgeContext);
            match context.get_error() {
                Some(msg) => {
                    // Store in global error for return
                    error::set_error(&msg);
                    error::get_error()
                }
                None => std::ptr::null(),
            }
        }
    });

    match result {
        Ok(ptr) => ptr,
        Err(_) => std::ptr::null(),
    }
}

/// Free a NudgeContext
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init` or NULL
/// * After calling this function, `ctx` must not be used again
#[no_mangle]
pub unsafe extern "C" fn nudge_free(ctx: NudgeContextHandle) {
    let _ = panic::catch_unwind(|| {
        if !ctx.is_null() {
            // Take ownership and drop
            // SAFETY: Caller guarantees ctx is a valid NudgeContext pointer
            let _ = Box::from_raw(ctx as *mut NudgeContext);
        }
    });
}

/// Get the library version
///
/// # Returns
/// * Pointer to null-terminated version string (e.g., "0.2.1")
///
/// # Safety
/// The returned string is statically allocated and always valid.
/// Do not free the returned pointer.
#[no_mangle]
pub extern "C" fn nudge_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_free() {
        unsafe {
            let ctx = nudge_init(std::ptr::null());
            // Context may be null if config doesn't exist, which is fine for this test
            if !ctx.is_null() {
                nudge_free(ctx);
            }
        }
    }

    #[test]
    fn test_null_context_error() {
        unsafe {
            let result = nudge_complete(
                std::ptr::null_mut(),
                std::ptr::null(),
                0,
                std::ptr::null(),
                std::ptr::null(),
                dummy_callback,
                std::ptr::null_mut(),
            );
            assert_eq!(result, NudgeError::NullPointer as c_int);
        }
    }

    #[test]
    fn test_version() {
        let version = nudge_version();
        assert!(!version.is_null());
        let version_str = unsafe { CStr::from_ptr(version) }.to_str().unwrap();
        assert!(!version_str.is_empty());
    }

    extern "C" fn dummy_callback(
        _suggestion: *const c_char,
        _warning: *const c_char,
        _error: *const c_char,
        _user_data: *mut c_void,
    ) {
        // Do nothing
    }
}
