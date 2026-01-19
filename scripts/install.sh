#!/usr/bin/env bash
# Nudge One-Click Installation Script for Unix/Linux/macOS
#
# This script automatically downloads and installs Nudge from GitHub Releases.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
#
# Or with options:
#   curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --version 0.1.0
#   curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --prefix ~/.local
#   curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --skip-shell

set -e

# Configuration
GITHUB_REPO="Zhangtiande/nudge"
VERSION=""
INSTALL_PREFIX=""
SKIP_SHELL=false
UNINSTALL=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print colored output
info() {
    echo -e "${CYAN}INFO:${NC} $1"
}

success() {
    echo -e "${GREEN}SUCCESS:${NC} $1"
}

warning() {
    echo -e "${YELLOW}WARNING:${NC} $1"
}

error() {
    echo -e "${RED}ERROR:${NC} $1" >&2
}

# Detect OS and architecture
detect_platform() {
    local os arch

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin)
            OS="macos"
            ;;
        Linux)
            OS="linux"
            ;;
        *)
            error "Unsupported operating system: $os"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    info "Detected platform: $OS-$ARCH"
}

# Get latest version from GitHub API
get_latest_version() {
    info "Fetching latest version from GitHub..."

    local api_url="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
    local response

    if command -v curl &> /dev/null; then
        response=$(curl -fsSL "$api_url")
    elif command -v wget &> /dev/null; then
        response=$(wget -qO- "$api_url")
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    VERSION=$(echo "$response" | grep '"tag_name"' | sed -E 's/.*"v?([^"]+)".*/\1/')

    if [[ -z "$VERSION" ]]; then
        error "Failed to fetch latest version"
        exit 1
    fi

    success "Latest version: $VERSION"
}

# Download and extract binary
download_and_extract() {
    local download_url filename tmpdir

    # Determine the appropriate binary name
    # Try linux-x86_64-musl for Linux first (static binary), fall back to dynamic
    if [[ "$OS" == "linux" ]]; then
        filename="nudge-${OS}-${ARCH}-musl.tar.gz"
    else
        filename="nudge-${OS}-${ARCH}.tar.gz"
    fi

    download_url="https://github.com/$GITHUB_REPO/releases/download/v${VERSION}/${filename}"

    info "Downloading from: $download_url"

    tmpdir=$(mktemp -d)
    trap "rm -rf $tmpdir" EXIT

    if command -v curl &> /dev/null; then
        if ! curl -fsSL "$download_url" -o "$tmpdir/$filename" 2>/dev/null; then
            # Fall back to dynamic build if musl not available
            if [[ "$OS" == "linux" ]]; then
                warning "Static build not available, trying dynamic build..."
                filename="nudge-${OS}-${ARCH}.tar.gz"
                download_url="https://github.com/$GITHUB_REPO/releases/download/v${VERSION}/${filename}"
                curl -fsSL "$download_url" -o "$tmpdir/$filename"
            else
                error "Download failed"
                exit 1
            fi
        fi
    elif command -v wget &> /dev/null; then
        if ! wget -q "$download_url" -O "$tmpdir/$filename" 2>/dev/null; then
            # Fall back to dynamic build if musl not available
            if [[ "$OS" == "linux" ]]; then
                warning "Static build not available, trying dynamic build..."
                filename="nudge-${OS}-${ARCH}.tar.gz"
                download_url="https://github.com/$GITHUB_REPO/releases/download/v${VERSION}/${filename}"
                wget -q "$download_url" -O "$tmpdir/$filename"
            else
                error "Download failed"
                exit 1
            fi
        fi
    fi

    success "Downloaded: $filename"

    info "Extracting archive..."
    tar -xzf "$tmpdir/$filename" -C "$tmpdir"

    BINARY_PATH="$tmpdir/nudge"
    if [[ ! -f "$BINARY_PATH" ]]; then
        error "Binary not found in archive"
        exit 1
    fi
}

# Choose installation location
choose_install_location() {
    if [[ -n "$INSTALL_PREFIX" ]]; then
        INSTALL_DIR="$INSTALL_PREFIX/bin"
        return
    fi

    echo ""
    echo "Choose installation location:"
    echo "  1) /usr/local/bin (system-wide, requires sudo)"
    echo "  2) ~/.local/bin (user-only, recommended)"
    echo ""

    while true; do
        read -p "Enter choice [1-2] (default: 2): " choice < /dev/tty
        choice=${choice:-2}

        case "$choice" in
            1)
                INSTALL_DIR="/usr/local/bin"
                NEED_SUDO=true
                break
                ;;
            2)
                INSTALL_DIR="$HOME/.local/bin"
                NEED_SUDO=false
                break
                ;;
            *)
                echo "Invalid choice. Please enter 1 or 2."
                ;;
        esac
    done

    info "Installation directory: $INSTALL_DIR"
}

