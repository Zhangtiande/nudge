#!/usr/bin/env bash
# Nudge Uninstallation Script
set -e

# Detect platform and get config directory
get_config_dir() {
    echo "$HOME/.nudge"
}

CONFIG_DIR="${NUDGE_CONFIG_DIR:-$(get_config_dir)}"

# Remove source line from RC file
remove_source_line() {
    local rc_file="$1"

    if [[ ! -f "$rc_file" ]]; then
        return 0
    fi

    if grep -q "Nudge integration" "$rc_file" 2>/dev/null; then
        echo "Removing Nudge from $rc_file"
        # Create backup
        cp "$rc_file" "$rc_file.bak"
        # Remove Nudge lines
        grep -v "Nudge integration\|nudge/integration" "$rc_file.bak" > "$rc_file" || true
        # Remove trailing empty lines
        sed -i.tmp -e :a -e '/^\s*$/{ $d; N; ba; }' "$rc_file" 2>/dev/null || true
        rm -f "$rc_file.tmp"
    fi
}

# Stop daemon if running
stop_daemon() {
    if command -v nudge &>/dev/null; then
        echo "Stopping Nudge daemon..."
        nudge stop 2>/dev/null || true
    fi
}

# Main uninstallation
main() {
    local keep_config=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --keep-config)
                keep_config=true
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    echo "========================================="
    echo "    Nudge Uninstallation"
    echo "========================================="
    echo ""

    stop_daemon

    # Remove from RC files
    remove_source_line "$HOME/.bashrc"
    remove_source_line "$HOME/.zshrc"

    # Remove config directory
    if [[ "$keep_config" == "false" && -d "$CONFIG_DIR" ]]; then
        echo "Removing config directory: $CONFIG_DIR"
        rm -rf "$CONFIG_DIR"
    elif [[ "$keep_config" == "true" ]]; then
        echo "Keeping config directory: $CONFIG_DIR"
    fi

    echo ""
    echo "========================================="
    echo "    Uninstallation Complete!"
    echo "========================================="
    echo ""
    echo "Nudge has been removed."
    echo "Open a new terminal or run: source ~/.bashrc (or ~/.zshrc)"
}

main "$@"
