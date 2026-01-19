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
    echo "Creating config directory: $CONFIG_DIR/config"
    mkdir -p "$CONFIG_DIR/config"
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

# Interactive configuration wizard
start_config_wizard() {
    echo ""
    echo "========================================="
    echo "    LLM Configuration Wizard"
    echo "========================================="
    echo ""
    echo "Let's configure your LLM settings interactively."
    echo ""

    # Step 1: Choose LLM provider
    echo "1. Which LLM provider do you want to use?"
    echo "   [1] Local Ollama (recommended for privacy, free)"
    echo "   [2] OpenAI (requires API key, paid)"
    echo "   [3] Other OpenAI-compatible API"
    echo "   [4] Skip configuration (I'll configure manually later)"
    echo ""
    read -p "Enter your choice (1-4): " provider_choice

    if [[ "$provider_choice" == "4" ]]; then
        echo "Skipping configuration. You can configure manually later."
        return 1
    fi

    local endpoint=""
    local model_name=""
    local api_key=""
    local api_key_env=""

    case "$provider_choice" in
        1)
            # Local Ollama
            echo ""
            echo "Configuring for local Ollama..."

            endpoint="http://localhost:11434/v1"

            echo ""
            echo "2. Which Ollama model do you want to use?"
            echo "   Common models:"
            echo "   - codellama:7b (fast, good for code)"
            echo "   - deepseek-coder:6.7b (excellent for code)"
            echo "   - qwen2.5-coder:7b (multilingual code support)"
            echo ""
            read -p "Enter model name (press Enter for 'codellama:7b'): " model_input

            if [[ -z "$model_input" ]]; then
                model_name="codellama:7b"
            else
                model_name="$model_input"
            fi

            echo ""
            echo "INFO: Make sure to run 'ollama serve' before using Nudge!"
            ;;
        2)
            # OpenAI
            echo ""
            echo "Configuring for OpenAI..."

            endpoint="https://api.openai.com/v1"

            echo ""
            echo "2. Which OpenAI model do you want to use?"
            echo "   [1] gpt-4o (best quality, expensive)"
            echo "   [2] gpt-4o-mini (good balance)"
            echo "   [3] gpt-3.5-turbo (fastest, cheapest)"
            echo ""
            read -p "Enter your choice (1-3): " model_choice

            case "$model_choice" in
                1) model_name="gpt-4o" ;;
                2) model_name="gpt-4o-mini" ;;
                3) model_name="gpt-3.5-turbo" ;;
                *) model_name="gpt-4o-mini" ;;
            esac

            echo ""
            echo "3. How do you want to provide your API key?"
            echo "   [1] Environment variable (recommended for security)"
            echo "   [2] Direct in config file (convenient but less secure)"
            echo ""
            read -p "Enter your choice (1-2): " key_choice

            if [[ "$key_choice" == "2" ]]; then
                echo ""
                read -sp "Enter your OpenAI API key (sk-...): " api_key
                echo ""
            else
                api_key_env="OPENAI_API_KEY"
                echo ""
                echo "WARNING: Please set the OPENAI_API_KEY environment variable with your API key"
                echo "Example: export OPENAI_API_KEY='sk-your-api-key-here'"
                echo "Add this to your ~/.bashrc or ~/.zshrc to make it permanent"
            fi
            ;;
        3)
            # Custom OpenAI-compatible API
            echo ""
            echo "Configuring for custom OpenAI-compatible API..."

            echo ""
            read -p "Enter API endpoint URL (e.g., https://api.example.com/v1): " endpoint

            echo ""
            read -p "Enter model name: " model_name

            echo ""
            read -p "Does this API require an API key? (y/N): " requires_key

            if [[ "$requires_key" =~ ^[Yy]$ ]]; then
                echo ""
                echo "How do you want to provide your API key?"
                echo "   [1] Environment variable (recommended)"
                echo "   [2] Direct in config file"
                echo ""
                read -p "Enter your choice (1-2): " key_choice

                if [[ "$key_choice" == "2" ]]; then
                    echo ""
                    read -sp "Enter your API key: " api_key
                    echo ""
                else
                    echo ""
                    read -p "Enter environment variable name (e.g., MY_API_KEY): " api_key_env
                    echo ""
                    echo "WARNING: Please set the $api_key_env environment variable with your API key"
                fi
            fi
            ;;
        *)
            echo "WARNING: Invalid choice. Using default Ollama configuration."
            endpoint="http://localhost:11434/v1"
            model_name="codellama:7b"
            ;;
    esac

    # Export for use in create_config_from_wizard
    export WIZARD_ENDPOINT="$endpoint"
    export WIZARD_MODEL_NAME="$model_name"
    export WIZARD_API_KEY="$api_key"
    export WIZARD_API_KEY_ENV="$api_key_env"

    return 0
}

# Create minimal user config from wizard results (overrides config.default.yaml)
create_config_from_wizard() {
    local config_file="$1"

    local api_key_line=""
    if [[ -n "$WIZARD_API_KEY" ]]; then
        api_key_line="  api_key: \"$WIZARD_API_KEY\""
    elif [[ -n "$WIZARD_API_KEY_ENV" ]]; then
        api_key_line="  api_key_env: \"$WIZARD_API_KEY_ENV\""
    fi

    # Generate minimal user config (overrides config.default.yaml)
    cat > "$config_file" << EOF
# Nudge User Configuration
# Generated by installation wizard
#
# Your customizations here override config.default.yaml
# This file is preserved across upgrades.

model:
  endpoint: "$WIZARD_ENDPOINT"
  model_name: "$WIZARD_MODEL_NAME"
$api_key_line
EOF

    echo "User configuration created: $config_file"
}

# Setup configuration files with layered approach
setup_config_files() {
    local config_subdir="$CONFIG_DIR/config"
    local default_config="$config_subdir/config.default.yaml"
    local user_config="$config_subdir/config.yaml"
    local template_default="$SCRIPT_DIR/../config/config.yaml.template"
    local template_user="$SCRIPT_DIR/../config/config.user.yaml.template"

    # Always update config.default.yaml (ships with app, updated on upgrade)
    echo "Updating default config: $default_config"
    if [[ -f "$template_default" ]]; then
        cp "$template_default" "$default_config"
        echo "Default config updated from template"
    else
        echo "Warning: Default config template not found"
    fi

    # Create user config only if it doesn't exist (preserve user customizations)
    if [[ -f "$user_config" ]]; then
        echo "User config preserved: $user_config"
        return 0
    fi

    # Run interactive configuration wizard
    if start_config_wizard; then
        create_config_from_wizard "$user_config"
        echo ""
        echo "Configuration completed!"
    else
        # User skipped wizard, create minimal template
        echo "Creating minimal user config: $user_config"
        if [[ -f "$template_user" ]]; then
            cp "$template_user" "$user_config"
            echo "User config created from template"
        else
            # Fallback: create minimal user config inline
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
        echo "Edit $user_config to customize your LLM settings"
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
