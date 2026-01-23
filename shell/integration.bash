#!/usr/bin/env bash
# Nudge - Bash Integration
# Installed by: nudge setup bash

# Get configuration from nudge CLI
NUDGE_CONFIG_DIR=$(nudge info --field config_dir 2>/dev/null)
NUDGE_SOCKET=$(nudge info --field socket_path 2>/dev/null)
NUDGE_TRIGGER_MODE=$(nudge info --field trigger_mode 2>/dev/null)
NUDGE_AUTO_DELAY=$(nudge info --field auto_delay_ms 2>/dev/null)

# Fallback if nudge binary not in PATH
if [[ -z "$NUDGE_CONFIG_DIR" ]]; then
    case "$(uname -s)" in
        Darwin)
            NUDGE_CONFIG_DIR="$HOME/Library/Application Support/nudge"
            ;;
        *)
            NUDGE_CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/nudge"
            ;;
    esac
    NUDGE_SOCKET="$NUDGE_CONFIG_DIR/nudge.sock"
    NUDGE_TRIGGER_MODE="manual"
    NUDGE_AUTO_DELAY="500"
fi

# Lock file for daemon startup
NUDGE_LOCK="/tmp/nudge.lock"

# Auto mode state
_nudge_auto_suggestion=""
_nudge_auto_timer_pid=""
_nudge_last_buffer=""

# Capture last exit code before any command
_nudge_last_exit=0
_nudge_capture_exit() {
    _nudge_last_exit=$?
}
PROMPT_COMMAND="_nudge_capture_exit${PROMPT_COMMAND:+; $PROMPT_COMMAND}"

# Ensure daemon is running (lazy load)
_nudge_ensure_daemon() {
    if [[ ! -S "$NUDGE_SOCKET" ]]; then
        # Use flock to prevent concurrent daemon starts
        (
            flock -n 200 2>/dev/null || exit 0
            nudge start 2>/dev/null
        ) 200>"$NUDGE_LOCK"
    fi
}

