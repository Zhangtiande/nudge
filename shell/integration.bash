#!/usr/bin/env bash
# Nudge - Bash Integration
# Installed by: nudge setup bash

# Get configuration from nudge CLI
NUDGE_CONFIG_DIR=$(nudge info --field config_dir 2>/dev/null)
NUDGE_SOCKET=$(nudge info --field socket_path 2>/dev/null)
NUDGE_TRIGGER_MODE=$(nudge info --field trigger_mode 2>/dev/null)
NUDGE_WARNING_PREFIX="NUDGE_WARNING:"
NUDGE_POPUP_ENABLED="${NUDGE_POPUP_ENABLED:-1}"
NUDGE_POPUP_KEY="${NUDGE_POPUP_KEY:-\e/}"       # Alt+/
NUDGE_POPUP_BACKEND="${NUDGE_POPUP_BACKEND:-auto}" # auto|fzf|sk|peco|builtin
NUDGE_POPUP_CONFIRM_RISKY="${NUDGE_POPUP_CONFIRM_RISKY:-1}"

# Fallback if nudge binary not in PATH
if [[ -z "$NUDGE_CONFIG_DIR" ]]; then
    NUDGE_CONFIG_DIR="$HOME/.nudge"
    NUDGE_SOCKET="$NUDGE_CONFIG_DIR/run/nudge.sock"
    NUDGE_TRIGGER_MODE="manual"
fi

# Lock files for daemon startup
NUDGE_LOCK_FILE="/tmp/nudge.lock"
NUDGE_LOCK_DIR="/tmp/nudge.lockdir"

# Display warning message for dangerous suggestions
_nudge_show_warning() {
    local message="$1"
    echo -e "\033[33mWarning:\033[0m $message"
}

# Capture last exit code before any command
_nudge_last_exit=0
_nudge_capture_exit() {
    _nudge_last_exit=$?
}
PROMPT_COMMAND="_nudge_capture_exit${PROMPT_COMMAND:+; $PROMPT_COMMAND}"

_nudge_start_daemon_locked() {
    if command -v flock >/dev/null 2>&1; then
        (
            flock -n 200 2>/dev/null || exit 0
            nudge start 2>/dev/null
        ) 200>"$NUDGE_LOCK_FILE"
        return
    fi

    # Fallback lock for systems without `flock` (e.g. minimal/macOS environments)
    if mkdir "$NUDGE_LOCK_DIR" 2>/dev/null; then
        nudge start 2>/dev/null
        rmdir "$NUDGE_LOCK_DIR" 2>/dev/null || true
    fi
}

# Ensure daemon is running (lazy load)
# Uses `nudge status` for reliable detection (checks PID file + process alive)
_nudge_ensure_daemon() {
    # Fast path: check if daemon is responding via status command
    if nudge status >/dev/null 2>&1; then
        return 0
    fi

    # Daemon not running, try to start it with lock to prevent concurrent starts
    _nudge_start_daemon_locked
}

# Request completion output in a specific format.
_nudge_request_completion() {
    local format="$1"
    local shell_mode="${2:-bash-inline}"
    _nudge_ensure_daemon

    nudge complete --format "$format" \
        --buffer "$READLINE_LINE" \
        --cursor "$READLINE_POINT" \
        --cwd "$PWD" \
        --session "bash-$$" \
        --shell-mode "$shell_mode" \
        --last-exit-code "$_nudge_last_exit" 2>/dev/null
}

# Main completion function (manual mode)
_nudge_complete() {
    local suggestion
    suggestion=$(_nudge_request_completion plain bash-inline)
    if [[ $? -eq 0 && -n "$suggestion" ]]; then
        if [[ "$suggestion" == ${NUDGE_WARNING_PREFIX}* ]]; then
            local warning_message="${suggestion#${NUDGE_WARNING_PREFIX}}"
            warning_message="${warning_message# }"
            _nudge_show_warning "$warning_message"
            return
        fi
        READLINE_LINE="$suggestion"
        READLINE_POINT=${#READLINE_LINE}
    fi
}

_nudge_resolve_popup_backend() {
    local preferred="$NUDGE_POPUP_BACKEND"
    preferred=$(printf '%s' "$preferred" | tr '[:upper:]' '[:lower:]')

    case "$preferred" in
        fzf|sk|peco|builtin)
            if [[ "$preferred" == "builtin" ]] || command -v "$preferred" >/dev/null 2>&1; then
                echo "$preferred"
                return
            fi
            ;;
        auto|"")
            ;;
        *)
            ;;
    esac

    if command -v fzf >/dev/null 2>&1; then
        echo "fzf"
    elif command -v sk >/dev/null 2>&1; then
        echo "sk"
    elif command -v peco >/dev/null 2>&1; then
        echo "peco"
    else
        echo "builtin"
    fi
}

_nudge_select_candidate_fzf() {
    local candidates="$1"
    printf '%s\n' "$candidates" | fzf \
        --height=40% \
        --layout=reverse \
        --border \
        --header="enter: apply  esc: cancel" \
        --prompt="nudge> " \
        --delimiter=$'\t' \
        --with-nth=1,2,4,3 \
        --preview='printf "risk: %s\n\ncommand:\n%s\n\nwhy: %s\n\ndiff:\n%s\n\nwarning:\n%s\n" {1} {2} {4} {5} {3}' \
        --preview-window='down,60%,wrap'
}

