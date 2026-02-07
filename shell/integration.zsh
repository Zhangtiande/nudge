#!/usr/bin/env zsh
# Nudge - Zsh Integration
# Installed by: nudge setup zsh

# Get configuration from nudge CLI
NUDGE_CONFIG_DIR=$(nudge info --field config_dir 2>/dev/null)
NUDGE_SOCKET=$(nudge info --field socket_path 2>/dev/null)
NUDGE_TRIGGER_MODE=$(nudge info --field trigger_mode 2>/dev/null)
NUDGE_AUTO_DELAY=$(nudge info --field auto_delay_ms 2>/dev/null)
NUDGE_ZSH_GHOST_OWNER=$(nudge info --field zsh_ghost_owner 2>/dev/null)
NUDGE_ZSH_OVERLAY_BACKEND=$(nudge info --field zsh_overlay_backend 2>/dev/null)
NUDGE_WARNING_PREFIX="NUDGE_WARNING:"

# Fallback if nudge binary not in PATH
if [[ -z "$NUDGE_CONFIG_DIR" ]]; then
    NUDGE_CONFIG_DIR="$HOME/.nudge"
    NUDGE_SOCKET="$NUDGE_CONFIG_DIR/run/nudge.sock"
    NUDGE_TRIGGER_MODE="manual"
    NUDGE_AUTO_DELAY="500"
    NUDGE_ZSH_GHOST_OWNER="auto"
    NUDGE_ZSH_OVERLAY_BACKEND="message"
fi

NUDGE_LOCK="/tmp/nudge.lock"

if [[ -z "$NUDGE_ZSH_GHOST_OWNER" ]]; then
    NUDGE_ZSH_GHOST_OWNER="auto"
fi
if [[ -z "$NUDGE_ZSH_OVERLAY_BACKEND" ]]; then
    NUDGE_ZSH_OVERLAY_BACKEND="message"
fi

# Auto mode state
typeset -g _nudge_auto_suggestion=""
typeset -g _nudge_auto_warning=""
typeset -g _nudge_last_buffer=""
typeset -g _nudge_last_warning_buffer=""
typeset -g _nudge_region_highlight_entry=""
typeset -g _nudge_ghost_owner_effective="nudge"
typeset -g _nudge_auto_mode_enabled="false"
typeset -g _nudge_overlay_mode_enabled="false"
typeset -g _nudge_overlay_hooks_installed="false"
typeset -g _nudge_overlay_last_message=""
typeset -g _nudge_overlay_backend_effective="message"
typeset -g _nudge_overlay_saved_rprompt=""
typeset -g _nudge_overlay_rprompt_active="false"
typeset -g _nudge_overlay_risk_level="low"
typeset -g _nudge_explain_expanded="false"

_nudge_has_autosuggestions() {
    (( ${+functions[_zsh_autosuggest_start]} )) && return 0
    (( ${+functions[_zsh_autosuggest_bind_widgets]} )) && return 0
    (( ${+widgets[autosuggest-accept]} )) && return 0
    (( ${+ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE} )) && return 0
    return 1
}

_nudge_resolve_ghost_owner() {
    local configured_owner="${NUDGE_ZSH_GHOST_OWNER:l}"

    case "$configured_owner" in
        auto|nudge|autosuggestions)
            ;;
        *)
            configured_owner="auto"
            ;;
    esac

    if [[ "$configured_owner" == "auto" ]]; then
        if _nudge_has_autosuggestions; then
            _nudge_ghost_owner_effective="autosuggestions"
        else
            _nudge_ghost_owner_effective="nudge"
        fi
    else
        _nudge_ghost_owner_effective="$configured_owner"
    fi

    if [[ "$NUDGE_TRIGGER_MODE" == "auto" && "$_nudge_ghost_owner_effective" == "nudge" ]]; then
        _nudge_auto_mode_enabled="true"
        _nudge_overlay_mode_enabled="false"
    elif [[ "$NUDGE_TRIGGER_MODE" == "auto" && "$_nudge_ghost_owner_effective" == "autosuggestions" ]]; then
        _nudge_auto_mode_enabled="false"
        _nudge_overlay_mode_enabled="true"
    else
        _nudge_auto_mode_enabled="false"
        _nudge_overlay_mode_enabled="false"
    fi
}

_nudge_resolve_ghost_owner

_nudge_resolve_overlay_backend() {
    local configured_backend="${NUDGE_ZSH_OVERLAY_BACKEND:l}"

    case "$configured_backend" in
        message|rprompt)
            _nudge_overlay_backend_effective="$configured_backend"
            ;;
        *)
            _nudge_overlay_backend_effective="message"
            ;;
    esac
}

_nudge_resolve_overlay_backend

_nudge_clear_own_highlight() {
    [[ -z "$_nudge_region_highlight_entry" ]] && return

    local -a filtered_highlights
    local entry
    for entry in "${region_highlight[@]}"; do
        if [[ "$entry" != "$_nudge_region_highlight_entry" ]]; then
            filtered_highlights+=("$entry")
        fi
    done
    region_highlight=("${filtered_highlights[@]}")
    _nudge_region_highlight_entry=""
}

