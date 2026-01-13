#!/usr/bin/env bash
# Nudge - Bash Integration
# Source this file from your .bashrc

# Detect platform and set config directory
_nudge_get_config_dir() {
    case "$(uname -s)" in
        Darwin)
            echo "$HOME/Library/Application Support/nudge"
            ;;
        *)
            echo "${XDG_CONFIG_HOME:-$HOME/.config}/nudge"
            ;;
    esac
}

# Configuration
NUDGE_HOTKEY="${NUDGE_HOTKEY:-\\C-e}"
NUDGE_CONFIG_DIR="${NUDGE_CONFIG_DIR:-$(_nudge_get_config_dir)}"
NUDGE_SOCKET="${NUDGE_SOCKET:-$NUDGE_CONFIG_DIR/nudge.sock}"
NUDGE_LOCK="/tmp/nudge.lock"

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
            nudge daemon --fork 2>/dev/null
        ) 200>"$NUDGE_LOCK"
    fi
}

# Main completion function
_nudge_complete() {
    # Ensure daemon is running
    _nudge_ensure_daemon

    # Call nudge with --format plain
    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$READLINE_LINE" \
        --cursor "$READLINE_POINT" \
        --cwd "$PWD" \
        --session "bash-$$" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    # Update buffer if we got a suggestion
    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        READLINE_LINE="$suggestion"
        READLINE_POINT=${#READLINE_LINE}
    fi
}

# Bind the hotkey
bind -x "\"$NUDGE_HOTKEY\": _nudge_complete"

# Print success message on first load
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    echo "Nudge loaded. Press Ctrl+E to trigger completion."
fi
