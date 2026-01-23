# Nudge FFI API Documentation

This document describes the C-compatible FFI (Foreign Function Interface) for Nudge, enabling shell integration scripts to call Nudge directly as a dynamic library for lower-latency completions.

## Overview

The FFI interface provides:
- Direct function calls instead of CLI invocation
- Embedded Tokio runtime for async operations
- Callback-based completion API
- Thread-safe error handling

## Platform Support

| Platform | Library Name | Status |
|----------|-------------|--------|
| macOS | `libnudge.dylib` | Supported |
| Linux | `libnudge.so` | Supported |
| Windows | N/A | Not supported (use CLI mode) |

## Building

Build the library with the `ffi` feature:

```bash
# Debug build
cargo build --features ffi

# Release build (recommended)
cargo build --release --features ffi
```

The library will be created at:
- `target/release/libnudge.dylib` (macOS)
- `target/release/libnudge.so` (Linux)

## Installation

Copy the library to the Nudge config directory:

```bash
# macOS
mkdir -p ~/Library/Application\ Support/nudge/lib
cp target/release/libnudge.dylib ~/Library/Application\ Support/nudge/lib/

# Linux
mkdir -p ~/.config/nudge/lib
cp target/release/libnudge.so ~/.config/nudge/lib/
```

## API Reference

### Header File

Include the header file in your C/C++ code:

```c
#include "nudge.h"
```

### Types

#### `NudgeContext`

Opaque handle to a Nudge context. Created by `nudge_init()`, freed by `nudge_free()`.

```c
typedef void* NudgeContext;
```

#### `NudgeError`

Error codes returned by FFI functions:

```c
typedef enum {
    NUDGE_SUCCESS = 0,
    NUDGE_ERROR_NULL_POINTER = -1,
    NUDGE_ERROR_INVALID_UTF8 = -2,
    NUDGE_ERROR_CONFIG_LOAD_FAILED = -3,
    NUDGE_ERROR_RUNTIME = -4,
    NUDGE_ERROR_CONTEXT_FREED = -5,
    NUDGE_ERROR_RUNTIME_CREATE_FAILED = -6
} NudgeError;
```

#### `CompletionCallback`

Callback function type for completion results:

```c
typedef void (*CompletionCallback)(
    const char* suggestion,  // Completed command (never NULL on success)
    const char* warning,     // Warning message (may be NULL)
    const char* error,       // Error message (may be NULL on success)
    void* user_data          // User-provided data pointer
);
```

### Functions

#### `nudge_init`

Initialize a new NudgeContext.

```c
NudgeContext nudge_init(const char* config_path);
```

**Parameters:**
- `config_path`: Path to configuration file (NULL for default)

**Returns:**
- Opaque handle on success
- NULL on failure (call `nudge_get_error(NULL)` for details)

**Example:**
```c
NudgeContext ctx = nudge_init(NULL);
if (!ctx) {
    fprintf(stderr, "Failed to init: %s\n", nudge_get_error(NULL));
    return 1;
}
```

#### `nudge_complete`

Request a command completion.

```c
int nudge_complete(
    NudgeContext ctx,
    const char* buffer,
    int cursor,
    const char* cwd,
    const char* session_id,
    CompletionCallback callback,
    void* user_data
);
```

**Parameters:**
- `ctx`: NudgeContext handle from `nudge_init()`
- `buffer`: Current command line buffer (null-terminated)
- `cursor`: Cursor position in buffer (0-indexed)
- `cwd`: Current working directory (null-terminated)
- `session_id`: Shell session identifier (null-terminated)
- `callback`: Function to call with completion result
- `user_data`: User data pointer passed to callback

**Returns:**
- 0 on success
- Negative error code on failure

**Example:**
```c
void on_completion(const char* suggestion, const char* warning,
                   const char* error, void* user_data) {
    if (error) {
        fprintf(stderr, "Error: %s\n", error);
    } else {
        printf("%s", suggestion);
        if (warning) {
            fprintf(stderr, "Warning: %s\n", warning);
        }
    }
}

int result = nudge_complete(ctx, "git sta", 7, "/home/user/project",
                            "session1", on_completion, NULL);
```