_nudge_set_own_highlight() {
    local start="$1"
    local end="$2"

    _nudge_clear_own_highlight
    _nudge_region_highlight_entry="$start $end fg=8"
    region_highlight+=("$_nudge_region_highlight_entry")
}

_nudge_overlay_clear_message() {
    [[ -z "$_nudge_overlay_last_message" ]] && return

    if [[ "$_nudge_overlay_backend_effective" == "rprompt" ]]; then
        if [[ "$_nudge_overlay_rprompt_active" == "true" ]]; then
            RPS1="$_nudge_overlay_saved_rprompt"
            RPROMPT="$_nudge_overlay_saved_rprompt"
            _nudge_overlay_rprompt_active="false"
            zle reset-prompt 2>/dev/null || zle -R 2>/dev/null
        fi
    else
        zle -M "" 2>/dev/null
    fi
    _nudge_overlay_last_message=""
}

_nudge_clear_autosuggest_preview() {
    [[ "$_nudge_ghost_owner_effective" != "autosuggestions" ]] && return

    typeset -g POSTDISPLAY=""
    if (( ${+widgets[autosuggest-clear]} )); then
        zle autosuggest-clear 2>/dev/null
    fi
}

_nudge_overlay_set_message() {
    local message="$1"
    local rendered_message="$message"

    if [[ "$_nudge_overlay_backend_effective" == "rprompt" ]]; then
        if [[ "$_nudge_overlay_rprompt_active" != "true" ]]; then
            _nudge_overlay_saved_rprompt="${RPROMPT:-$RPS1}"
            _nudge_overlay_rprompt_active="true"
        fi

        local label_color="2"
        local body_color="8"
        if [[ "$_nudge_overlay_risk_level" == "high" ]]; then
            label_color="1"
            body_color="1"
        fi

        local prompt_msg="${message//\%/%%}"
        rendered_message="%F{${label_color}}diff%f %F{${body_color}}${prompt_msg}%f"
        if [[ "$rendered_message" == "$_nudge_overlay_last_message" \
            && "$RPROMPT" == "$rendered_message" \
            && "$RPS1" == "$rendered_message" ]]; then
            return
        fi
        RPS1="$rendered_message"
        RPROMPT="$rendered_message"
        zle reset-prompt 2>/dev/null || zle -R 2>/dev/null
    else
        local plain_prefix="[nudge]"
        local plain_risk="[low]"
        if [[ "$_nudge_overlay_risk_level" == "high" ]]; then
            plain_risk="[high]"
        fi
        rendered_message="${plain_prefix} ${plain_risk} ${message}"
        [[ "$rendered_message" == "$_nudge_overlay_last_message" ]] && return
        zle -M -- "$rendered_message" 2>/dev/null
    fi
    _nudge_overlay_last_message="$rendered_message"
}

# Widget classification for auto mode (inspired by zsh-autosuggestions)
# Widgets that modify the buffer - trigger new suggestion fetch
typeset -ga NUDGE_MODIFY_WIDGETS=(
    self-insert
    backward-delete-char
    delete-char
    delete-word
    backward-delete-word
    kill-word
    backward-kill-word
    kill-line
    backward-kill-line
    kill-whole-line
    quoted-insert
    yank
    yank-pop
    vi-delete
    vi-change
    vi-substitute
)

# Widgets that clear the suggestion (history navigation)
typeset -ga NUDGE_CLEAR_WIDGETS=(
    up-line-or-history
    down-line-or-history
    up-line-or-beginning-search
    down-line-or-beginning-search
    history-search-forward
    history-search-backward
    history-beginning-search-forward
    history-beginning-search-backward
    history-substring-search-up
    history-substring-search-down
    accept-line
    accept-and-hold
)

# Widgets that accept the entire suggestion
typeset -ga NUDGE_ACCEPT_WIDGETS=(
    end-of-line
    vi-end-of-line
    vi-add-eol
)

# Widgets that accept suggestion partially (word by word)
typeset -ga NUDGE_PARTIAL_ACCEPT_WIDGETS=(
    forward-word
    vi-forward-word
    vi-forward-word-end
    emacs-forward-word
)

# Widgets to ignore completely
typeset -ga NUDGE_IGNORE_WIDGETS=(
    beep
    run-help
    set-local-history
    which-command
    'zle-*'
    'orig-*'
    'autosuggest-*'
    '_nudge_*'
)

# Prefix for saving original widgets
typeset -g NUDGE_ORIG_WIDGET_PREFIX="_nudge_orig_"

# Diagnosis state
typeset -g _nudge_stderr_file=""
typeset -g _nudge_stderr_fd=""
typeset -g _nudge_last_command=""
typeset -g _nudge_skip_capture=""
NUDGE_DIAGNOSIS_ENABLED=$(nudge info --field diagnosis_enabled 2>/dev/null)