# Install binary
install_binary() {
    info "Installing nudge to $INSTALL_DIR..."

    # Create directory if needed
    if [[ ! -d "$INSTALL_DIR" ]]; then
        if [[ "$NEED_SUDO" == true ]]; then
            sudo mkdir -p "$INSTALL_DIR"
        else
            mkdir -p "$INSTALL_DIR"
        fi
    fi

    # Copy binary
    if [[ "$NEED_SUDO" == true ]]; then
        sudo cp "$BINARY_PATH" "$INSTALL_DIR/nudge"
        sudo chmod +x "$INSTALL_DIR/nudge"
    else
        cp "$BINARY_PATH" "$INSTALL_DIR/nudge"
        chmod +x "$INSTALL_DIR/nudge"
    fi

    success "Binary installed: $INSTALL_DIR/nudge"

    # Check if install dir is in PATH
    if [[ "$INSTALL_DIR" == "$HOME/.local/bin" ]]; then
        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            warning "$HOME/.local/bin is not in your PATH"
            echo "Add this line to your ~/.bashrc or ~/.zshrc:"
            echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
            echo ""
        fi
    fi
}

# Setup shell integration
setup_shell_integration() {
    if [[ "$SKIP_SHELL" == true ]]; then
        info "Skipping shell integration (--skip-shell flag)"
        return
    fi

    echo ""
    info "Setting up shell integration..."

    # Determine where the setup script is
    local setup_script=""

    # Check if running from repository
    if [[ -f "$(dirname "$0")/../shell/setup-shell.sh" ]]; then
        setup_script="$(cd "$(dirname "$0")/../shell" && pwd)/setup-shell.sh"
    # Check if running from a cloned repo
    elif [[ -f "./shell/setup-shell.sh" ]]; then
        setup_script="$(cd "./shell" && pwd)/setup-shell.sh"
    else
        # Download shell integration files
        local tmpdir
        tmpdir=$(mktemp -d)
        trap "rm -rf $tmpdir" EXIT

        info "Downloading shell integration files..."

        local base_url="https://raw.githubusercontent.com/$GITHUB_REPO/main"

        # Create directory structure matching repository layout
        # setup-shell.sh expects: $SCRIPT_DIR/../config/config.yaml.template
        mkdir -p "$tmpdir/shell"
        mkdir -p "$tmpdir/config"

        for file in setup-shell.sh integration.bash integration.zsh; do
            local url="$base_url/shell/$file"
            if command -v curl &> /dev/null; then
                curl -fsSL "$url" -o "$tmpdir/shell/$file"
            else
                wget -q "$url" -O "$tmpdir/shell/$file"
            fi
        done

        # Download config template to proper location
        if command -v curl &> /dev/null; then
            curl -fsSL "$base_url/config/config.yaml.template" -o "$tmpdir/config/config.yaml.template"
        else
            wget -q "$base_url/config/config.yaml.template" -O "$tmpdir/config/config.yaml.template"
        fi

        setup_script="$tmpdir/shell/setup-shell.sh"
        chmod +x "$setup_script"
    fi

    if [[ -f "$setup_script" ]]; then
        bash "$setup_script"
    else
        warning "Shell setup script not found. You'll need to set up shell integration manually."
        echo "See: https://github.com/$GITHUB_REPO#installation"
    fi
}