#### `nudge_get_error`

Get the last error message.

```c
const char* nudge_get_error(NudgeContext ctx);
```

**Parameters:**
- `ctx`: NudgeContext handle (can be NULL for global error)

**Returns:**
- Pointer to error string
- NULL if no error is set

**Note:** The returned string is valid until the next FFI call. Do not free it.

#### `nudge_free`

Free a NudgeContext.

```c
void nudge_free(NudgeContext ctx);
```

**Parameters:**
- `ctx`: NudgeContext handle from `nudge_init()` (can be NULL)

**Note:** After calling this function, the handle must not be used again.

#### `nudge_version`

Get the library version.

```c
const char* nudge_version(void);
```

**Returns:**
- Pointer to version string (e.g., "0.2.1")

**Note:** The returned string is statically allocated and always valid.

## Complete Example

```c
#include <stdio.h>
#include "nudge.h"

void on_completion(const char* suggestion, const char* warning,
                   const char* error, void* user_data) {
    int* completed = (int*)user_data;
    *completed = 1;

    if (error) {
        fprintf(stderr, "Error: %s\n", error);
    } else {
        printf("Suggestion: %s\n", suggestion);
        if (warning) {
            fprintf(stderr, "Warning: %s\n", warning);
        }
    }
}

int main() {
    printf("Nudge version: %s\n", nudge_version());

    // Initialize context
    NudgeContext ctx = nudge_init(NULL);
    if (!ctx) {
        fprintf(stderr, "Failed to initialize: %s\n", nudge_get_error(NULL));
        return 1;
    }

    // Request completion
    int completed = 0;
    int result = nudge_complete(
        ctx,
        "git sta",           // buffer
        7,                   // cursor position
        "/home/user/repo",   // current directory
        "shell-session-1",   // session ID
        on_completion,       // callback
        &completed           // user data
    );

    if (result != NUDGE_SUCCESS) {
        fprintf(stderr, "Completion failed: %s\n", nudge_get_error(ctx));
    }

    // Cleanup
    nudge_free(ctx);
    return 0;
}
```

## Shell Integration

### Bash/Zsh

The FFI library can be loaded using `dlopen` in shell integration scripts. Example using a helper binary:

```bash
# Check if library exists
LIB_PATH=$(nudge info --field lib_path)
if [ -f "$LIB_PATH" ]; then
    # Use FFI mode (faster)
    _nudge_complete_ffi "$BUFFER" "$CURSOR" "$PWD" "$$"
else
    # Fallback to CLI mode
    nudge complete --buffer "$BUFFER" --cursor "$CURSOR"
fi
```

## Thread Safety

- `nudge_init()` and `nudge_free()` are not thread-safe for the same context
- `nudge_complete()` is thread-safe and can be called from multiple threads
- Error state is thread-local

## Memory Management

- Strings returned by `nudge_get_error()` and `nudge_version()` should NOT be freed
- The `NudgeContext` handle MUST be freed with `nudge_free()` when no longer needed
- Strings passed to the callback are valid only during the callback invocation

## Error Handling

Always check return values:

```c
NudgeContext ctx = nudge_init(NULL);
if (!ctx) {
    // Handle initialization error
    const char* error = nudge_get_error(NULL);
    // ...
}

int result = nudge_complete(ctx, ...);
if (result != NUDGE_SUCCESS) {
    // Handle completion error
    const char* error = nudge_get_error(ctx);
    // ...
}
```

## Debugging

Enable debug logging by setting the `RUST_LOG` environment variable:

```bash
export RUST_LOG=nudge=debug
```

## Limitations

1. **Windows not supported**: FFI mode is only available on Unix systems (macOS, Linux)
2. **Synchronous callback**: The callback is invoked synchronously before `nudge_complete()` returns
3. **No streaming**: Completions are returned as a single result, not streamed
4. **Config reload**: To reload configuration, create a new context
