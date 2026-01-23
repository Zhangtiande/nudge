# Installation Guide

This guide covers all installation methods for Nudge.

## Quick Install (Recommended)

### Unix/Linux/macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

The installer will:
1. Download the latest release binary
2. Install to `~/.local/bin` (Unix) or `%LOCALAPPDATA%\nudge\bin` (Windows)
3. Add to PATH
4. Run `nudge setup` to configure shell integration
5. Start the daemon

## Installation Options

### Specify Version

```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --version 0.3.0

# Windows
$script = Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1"
& ([scriptblock]::Create($script.Content)) -Version "0.3.0"
```

### Custom Install Location

```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --prefix /opt/nudge

# Windows (download script first)
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1" -OutFile install.ps1
.\install.ps1 -InstallDir "C:\Tools\nudge"
```

### Skip Shell Integration

If you want to configure shell integration manually:

```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --skip-shell

# Windows
.\install.ps1 -SkipShell
```

### Uninstall

```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --uninstall

# Windows
.\install.ps1 -Uninstall
```

## Manual Installation

### From Pre-built Binaries

1. Download the binary for your platform from [Releases](https://github.com/Zhangtiande/nudge/releases/latest)

| Platform | Architecture | Binary |
|----------|--------------|--------|
| Linux | x86_64 (glibc) | `nudge-linux-x86_64.tar.gz` |
| Linux | x86_64 (musl) | `nudge-linux-x86_64-musl.tar.gz` |
| Linux | aarch64 (ARM64) | `nudge-linux-aarch64.tar.gz` |
| macOS | x86_64 (Intel) | `nudge-macos-x86_64.tar.gz` |
| macOS | aarch64 (Apple Silicon) | `nudge-macos-aarch64.tar.gz` |
| Windows | x86_64 | `nudge-windows-x86_64.zip` |

2. Extract and install:

**Linux/macOS:**
```bash
# Download (replace with your platform)
curl -LO https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64.tar.gz

# Extract
tar xzf nudge-linux-x86_64.tar.gz

# Install to PATH
sudo mv nudge /usr/local/bin/
# Or for user-only install:
mkdir -p ~/.local/bin
mv nudge ~/.local/bin/
```

**Windows:**
```powershell
# Download and extract
Invoke-WebRequest -Uri "https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-windows-x86_64.zip" -OutFile nudge.zip
Expand-Archive nudge.zip -DestinationPath .

# Move to a directory in PATH
Move-Item nudge.exe "$env:LOCALAPPDATA\nudge\bin\"
```

3. Configure shell integration:
```bash
nudge setup
```

4. Restart your shell or source your profile.

### Build from Source

**Prerequisites:**
- Rust toolchain (1.70+)
- Git

```bash
# Clone the repository
git clone https://github.com/Zhangtiande/nudge.git
cd nudge

# Build release binary
cargo build --release

# Install binary
# Unix/Linux/macOS:
sudo cp target/release/nudge /usr/local/bin/

# Windows PowerShell:
Copy-Item target\release\nudge.exe "$env:LOCALAPPDATA\nudge\bin\"

# Configure shell integration
nudge setup
```

**Build with FFI support (Unix only):**
```bash
cargo build --release --features ffi

# This creates libnudge.so (Linux) or libnudge.dylib (macOS)
ls target/release/libnudge.*
```

## Shell Integration

### Automatic Setup

The recommended way to configure shell integration:

```bash
nudge setup
```

This will:
- Detect your shell (Bash, Zsh, PowerShell, CMD)
- Install integration scripts to the config directory
- Add source line to your shell profile
- Start the daemon if not running

### Manual Setup

If you prefer manual configuration:

**Bash** (`~/.bashrc`):
```bash
# Nudge - LLM-powered CLI completion
[ -f "$HOME/.config/nudge/shell/integration.bash" ] && source "$HOME/.config/nudge/shell/integration.bash"
```

**Zsh** (`~/.zshrc`):
```zsh
# Nudge - LLM-powered CLI completion
[ -f "$HOME/.config/nudge/shell/integration.zsh" ] && source "$HOME/.config/nudge/shell/integration.zsh"
```

**PowerShell** (`$PROFILE`):
```powershell
# Nudge - LLM-powered CLI completion
$nudgeIntegration = Join-Path $env:APPDATA "nudge\shell\integration.ps1"
if (Test-Path $nudgeIntegration) {
    . $nudgeIntegration
}
```

**CMD** (Registry: `HKCU:\Software\Microsoft\Command Processor\AutoRun`):
```cmd
"%APPDATA%\nudge\shell\integration.cmd"
```

## Post-Installation

### Verify Installation

```bash
# Check version
nudge --version

# Check daemon status
nudge status

# Show runtime information
nudge info
```

### Start the Daemon

The daemon should start automatically. If not:

```bash
nudge start
```

### Configure LLM

Edit your config file to set up your LLM provider:

**Location:**
- Linux: `~/.config/nudge/config.yaml`
- macOS: `~/Library/Application Support/nudge/config.yaml`
- Windows: `%APPDATA%\nudge\config\config.yaml`

**Example for Ollama (local):**
```yaml
model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
```

**Example for OpenAI:**
```yaml
model:
  endpoint: "https://api.openai.com/v1"
  model_name: "gpt-3.5-turbo"
  api_key_env: "OPENAI_API_KEY"
```

Then set your API key:
```bash
export OPENAI_API_KEY="sk-..."
```

### Enable Auto Mode (Optional)

To enable ghost text suggestions:

```yaml
trigger:
  mode: auto
  auto_delay_ms: 500
```

See [Auto Mode Guide](auto-mode.md) for details.

## Troubleshooting

### Common Issues

**"nudge: command not found"**
- Ensure the binary is in your PATH
- Try opening a new terminal window
- Check: `echo $PATH` (Unix) or `$env:PATH` (PowerShell)

**Daemon not starting**
- Check logs: `nudge daemon --foreground`
- Verify config syntax: `nudge info`

**Shell integration not working**
- Re-run: `nudge setup`
- Source your profile: `source ~/.bashrc` or `. $PROFILE`

See [Troubleshooting Guide](troubleshooting.md) for more solutions.

## Updating

To update to the latest version, simply re-run the installer:

```bash
# Unix/Linux/macOS
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash

# Windows
irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
```

Or specify a version:
```bash
curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash -s -- --version 0.3.0
```

## Platform-Specific Notes

### macOS

If you see "nudge cannot be opened because the developer cannot be verified":
```bash
xattr -d com.apple.quarantine /usr/local/bin/nudge
```

### Windows

If you encounter execution policy issues:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Linux

For musl-based distributions (Alpine, etc.), use the musl binary:
```bash
curl -LO https://github.com/Zhangtiande/nudge/releases/latest/download/nudge-linux-x86_64-musl.tar.gz
```
