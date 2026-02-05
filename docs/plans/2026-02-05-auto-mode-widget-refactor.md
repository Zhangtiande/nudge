# Auto Mode Widget-Based Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor Zsh auto-completion mode to use widget wrapping instead of `zle-line-pre-redraw`, eliminating terminal freezing and arrow key issues.

**Architecture:** Adopt zsh-autosuggestions' proven architecture: wrap individual widgets (self-insert, delete-char, etc.) instead of hooking into the global redraw event. Use `$PENDING` and `$KEYS_QUEUED_COUNT` for input queue detection instead of `sleep` process debouncing. Classify widgets into modify/clear/accept/partial_accept/ignore categories.

**Tech Stack:** Zsh ZLE (Zsh Line Editor), shell scripting

---

## Background

### Current Problems
1. **Terminal freezing**: `zle-line-pre-redraw` triggers on every redraw, creating `sleep` processes for debouncing
2. **Arrow key issues**: History navigation (up/down) triggers auto-completion instead of just clearing suggestions
3. **Resource waste**: Each keystroke spawns a new background process

### Solution: Widget Wrapping
Instead of hooking `zle-line-pre-redraw`, wrap individual widgets and classify them:
- **modify**: Widgets that change buffer content → fetch new suggestion
- **clear**: History navigation widgets → only clear suggestion
- **accept**: End-of-line widgets → accept entire suggestion
- **partial_accept**: Forward-word widgets → accept partial suggestion
- **ignore**: Internal widgets → do nothing

---

## Task 1: Add Widget Classification Configuration

**Files:**
- Modify: `shell/integration.zsh:28-33` (auto mode state section)

**Step 1: Add widget classification arrays after existing state variables**

Add these arrays after line 33 (`typeset -g _nudge_pending_buffer=""`):

```zsh
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
```

**Step 2: Verify syntax**

Run: `zsh -n shell/integration.zsh`
Expected: No output (no syntax errors)

**Step 3: Commit**

```bash
git add shell/integration.zsh
git commit -m "feat(auto-mode): add widget classification arrays"
```

---

## Task 2: Implement Widget Wrapping Infrastructure

**Files:**
- Modify: `shell/integration.zsh` (add new functions before Widget Registration section, around line 424)

**Step 1: Add widget binding helper functions**

Insert before the `# Widget Registration` section (before line 424):

```zsh
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
```

**Step 2: Verify syntax**

Run: `zsh -n shell/integration.zsh`
Expected: No output (no syntax errors)

**Step 3: Commit**

```bash
git add shell/integration.zsh
git commit -m "feat(auto-mode): add widget wrapping infrastructure"
```

---

## Task 3: Implement Widget Action Handlers

**Files:**
- Modify: `shell/integration.zsh` (add after widget wrapping infrastructure)

**Step 1: Add the four action handler functions**

Insert after `_nudge_bind_all_widgets` function:

```zsh
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
```

**Step 2: Verify syntax**

Run: `zsh -n shell/integration.zsh`
Expected: No output (no syntax errors)

**Step 3: Commit**

```bash
git add shell/integration.zsh
git commit -m "feat(auto-mode): implement widget action handlers"
```

---

## Task 4: Implement Async Fetch Without Sleep Debounce

**Files:**
- Modify: `shell/integration.zsh` (replace `_nudge_auto_trigger` and related functions)

**Step 1: Replace the old debounce mechanism with async fetch**

Replace the functions `_nudge_auto_trigger`, `_nudge_auto_on_timer_ready`, `_nudge_auto_update_display`, and `_nudge_auto_fetch` (lines 341-409) with:

```zsh
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
```

**Step 2: Verify syntax**

Run: `zsh -n shell/integration.zsh`
Expected: No output (no syntax errors)

**Step 3: Commit**

```bash
git add shell/integration.zsh
git commit -m "feat(auto-mode): replace sleep debounce with async fetch"
```

---

## Task 5: Update Auto Mode Setup

**Files:**
- Modify: `shell/integration.zsh` (update the auto mode setup section, lines 437-460)

**Step 1: Replace the old auto mode setup with widget-based approach**

Replace the auto mode setup section (starting at `if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then`) with:

```zsh
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
```

**Step 2: Remove the old `_nudge_auto_line_change` function and `zle-line-pre-redraw` hook**

Delete the following code (around lines 411-422):

```zsh
# Hook into line editing (called on every buffer change)
_nudge_auto_line_change() {
    if [[ "$NUDGE_TRIGGER_MODE" == "auto" ]]; then
        # Clear preview if buffer changed
        if [[ "$BUFFER" != "$_nudge_last_buffer" ]]; then
            typeset -g POSTDISPLAY=""
            # Clear region_highlight
            region_highlight=(\"${(@)region_highlight:#*}\")
            _nudge_auto_trigger
        fi
    fi
}
```