# Interactive commands list (loaded from config, cached for performance)
typeset -ga _nudge_interactive_commands
_nudge_interactive_commands=($(nudge info --field interactive_commands 2>/dev/null | tr ',' ' '))
# Fallback if nudge info fails
if [[ ${#_nudge_interactive_commands[@]} -eq 0 ]]; then
    _nudge_interactive_commands=(
        vim nvim vi nano emacs code
        ssh telnet mosh
        top htop btop less more man
        fzf sk
        tmux screen
        python python3 ipython node irb psql mysql sqlite3
        watch tail
    )
fi

# Capture last exit code
_nudge_last_exit=0
_nudge_capture_exit() {
    _nudge_last_exit=$?
}
precmd_functions+=(_nudge_capture_exit)

# ============================================================================
# Error Diagnosis Functions
# ============================================================================

# Check if command is interactive (should skip stderr capture)
_nudge_is_interactive_command() {
    local cmd="$1"
    # Extract the first word (command name), handling pipes and redirects
    local first_word="${cmd%% *}"
    # Remove any leading env vars like VAR=value
    first_word="${first_word##*=}"
    # Handle commands with path like /usr/bin/vim
    first_word="${first_word##*/}"

    # Check against interactive commands list
    local interactive_cmd
    for interactive_cmd in "${_nudge_interactive_commands[@]}"; do
        [[ "$first_word" == "$interactive_cmd" ]] && return 0
    done
    return 1
}

# Display warning message for dangerous suggestions
_nudge_show_warning() {
    local message="$1"
    echo -e "\033[33mWarning:\033[0m $message"
}

# Diagnosis preexec - capture stderr before command runs
_nudge_diagnosis_preexec() {
    [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]] && return

    # Clear any pending diagnosis suggestion when user executes a new command
    # This prevents stale suggestions from appearing on next Tab press
    _nudge_auto_suggestion=""
    _nudge_overlay_clear_message

    _nudge_last_command="$1"
    _nudge_skip_capture=""

    # Skip stderr capture for interactive commands
    if _nudge_is_interactive_command "$1"; then
        _nudge_skip_capture="1"
        return
    fi

    _nudge_stderr_file="/tmp/nudge_stderr_$$"

    # Save original stderr and redirect to file
    exec {_nudge_stderr_fd}>&2
    exec 2>"$_nudge_stderr_file"
}

# Diagnosis precmd - analyze errors after command runs
_nudge_diagnosis_precmd() {
    local exit_code=$?

    # Clear any stale diagnosis suggestion at the start of each prompt
    # This handles cases where user pressed Enter without accepting the suggestion
    # or when preexec wasn't called (e.g., empty command)
    _nudge_auto_suggestion=""
    _nudge_overlay_clear_message

    # Restore stderr immediately (only if we captured it)
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
        _nudge_skip_capture=""
    }

    # Only proceed if diagnosis enabled
    if [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]]; then
        _nudge_diagnosis_cleanup
        return
    fi
    if [[ -z "$_nudge_last_command" ]]; then
        _nudge_diagnosis_cleanup
        return
    fi

    # Skip diagnosis for interactive commands (stderr was not captured)
    if [[ -n "$_nudge_skip_capture" ]]; then
        _nudge_diagnosis_cleanup
        return
    fi

    # Always output original stderr first (for progress output like cargo build)
    if [[ -s "$_nudge_stderr_file" ]]; then
        cat "$_nudge_stderr_file" >&2
    fi

    # Only run diagnosis if command failed
    if [[ $exit_code -eq 0 ]]; then
        _nudge_diagnosis_cleanup
        return
    fi

    # Check if stderr file has content for diagnosis
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

            # Print diagnosis with visual separator
            echo -e "\n\033[90m── nudge ──────────────────────────────────────\033[0m"
            if [[ -n "$message" ]]; then
                echo "$message"
            fi

            # Show suggestion with prompt to accept
            if [[ -n "$suggestion" && "$suggestion" != "$message" ]]; then
                _nudge_auto_suggestion="$suggestion"
                # Print suggestion with visual hint
                echo -e "\033[90m💡 Suggested fix: \033[0m\033[1m$suggestion\033[0m \033[90m(press Tab to accept)\033[0m"
            fi
            echo -e "\033[90m───────────────────────────────────────────────\033[0m"
        fi
    fi

    # Cleanup
    _nudge_diagnosis_cleanup
}

# Ensure daemon is running
# Uses `nudge status` for reliable detection (checks PID file + process alive)
# Falls back to socket check if nudge binary is unavailable
_nudge_ensure_daemon() {
    # Fast path: check if daemon is responding via status command
    if nudge status >/dev/null 2>&1; then
        return 0
    fi

    # Daemon not running, try to start it with lock to prevent concurrent starts
    if zsystem flock -t 0 "$NUDGE_LOCK" 2>/dev/null; then
        nudge start 2>/dev/null
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
        --shell-mode "zsh-inline" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null)

    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        if [[ "$suggestion" == ${NUDGE_WARNING_PREFIX}* ]]; then
            local warning_message="${suggestion#${NUDGE_WARNING_PREFIX}}"
            warning_message="${warning_message# }"
            _nudge_auto_suggestion=""
            _nudge_auto_warning=""
            _nudge_show_warning "$warning_message"
            return
        fi
        BUFFER="$suggestion"
        CURSOR=${#BUFFER}
        # Clear any auto suggestion
        _nudge_auto_suggestion=""
        _nudge_auto_warning=""
        POSTDISPLAY=""
        _nudge_clear_own_highlight
        _nudge_overlay_clear_message
        _nudge_clear_autosuggest_preview
        zle -R 2>/dev/null
    fi
}

# ============================================================================
# Auto Mode Functions
# ============================================================================

