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
    zle-*
    orig-*
    autosuggest-*
    _nudge_*
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

# Diagnosis preexec - capture stderr before command runs
_nudge_diagnosis_preexec() {
    [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]] && return

    # Clear any pending diagnosis suggestion when user executes a new command
    # This prevents stale suggestions from appearing on next Tab press
    _nudge_auto_suggestion=""

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

# ============================================================================
# Async Suggestion Fetching (No Sleep Debounce)
# ============================================================================

# State for async operations
typeset -g _nudge_async_fd=""
typeset -g _nudge_child_pid=""

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
    fi
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

    # Fork process to fetch suggestion
    builtin exec {_nudge_async_fd}< <(
        # Send PID first for cancellation
        echo $sysparams[pid]

        # Fetch suggestion
        local suggestion
        suggestion=$(nudge complete --format plain \
            --buffer "$current_buffer" \
            --cursor "$CURSOR" \
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

        # Only update if buffer hasn't changed
        if [[ "$BUFFER" == "$_nudge_last_buffer" && -n "$suggestion" ]]; then
            _nudge_auto_suggestion="$suggestion"
            _nudge_auto_display_preview
            zle -R
        fi
    fi

    # Clean up
    builtin exec {fd}<&- 2>/dev/null
    zle -F "$fd" 2>/dev/null
    _nudge_async_fd=""
    _nudge_child_pid=""
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
    region_highlight=("${(@)region_highlight:#*fg=8*}")

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
        region_highlight=("${(@)region_highlight:#*fg=8*}")
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
        # Remove old suggestion highlights, add new one
        region_highlight=("${(@)region_highlight:#*fg=8*}")
        region_highlight+=("$start $end fg=8")
    fi
}

# ============================================================================
# Widget Registration
# ============================================================================

# Register widgets
zle -N _nudge_complete
zle -N _nudge_auto_accept
zle -N _nudge_auto_accept_word

# Bind manual mode hotkey
bindkey '^E' _nudge_complete

# Setup auto mode if enabled
if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
    # Disable job notifications for background processes
    setopt NO_NOTIFY NO_MONITOR

    # Bind all widgets based on classification
    _nudge_bind_all_widgets

    # Bind Tab to accept suggestion (override default)
    bindkey '^I' _nudge_auto_accept

    # Bind Right Arrow to accept word
    bindkey '^[[C' _nudge_auto_accept_word

    # Clean up on exit
    _nudge_cleanup() {
        _nudge_async_cancel
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

# Print success message on first load (only in interactive shells)
if [[ -z "$_NUDGE_LOADED" ]]; then
    export _NUDGE_LOADED=1
    # Only print messages in interactive shells to avoid breaking scp, rsync, etc.
    if [[ $- == *i* ]]; then
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
fi
