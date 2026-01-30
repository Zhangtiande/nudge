#!/usr/bin/env zsh
# Nudge - Zsh Integration
# Installed by: nudge setup zsh

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

NUDGE_LOCK="/tmp/nudge.lock"

# Auto mode state
typeset -g _nudge_auto_suggestion=""
typeset -g _nudge_timer_fd=""
typeset -g _nudge_last_buffer=""
typeset -g _nudge_pending_buffer=""

# Diagnosis state
typeset -g _nudge_stderr_file=""
typeset -g _nudge_stderr_fd=""
typeset -g _nudge_last_command=""
NUDGE_DIAGNOSIS_ENABLED=$(nudge info --field diagnosis_enabled 2>/dev/null)

# Capture last exit code
_nudge_last_exit=0
_nudge_capture_exit() {
    _nudge_last_exit=$?
}
precmd_functions+=(_nudge_capture_exit)

# ============================================================================
# Error Diagnosis Functions
# ============================================================================

# Diagnosis preexec - capture stderr before command runs
_nudge_diagnosis_preexec() {
    [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]] && return

    _nudge_last_command="$1"
    _nudge_stderr_file="/tmp/nudge_stderr_$$"

    # Save original stderr and redirect to file
    exec {_nudge_stderr_fd}>&2
    exec 2>"$_nudge_stderr_file"
}

# Diagnosis precmd - analyze errors after command runs
_nudge_diagnosis_precmd() {
    local exit_code=$?

    # Restore stderr immediately
    if [[ -n "$_nudge_stderr_fd" ]]; then
        exec 2>&$_nudge_stderr_fd
        exec {_nudge_stderr_fd}>&-
        _nudge_stderr_fd=""
    fi

    # Cleanup function (always called)
    _nudge_diagnosis_cleanup() {
        rm -f "$_nudge_stderr_file"
        _nudge_stderr_file=""
        _nudge_last_command=""
    }

    # Only proceed if diagnosis enabled and command failed
    if [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]]; then
        _nudge_diagnosis_cleanup
        return
    fi
    if [[ $exit_code -eq 0 ]]; then
        _nudge_diagnosis_cleanup
        return
    fi
    if [[ -z "$_nudge_last_command" ]]; then
        _nudge_diagnosis_cleanup
        return
    fi

    # Check if stderr file has content
    if [[ -s "$_nudge_stderr_file" ]]; then
        _nudge_ensure_daemon

        # Get diagnosis (format: message\nsuggestion)
        local diagnosis
        diagnosis=$(nudge diagnose \
            --exit-code "$exit_code" \
            --command "$_nudge_last_command" \
            --stderr-file "$_nudge_stderr_file" \
            --cwd "$PWD" \
            --session "zsh-$$" \
            --format plain 2>/dev/null)

        if [[ $? -eq 0 && -n "$diagnosis" ]]; then
            # Split into message and suggestion
            local message="${diagnosis%%$'\n'*}"
            local suggestion="${diagnosis#*$'\n'}"

            # Print diagnosis message (replaces stderr)
            if [[ -n "$message" ]]; then
                echo "$message"
            fi

            # Show suggestion with prompt to accept
            if [[ -n "$suggestion" && "$suggestion" != "$message" ]]; then
                _nudge_auto_suggestion="$suggestion"
                # Print suggestion with visual hint
                echo -e "\033[90m💡 Suggested fix: \033[0m\033[1m$suggestion\033[0m \033[90m(press Tab to accept)\033[0m"
            fi
        fi
    fi

    # Cleanup
    _nudge_diagnosis_cleanup
}

# Ensure daemon is running
_nudge_ensure_daemon() {
    if [[ ! -S "$NUDGE_SOCKET" ]]; then
        # Use zsystem flock to prevent concurrent daemon starts
        if zsystem flock -t 0 "$NUDGE_LOCK" 2>/dev/null; then
            nudge start 2>/dev/null
        fi
    fi
}