# Uninstall nudge
uninstall() {
    echo ""
    echo "========================================="
    echo "    Uninstalling Nudge"
    echo "========================================="
    echo ""

    # Remove binary
    local removed=false

    for bindir in /usr/local/bin "$HOME/.local/bin"; do
        if [[ -f "$bindir/nudge" ]]; then
            info "Removing binary: $bindir/nudge"
            if [[ "$bindir" == "/usr/local/bin" ]]; then
                sudo rm "$bindir/nudge"
            else
                rm "$bindir/nudge"
            fi
            removed=true
        fi
    done

    if [[ "$removed" == false ]]; then
        warning "Nudge binary not found"
    else
        success "Binary removed"
    fi

    # Remove shell integration
    for shell_rc in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [[ -f "$shell_rc" ]] && grep -q "Nudge integration" "$shell_rc"; then
            info "Removing integration from $shell_rc"
            sed -i.bak '/# Nudge integration/,+1d' "$shell_rc"
        fi
    done

    echo ""
    warning "Configuration files in ~/.config/nudge (or ~/Library/Application Support/nudge) were not removed."
    echo "To remove them manually, run:"
    if [[ "$(uname -s)" == "Darwin" ]]; then
        echo "  rm -rf ~/Library/Application\\ Support/nudge"
    else
        echo "  rm -rf ~/.config/nudge"
    fi

    echo ""
    success "Uninstallation complete!"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --prefix)
                INSTALL_PREFIX="$2"
                shift 2
                ;;
            --skip-shell)
                SKIP_SHELL=true
                shift
                ;;
            --uninstall)
                UNINSTALL=true
                shift
                ;;
            --help)
                echo "Nudge Installation Script"
                echo ""
                echo "Usage: $0 [options]"
                echo ""
                echo "Options:"
                echo "  --version VERSION    Install specific version (default: latest)"
                echo "  --prefix PATH        Install to PATH/bin (default: interactive)"
                echo "  --skip-shell         Skip shell integration setup"
                echo "  --uninstall          Remove Nudge"
                echo "  --help               Show this help message"
                exit 0
                ;;
            *)
                error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
}

# Main installation
main() {
    echo ""
    echo "========================================="
    echo "    Nudge Installation"
    echo "========================================="
    echo ""

    parse_args "$@"

    if [[ "$UNINSTALL" == true ]]; then
        uninstall
        exit 0
    fi

    detect_platform

    if [[ -z "$VERSION" ]]; then
        get_latest_version
    else
        info "Using specified version: $VERSION"
    fi

    download_and_extract
    choose_install_location
    install_binary
    setup_shell_integration

    echo ""
    echo "========================================="
    echo "    Installation Complete!"
    echo "========================================="
    echo ""
    success "Nudge $VERSION has been installed successfully!"
    echo ""
    
    # Determine config file location
    local config_file=""
    if [[ "$(uname -s)" == "Darwin" ]]; then
        config_file="$HOME/Library/Application Support/nudge/config.yaml"
    else
        config_file="${XDG_CONFIG_HOME:-$HOME/.config}/nudge/config.yaml"
    fi
    
    echo "========================================="
    echo "    Configuration Required"
    echo "========================================="
    echo ""
    warning "Please configure your LLM settings before using Nudge!"
    echo ""
    echo "Configuration file location:"
    echo "  $config_file"
    echo ""
    echo "You need to edit the following settings:"
    echo ""
    echo "  model:"
    echo "    endpoint: \"http://localhost:11434/v1\"  # Change if using different LLM"
    echo "    model_name: \"codellama:7b\"              # Change to your preferred model"
    echo "    # api_key: \"sk-xxx\"                     # Direct API key (option 1)"
    echo "    # api_key_env: \"OPENAI_API_KEY\"         # Or use env variable (option 2)"
    echo ""
    echo "Example configurations:"
    echo ""
    echo "  # Local Ollama (default):"
    echo "  model:"
    echo "    endpoint: \"http://localhost:11434/v1\""
    echo "    model_name: \"codellama:7b\""
    echo ""
    echo "  # OpenAI (with direct API key):"
    echo "  model:"
    echo "    endpoint: \"https://api.openai.com/v1\""
    echo "    model_name: \"gpt-3.5-turbo\""
    echo "    api_key: \"sk-your-api-key-here\""
    echo ""
    echo "  # OpenAI (with env variable, recommended for security):"
    echo "  model:"
    echo "    endpoint: \"https://api.openai.com/v1\""
    echo "    model_name: \"gpt-3.5-turbo\""
    echo "    api_key_env: \"OPENAI_API_KEY\""
    echo ""
    echo "To edit the config file, run:"
    echo "  ${EDITOR:-nano} \"$config_file\""
    echo ""
    echo "Next steps:"
    echo "  1. Edit the configuration file above"
    echo "  2. Open a new terminal or run: source ~/.bashrc (or ~/.zshrc)"
    echo "  3. Start Ollama if using local LLM: ollama serve"
    echo "  4. Press Ctrl+E in your terminal to trigger AI completion"
    echo ""
    info "For more information, visit: https://github.com/$GITHUB_REPO"
    echo ""
}

main "$@"