# Main completion function (manual mode)
_nudge_complete() {
    _nudge_ensure_daemon

    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$READLINE_LINE" \
        --cursor "$READLINE_POINT" \
        --cwd "$PWD" \
        --session "bash-$$" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        READLINE_LINE="$suggestion"
        READLINE_POINT=${#READLINE_LINE}
        # Clear any auto suggestion
        _nudge_auto_suggestion=""
    fi
}

# ============================================================================
# Auto Mode Functions
# ============================================================================

# Cancel any pending auto completion
_nudge_auto_cancel() {
    if [[ -n "$_nudge_auto_timer_pid" ]]; then
        kill "$_nudge_auto_timer_pid" 2>/dev/null
        wait "$_nudge_auto_timer_pid" 2>/dev/null
        _nudge_auto_timer_pid=""
    fi
    _nudge_auto_suggestion=""
}

# Fetch completion (called after debounce)
_nudge_auto_fetch() {
    _nudge_ensure_daemon

    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$READLINE_LINE" \
        --cursor "$READLINE_POINT" \
        --cwd "$PWD" \
        --session "bash-$$" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        # Only update if buffer hasn't changed
        if [[ "$READLINE_LINE" == "$_nudge_last_buffer" ]]; then
            _nudge_auto_suggestion="$suggestion"
            _nudge_auto_display_preview
        fi
    fi
}

# Display inline preview (gray text after cursor)
# Note: Bash's readline has limited support for inline preview
# This implementation uses a simple approach that may not work in all terminals
_nudge_auto_display_preview() {
    if [[ -n "$_nudge_auto_suggestion" && "$_nudge_auto_suggestion" != "$READLINE_LINE" ]]; then
        # Calculate the preview text (suggestion minus current buffer)
        local preview="${_nudge_auto_suggestion:${#READLINE_LINE}}"
        if [[ -n "$preview" ]]; then
            # Save cursor position, print gray preview, restore cursor
            # \e[s = save cursor, \e[u = restore cursor
            # \e[90m = gray, \e[0m = reset
            printf '\e[s\e[90m%s\e[0m\e[u' "$preview"
        fi
    fi
}

# Clear the preview display
_nudge_auto_clear_preview() {
    if [[ -n "$_nudge_auto_suggestion" ]]; then
        local preview="${_nudge_auto_suggestion:${#READLINE_LINE}}"
        if [[ -n "$preview" ]]; then
            # Clear the preview by printing spaces
            printf '\e[s%*s\e[u' "${#preview}" ""
        fi
    fi
}

# Accept auto suggestion
_nudge_auto_accept() {
    if [[ -n "$_nudge_auto_suggestion" ]]; then
        _nudge_auto_clear_preview
        READLINE_LINE="$_nudge_auto_suggestion"
        READLINE_POINT=${#READLINE_LINE}
        _nudge_auto_suggestion=""
    fi
}

# Accept partial suggestion (word by word)
_nudge_auto_accept_word() {
    if [[ -n "$_nudge_auto_suggestion" ]]; then
        _nudge_auto_clear_preview
        # Get the next word from suggestion
        local remaining="${_nudge_auto_suggestion:${#READLINE_LINE}}"
        local next_word="${remaining%% *}"
        if [[ "$remaining" == "$next_word" ]]; then
            # No space found, accept all
            READLINE_LINE="$_nudge_auto_suggestion"
        else
            READLINE_LINE="$READLINE_LINE$next_word "
        fi
        READLINE_POINT=${#READLINE_LINE}
        _nudge_auto_display_preview
    fi
}

# Trigger auto completion after debounce
_nudge_auto_trigger() {
    # Cancel previous timer
    _nudge_auto_cancel

    # Don't trigger for empty or very short input
    if [[ ${#READLINE_LINE} -lt 2 ]]; then
        return
    fi

    # Save current buffer for comparison
    _nudge_last_buffer="$READLINE_LINE"

    # Start debounce timer in background
    {
        sleep "$(echo "scale=3; $NUDGE_AUTO_DELAY / 1000" | bc)"
        # Signal parent to fetch completion
        kill -USR1 $$ 2>/dev/null
    } &
    _nudge_auto_timer_pid=$!
}

# Handle USR1 signal (debounce timer expired)
_nudge_auto_on_timer() {
    _nudge_auto_timer_pid=""
    _nudge_auto_fetch
}

# Hook for buffer changes (called periodically)
# Note: Bash doesn't have a native hook for every keystroke
# This is a workaround using PROMPT_COMMAND
_nudge_auto_check_buffer() {
    if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
        if [[ "$READLINE_LINE" != "$_nudge_last_buffer" ]]; then
            _nudge_auto_clear_preview
            _nudge_auto_trigger
        fi
    fi
}

# ============================================================================
# Key Bindings
# ============================================================================

# Bind Ctrl+E hotkey (manual mode)
bind -x '"\C-e": _nudge_complete'

# Setup auto mode if enabled
if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
    # Set up signal handler for debounce timer
    trap '_nudge_auto_on_timer' USR1

    # Bind Tab to accept suggestion
    # Note: This overrides default Tab completion
    bind -x '"\t": _nudge_auto_accept'

    # Bind Ctrl+Right to accept word
    bind -x '"\e[1;5C": _nudge_auto_accept_word'

    # Hook into readline for buffer change detection
    # This is a workaround since Bash doesn't have native keystroke hooks
    # We use a custom function bound to common keys
    _nudge_auto_key_hook() {
        # Check if buffer changed and trigger auto completion
        if [[ "$READLINE_LINE" != "$_nudge_last_buffer" ]]; then
            _nudge_auto_clear_preview
            _nudge_auto_trigger
        fi
    }

    # Bind the hook to self-insert (called after each character)
    # Note: This is a hack and may not work perfectly in all cases
    # For better auto mode support, consider using ble.sh
    bind -x '"\C-x\C-n": _nudge_auto_key_hook'

    # Clean up on exit
    trap '_nudge_auto_cancel' EXIT
fi

# Print success message on first load
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
        echo "Nudge loaded (auto mode - limited in Bash). Press Tab to accept suggestions."
        echo "Note: For better auto mode support, consider using Zsh or ble.sh."
    else
        echo "Nudge loaded. Press Ctrl+E to trigger completion."
    fi
fi