And remove the `zle-line-pre-redraw` registration:

```zsh
    _nudge_zle_line_pre_redraw() {
        _nudge_auto_line_change
    }
    zle -N zle-line-pre-redraw _nudge_zle_line_pre_redraw
```

**Step 3: Verify syntax**

Run: `zsh -n shell/integration.zsh`
Expected: No output (no syntax errors)

**Step 4: Commit**

```bash
git add shell/integration.zsh
git commit -m "feat(auto-mode): switch to widget-based triggering"
```

---

## Task 6: Update Cancel Function and Cleanup

**Files:**
- Modify: `shell/integration.zsh`

**Step 1: Update `_nudge_auto_cancel` to use new async cancel**

Replace the old `_nudge_auto_cancel` function (lines 240-251) with:

```zsh
# Cancel any pending auto completion (wrapper for compatibility)
_nudge_auto_cancel() {
    _nudge_async_cancel
    _nudge_pending_buffer=""
    _nudge_auto_suggestion=""
    POSTDISPLAY=""
}
```

**Step 2: Remove old timer-related state variables**

Remove or comment out these lines (around line 30):

```zsh
typeset -g _nudge_timer_fd=""
```

**Step 3: Verify syntax**

Run: `zsh -n shell/integration.zsh`
Expected: No output (no syntax errors)

**Step 4: Commit**

```bash
git add shell/integration.zsh
git commit -m "refactor(auto-mode): cleanup old timer-based code"
```

---

## Task 7: Manual Testing

**Files:** None (testing only)

**Step 1: Reload shell integration**

```bash
source ~/.zshrc
# Or restart terminal
```

**Step 2: Test basic auto-completion**

1. Type `git st` slowly - should see suggestion appear
2. Type `git st` quickly - should not freeze
3. Press Tab - should accept suggestion

Expected: Suggestions appear without terminal freezing

**Step 3: Test history navigation**

1. Type `git` and wait for suggestion
2. Press Up arrow - suggestion should clear, history should work
3. Press Down arrow - should navigate history normally

Expected: Arrow keys work normally, suggestions clear on history navigation

**Step 4: Test fast typing**

1. Type `echo hello world` as fast as possible
2. Should not see any lag or freezing

Expected: No freezing, smooth typing experience

**Step 5: Test suggestion truncation optimization**

1. Type `git c` - wait for suggestion like `git commit -m "..."`
2. Continue typing `o` to make `git co`
3. Suggestion should update instantly (truncated, not re-fetched)

Expected: Instant update when typing matches suggestion

**Step 6: Document test results**

Create a test log noting any issues found.

---

## Task 8: Add Debug Logging (Optional)

**Files:**
- Modify: `shell/integration.zsh`

**Step 1: Add debug mode toggle**

Add near the top of the file:

```zsh
# Debug mode (set NUDGE_DEBUG=1 to enable)
NUDGE_DEBUG=${NUDGE_DEBUG:-0}

_nudge_debug() {
    (( NUDGE_DEBUG )) && echo "[nudge] $*" >> /tmp/nudge-debug.log
}
```

**Step 2: Add debug calls to key functions**

In `_nudge_widget_modify`:
```zsh
_nudge_debug "modify: widget=$orig_widget buffer=$BUFFER pending=$PENDING"
```

In `_nudge_fetch_async`:
```zsh
_nudge_debug "fetch_async: buffer=$BUFFER"
```

In `_nudge_async_response`:
```zsh
_nudge_debug "async_response: suggestion=${suggestion:0:50}..."
```

**Step 3: Commit**

```bash
git add shell/integration.zsh
git commit -m "feat(auto-mode): add optional debug logging"
```

---

## Summary

After completing all tasks, the auto mode will:

1. ✅ **No more terminal freezing** - No `sleep` processes, uses `$PENDING` detection
2. ✅ **Arrow keys work correctly** - History widgets only clear suggestions
3. ✅ **Instant suggestion truncation** - When typing matches suggestion
4. ✅ **Proper async cancellation** - Kill child process when new request starts
5. ✅ **Resource efficient** - Only one async process at a time

### Architecture Comparison

| Aspect | Before | After |
|--------|--------|-------|
| Trigger | `zle-line-pre-redraw` (every redraw) | Widget wrapping (specific widgets) |
| Debounce | `sleep` process + fd | `$PENDING` + `$KEYS_QUEUED_COUNT` |
| History nav | Triggers new fetch | Only clears suggestion |
| Fast typing | Creates many processes | Skips fetch, keeps typing |
| Suggestion match | Always re-fetch | Truncate existing |
