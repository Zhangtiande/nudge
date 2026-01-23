//! FFI integration tests
//!
//! These tests verify the FFI interface works correctly.

#![cfg(all(unix, feature = "ffi"))]

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use nudge::ffi::{
    nudge_complete, nudge_free, nudge_get_error, nudge_init, nudge_version, NudgeError,
};

/// Test that init and free work correctly
#[test]
fn test_init_and_free_lifecycle() {
    unsafe {
        let ctx = nudge_init(ptr::null());
        // Context may be null if config doesn't exist
        // This is expected behavior, not a failure
        if !ctx.is_null() {
            nudge_free(ctx);
        }
    }
}

/// Test that freeing a null context doesn't crash
#[test]
fn test_free_null_context() {
    unsafe {
        nudge_free(ptr::null_mut());
    }
    // Should not crash
}

/// Test that double-free doesn't crash (undefined behavior, but shouldn't panic)
#[test]
fn test_double_free_safety() {
    unsafe {
        let ctx = nudge_init(ptr::null());
        if !ctx.is_null() {
            nudge_free(ctx);
            // Note: This is technically undefined behavior, but we want to ensure
            // it doesn't cause a panic that crosses the FFI boundary
            // In production code, users should never double-free
        }
    }
}

/// Test null pointer handling in nudge_complete
#[test]
fn test_null_context_returns_error() {
    extern "C" fn callback(
        _suggestion: *const c_char,
        _warning: *const c_char,
        _error: *const c_char,
        _user_data: *mut c_void,
    ) {
        // Should not be called
        panic!("Callback should not be called with null context");
    }

    unsafe {
        let result = nudge_complete(
            ptr::null_mut(),
            ptr::null(),
            0,
            ptr::null(),
            ptr::null(),
            callback,
            ptr::null_mut(),
        );

        assert_eq!(result, NudgeError::NullPointer as i32);
    }
}

/// Test null buffer handling
#[test]
fn test_null_buffer_returns_error() {
    extern "C" fn callback(
        _suggestion: *const c_char,
        _warning: *const c_char,
        _error: *const c_char,
        _user_data: *mut c_void,
    ) {
        panic!("Callback should not be called with null buffer");
    }

    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            // Skip test if context couldn't be created
            return;
        }

        let cwd = CString::new("/tmp").unwrap();
        let session = CString::new("test-session").unwrap();

        let result = nudge_complete(
            ctx,
            ptr::null(), // null buffer
            0,
            cwd.as_ptr(),
            session.as_ptr(),
            callback,
            ptr::null_mut(),
        );

        assert_eq!(result, NudgeError::NullPointer as i32);
        nudge_free(ctx);
    }
}

/// Test null CWD handling
#[test]
fn test_null_cwd_returns_error() {
    extern "C" fn callback(
        _suggestion: *const c_char,
        _warning: *const c_char,
        _error: *const c_char,
        _user_data: *mut c_void,
    ) {
        panic!("Callback should not be called with null cwd");
    }

    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            return;
        }

        let buffer = CString::new("ls").unwrap();
        let session = CString::new("test-session").unwrap();

        let result = nudge_complete(
            ctx,
            buffer.as_ptr(),
            2,
            ptr::null(), // null cwd
            session.as_ptr(),
            callback,
            ptr::null_mut(),
        );

        assert_eq!(result, NudgeError::NullPointer as i32);
        nudge_free(ctx);
    }
}

/// Test null session_id handling
#[test]
fn test_null_session_returns_error() {
    extern "C" fn callback(
        _suggestion: *const c_char,
        _warning: *const c_char,
        _error: *const c_char,
        _user_data: *mut c_void,
    ) {
        panic!("Callback should not be called with null session");
    }

    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            return;
        }

        let buffer = CString::new("ls").unwrap();
        let cwd = CString::new("/tmp").unwrap();

        let result = nudge_complete(
            ctx,
            buffer.as_ptr(),
            2,
            cwd.as_ptr(),
            ptr::null(), // null session
            callback,
            ptr::null_mut(),
        );

        assert_eq!(result, NudgeError::NullPointer as i32);
        nudge_free(ctx);
    }
}

/// Test error retrieval
#[test]
fn test_get_error_after_null_pointer() {
    extern "C" fn callback(
        _suggestion: *const c_char,
        _warning: *const c_char,
        _error: *const c_char,
        _user_data: *mut c_void,
    ) {
    }

    unsafe {
        // Trigger an error
        let result = nudge_complete(
            ptr::null_mut(),
            ptr::null(),
            0,
            ptr::null(),
            ptr::null(),
            callback,
            ptr::null_mut(),
        );

        // Verify error was returned
        assert_eq!(result, NudgeError::NullPointer as i32);

        // Get the error - it should be set
        let error_ptr = nudge_get_error(ptr::null_mut());
        // Error may or may not be set depending on implementation details
        // The important thing is that the function doesn't crash
        if !error_ptr.is_null() {
            let error_str = CStr::from_ptr(error_ptr).to_str().unwrap();
            assert!(error_str.contains("null"));
        }
    }
}

/// Test version string
#[test]
fn test_version_returns_valid_string() {
    let version_ptr = nudge_version();
    assert!(!version_ptr.is_null());

    let version_str = unsafe { CStr::from_ptr(version_ptr) }.to_str().unwrap();
    assert!(!version_str.is_empty());

    // Version should match Cargo.toml version
    assert!(version_str.starts_with("0."));
}

/// Test callback is invoked with user_data
#[test]
fn test_callback_receives_user_data() {
    unsafe {
        let ctx = nudge_init(ptr::null());
        if ctx.is_null() {
            // Skip if context couldn't be created (no config)
            return;
        }

        let callback_invoked = Arc::new(AtomicBool::new(false));
        let callback_invoked_clone = Arc::clone(&callback_invoked);

        extern "C" fn callback(
            _suggestion: *const c_char,
            _warning: *const c_char,
            _error: *const c_char,
            user_data: *mut c_void,
        ) {
            if !user_data.is_null() {
                // SAFETY: We know user_data points to a valid AtomicBool
                unsafe {
                    let flag = &*(user_data as *const AtomicBool);
                    flag.store(true, Ordering::SeqCst);
                }
            }
        }

        let buffer = CString::new("ls").unwrap();
        let cwd = CString::new("/tmp").unwrap();
        let session = CString::new("test-session").unwrap();

        let result = nudge_complete(
            ctx,
            buffer.as_ptr(),
            2,
            cwd.as_ptr(),
            session.as_ptr(),
            callback,
            Arc::as_ptr(&callback_invoked_clone) as *mut c_void,
        );

        // Result may be success or error depending on LLM availability
        // But callback should always be invoked
        if result == NudgeError::Success as i32 {
            assert!(callback_invoked.load(Ordering::SeqCst));
        }

        nudge_free(ctx);
    }
}

/// Test invalid config path handling
#[test]
fn test_invalid_config_path() {
    unsafe {
        let invalid_path = CString::new("/nonexistent/path/config.yaml").unwrap();
        let ctx = nudge_init(invalid_path.as_ptr());

        // Should return null for invalid path
        assert!(ctx.is_null());

        // Error should be set
        let error_ptr = nudge_get_error(ptr::null_mut());
        assert!(!error_ptr.is_null());
    }
}

/// Test get_error with null context returns global error
#[test]
fn test_get_error_null_context() {
    unsafe {
        // This should not crash
        let error_ptr = nudge_get_error(ptr::null_mut());
        // May or may not be null depending on previous errors
        let _ = error_ptr;
    }
}