# Cancel any pending auto completion (wrapper for compatibility)
_nudge_auto_cancel() {
    _nudge_async_cancel
    _nudge_auto_suggestion=""
    if [[ "$_nudge_overlay_mode_enabled" != "true" ]]; then
        POSTDISPLAY=""
    fi
    _nudge_clear_own_highlight
    _nudge_overlay_clear_message
}

# ============================================================================
# Async Suggestion Fetching (event-driven)
# ============================================================================

# State for async operations
typeset -g _nudge_async_fd=""
typeset -g _nudge_child_pid=""
typeset -g _nudge_child_generation=0
typeset -g _nudge_generation_seq=0
typeset -g _nudge_last_applied_generation=0
typeset -g _nudge_async_suggestion_temp=""
typeset -g _nudge_async_generation_temp=0

# Cancel any pending async request
_nudge_async_cancel() {
    if [[ -n "$_nudge_async_fd" ]]; then
        # Remove fd handler
        zle -F "$_nudge_async_fd" 2>/dev/null
        # Close fd
        builtin exec {_nudge_async_fd}<&- 2>/dev/null

        # Kill child process if we have its PID
        if [[ -n "$_nudge_child_pid" ]]; then
            if [[ -o MONITOR ]]; then
                # Kill process group
                kill -TERM -$_nudge_child_pid 2>/dev/null
            else
                # Kill just the process
                kill -TERM $_nudge_child_pid 2>/dev/null
            fi
        fi

        _nudge_async_fd=""
        _nudge_child_pid=""
        _nudge_child_generation=0
    fi
}

_nudge_should_fetch_on_event() {
    # Only available in zsh >= 5.4
    local -i KEYS_QUEUED_COUNT 2>/dev/null
    (( PENDING > 0 || KEYS_QUEUED_COUNT > 0 )) && return 1
    return 0
}