# Main completion widget (manual mode)
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
        # Clear any auto suggestion
        _nudge_auto_suggestion=""
    fi
}

# ============================================================================
# Auto Mode Functions
# ============================================================================

# Cancel any pending auto completion
_nudge_auto_cancel() {
    if [[ -n "$_nudge_timer_fd" ]]; then
        # Unregister the fd handler
        zle -F "$_nudge_timer_fd" 2>/dev/null
        # Close the fd
        exec {_nudge_timer_fd}<&- 2>/dev/null
        _nudge_timer_fd=""
    fi
    _nudge_pending_buffer=""
    _nudge_auto_suggestion=""
}

# Fetch completion in background
_nudge_auto_fetch() {
    _nudge_ensure_daemon

    local suggestion
    suggestion=$(nudge complete --format plain \
        --buffer "$BUFFER" \
        --cursor "$CURSOR" \
        --cwd "$PWD" \
        --session "zsh-$$" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    local exit_code=$?

    if [[ $exit_code -eq 0 && -n "$suggestion" ]]; then
        # Only update if buffer hasn't changed
        if [[ "$BUFFER" == "$_nudge_last_buffer" ]]; then
            _nudge_auto_suggestion="$suggestion"
        fi
    fi
}

# Display inline preview (gray text after cursor)
_nudge_auto_display_preview() {
    # Ensure POSTDISPLAY is writable
    typeset -g POSTDISPLAY

    if [[ -n "$_nudge_auto_suggestion" && "$_nudge_auto_suggestion" != "$BUFFER" ]]; then
        # Calculate the preview text (suggestion minus current buffer)
        local preview="${_nudge_auto_suggestion:${#BUFFER}}"
        if [[ -n "$preview" ]]; then
            # Set POSTDISPLAY to the preview text
            POSTDISPLAY="$preview"

            # Use region_highlight to color it gray
            # Format: "start end style"
            # Start is after BUFFER, end is after BUFFER + preview length
            local start=${#BUFFER}
            local end=$((start + ${#preview}))

            # fg=8 is gray (bright black)
            region_highlight+=("$start $end fg=8")
        else
            POSTDISPLAY=""
        fi
    else
        POSTDISPLAY=""
    fi
}

# Accept auto suggestion
_nudge_auto_accept() {
    if [[ -n "$_nudge_auto_suggestion" ]]; then
        BUFFER="$_nudge_auto_suggestion"
        CURSOR=${#BUFFER}
        _nudge_auto_suggestion=""
        typeset -g POSTDISPLAY=""
        region_highlight=("${(@)region_highlight:#*}")
        zle -R
    else
        # Fall back to default Tab behavior (completion)
        zle expand-or-complete
    fi
}

# Accept partial suggestion (word by word)
_nudge_auto_accept_word() {
    if [[ -n "$_nudge_auto_suggestion" ]]; then
        # Get the next word from suggestion
        local remaining="${_nudge_auto_suggestion:${#BUFFER}}"
        local next_word="${remaining%% *}"
        if [[ "$remaining" == "$next_word" ]]; then
            # No space found, accept all
            BUFFER="$_nudge_auto_suggestion"
        else
            BUFFER="$BUFFER$next_word "
        fi
        CURSOR=${#BUFFER}
        _nudge_auto_display_preview
        zle -R
    else
        # Fall back to default Right Arrow behavior
        zle forward-char
    fi
}

# Trigger auto completion after debounce
_nudge_auto_trigger() {
    # Cancel previous timer
    _nudge_auto_cancel

    # Don't trigger for empty or very short input
    if [[ ${#BUFFER} -lt 2 ]]; then
        typeset -g POSTDISPLAY=""
        return
    fi

    # Save buffer for comparison
    _nudge_pending_buffer="$BUFFER"

    # Calculate delay in seconds
    local delay_sec
    delay_sec=$(printf "%.3f" "$(echo "scale=3; $NUDGE_AUTO_DELAY / 1000" | bc)")

    # Create an anonymous pipe and start background sleep process
    # The process will write to the pipe after delay
    {
        # Open anonymous pipe for reading
        exec {_nudge_timer_fd}< <(
            setopt LOCAL_OPTIONS NO_NOTIFY NO_MONITOR
            sleep "$delay_sec"
            echo "ready"
        )

        # Register fd handler - this will call our widget when pipe is readable
        zle -F "$_nudge_timer_fd" _nudge_auto_on_timer_ready
    } 2>/dev/null
}

# This is called by zle -F when the timer fd becomes readable
# It's NOT a widget, so we need to trigger a real widget
_nudge_auto_on_timer_ready() {
    local fd=$1

    # Read and discard the message
    local dummy
    IFS= read -r -u "$fd" dummy 2>/dev/null

    # Clean up fd
    zle -F "$fd" 2>/dev/null
    exec {fd}<&- 2>/dev/null
    _nudge_timer_fd=""

    # Trigger the actual widget that can call zle -R
    zle _nudge_auto_update_display
}

# This is the actual widget that updates the display
# Because it's a widget, zle -R works here
_nudge_auto_update_display() {
    # Only update if buffer hasn't changed
    if [[ "$BUFFER" == "$_nudge_pending_buffer" && -n "$_nudge_pending_buffer" ]]; then
        _nudge_last_buffer="$BUFFER"
        _nudge_pending_buffer=""

        # Fetch completion
        _nudge_auto_fetch

        # Display preview
        _nudge_auto_display_preview

        # Force redraw (this works because we're in a widget)
        zle -R
    fi
}

# Hook into line editing (called on every buffer change)
_nudge_auto_line_change() {
    if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
        # Clear preview if buffer changed
        if [[ "$BUFFER" != "$_nudge_last_buffer" ]]; then
            typeset -g POSTDISPLAY=""
            # Clear region_highlight
            region_highlight=("${(@)region_highlight:#*}")
            _nudge_auto_trigger
        fi
    fi
}

# ============================================================================
# Widget Registration
# ============================================================================

# Register widgets
zle -N _nudge_complete
zle -N _nudge_auto_accept
zle -N _nudge_auto_accept_word
zle -N _nudge_auto_update_display  # Register the update display widget

# Bind manual mode hotkey
bindkey '^E' _nudge_complete

# Setup auto mode if enabled
if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
    # Disable job notifications for background processes
    setopt NO_NOTIFY NO_MONITOR

    # Hook into line editing
    # Use zle-line-pre-redraw for buffer change detection
    _nudge_zle_line_pre_redraw() {
        _nudge_auto_line_change
    }
    zle -N zle-line-pre-redraw _nudge_zle_line_pre_redraw

    # Bind Tab to accept suggestion
    bindkey '^I' _nudge_auto_accept

    # Bind Right Arrow to accept word
    bindkey '^[[C' _nudge_auto_accept_word

    # Clean up on exit
    _nudge_cleanup() {
        _nudge_auto_cancel
    }
    zshexit_functions+=(_nudge_cleanup)
fi

# Setup diagnosis if enabled
if [[ "$NUDGE_DIAGNOSIS_ENABLED" == "true" ]]; then
    preexec_functions+=(_nudge_diagnosis_preexec)
    # Insert at beginning to capture exit code first
    precmd_functions=(_nudge_diagnosis_precmd "${precmd_functions[@]}")

    # Bind Tab to accept diagnosis suggestion (if not already bound by auto mode)
    if [[ "$NUDGE_TRIGGER_MODE" != "auto" ]]; then
        bindkey '^I' _nudge_auto_accept
    fi
fi

# Print success message on first load
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    local mode_msg=""
    if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
        mode_msg="auto mode"
    else
        mode_msg="manual mode (Ctrl+E)"
    fi
    if [[ "$NUDGE_DIAGNOSIS_ENABLED" == "true" ]]; then
        mode_msg="$mode_msg + error diagnosis"
    fi
    echo "Nudge loaded ($mode_msg)."
fi
