//! Auto mode implementation for FFI
//!
//! This module provides auto mode functionality with debouncing and
//! background completion requests.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::completion;
use super::context::NudgeContext;
use super::error;
use super::types::{CompletionCallback, NudgeError};

/// Auto mode state stored in NudgeContext
pub struct AutoModeState {
    /// Whether auto mode is active
    pub active: AtomicBool,
    /// Current buffer content
    pub buffer: Mutex<String>,
    /// Current cursor position
    pub cursor: AtomicU64,
    /// Current working directory
    pub cwd: Mutex<String>,
    /// Session ID
    pub session_id: Mutex<String>,
    /// Last suggestion (for inline preview)
    pub last_suggestion: Mutex<Option<String>>,
    /// Channel to send cancel signals
    pub cancel_tx: Mutex<Option<mpsc::Sender<()>>>,
    /// Handle to the debounce task
    pub debounce_handle: Mutex<Option<JoinHandle<()>>>,
}

impl AutoModeState {
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            buffer: Mutex::new(String::new()),
            cursor: AtomicU64::new(0),
            cwd: Mutex::new(String::new()),
            session_id: Mutex::new(String::new()),
            last_suggestion: Mutex::new(None),
            cancel_tx: Mutex::new(None),
            debounce_handle: Mutex::new(None),
        }
    }

    /// Update buffer and cursor position
    pub fn update_buffer(&self, buffer: &str, cursor: usize) {
        if let Ok(mut buf) = self.buffer.lock() {
            *buf = buffer.to_string();
        }
        self.cursor.store(cursor as u64, Ordering::SeqCst);
    }

    /// Get current buffer
    pub fn get_buffer(&self) -> String {
        self.buffer.lock().map(|b| b.clone()).unwrap_or_default()
    }

    /// Get current cursor position
    pub fn get_cursor(&self) -> usize {
        self.cursor.load(Ordering::SeqCst) as usize
    }

    /// Set last suggestion
    pub fn set_suggestion(&self, suggestion: Option<String>) {
        if let Ok(mut s) = self.last_suggestion.lock() {
            *s = suggestion;
        }
    }

    /// Get last suggestion
    pub fn get_suggestion(&self) -> Option<String> {
        self.last_suggestion.lock().ok().and_then(|s| s.clone())
    }

    /// Cancel any pending debounce
    pub fn cancel_pending(&self) {
        // Send cancel signal
        if let Ok(tx) = self.cancel_tx.lock() {
            if let Some(tx) = tx.as_ref() {
                let _ = tx.try_send(());
            }
        }

        // Abort debounce task
        if let Ok(mut handle) = self.debounce_handle.lock() {
            if let Some(h) = handle.take() {
                h.abort();
            }
        }
    }
}

impl Default for AutoModeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Start auto mode for the given context
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
/// * `delay_ms` - Debounce delay in milliseconds
/// * `callback` - Function to call with completion result
/// * `user_data` - User data pointer passed to callback
///
/// # Returns
/// * 0 on success
/// * Negative error code on failure
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init`
/// * `callback` must be a valid function pointer
#[no_mangle]
pub unsafe extern "C" fn nudge_auto_start(
    ctx: *mut c_void,
    delay_ms: c_uint,
    _callback: CompletionCallback,
    _user_data: *mut c_void,
) -> c_int {
    let result = std::panic::catch_unwind(|| {
        if ctx.is_null() {
            error::set_error("Context handle is null");
            return NudgeError::NullPointer.into();
        }

        let context = &*(ctx as *const NudgeContext);

        // Mark auto mode as active
        context.auto_mode.active.store(true, Ordering::SeqCst);

        // Store delay for later use
        context.auto_delay_ms.store(delay_ms, Ordering::SeqCst);

        // Store callback info (we'll use it when triggering completions)
        // Note: In a real implementation, we'd store these in the context
        // For now, auto mode is triggered via nudge_auto_trigger

        NudgeError::Success.into()
    });

    match result {
        Ok(code) => code,
        Err(_) => {
            error::set_error("Panic during auto_start");
            NudgeError::RuntimeError.into()
        }
    }
}

/// Update buffer content for auto mode
///
/// Call this function whenever the command line buffer changes.
/// This will reset the debounce timer.
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
/// * `buffer` - Current command line buffer (null-terminated C string)
/// * `cursor` - Cursor position in buffer
/// * `cwd` - Current working directory (null-terminated C string)
/// * `session_id` - Shell session identifier (null-terminated C string)
///
/// # Returns
/// * 0 on success
/// * Negative error code on failure
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init`
/// * All string parameters must be valid null-terminated UTF-8 strings
#[no_mangle]
pub unsafe extern "C" fn nudge_auto_update_buffer(
    ctx: *mut c_void,
    buffer: *const c_char,
    cursor: c_int,
    cwd: *const c_char,
    session_id: *const c_char,
) -> c_int {
    let result = std::panic::catch_unwind(|| {
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

        let context = &*(ctx as *const NudgeContext);

        // Check if auto mode is active
        if !context.auto_mode.active.load(Ordering::SeqCst) {
            return NudgeError::Success.into();
        }

        // Convert C strings
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

        // Update auto mode state
        context.auto_mode.update_buffer(buffer_str, cursor as usize);

        if let Ok(mut cwd_guard) = context.auto_mode.cwd.lock() {
            *cwd_guard = cwd_str.to_string();
        }

        if let Ok(mut session_guard) = context.auto_mode.session_id.lock() {
            *session_guard = session_str.to_string();
        }

        // Cancel any pending completion
        context.auto_mode.cancel_pending();

        // Clear last suggestion (new input invalidates it)
        context.auto_mode.set_suggestion(None);

        NudgeError::Success.into()
    });

    match result {
        Ok(code) => code,
        Err(_) => {
            error::set_error("Panic during auto_update_buffer");
            NudgeError::RuntimeError.into()
        }
    }
}

