#!/usr/bin/env bash
# Nudge Installation Script
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="${NUDGE_CONFIG_DIR:-$HOME/.config/nudge}"

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

# Create default config if not exists
create_default_config() {
    local config_file="$CONFIG_DIR/config.yaml"

    if [[ -f "$config_file" ]]; then
        echo "Config file already exists: $config_file"
        return 0
    fi

    echo "Creating default config: $config_file"
    cat > "$config_file" << 'EOF'
# Nudge Configuration
# Documentation: https://github.com/user/nudge

model:
  # LLM endpoint (Ollama default)
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
  # api_key_env: "OPENAI_API_KEY"  # Uncomment for OpenAI
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
    depth: standard  # light, standard, or detailed
    recent_commits: 5

trigger:
  mode: manual  # manual or auto
  hotkey: "\\C-e"  # Ctrl+E

privacy:
  sanitize_enabled: true
  custom_patterns: []
  block_dangerous: true
  custom_blocked: []

# system_prompt: "Custom system prompt here..."
EOF
}

# Main installation
main() {
    echo "========================================="
    echo "    Nudge Installation"
    echo "========================================="
    echo ""

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
    create_default_config

    echo ""
    echo "========================================="
    echo "    Installation Complete!"
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
