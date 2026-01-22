#!/usr/bin/env zsh
# Nudge - Zsh Integration
# Installed by: nudge setup zsh

# Get configuration from nudge CLI
NUDGE_CONFIG_DIR=$(nudge info --field config_dir 2>/dev/null)
NUDGE_SOCKET=$(nudge info --field socket_path 2>/dev/null)

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
fi

NUDGE_LOCK="/tmp/nudge.lock"

# Capture last exit code
_nudge_last_exit=0
_nudge_capture_exit() {
    _nudge_last_exit=$?
}
precmd_functions+=(_nudge_capture_exit)

# Ensure daemon is running
_nudge_ensure_daemon() {
    if [[ ! -S "$NUDGE_SOCKET" ]]; then
        # Use zsystem flock to prevent concurrent daemon starts
        if zsystem flock -t 0 "$NUDGE_LOCK" 2>/dev/null; then
            nudge start 2>/dev/null
        fi
    fi
}

# Main completion widget
_nudge_complete() {
    _nudge_ensure_daemon

    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$BUFFER" \
        --cursor "$CURSOR" \
        --cwd "$PWD" \
        --session "zsh-$$" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        BUFFER="$suggestion"
        CURSOR=${#BUFFER}
    fi
}

# Register widget and bind key
zle -N _nudge_complete
bindkey '^E' _nudge_complete

# Print success message on first load
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    echo "Nudge loaded. Press Ctrl+E to trigger completion."
fi
