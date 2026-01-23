//! Auto mode integration tests
//!
//! These tests verify the auto mode FFI interface works correctly.

#![cfg(all(unix, feature = "ffi"))]

use std::ffi::CString;
use std::os::raw::{c_char, c_void};
use std::ptr;

use nudge::ffi::{nudge_free, nudge_init, NudgeError};

// Import auto mode functions
extern "C" {
    fn nudge_auto_start(
        ctx: *mut c_void,
        delay_ms: u32,
        callback: extern "C" fn(*const c_char, *const c_char, *const c_char, *mut c_void),
        user_data: *mut c_void,
    ) -> i32;

    fn nudge_auto_update_buffer(
        ctx: *mut c_void,
        buffer: *const c_char,
        cursor: i32,
        cwd: *const c_char,
        session_id: *const c_char,
    ) -> i32;

    fn nudge_auto_stop(ctx: *mut c_void) -> i32;

    fn nudge_auto_is_active(ctx: *mut c_void) -> i32;

    fn nudge_auto_get_delay_ms(ctx: *mut c_void) -> u32;

    fn nudge_auto_get_suggestion(ctx: *mut c_void) -> *const c_char;
}

extern "C" fn dummy_callback(
    _suggestion: *const c_char,
    _warning: *const c_char,
    _error: *const c_char,
    _user_data: *mut c_void,
) {
    // Do nothing
}

/// Test auto mode start and stop
#[test]
fn test_auto_mode_start_stop() {
    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            // Skip if context couldn't be created (no config)
            return;
        }

        // Initially not active
        assert_eq!(nudge_auto_is_active(ctx), 0);

        // Start auto mode
        let result = nudge_auto_start(ctx, 500, dummy_callback, ptr::null_mut());
        assert_eq!(result, NudgeError::Success as i32);

        // Now active
        assert_eq!(nudge_auto_is_active(ctx), 1);

        // Stop auto mode
        let result = nudge_auto_stop(ctx);
        assert_eq!(result, NudgeError::Success as i32);

        // No longer active
        assert_eq!(nudge_auto_is_active(ctx), 0);

        nudge_free(ctx);
    }
}

/// Test auto mode with null context
#[test]
fn test_auto_mode_null_context() {
    unsafe {
        // All functions should handle null context gracefully
        let result = nudge_auto_start(ptr::null_mut(), 500, dummy_callback, ptr::null_mut());
        assert_eq!(result, NudgeError::NullPointer as i32);

        let result = nudge_auto_stop(ptr::null_mut());
        assert_eq!(result, NudgeError::NullPointer as i32);

        assert_eq!(nudge_auto_is_active(ptr::null_mut()), 0);
        assert_eq!(nudge_auto_get_delay_ms(ptr::null_mut()), 500);
        assert!(nudge_auto_get_suggestion(ptr::null_mut()).is_null());
    }
}

/// Test auto mode buffer update
#[test]
fn test_auto_mode_buffer_update() {
    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            return;
        }

        // Start auto mode
        nudge_auto_start(ctx, 500, dummy_callback, ptr::null_mut());

        let buffer = CString::new("git sta").unwrap();
        let cwd = CString::new("/tmp").unwrap();
        let session = CString::new("test-session").unwrap();

        // Update buffer
        let result =
            nudge_auto_update_buffer(ctx, buffer.as_ptr(), 7, cwd.as_ptr(), session.as_ptr());
        assert_eq!(result, NudgeError::Success as i32);

        nudge_auto_stop(ctx);
        nudge_free(ctx);
    }
}

/// Test auto mode buffer update with null parameters
#[test]
fn test_auto_mode_buffer_update_null_params() {
    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            return;
        }

        nudge_auto_start(ctx, 500, dummy_callback, ptr::null_mut());

        let buffer = CString::new("git sta").unwrap();
        let cwd = CString::new("/tmp").unwrap();
        let session = CString::new("test-session").unwrap();

        // Null buffer
        let result = nudge_auto_update_buffer(ctx, ptr::null(), 7, cwd.as_ptr(), session.as_ptr());
        assert_eq!(result, NudgeError::NullPointer as i32);

        // Null cwd
        let result =
            nudge_auto_update_buffer(ctx, buffer.as_ptr(), 7, ptr::null(), session.as_ptr());
        assert_eq!(result, NudgeError::NullPointer as i32);

        // Null session
        let result = nudge_auto_update_buffer(ctx, buffer.as_ptr(), 7, cwd.as_ptr(), ptr::null());
        assert_eq!(result, NudgeError::NullPointer as i32);

        nudge_auto_stop(ctx);
        nudge_free(ctx);
    }
}

/// Test auto mode delay configuration
#[test]
fn test_auto_mode_delay_config() {
    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            return;
        }

        // Get default delay (should be from config or 500)
        let delay = nudge_auto_get_delay_ms(ctx);
        assert!(delay > 0);

        // Start with custom delay
        nudge_auto_start(ctx, 300, dummy_callback, ptr::null_mut());

        // Delay should be updated
        let delay = nudge_auto_get_delay_ms(ctx);
        assert_eq!(delay, 300);

        nudge_auto_stop(ctx);
        nudge_free(ctx);
    }
}

/// Test that auto mode doesn't trigger for inactive context
#[test]
fn test_auto_mode_inactive_no_trigger() {
    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            return;
        }

        // Don't start auto mode
        let buffer = CString::new("git sta").unwrap();
        let cwd = CString::new("/tmp").unwrap();
        let session = CString::new("test-session").unwrap();

        // Update buffer should succeed but not trigger anything
        let result =
            nudge_auto_update_buffer(ctx, buffer.as_ptr(), 7, cwd.as_ptr(), session.as_ptr());
        assert_eq!(result, NudgeError::Success as i32);

        // No suggestion should be available
        let suggestion = nudge_auto_get_suggestion(ctx);
        assert!(suggestion.is_null());

        nudge_free(ctx);
    }
}
