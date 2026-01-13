#!/usr/bin/env zsh
# Nudge - Zsh Integration
# Source this file from your .zshrc

# Configuration
NUDGE_HOTKEY="${NUDGE_HOTKEY:-^E}"
NUDGE_SOCKET="${NUDGE_SOCKET:-$HOME/.config/nudge/nudge.sock}"
NUDGE_LOCK="/tmp/nudge.lock"

# Capture last exit code
typeset -g _nudge_last_exit=0
_nudge_capture_exit() {
    _nudge_last_exit=$?
}
precmd_functions+=(_nudge_capture_exit)

# Ensure daemon is running (lazy load)
_nudge_ensure_daemon() {
    if [[ ! -S "$NUDGE_SOCKET" ]]; then
        # Use zsystem flock to prevent concurrent daemon starts
        if zsystem flock -t 0 "$NUDGE_LOCK" 2>/dev/null; then
            nudge daemon --fork 2>/dev/null
        fi
    fi
}

# Main completion widget
_nudge_complete() {
    # Ensure daemon is running
    _nudge_ensure_daemon

    # Call nudge with --format plain
    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$BUFFER" \
        --cursor "$CURSOR" \
        --cwd "$PWD" \
        --session "zsh-$$" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    # Update buffer if we got a suggestion
    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        BUFFER="$suggestion"
        CURSOR=${#BUFFER}
        zle redisplay
    fi
}

# Register widget and bind hotkey
zle -N nudge-complete _nudge_complete
bindkey "$NUDGE_HOTKEY" nudge-complete

# Print success message on first load
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    print "Nudge loaded. Press Ctrl+E to trigger completion."
fi
