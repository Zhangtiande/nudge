/**
 * Nudge FFI Header
 *
 * C-compatible interface for the Nudge LLM-powered CLI auto-completion library.
 *
 * This header provides functions to:
 * - Initialize and free the Nudge context
 * - Request command completions with async callbacks
 * - Retrieve error messages
 *
 * Example usage:
 *
 *   #include "nudge.h"
 *
 *   void on_completion(const char* suggestion, const char* warning,
 *                      const char* error, void* user_data) {
 *       if (error) {
 *           fprintf(stderr, "Error: %s\n", error);
 *       } else {
 *           printf("Suggestion: %s\n", suggestion);
 *           if (warning) {
 *               printf("Warning: %s\n", warning);
 *           }
 *       }
 *   }
 *
 *   int main() {
 *       NudgeContext ctx = nudge_init(NULL);
 *       if (!ctx) {
 *           fprintf(stderr, "Failed to init: %s\n", nudge_get_error(NULL));
 *           return 1;
 *       }
 *
 *       nudge_complete(ctx, "git sta", 7, "/home/user/project", "session1",
 *                      on_completion, NULL);
 *
 *       nudge_free(ctx);
 *       return 0;
 *   }
 */

#ifndef NUDGE_H
#define NUDGE_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque handle to a NudgeContext.
 * Created by nudge_init(), freed by nudge_free().
 */
typedef void* NudgeContext;

/**
 * Error codes returned by FFI functions.
 */
typedef enum {
    NUDGE_SUCCESS = 0,
    NUDGE_ERROR_NULL_POINTER = -1,
    NUDGE_ERROR_INVALID_UTF8 = -2,
    NUDGE_ERROR_CONFIG_LOAD_FAILED = -3,
    NUDGE_ERROR_RUNTIME = -4,
    NUDGE_ERROR_CONTEXT_FREED = -5,
    NUDGE_ERROR_RUNTIME_CREATE_FAILED = -6
} NudgeError;

/**
 * Callback function type for completion results.
 *
 * @param suggestion  The completed command suggestion (never NULL on success)
 * @param warning     Warning message if command is dangerous (may be NULL)
 * @param error       Error message if completion failed (may be NULL on success)
 * @param user_data   User-provided data pointer passed to nudge_complete
 *
 * Note: The strings are valid only for the duration of the callback.
 *       Copy them if you need to retain them.
 */
typedef void (*CompletionCallback)(
    const char* suggestion,
    const char* warning,
    const char* error,
    void* user_data
);

/**
 * Initialize a new NudgeContext.
 *
 * @param config_path  Path to configuration file (NULL for default)
 * @return             Opaque handle on success, NULL on failure
 *
 * On failure, call nudge_get_error(NULL) to get the error message.
 * The returned handle must be freed with nudge_free() when no longer needed.
 */
NudgeContext nudge_init(const char* config_path);

/**
 * Request a command completion.
 *
 * @param ctx         NudgeContext handle from nudge_init()
 * @param buffer      Current command line buffer (null-terminated)
 * @param cursor      Cursor position in buffer (0-indexed)
 * @param cwd         Current working directory (null-terminated)
 * @param session_id  Shell session identifier (null-terminated)
 * @param callback    Function to call with completion result
 * @param user_data   User data pointer passed to callback
 * @return            0 on success, negative error code on failure
 *
 * The callback is invoked synchronously before this function returns.
 * All string parameters must be valid null-terminated UTF-8 strings.
 */
int nudge_complete(
    NudgeContext ctx,
    const char* buffer,
    int cursor,
    const char* cwd,
    const char* session_id,
    CompletionCallback callback,
    void* user_data
);

/**
 * Get the last error message.
 *
 * @param ctx  NudgeContext handle (can be NULL for global error)
 * @return     Pointer to error string, or NULL if no error
 *
 * The returned string is valid until the next FFI call.
 * Do not free the returned pointer.
 */
const char* nudge_get_error(NudgeContext ctx);

/**
 * Free a NudgeContext.
 *
 * @param ctx  NudgeContext handle from nudge_init() (can be NULL)
 *
 * After calling this function, the handle must not be used again.
 */
void nudge_free(NudgeContext ctx);

/**
 * Get the library version.
 *
 * @return  Pointer to version string (e.g., "0.2.1")
 *
 * The returned string is statically allocated and always valid.
 * Do not free the returned pointer.
 */
const char* nudge_version(void);

/* ============================================================================
 * Auto Mode Functions
 * ============================================================================
 * Auto mode provides automatic completion suggestions as the user types.
 * It uses debouncing to avoid excessive API calls.
 */

/**
 * Start auto mode for the given context.
 *
 * @param ctx        NudgeContext handle from nudge_init()
 * @param delay_ms   Debounce delay in milliseconds (e.g., 500)
 * @param callback   Function to call with completion results
 * @param user_data  User data pointer passed to callback
 * @return           0 on success, negative error code on failure
 *
 * After calling this function, use nudge_auto_update_buffer() to notify
 * the library of buffer changes. The callback will be invoked after the
 * debounce delay with completion suggestions.
 */
int nudge_auto_start(
    NudgeContext ctx,
    unsigned int delay_ms,
    CompletionCallback callback,
    void* user_data
);

/**
 * Update buffer content for auto mode.
 *
 * Call this function whenever the command line buffer changes.
 * This will reset the debounce timer.
 *
 * @param ctx         NudgeContext handle from nudge_init()
 * @param buffer      Current command line buffer (null-terminated)
 * @param cursor      Cursor position in buffer (0-indexed)
 * @param cwd         Current working directory (null-terminated)
 * @param session_id  Shell session identifier (null-terminated)
 * @return            0 on success, negative error code on failure
 */
int nudge_auto_update_buffer(
    NudgeContext ctx,
    const char* buffer,
    int cursor,
    const char* cwd,
    const char* session_id
);

/**
 * Trigger auto completion after debounce delay.
 *
 * This function should be called after the debounce delay has elapsed.
 * It will perform the completion and invoke the callback with the result.
 *
 * @param ctx        NudgeContext handle from nudge_init()
 * @param callback   Function to call with completion result
 * @param user_data  User data pointer passed to callback
 * @return           0 on success, negative error code on failure
 */
int nudge_auto_trigger(
    NudgeContext ctx,
    CompletionCallback callback,
    void* user_data
);

/**
 * Get the last auto mode suggestion.
 *
 * Returns the most recent suggestion from auto mode, which can be used
 * for inline preview display.
 *
 * @param ctx  NudgeContext handle from nudge_init()
 * @return     Pointer to suggestion string, or NULL if no suggestion
 *
 * The returned string is valid until the next auto mode operation.
 * Do not free the returned pointer.
 */
const char* nudge_auto_get_suggestion(NudgeContext ctx);

/**
 * Stop auto mode.
 *
 * @param ctx  NudgeContext handle from nudge_init()
 * @return     0 on success, negative error code on failure
 */
int nudge_auto_stop(NudgeContext ctx);

/**
 * Check if auto mode is active.
 *
 * @param ctx  NudgeContext handle from nudge_init()
 * @return     1 if active, 0 if inactive or ctx is NULL
 */
int nudge_auto_is_active(NudgeContext ctx);

/**
 * Get the configured auto mode delay in milliseconds.
 *
 * @param ctx  NudgeContext handle from nudge_init()
 * @return     Delay in milliseconds (default 500 if not configured)
 */
unsigned int nudge_auto_get_delay_ms(NudgeContext ctx);

#ifdef __cplusplus
}
#endif

#endif /* NUDGE_H */
