#!/usr/bin/env bash
# Nudge - Bash Integration
# Installed by: nudge setup bash

# Get configuration from nudge CLI
NUDGE_CONFIG_DIR=$(nudge info --field config_dir 2>/dev/null)
NUDGE_SOCKET=$(nudge info --field socket_path 2>/dev/null)
NUDGE_TRIGGER_MODE=$(nudge info --field trigger_mode 2>/dev/null)

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
fi

# Lock file for daemon startup
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
        --shell-mode "bash-popup" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        READLINE_LINE="$suggestion"
        READLINE_POINT=${#READLINE_LINE}
    fi
}

# ============================================================================
# Key Bindings
# ============================================================================

# Bind Ctrl+E hotkey (manual mode)
bind -x '"\C-e": _nudge_complete'

# Print success message on first load (only in interactive shells)
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    # Only print messages in interactive shells to avoid breaking scp, rsync, etc.
    if [[ $- == *i* ]]; then
        if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
            echo "Warning: Bash does not support auto mode due to readline limitations."
            echo "Falling back to manual mode. Press Ctrl+E to trigger completion."
            echo "For auto mode support, consider using Zsh or Fish."
        else
            echo "Nudge loaded. Press Ctrl+E to trigger completion."
        fi
    fi
fi