# Fetch suggestion asynchronously
_nudge_fetch_async() {

    zmodload zsh/system 2>/dev/null  # For $sysparams

    # Cancel any pending request
    _nudge_async_cancel

    # Don't fetch for very short input
    if [[ ${#BUFFER} -lt 2 ]]; then
        return
    fi

    _nudge_ensure_daemon

    local current_buffer="$BUFFER"
    local current_cursor="$CURSOR"
    _nudge_generation_seq=$((_nudge_generation_seq + 1))
    local current_generation=$_nudge_generation_seq

    # Fork process to fetch suggestion
    builtin exec {_nudge_async_fd}< <(
        # Send PID first for cancellation
        echo $sysparams[pid]
        # Send request generation for response arbitration
        echo "$current_generation"

        # Fetch suggestion
        local suggestion
        suggestion=$(nudge complete --format plain \
            --buffer "$current_buffer" \
            --cursor "$current_cursor" \
            --cwd "$PWD" \
            --session "zsh-$$" \
            --shell-mode "zsh-auto" \
            --time-bucket $((EPOCHSECONDS / 2)) \
            --last-exit-code "$_nudge_last_exit" 2>/dev/null)

        # Output suggestion
        echo -nE "$suggestion"
    )

    # Workaround for ^C bug in older zsh versions
    command true

    # Read child PID
    read _nudge_child_pid <&$_nudge_async_fd
    # Read request generation
    _nudge_child_generation=0
    read _nudge_child_generation <&$_nudge_async_fd || _nudge_child_generation=0

    # Register handler for when result is ready
    zle -F "$_nudge_async_fd" _nudge_async_response
}

# Handle async response
_nudge_async_response() {
    emulate -L zsh

    local fd=$1
    local error=$2


    if [[ -z "$error" || "$error" == "hup" ]]; then
        # Read the suggestion
        local suggestion
        IFS='' read -rd '' -u $fd suggestion 2>/dev/null


        # Store suggestion for widget to use (can't access $BUFFER in fd handler)
        if [[ -n "$suggestion" ]]; then
            _nudge_async_suggestion_temp="$suggestion"
            _nudge_async_generation_temp="${_nudge_child_generation:-0}"
            # Trigger widget to update display
            zle _nudge_async_update
        fi
    fi

    # Clean up
    builtin exec {fd}<&- 2>/dev/null
    zle -F "$fd" 2>/dev/null
    _nudge_async_fd=""
    _nudge_child_pid=""
    _nudge_child_generation=0
}

# Widget to update display after async response
_nudge_async_update() {

    # Drop stale responses by generation before touching UI state
    if (( _nudge_async_generation_temp < _nudge_last_applied_generation )); then
        _nudge_async_suggestion_temp=""
        _nudge_async_generation_temp=0
        return
    fi

    # Only update if buffer hasn't changed
    if [[ "$BUFFER" == "$_nudge_last_buffer" && -n "$_nudge_async_suggestion_temp" ]]; then
        local suggestion="$_nudge_async_suggestion_temp"
        _nudge_async_suggestion_temp=""
        _nudge_last_applied_generation="$_nudge_async_generation_temp"
        _nudge_async_generation_temp=0

        # Check for warning prefix
        if [[ "$suggestion" == ${NUDGE_WARNING_PREFIX}* ]]; then
            local warning_message="${suggestion#${NUDGE_WARNING_PREFIX}}"
            warning_message="${warning_message# }"
            _nudge_auto_warning="$warning_message"
            _nudge_auto_suggestion=""
        else
            _nudge_auto_warning=""
            _nudge_auto_suggestion="$suggestion"
        fi

        _nudge_auto_display_preview
        _nudge_overlay_render
        zle -R
    else
        _nudge_async_suggestion_temp=""
        _nudge_async_generation_temp=0
    fi
}

# Display inline preview (gray text after cursor)
_nudge_auto_display_preview() {
    if [[ "$_nudge_overlay_mode_enabled" == "true" ]]; then
        _nudge_clear_own_highlight
        return
    fi

    # Ensure POSTDISPLAY is writable
    typeset -g POSTDISPLAY


    if [[ -n "$_nudge_auto_suggestion" && "$_nudge_auto_suggestion" != "$BUFFER" ]]; then
        # Check if suggestion starts with buffer
        if [[ "$_nudge_auto_suggestion" == "$BUFFER"* ]]; then
            # Calculate the preview text (suggestion minus current buffer)
            local preview="${_nudge_auto_suggestion:${#BUFFER}}"
            if [[ -n "$preview" ]]; then
                # Set POSTDISPLAY to the preview text
                POSTDISPLAY="$preview"

                # Use region_highlight to color it gray
                local start=${#BUFFER}
                local end=$((start + ${#preview}))

                # Keep other plugin highlights intact and manage only nudge highlight
                _nudge_set_own_highlight "$start" "$end"

            else
                POSTDISPLAY=""
                _nudge_clear_own_highlight
            fi
        else
            # Suggestion doesn't start with buffer, show full suggestion as replacement
            POSTDISPLAY=""
            _nudge_clear_own_highlight
        fi
    else
        POSTDISPLAY=""
        _nudge_clear_own_highlight
    fi
}

_nudge_overlay_render() {
    if [[ "$_nudge_auto_mode_enabled" != "true" && "$_nudge_overlay_mode_enabled" != "true" ]]; then
        return
    fi

    local message=""
    local key_hint="Tab"
    local detail_hint="F1 details"
    local suggestion="$BUFFER"
    local why="prefix completion"
    local risk="low"
    local diff="+<none>"

    if [[ "$_nudge_overlay_mode_enabled" == "true" || "$_nudge_ghost_owner_effective" == "autosuggestions" ]]; then
        key_hint="Ctrl+G"
    fi

    if [[ -n "$_nudge_auto_suggestion" ]]; then
        suggestion="$_nudge_auto_suggestion"
    fi

    if [[ -n "$_nudge_auto_warning" ]]; then
        risk="high"
        why="safety check flagged"
    elif [[ "$suggestion" == "$BUFFER"* ]]; then
        local tail="${suggestion:${#BUFFER}}"
        if [[ -n "$tail" ]]; then
            diff="+$tail"
        fi
    else
        why="context rewrite"
        diff="~ ${BUFFER} -> ${suggestion}"
    fi

    diff="${diff//$'\n'/ }"
    diff="${diff//$'\t'/ }"
    suggestion="${suggestion//$'\n'/ }"
    suggestion="${suggestion//$'\t'/ }"
    _nudge_overlay_risk_level="$risk"

    if [[ "$_nudge_overlay_backend_effective" == "rprompt" ]]; then
        if [[ -n "$_nudge_auto_suggestion" && "$_nudge_auto_suggestion" != "$BUFFER" ]]; then
            local compact="$diff"
            if (( ${#compact} > 42 )); then
                compact="${compact:0:39}..."
            fi
            _nudge_overlay_set_message "$compact"
        else
            _nudge_overlay_clear_message
        fi
        return
    fi

    if [[ -n "$_nudge_auto_warning" ]]; then
        local warning_text="${_nudge_auto_warning//$'\n'/ }"
        warning_text="${warning_text//$'\t'/ }"
        if [[ "$_nudge_explain_expanded" == "true" ]]; then
            message="why:${why} | risk:${risk} | warn:${warning_text} | diff:${diff} | $key_hint accept"
        else
            message="why:${why} | risk:${risk} | diff:${diff} (${detail_hint})"
        fi
    elif [[ -n "$_nudge_auto_suggestion" && "$_nudge_auto_suggestion" != "$BUFFER" ]]; then
        if [[ "$_nudge_explain_expanded" == "true" ]]; then
            local full_diff="$diff"
            if (( ${#full_diff} > 56 )); then
                full_diff="${full_diff:0:53}..."
            fi
            message="why:${why} | risk:${risk} | diff:${full_diff} | suggest=${suggestion} | $key_hint accept"
        else
            local preview="$diff"
            if (( ${#preview} > 50 )); then
                preview="${preview:0:47}..."
            fi
            message="why:${why} | risk:${risk} | diff:${preview} (${detail_hint})"
        fi
    fi

    if [[ -n "$message" ]]; then
        _nudge_overlay_set_message "$message"
    else
        _nudge_overlay_clear_message
    fi
}

_nudge_overlay_line_pre_redraw() {
    [[ "$_nudge_overlay_mode_enabled" != "true" ]] && return

    case "$LASTWIDGET" in
        up-line-or-history|down-line-or-history|up-line-or-beginning-search|down-line-or-beginning-search|history-*)
            _nudge_auto_suggestion=""
            _nudge_auto_warning=""
            _nudge_overlay_clear_message
            return
            ;;
    esac

    if [[ "$BUFFER" == "$_nudge_last_buffer" ]]; then
        return
    fi

    _nudge_last_buffer="$BUFFER"
    _nudge_auto_suggestion=""
    _nudge_auto_warning=""
    _nudge_last_warning_buffer=""

    if (( ${#BUFFER} >= 2 )); then
        if _nudge_should_fetch_on_event; then
            _nudge_fetch_async
        fi
    else
        _nudge_auto_cancel
    fi
}

_nudge_overlay_line_finish() {
    [[ "$_nudge_overlay_mode_enabled" != "true" ]] && return
    _nudge_auto_cancel
}

_nudge_toggle_explanation() {
    if [[ -z "$_nudge_auto_suggestion" && -z "$_nudge_auto_warning" ]]; then
        zle run-help 2>/dev/null || return 0
        return 0
    fi

    if [[ "$_nudge_explain_expanded" == "true" ]]; then
        _nudge_explain_expanded="false"
    else
        _nudge_explain_expanded="true"
    fi

    _nudge_overlay_render
    zle -R
}

# Accept auto suggestion
_nudge_auto_accept() {
    if [[ -n "$_nudge_auto_warning" ]]; then
        if [[ "$_nudge_last_warning_buffer" != "$BUFFER" ]]; then
            _nudge_show_warning "$_nudge_auto_warning"
            _nudge_last_warning_buffer="$BUFFER"
        fi
        _nudge_auto_warning=""
        _nudge_auto_suggestion=""
        _nudge_overlay_clear_message
        return
    fi
    if [[ -n "$_nudge_auto_suggestion" ]]; then
        BUFFER="$_nudge_auto_suggestion"
        CURSOR=${#BUFFER}
        _nudge_auto_suggestion=""
        if [[ "$_nudge_overlay_mode_enabled" != "true" ]]; then
            typeset -g POSTDISPLAY=""
        fi
        _nudge_clear_autosuggest_preview
        _nudge_clear_own_highlight
        _nudge_overlay_clear_message
        zle -R
    else
        # Fall back to default Tab behavior (completion)
        zle expand-or-complete
    fi
}

_nudge_overlay_accept() {
    if [[ -n "$_nudge_auto_warning" || -n "$_nudge_auto_suggestion" ]]; then
        _nudge_auto_accept
    else
        zle send-break
    fi
}

# Accept partial suggestion (word by word)
_nudge_auto_accept_word() {
    if [[ -n "$_nudge_auto_warning" ]]; then
        _nudge_auto_accept
        return
    fi
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
        _nudge_overlay_render
        zle -R
    else
        # Fall back to default Right Arrow behavior
        zle forward-char
    fi
}

# ============================================================================
# Widget Wrapping Infrastructure
# ============================================================================

# Track bind counts to handle multiple bindings of same widget
typeset -gA _NUDGE_BIND_COUNTS

# Increment and return bind count for a widget
_nudge_incr_bind_count() {
    local widget=$1
    typeset -gi bind_count=$((_NUDGE_BIND_COUNTS[$widget]+1))
    _NUDGE_BIND_COUNTS[$widget]=$bind_count
}

# Bind a widget to a nudge action, saving reference to original
_nudge_bind_widget() {
    local widget=$1
    local action=$2
    local prefix=$NUDGE_ORIG_WIDGET_PREFIX
    local -i bind_count

    # Check widget type and save original
    case $widgets[$widget] in
        # Already bound by us
        user:_nudge_bound_*|user:_nudge_orig_*)
            bind_count=$((_NUDGE_BIND_COUNTS[$widget]))
            ;;
        # User-defined widget
        user:*)
            _nudge_incr_bind_count $widget
            bind_count=$_NUDGE_BIND_COUNTS[$widget]
            zle -N $prefix$bind_count-$widget ${widgets[$widget]#*:}
            ;;
        # Built-in widget
        builtin)
            _nudge_incr_bind_count $widget
            bind_count=$_NUDGE_BIND_COUNTS[$widget]
            eval "_nudge_orig_${(q)widget}() { zle .${(q)widget} }"
            zle -N $prefix$bind_count-$widget _nudge_orig_$widget
            ;;
        # Completion widget
        completion:*)
            _nudge_incr_bind_count $widget
            bind_count=$_NUDGE_BIND_COUNTS[$widget]
            eval "zle -C $prefix$bind_count-${(q)widget} ${${(s.:.)widgets[$widget]}[2,3]}"
            ;;
        # Unknown - skip
        *)
            return 1
            ;;
    esac

    # Create bound widget that calls our action handler
    eval "_nudge_bound_${bind_count}_${(q)widget}() {
        _nudge_widget_$action $prefix$bind_count-${(q)widget} \$@
    }"

    # Register the new widget
    zle -N -- $widget _nudge_bound_${bind_count}_$widget
}

# Invoke original widget by name
_nudge_invoke_original_widget() {
    (( $# )) || return 0
    local original_widget_name="$1"
    shift
    if (( ${+widgets[$original_widget_name]} )); then
        zle $original_widget_name -- $@
    fi
}

# Check if widget matches any pattern in an array
_nudge_widget_in_list() {
    local widget=$1
    shift
    local pattern
    for pattern in $@; do
        [[ $widget == $~pattern ]] && return 0
    done
    return 1
}

# Bind all widgets based on classification
_nudge_bind_all_widgets() {
    emulate -L zsh
    local widget

    # Patterns to ignore
    local ignore_patterns=(
        '.*'
        '_*'
        $NUDGE_IGNORE_WIDGETS
    )

    # Iterate all widgets
    for widget in ${${(f)"$(builtin zle -la)"}:#${(j:|:)~ignore_patterns}}; do
        if _nudge_widget_in_list $widget $NUDGE_CLEAR_WIDGETS; then
            _nudge_bind_widget $widget clear
        elif _nudge_widget_in_list $widget $NUDGE_ACCEPT_WIDGETS; then
            _nudge_bind_widget $widget accept
        elif _nudge_widget_in_list $widget $NUDGE_PARTIAL_ACCEPT_WIDGETS; then
            _nudge_bind_widget $widget partial_accept
        elif _nudge_widget_in_list $widget $NUDGE_MODIFY_WIDGETS; then
            _nudge_bind_widget $widget modify
        fi
        # Unclassified widgets are not wrapped (pass through)
    done
}

# ============================================================================
# Widget Action Handlers
# ============================================================================

# Handler for widgets that modify the buffer
_nudge_widget_modify() {
    local orig_widget=$1
    shift
    local -i retval


    # Only available in zsh >= 5.4
    local -i KEYS_QUEUED_COUNT 2>/dev/null

    # Save original state
    local orig_buffer="$BUFFER"
    local orig_postdisplay="$POSTDISPLAY"

    # Clear suggestion while processing
    POSTDISPLAY=

    # Call original widget
    _nudge_invoke_original_widget $orig_widget $@
    retval=$?

    emulate -L zsh


    # If more keys are queued, skip fetching (user is typing fast)
    if (( PENDING > 0 || KEYS_QUEUED_COUNT > 0 )); then
        POSTDISPLAY="$orig_postdisplay"
        return $retval
    fi

    # Optimization: if user is typing into the suggestion, just truncate
    if [[ "$BUFFER" = "$orig_buffer"* && -n "$orig_postdisplay" ]]; then
        local typed_len=$((${#BUFFER} - ${#orig_buffer}))
        if [[ "$orig_postdisplay" = "${BUFFER:${#orig_buffer}}"* ]]; then
            POSTDISPLAY="${orig_postdisplay:$typed_len}"
            _nudge_auto_suggestion="$BUFFER$POSTDISPLAY"
            _nudge_highlight_suggestion
            _nudge_overlay_render
            return $retval
        fi
    fi

    # Fetch new suggestion if buffer is not empty
    if (( ${#BUFFER} >= 2 )); then
        _nudge_last_buffer="$BUFFER"
        _nudge_fetch_async
    else
        _nudge_auto_suggestion=""
        POSTDISPLAY=""
        _nudge_clear_own_highlight
        _nudge_overlay_clear_message
    fi

    return $retval
}

# Handler for widgets that clear the suggestion (history navigation)
_nudge_widget_clear() {
    local orig_widget=$1
    shift

    # Clear suggestion
    _nudge_auto_suggestion=""
    POSTDISPLAY=""
    _nudge_clear_own_highlight
    _nudge_overlay_clear_message

    # Call original widget
    _nudge_invoke_original_widget $orig_widget $@
}

# Handler for widgets that accept the entire suggestion
_nudge_widget_accept() {
    local orig_widget=$1
    shift
    local -i retval

    # If we have a suggestion and cursor is at end, accept it
    if [[ -n "$_nudge_auto_suggestion" && $CURSOR -eq ${#BUFFER} && -n "$POSTDISPLAY" ]]; then
        BUFFER="$_nudge_auto_suggestion"
        _nudge_auto_suggestion=""
        POSTDISPLAY=""
        _nudge_clear_own_highlight
        _nudge_overlay_clear_message
        CURSOR=${#BUFFER}
    fi

    # Call original widget
    _nudge_invoke_original_widget $orig_widget $@
    retval=$?

    return $retval
}

# Handler for widgets that accept suggestion partially
_nudge_widget_partial_accept() {
    local orig_widget=$1
    shift
    local -i retval

    if [[ -n "$_nudge_auto_suggestion" && -n "$POSTDISPLAY" ]]; then
        # Temporarily accept full suggestion
        local original_buffer="$BUFFER"
        BUFFER="$_nudge_auto_suggestion"

        # Let original widget move cursor
        _nudge_invoke_original_widget $orig_widget $@
        retval=$?

        local cursor_pos=$CURSOR

        # If cursor moved past original buffer end
        if (( cursor_pos > ${#original_buffer} )); then
            # Keep buffer up to cursor, rest becomes POSTDISPLAY
            POSTDISPLAY="${BUFFER:$cursor_pos}"
            BUFFER="${BUFFER:0:$cursor_pos}"
            _nudge_highlight_suggestion
            _nudge_overlay_render
        else
            # Restore original buffer
            BUFFER="$original_buffer"
        fi
    else
        _nudge_invoke_original_widget $orig_widget $@
        retval=$?
    fi

    return $retval
}

# Helper to apply highlight to POSTDISPLAY
_nudge_highlight_suggestion() {
    if [[ -n "$POSTDISPLAY" ]]; then
        local start=${#BUFFER}
        local end=$((start + ${#POSTDISPLAY}))
        _nudge_set_own_highlight "$start" "$end"
    else
        _nudge_clear_own_highlight
    fi
}

_nudge_bind_widget_to_keymaps() {
    local key="$1"
    local widget="$2"

    bindkey "$key" "$widget" 2>/dev/null
    bindkey -M emacs "$key" "$widget" 2>/dev/null
    bindkey -M viins "$key" "$widget" 2>/dev/null
}

_nudge_bind_accept_navigation_keys() {
    zmodload zsh/terminfo 2>/dev/null

    # Word acceptance (Right Arrow)
    _nudge_bind_widget_to_keymaps '^[[C' _nudge_auto_accept_word
    if [[ -n "${terminfo[kcuf1]}" && "${terminfo[kcuf1]}" != '^[[C' ]]; then
        _nudge_bind_widget_to_keymaps "${terminfo[kcuf1]}" _nudge_auto_accept_word
    fi
}

_nudge_bind_explanation_toggle_keys() {
    zmodload zsh/terminfo 2>/dev/null

    _nudge_bind_widget_to_keymaps $'\eOP' _nudge_toggle_explanation
    _nudge_bind_widget_to_keymaps $'\e[11~' _nudge_toggle_explanation
    if [[ -n "${terminfo[kf1]}" ]]; then
        _nudge_bind_widget_to_keymaps "${terminfo[kf1]}" _nudge_toggle_explanation
    fi
}

_nudge_setup_overlay_hooks() {
    autoload -Uz add-zle-hook-widget 2>/dev/null
    (( ${+functions[add-zle-hook-widget]} )) || return
    [[ "$_nudge_overlay_hooks_installed" == "true" ]] && return

    add-zle-hook-widget line-pre-redraw _nudge_overlay_line_pre_redraw
    add-zle-hook-widget line-finish _nudge_overlay_line_finish
    _nudge_overlay_hooks_installed="true"
}

# ============================================================================
# Widget Registration
# ============================================================================

# Register widgets
zle -N _nudge_complete
zle -N _nudge_auto_accept
zle -N _nudge_overlay_accept
zle -N _nudge_auto_accept_word
zle -N _nudge_toggle_explanation
zle -N _nudge_async_update
zle -N _nudge_overlay_line_pre_redraw
zle -N _nudge_overlay_line_finish

# Bind manual mode hotkey
bindkey '^E' _nudge_complete

# Setup auto mode if enabled
if [[ "$_nudge_auto_mode_enabled" == "true" ]]; then
    # Disable job notifications for background processes
    setopt NO_NOTIFY NO_MONITOR

    # Bind all widgets based on classification
    _nudge_bind_all_widgets

    # Bind Tab to accept suggestion (override default)
    _nudge_bind_widget_to_keymaps '^I' _nudge_auto_accept

    # Bind Right Arrow for progressive word acceptance
    _nudge_bind_accept_navigation_keys
    _nudge_bind_explanation_toggle_keys

    # Clean up on exit
    _nudge_cleanup() {
        _nudge_async_cancel
    }
    zshexit_functions+=(_nudge_cleanup)
elif [[ "$_nudge_overlay_mode_enabled" == "true" ]]; then
    # Keep async requests for slow suggestions without owning ghost text
    setopt NO_NOTIFY NO_MONITOR
    _nudge_setup_overlay_hooks
    _nudge_bind_widget_to_keymaps '^G' _nudge_overlay_accept
    _nudge_bind_explanation_toggle_keys
fi

# Setup diagnosis if enabled
if [[ "$NUDGE_DIAGNOSIS_ENABLED" == "true" ]]; then
    preexec_functions+=(_nudge_diagnosis_preexec)
    # Insert at beginning to capture exit code first
    precmd_functions=(_nudge_diagnosis_precmd "${precmd_functions[@]}")

    # Bind Tab to accept diagnosis suggestion (if not already bound by auto mode)
    if [[ "$_nudge_auto_mode_enabled" != "true" && "$_nudge_ghost_owner_effective" != "autosuggestions" ]]; then
        _nudge_bind_widget_to_keymaps '^I' _nudge_auto_accept
    elif [[ "$_nudge_ghost_owner_effective" == "autosuggestions" ]]; then
        _nudge_bind_widget_to_keymaps '^G' _nudge_overlay_accept
    fi
fi

# Print success message on first load (only in interactive shells)
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    # Only print messages in interactive shells to avoid breaking scp, rsync, etc.
    if [[ $- == *i* ]]; then
        local mode_msg=""
        if [[ "$_nudge_auto_mode_enabled" == "true" ]]; then
            mode_msg="auto mode"
        elif [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
            mode_msg="auto mode (ghost owned by $_nudge_ghost_owner_effective)"
        else
            mode_msg="manual mode (Ctrl+E)"
        fi
        if [[ "$NUDGE_DIAGNOSIS_ENABLED" == "true" ]]; then
            mode_msg="$mode_msg + error diagnosis"
        fi
        echo "Nudge loaded ($mode_msg)."
    fi
fi
