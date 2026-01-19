#!/usr/bin/env bash
# Nudge Shell Integration Setup Script
# This script sets up shell integration for Nudge.
# It assumes that the 'nudge' binary is already installed and in PATH.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Detect platform and get config directory
get_config_dir() {
    case "$(uname -s)" in
        Darwin)
            echo "$HOME/Library/Application Support/nudge"
            ;;
        *)
            echo "${XDG_CONFIG_HOME:-$HOME/.config}/nudge"
            ;;
    esac
}

CONFIG_DIR="${NUDGE_CONFIG_DIR:-$(get_config_dir)}"

# Detect shell type
detect_shell() {
    local shell_name
    shell_name=$(basename "$SHELL")
    echo "$shell_name"
}

# Get RC file for shell
get_rc_file() {
    local shell_name="$1"
    case "$shell_name" in
        bash)
            echo "$HOME/.bashrc"
            ;;
        zsh)
            echo "$HOME/.zshrc"
            ;;
        *)
            echo ""
            ;;
    esac
}

# Create config directory
setup_config_dir() {
    echo "Creating config directory: $CONFIG_DIR"
    mkdir -p "$CONFIG_DIR"
}

# Copy integration script
copy_integration() {
    local shell_name="$1"
    local source_file="$SCRIPT_DIR/integration.$shell_name"
    local dest_file="$CONFIG_DIR/integration.$shell_name"

    if [[ -f "$source_file" ]]; then
        echo "Installing integration script: $dest_file"
        cp "$source_file" "$dest_file"
        chmod 644 "$dest_file"
    else
        echo "Warning: Integration script not found: $source_file"
        return 1
    fi
}

# Add source line to RC file
add_source_line() {
    local rc_file="$1"
    local integration_file="$2"
    local source_line="# Nudge integration"
    local source_cmd="[ -f \"$integration_file\" ] && source \"$integration_file\""

    if [[ ! -f "$rc_file" ]]; then
        echo "Creating RC file: $rc_file"
        touch "$rc_file"
    fi

    # Check if already installed
    if grep -q "Nudge integration" "$rc_file" 2>/dev/null; then
        echo "Nudge is already configured in $rc_file"
        return 0
    fi

    echo "Adding Nudge to $rc_file"
    {
        echo ""
        echo "$source_line"
        echo "$source_cmd"
    } >> "$rc_file"
}

# Setup configuration files with layered approach:
# - config.default.yaml: Full default config (updated on every install/upgrade)
# - config.yaml: User customizations only (preserved across upgrades)
setup_config_files() {
    local default_config="$CONFIG_DIR/config.default.yaml"
    local user_config="$CONFIG_DIR/config.yaml"
    local default_template="$SCRIPT_DIR/../config/config.yaml.template"
    local user_template="$SCRIPT_DIR/../config/config.user.yaml.template"

    # Always update config.default.yaml (ships with app)
    echo "Updating default config: $default_config"
    if [[ -f "$default_template" ]]; then
        cp "$default_template" "$default_config"
        echo "Default config updated from template"
    else
        # Fallback: create basic default config inline
        cat > "$default_config" << 'EOF'
# Nudge Default Configuration
# DO NOT EDIT - This file is overwritten on upgrades.
# Put your customizations in config.yaml instead.

model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
  timeout_ms: 5000

context:
  history_window: 20
  include_cwd_listing: true
  include_exit_code: true
  max_files_in_listing: 50
  max_total_tokens: 4000
  priorities:
    history: 80
    cwd_listing: 60
    plugins: 40

plugins:
  git:
    enabled: true
    depth: standard
    recent_commits: 5

trigger:
  mode: manual
  hotkey: "\\C-e"

privacy:
  sanitize_enabled: true
  custom_patterns: []
  block_dangerous: true
  custom_blocked: []

log:
  level: "info"
  file_enabled: false
EOF
        echo "Default config created with built-in settings"
    fi

    # Create user config only if it doesn't exist (preserve user customizations)
    if [[ -f "$user_config" ]]; then
        echo "User config preserved: $user_config"
    else
        echo "Creating user config: $user_config"
        if [[ -f "$user_template" ]]; then
            cp "$user_template" "$user_config"
            echo "User config created from template"
        else
            # Fallback: create minimal user config
            cat > "$user_config" << 'EOF'
# Nudge User Configuration
#
# Add your custom settings here. They will override config.default.yaml.
# This file is preserved across upgrades.
#
# Example - To use OpenAI instead of local Ollama:
#
# model:
#   endpoint: "https://api.openai.com/v1"
#   model_name: "gpt-3.5-turbo"
#   api_key_env: "OPENAI_API_KEY"
EOF
            echo "User config created with minimal template"
        fi
    fi
}

# Main installation
main() {
    echo "========================================="
    echo "    Nudge Shell Integration Setup"
    echo "========================================="
    echo ""

    # Check if nudge binary is available
    if ! command -v nudge &> /dev/null; then
        echo "Warning: 'nudge' command not found in PATH."
        echo "Make sure the nudge binary is installed and in your PATH."
        echo ""
    fi

    local shell_name
    shell_name=$(detect_shell)
    echo "Detected shell: $shell_name"

    local rc_file
    rc_file=$(get_rc_file "$shell_name")

    if [[ -z "$rc_file" ]]; then
        echo "Error: Unsupported shell: $shell_name"
        echo "Supported shells: bash, zsh"
        exit 1
    fi

    setup_config_dir
    copy_integration "$shell_name"
    add_source_line "$rc_file" "$CONFIG_DIR/integration.$shell_name"
    setup_config_files

    echo ""
    echo "========================================="
    echo "    Shell Integration Complete!"
    echo "========================================="
    echo ""
    echo "To activate Nudge, either:"
    echo "  1. Open a new terminal"
    echo "  2. Run: source $rc_file"
    echo ""
    echo "Make sure Ollama is running for local LLM:"
    echo "  ollama serve"
    echo ""
    echo "Then press Ctrl+E to trigger completion!"
}

main "$@"