_nudge_select_candidate_sk() {
    local candidates="$1"
    printf '%s\n' "$candidates" | sk \
        --height=40% \
        --prompt="nudge> " \
        --delimiter=$'\t' \
        --with-nth=1,2,4,3 \
        --preview='printf "risk: %s\n\ncommand:\n%s\n\nwhy: %s\n\ndiff:\n%s\n\nwarning:\n%s\n" {1} {2} {4} {5} {3}' \
        --preview-window='down:60%'
}

_nudge_select_candidate_peco() {
    local candidates="$1"
    local -a raw_rows=()
    local -a display_rows=()
    local line
    local idx=1

    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        raw_rows+=("$line")

        local risk command warning why diff
        IFS=$'\t' read -r risk command warning why diff <<< "$line"
        display_rows+=("$idx"$'\t'"[$risk] $command | why: ${why:-n/a}")
        idx=$((idx + 1))
    done <<< "$candidates"

    local selected
    selected=$(printf '%s\n' "${display_rows[@]}" | peco --prompt="nudge> " 2>/dev/null)
    [[ -z "$selected" ]] && return

    local selected_idx="${selected%%$'\t'*}"
    if [[ ! "$selected_idx" =~ ^[0-9]+$ ]]; then
        return
    fi

    local raw_index=$((selected_idx - 1))
    if (( raw_index >= 0 && raw_index < ${#raw_rows[@]} )); then
        printf '%s' "${raw_rows[$raw_index]}"
    fi
}

_nudge_select_candidate_builtin() {
    local candidates="$1"
    local -a lines=()
    local line
    while IFS= read -r line; do
        [[ -n "$line" ]] && lines+=("$line")
    done <<< "$candidates"

    local count="${#lines[@]}"
    [[ "$count" -eq 0 ]] && return

    echo
    local i
    for (( i=0; i<count; i++ )); do
        local risk command warning why diff
        IFS=$'\t' read -r risk command warning why diff <<< "${lines[$i]}"
        printf "  %d) [%s] %s\n" "$((i + 1))" "$risk" "$command"
        [[ -n "$why" ]] && printf "     why: %s\n" "$why"
        [[ -n "$diff" ]] && printf "     diff: %s\n" "$diff"
        [[ -n "$warning" ]] && printf "     warn: %s\n" "$warning"
    done

    local choice
    printf "Select candidate [1-%d, Enter to cancel]: " "$count"
    read -r choice
    echo

    if [[ "$choice" =~ ^[0-9]+$ ]] && (( choice >= 1 && choice <= count )); then
        printf '%s' "${lines[$((choice - 1))]}"
    fi
}

_nudge_popup_complete() {
    local candidates
    candidates=$(_nudge_request_completion list bash-popup)
    if [[ $? -ne 0 || -z "$candidates" ]]; then
        _nudge_complete
        return
    fi

    local backend selected
    backend=$(_nudge_resolve_popup_backend)
    case "$backend" in
        fzf)
            selected=$(_nudge_select_candidate_fzf "$candidates")
            ;;
        sk)
            selected=$(_nudge_select_candidate_sk "$candidates")
            ;;
        peco)
            selected=$(_nudge_select_candidate_peco "$candidates")
            ;;
        *)
            selected=$(_nudge_select_candidate_builtin "$candidates")
            ;;
    esac

    [[ -z "$selected" ]] && return

    local risk command warning why diff
    IFS=$'\t' read -r risk command warning why diff <<< "$selected"
    [[ -z "$command" ]] && return

    if [[ "$risk" == "high" ]]; then
        _nudge_show_warning "${warning:-This suggestion is marked high risk.}"
        if [[ "$NUDGE_POPUP_CONFIRM_RISKY" != "0" ]]; then
            local confirm
            if [[ -r /dev/tty && -w /dev/tty ]]; then
                printf "Apply high-risk suggestion? [y/N] " > /dev/tty
                IFS= read -r confirm < /dev/tty || return
                printf "\n" > /dev/tty
            else
                return
            fi
            case "$confirm" in
                y|Y|yes|YES)
                    ;;
                *)
                    return
                    ;;
            esac
        fi
    fi

    READLINE_LINE="$command"
    READLINE_POINT=${#READLINE_LINE}
    if [[ -n "$warning" ]]; then
        _nudge_show_warning "$warning"
    fi
}

# ============================================================================
# Key Bindings
# ============================================================================

# Bind Ctrl+E hotkey (manual mode)
bind -x '"\C-e": _nudge_complete'
if [[ "$NUDGE_POPUP_ENABLED" != "0" ]]; then
    bind -x "\"${NUDGE_POPUP_KEY}\": _nudge_popup_complete"
fi

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
            if [[ "$NUDGE_POPUP_ENABLED" != "0" ]]; then
                echo "Nudge loaded. Ctrl+E: quick completion | Alt+/: popup selector."
            else
                echo "Nudge loaded. Press Ctrl+E to trigger completion."
            fi
        fi
    fi
fi