/// Trigger auto completion after debounce delay
///
/// This function should be called after the debounce delay has elapsed.
/// It will perform the completion and invoke the callback with the result.
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
/// * `callback` - Function to call with completion result
/// * `user_data` - User data pointer passed to callback
///
/// # Returns
/// * 0 on success
/// * Negative error code on failure
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init`
/// * `callback` must be a valid function pointer
#[no_mangle]
pub unsafe extern "C" fn nudge_auto_trigger(
    ctx: *mut c_void,
    callback: CompletionCallback,
    user_data: *mut c_void,
) -> c_int {
    let result = std::panic::catch_unwind(|| {
        if ctx.is_null() {
            error::set_error("Context handle is null");
            return NudgeError::NullPointer.into();
        }

        let context = &*(ctx as *const NudgeContext);

        // Check if auto mode is active
        if !context.auto_mode.active.load(Ordering::SeqCst) {
            return NudgeError::Success.into();
        }

        // Get current buffer state
        let buffer = context.auto_mode.get_buffer();
        let cursor = context.auto_mode.get_cursor();

        // Don't trigger for empty or very short input
        if buffer.is_empty() || buffer.len() < 2 {
            return NudgeError::Success.into();
        }

        let cwd = context
            .auto_mode
            .cwd
            .lock()
            .map(|c| c.clone())
            .unwrap_or_default();

        let session_id = context
            .auto_mode
            .session_id
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default();

        // Run completion
        let result = context.runtime.block_on(async {
            completion::complete(&buffer, cursor, &cwd, &session_id, &context.config).await
        });

        // Store suggestion for later retrieval
        if result.error.is_none() && !result.suggestion.is_empty() {
            context
                .auto_mode
                .set_suggestion(Some(result.suggestion.clone()));
        }

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
            error::set_error("Panic during auto_trigger");
            NudgeError::RuntimeError.into()
        }
    }
}

/// Get the last auto mode suggestion
///
/// Returns the most recent suggestion from auto mode, which can be used
/// for inline preview display.
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
///
/// # Returns
/// * Pointer to null-terminated suggestion string
/// * NULL if no suggestion is available
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init`
/// * The returned string is valid until the next auto mode operation
#[no_mangle]
pub unsafe extern "C" fn nudge_auto_get_suggestion(ctx: *mut c_void) -> *const c_char {
    let result = std::panic::catch_unwind(|| {
        if ctx.is_null() {
            return std::ptr::null();
        }

        let context = &*(ctx as *const NudgeContext);

        match context.auto_mode.get_suggestion() {
            Some(suggestion) => {
                // Store in global error state for lifetime management
                // (This is a hack, but ensures the string lives long enough)
                error::set_error(&suggestion);
                error::get_error()
            }
            None => std::ptr::null(),
        }
    });

    match result {
        Ok(ptr) => ptr,
        Err(_) => std::ptr::null(),
    }
}

/// Stop auto mode
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
///
/// # Returns
/// * 0 on success
/// * Negative error code on failure
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init`
#[no_mangle]
pub unsafe extern "C" fn nudge_auto_stop(ctx: *mut c_void) -> c_int {
    let result = std::panic::catch_unwind(|| {
        if ctx.is_null() {
            error::set_error("Context handle is null");
            return NudgeError::NullPointer.into();
        }

        let context = &*(ctx as *const NudgeContext);

        // Mark auto mode as inactive
        context.auto_mode.active.store(false, Ordering::SeqCst);

        // Cancel any pending completion
        context.auto_mode.cancel_pending();

        // Clear suggestion
        context.auto_mode.set_suggestion(None);

        NudgeError::Success.into()
    });

    match result {
        Ok(code) => code,
        Err(_) => {
            error::set_error("Panic during auto_stop");
            NudgeError::RuntimeError.into()
        }
    }
}

/// Check if auto mode is active
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
///
/// # Returns
/// * 1 if auto mode is active
/// * 0 if auto mode is inactive or ctx is null
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init` or NULL
#[no_mangle]
pub unsafe extern "C" fn nudge_auto_is_active(ctx: *mut c_void) -> c_int {
    let result = std::panic::catch_unwind(|| {
        if ctx.is_null() {
            return 0;
        }

        let context = &*(ctx as *const NudgeContext);

        if context.auto_mode.active.load(Ordering::SeqCst) {
            1
        } else {
            0
        }
    });

    result.unwrap_or_default()
}

/// Get the configured auto mode delay in milliseconds
///
/// # Arguments
/// * `ctx` - NudgeContext handle from `nudge_init`
///
/// # Returns
/// * Delay in milliseconds (default 500 if not configured)
///
/// # Safety
/// * `ctx` must be a valid handle from `nudge_init` or NULL
#[no_mangle]
pub unsafe extern "C" fn nudge_auto_get_delay_ms(ctx: *mut c_void) -> c_uint {
    let result = std::panic::catch_unwind(|| {
        if ctx.is_null() {
            return 500; // Default delay
        }

        let context = &*(ctx as *const NudgeContext);
        context.auto_delay_ms.load(Ordering::SeqCst)
    });

    result.unwrap_or(500)
}
