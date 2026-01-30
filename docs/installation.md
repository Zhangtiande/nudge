# Installation Guide

## Quick Install (Recommended)

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

The installer will:
1. Download the latest release binary
2. Install to `~/.local/bin` (Unix) or `%LOCALAPPDATA%\nudge\bin` (Windows)
3. Add to PATH
4. Configure shell integration
5. Start the daemon

## Installation Options

```bash
# Specify version
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --version 0.4.0

# Custom install location
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --prefix /opt/nudge

# Skip shell integration
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --skip-shell

# Uninstall
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --uninstall
```

## Manual Installation

### Download Binary

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 (glibc) | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz) |
| Linux | x86_64 (musl) | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64-musl.tar.gz) |
| Linux | aarch64 | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-aarch64.tar.gz) |
| macOS | x86_64 (Intel) | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-x86_64.tar.gz) |
| macOS | aarch64 (Apple Silicon) | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-macos-aarch64.tar.gz) |
| Windows | x86_64 | [Download](https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-windows-x86_64.zip) |

### Install Steps

**Linux/macOS:**
```bash
# Download
curl -LO https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz

# Extract
tar xzf nudge-linux-x86_64.tar.gz

# Install
sudo mv nudge /usr/local/bin/
# Or user-only:
mkdir -p ~/.local/bin && mv nudge ~/.local/bin/

# Setup shell
nudge setup
```

**Windows:**
```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-windows-x86_64.zip" -OutFile nudge.zip
Expand-Archive nudge.zip -DestinationPath .

# Install
New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\nudge\bin"
Move-Item nudge.exe "$env:LOCALAPPDATA\nudge\bin\"

# Setup shell
nudge setup
```

## Build from Source

**Prerequisites:** Rust 1.70+

```bash
git clone https://github.com/Zhangtiande/nudge.git
cd nudge
cargo build --release
sudo cp target/release/nudge /usr/local/bin/
nudge setup
```

## Shell Integration

### Automatic

```bash
nudge setup
```

### Manual

**Bash** (`~/.bashrc`):
```bash
[ -f "$HOME/.config/nudge/shell/integration.bash" ] && source "$HOME/.config/nudge/shell/integration.bash"
```

**Zsh** (`~/.zshrc`):
```zsh
[ -f "$HOME/.config/nudge/shell/integration.zsh" ] && source "$HOME/.config/nudge/shell/integration.zsh"
```

**PowerShell** (`$PROFILE`):
```powershell
$nudgeIntegration = Join-Path $env:APPDATA "nudge\shell\integration.ps1"
if (Test-Path $nudgeIntegration) { . $nudgeIntegration }
```

## Post-Installation

```bash
# Verify installation
nudge --version

# Check status
nudge status

# Show runtime info
nudge info
```

### Configure LLM

Edit `~/.config/nudge/config.yaml`:

```yaml
# Ollama (local)
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"

# Or OpenAI
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-4o-mini"
  api_key_env: "OPENAI_API_KEY"
```

## Troubleshooting

**"nudge: command not found"**
- Ensure binary is in PATH
- Open a new terminal window
- Check: `echo $PATH`

**Daemon not starting**
- Run: `nudge daemon --foreground`
- Check logs for errors

**Shell integration not working**
- Re-run: `nudge setup`
- Source profile: `source ~/.bashrc`

**macOS: "cannot be opened because the developer cannot be verified"**
```bash
xattr -d com.apple.quarantine /usr/local/bin/nudge
```

**Windows: execution policy error**
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## Updating

Re-run the installer to update:

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

## See Also

- [Configuration Reference](configuration.md)
- [CLI Reference](cli-reference.md)
- [Auto Mode Guide](auto-mode.md)
