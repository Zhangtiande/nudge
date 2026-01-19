# Nudge One-Click Installation Script for Windows
#
# This script automatically downloads and installs Nudge from GitHub Releases.
#
# Usage:
#   irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
#
# Or with options:
#   .\install.ps1
#   .\install.ps1 -Version "0.1.0"
#   .\install.ps1 -InstallDir "C:\Tools\nudge"
#   .\install.ps1 -SkipShell
#   .\install.ps1 -Uninstall

param(
    [string]$Version = "",
    [string]$InstallDir = "",
    [switch]$SkipShell,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

# Configuration
$GitHubRepo = "Zhangtiande/nudge"
$DefaultInstallDir = Join-Path $env:LOCALAPPDATA "nudge"

# Color output functions
function Write-Info {
    param([string]$Message)
    Write-Host "INFO: " -ForegroundColor Cyan -NoNewline
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "SUCCESS: " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warning {
    param([string]$Message)
    Write-Host "WARNING: " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-ErrorMsg {
    param([string]$Message)
    Write-Host "ERROR: " -ForegroundColor Red -NoNewline
    Write-Host $Message
}

# Detect system architecture
function Get-SystemArchitecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default {
            Write-ErrorMsg "Unsupported architecture: $arch"
            exit 1
        }
    }
}

# Get latest version from GitHub API
function Get-LatestVersion {
    Write-Info "Fetching latest version from GitHub..."

    try {
        $apiUrl = "https://api.github.com/repos/$GitHubRepo/releases/latest"
        $response = Invoke-RestMethod -Uri $apiUrl -ErrorAction Stop

        $version = $response.tag_name.TrimStart('v')

        if ([string]::IsNullOrEmpty($version)) {
            throw "Failed to parse version from response"
        }

        Write-Success "Latest version: $version"
        return $version
    }
    catch {
        Write-ErrorMsg "Failed to fetch latest version: $_"
        exit 1
    }
}

# Download and extract binary
function Install-Binary {
    param(
        [string]$Version,
        [string]$Architecture,
        [string]$InstallPath
    )

    $filename = "nudge-windows-${Architecture}.zip"
    $downloadUrl = "https://github.com/$GitHubRepo/releases/download/v${Version}/${filename}"

    Write-Info "Downloading from: $downloadUrl"

    $tmpDir = Join-Path $env:TEMP "nudge-install-$(Get-Random)"
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        $zipPath = Join-Path $tmpDir $filename

        # Download with progress
        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -ErrorAction Stop
        $ProgressPreference = 'Continue'

        Write-Success "Downloaded: $filename"

        # Extract
        Write-Info "Extracting archive..."
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        # Find the binary
        $binaryPath = Join-Path $tmpDir "nudge.exe"
        if (-not (Test-Path $binaryPath)) {
            throw "Binary not found in archive"
        }

        # Create install directory
        $binDir = Join-Path $InstallPath "bin"
        if (-not (Test-Path $binDir)) {
            Write-Info "Creating directory: $binDir"
            New-Item -ItemType Directory -Path $binDir -Force | Out-Null
        }

        # Copy binary
        $destBinary = Join-Path $binDir "nudge.exe"
        Write-Info "Installing to: $destBinary"
        Copy-Item -Path $binaryPath -Destination $destBinary -Force

        Write-Success "Binary installed successfully"

        return $binDir
    }
    catch {
        Write-ErrorMsg "Installation failed: $_"
        exit 1
    }
    finally {
        # Cleanup
        if (Test-Path $tmpDir) {
            Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

# Add directory to user PATH
function Add-ToPath {
    param([string]$Directory)

    # Get current user PATH
    $regPath = "HKCU:\Environment"
    $currentPath = (Get-ItemProperty -Path $regPath -Name "Path" -ErrorAction SilentlyContinue).Path

    if ([string]::IsNullOrEmpty($currentPath)) {
        $currentPath = ""
    }

    # Check if already in PATH
    $pathDirs = $currentPath.Split(';', [StringSplitOptions]::RemoveEmptyEntries)
    if ($pathDirs -contains $Directory) {
        Write-Info "Directory already in PATH: $Directory"
        return
    }

    # Add to PATH
    Write-Info "Adding to PATH: $Directory"

    if ([string]::IsNullOrEmpty($currentPath)) {
        $newPath = $Directory
    }
    else {
        $newPath = "$currentPath;$Directory"
    }

    Set-ItemProperty -Path $regPath -Name "Path" -Value $newPath

    # Update current session PATH
    $env:Path = "$env:Path;$Directory"

    Write-Success "Added to PATH successfully"
    Write-Warning "You may need to restart your terminal for PATH changes to take effect"
}

# Interactive configuration wizard
function Start-ConfigWizard {
    param([string]$ConfigFile)

    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "    LLM Configuration Wizard" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Let's configure your LLM settings interactively." -ForegroundColor White
    Write-Host ""

    # Step 1: Choose LLM provider
    Write-Host "1. Which LLM provider do you want to use?" -ForegroundColor Cyan
    Write-Host "   [1] Local Ollama (recommended for privacy, free)" -ForegroundColor White
    Write-Host "   [2] OpenAI (requires API key, paid)" -ForegroundColor White
    Write-Host "   [3] Other OpenAI-compatible API" -ForegroundColor White
    Write-Host "   [4] Skip configuration (I'll configure manually later)" -ForegroundColor Gray
    Write-Host ""

    $providerChoice = Read-Host "Enter your choice (1-4)"

    if ($providerChoice -eq "4") {
        Write-Info "Skipping configuration. You can configure manually later."
        return $null
    }

    $config = @{
        endpoint = ""
        model_name = ""
        api_key = $null
        api_key_env = $null
    }

    switch ($providerChoice) {
        "1" {
            # Local Ollama
            Write-Host ""
            Write-Host "Configuring for local Ollama..." -ForegroundColor Green

            $config.endpoint = "http://localhost:11434/v1"

            Write-Host ""
            Write-Host "2. Which Ollama model do you want to use?" -ForegroundColor Cyan
            Write-Host "   Common models:" -ForegroundColor Gray
            Write-Host "   - codellama:7b (fast, good for code)" -ForegroundColor Gray
            Write-Host "   - deepseek-coder:6.7b (excellent for code)" -ForegroundColor Gray
            Write-Host "   - qwen2.5-coder:7b (multilingual code support)" -ForegroundColor Gray
            Write-Host ""

            $modelInput = Read-Host "Enter model name (press Enter for 'codellama:7b')"
            if ([string]::IsNullOrWhiteSpace($modelInput)) {
                $config.model_name = "codellama:7b"
            } else {
                $config.model_name = $modelInput.Trim()
            }

            Write-Host ""
            Write-Info "Make sure to run 'ollama serve' before using Nudge!"
        }
        "2" {
            # OpenAI
            Write-Host ""
            Write-Host "Configuring for OpenAI..." -ForegroundColor Green

            $config.endpoint = "https://api.openai.com/v1"

            Write-Host ""
            Write-Host "2. Which OpenAI model do you want to use?" -ForegroundColor Cyan
            Write-Host "   [1] gpt-4o (best quality, expensive)" -ForegroundColor White
            Write-Host "   [2] gpt-4o-mini (good balance)" -ForegroundColor White
            Write-Host "   [3] gpt-3.5-turbo (fastest, cheapest)" -ForegroundColor White
            Write-Host ""

            $modelChoice = Read-Host "Enter your choice (1-3)"
            switch ($modelChoice) {
                "1" { $config.model_name = "gpt-4o" }
                "2" { $config.model_name = "gpt-4o-mini" }
                "3" { $config.model_name = "gpt-3.5-turbo" }
                default { $config.model_name = "gpt-4o-mini" }
            }

            Write-Host ""
            Write-Host "3. How do you want to provide your API key?" -ForegroundColor Cyan
            Write-Host "   [1] Environment variable (recommended for security)" -ForegroundColor White
            Write-Host "   [2] Direct in config file (convenient but less secure)" -ForegroundColor White
            Write-Host ""

            $keyChoice = Read-Host "Enter your choice (1-2)"
            if ($keyChoice -eq "2") {
                Write-Host ""
                $apiKey = Read-Host "Enter your OpenAI API key (sk-...)" -AsSecureString
                $BSTR = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($apiKey)
                $config.api_key = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($BSTR)
            } else {
                $config.api_key_env = "OPENAI_API_KEY"
                Write-Host ""
                Write-Warning "Please set the OPENAI_API_KEY environment variable with your API key"
                Write-Host "Example: `$env:OPENAI_API_KEY = 'sk-your-api-key-here'" -ForegroundColor Gray
            }
        }
        "3" {
            # Custom OpenAI-compatible API
            Write-Host ""
            Write-Host "Configuring for custom OpenAI-compatible API..." -ForegroundColor Green

            Write-Host ""
            $config.endpoint = Read-Host "Enter API endpoint URL (e.g., https://api.example.com/v1)"

            Write-Host ""
            $config.model_name = Read-Host "Enter model name"

            Write-Host ""
            Write-Host "Does this API require an API key? (Y/N)" -ForegroundColor Cyan
            $requiresKey = Read-Host

            if ($requiresKey -eq "Y" -or $requiresKey -eq "y") {
                Write-Host ""
                Write-Host "How do you want to provide your API key?" -ForegroundColor Cyan
                Write-Host "   [1] Environment variable (recommended)" -ForegroundColor White
                Write-Host "   [2] Direct in config file" -ForegroundColor White
                Write-Host ""

                $keyChoice = Read-Host "Enter your choice (1-2)"
                if ($keyChoice -eq "2") {
                    Write-Host ""
                    $apiKey = Read-Host "Enter your API key" -AsSecureString
                    $BSTR = [System.Runtime.InteropServices.Marshal]::SecureStringToBSTR($apiKey)
                    $config.api_key = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto($BSTR)
                } else {
                    Write-Host ""
                    $envVarName = Read-Host "Enter environment variable name (e.g., MY_API_KEY)"
                    $config.api_key_env = $envVarName
                    Write-Host ""
                    Write-Warning "Please set the $envVarName environment variable with your API key"
                }
            }
        }
        default {
            Write-Warning "Invalid choice. Using default Ollama configuration."
            $config.endpoint = "http://localhost:11434/v1"
            $config.model_name = "codellama:7b"
        }
    }

    return $config
}

# Create config file from wizard results
function New-ConfigFromWizard {
    param(
        [string]$ConfigFile,
        [hashtable]$Config
    )

    $apiKeyLine = ""
    if ($Config.api_key) {
        $apiKeyLine = "  api_key: `"$($Config.api_key)`""
    } elseif ($Config.api_key_env) {
        $apiKeyLine = "  api_key_env: `"$($Config.api_key_env)`""
    } else {
        $apiKeyLine = "  # api_key_env: `"OPENAI_API_KEY`"  # Uncomment and set if needed"
    }

    $configContent = @"
# Nudge Configuration
# Generated by installation wizard
# Documentation: https://github.com/$GitHubRepo

model:
  endpoint: "$($Config.endpoint)"
  model_name: "$($Config.model_name)"
$apiKeyLine
  timeout_ms: 5000

context:
  history_window: 20
  include_cwd_listing: true
  include_exit_code: true
  include_system_info: true
  similar_commands_enabled: true
  similar_commands_window: 200
  similar_commands_max: 5
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
  hotkey: "\C-e"

privacy:
  sanitize_enabled: true
  custom_patterns: []
  block_dangerous: true
  custom_blocked: []

log:
  level: "info"
  file_enabled: false
"@

    Set-Content -Path $ConfigFile -Value $configContent -Encoding UTF8
    Write-Success "Configuration file created: $ConfigFile"
}

# Download shell integration files
function Get-ShellIntegrationFiles {
    Write-Info "Downloading shell integration files..."

    $baseUrl = "https://raw.githubusercontent.com/$GitHubRepo/main"
    $shellDir = Join-Path $env:TEMP "nudge-shell-$(Get-Random)"
    New-Item -ItemType Directory -Path $shellDir -Force | Out-Null

    try {
        $files = @(
            "shell/setup-shell.ps1",
            "shell/integration.ps1",
            "shell/integration.cmd",
            "config/config.yaml.template"
        )

        foreach ($file in $files) {
            $url = "$baseUrl/$file"
            $destPath = Join-Path $shellDir ($file -replace '/', '\')
            $destDir = Split-Path -Parent $destPath

            if (-not (Test-Path $destDir)) {
                New-Item -ItemType Directory -Path $destDir -Force | Out-Null
            }

            $ProgressPreference = 'SilentlyContinue'
            Invoke-WebRequest -Uri $url -OutFile $destPath -ErrorAction Stop
            $ProgressPreference = 'Continue'
        }

        Write-Success "Shell integration files downloaded"
        return $shellDir
    }
    catch {
        Write-ErrorMsg "Failed to download shell integration files: $_"
        return $null
    }
}

# Setup shell integration
function Setup-ShellIntegration {
    param([string]$ShellDir)

    if ($SkipShell) {
        Write-Info "Skipping shell integration (SkipShell flag)"
        return
    }

    Write-Info "Setting up shell integration..."

    $setupScript = Join-Path $ShellDir "shell\setup-shell.ps1"

    if (Test-Path $setupScript) {
        & $setupScript
    }
    else {
        Write-Warning "Shell setup script not found"
        Write-Info "You can set up shell integration manually later"
    }
}

# Uninstall nudge
function Uninstall-Nudge {
    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "    Uninstalling Nudge" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host ""

    $removed = $false

    # Remove from default location
    $defaultBin = Join-Path $DefaultInstallDir "bin\nudge.exe"
    if (Test-Path $defaultBin) {
        Write-Info "Removing binary: $defaultBin"
        Remove-Item -Path $defaultBin -Force
        $removed = $true

        # Remove directory if empty
        $binDir = Split-Path -Parent $defaultBin
        if ((Get-ChildItem -Path $binDir -Force | Measure-Object).Count -eq 0) {
            Remove-Item -Path $binDir -Force
        }

        $installDir = Split-Path -Parent $binDir
        if ((Get-ChildItem -Path $installDir -Force | Measure-Object).Count -eq 0) {
            Remove-Item -Path $installDir -Force
        }
    }

    # Check other locations in PATH
    $nudgePath = Get-Command "nudge" -ErrorAction SilentlyContinue
    if ($nudgePath) {
        $nudgeExe = $nudgePath.Source
        Write-Info "Removing binary: $nudgeExe"
        Remove-Item -Path $nudgeExe -Force -ErrorAction SilentlyContinue
        $removed = $true
    }

    if (-not $removed) {
        Write-Warning "Nudge binary not found"
    }
    else {
        Write-Success "Binary removed"
    }

    # Remove from PATH
    $regPath = "HKCU:\Environment"
    $currentPath = (Get-ItemProperty -Path $regPath -Name "Path" -ErrorAction SilentlyContinue).Path

    if ($currentPath) {
        $newPath = ($currentPath.Split(';', [StringSplitOptions]::RemoveEmptyEntries) |
            Where-Object { $_ -notmatch 'nudge' }) -join ';'

        if ($newPath -ne $currentPath) {
            Set-ItemProperty -Path $regPath -Name "Path" -Value $newPath
            Write-Info "Removed from PATH"
        }
    }

    # Remove shell integration
    $shellSetupDir = Join-Path $env:TEMP "nudge-shell-uninstall"
    New-Item -ItemType Directory -Path $shellSetupDir -Force | Out-Null

    try {
        $baseUrl = "https://raw.githubusercontent.com/$GitHubRepo/main/shell"
        $setupScriptPath = Join-Path $shellSetupDir "setup-shell.ps1"

        $ProgressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri "$baseUrl/setup-shell.ps1" -OutFile $setupScriptPath -ErrorAction Stop
        $ProgressPreference = 'Continue'

        & $setupScriptPath -Uninstall
    }
    catch {
        Write-Warning "Could not remove shell integration automatically"
        Write-Info "You may need to remove it manually from your PowerShell profile"
    }
    finally {
        Remove-Item -Path $shellSetupDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host ""
    Write-Warning "Configuration files in $env:APPDATA\nudge were not removed."
    Write-Host "To remove them manually, run:"
    Write-Host "  Remove-Item -Path `"$env:APPDATA\nudge`" -Recurse -Force"
    Write-Host ""

    Write-Success "Uninstallation complete!"
}

# Main installation
function Main {
    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "    Nudge Installation for Windows" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host ""

    if ($Uninstall) {
        Uninstall-Nudge
        return
    }

    # Detect architecture
    $arch = Get-SystemArchitecture
    Write-Info "Detected architecture: $arch"

    # Get version
    if ([string]::IsNullOrEmpty($Version)) {
        $Version = Get-LatestVersion
    }
    else {
        Write-Info "Using specified version: $Version"
    }

    # Determine install directory
    if ([string]::IsNullOrEmpty($InstallDir)) {
        $InstallDir = $DefaultInstallDir
    }

    Write-Info "Install directory: $InstallDir"

    # Download and install binary
    $binDir = Install-Binary -Version $Version -Architecture $arch -InstallPath $InstallDir

    # Add to PATH
    Add-ToPath -Directory $binDir

    # Setup shell integration
    $shellDir = Get-ShellIntegrationFiles
    if ($shellDir) {
        Setup-ShellIntegration -ShellDir $shellDir

        # Cleanup
        Remove-Item -Path $shellDir -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "    Installation Complete!" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Success "Nudge $Version has been installed successfully!"
    Write-Host ""

    # Determine config file location
    $configFile = Join-Path $env:APPDATA "nudge\config\config.yaml"
    $configDir = Split-Path -Parent $configFile

    # Ensure config directory exists
    if (-not (Test-Path $configDir)) {
        New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    }

    # Run interactive configuration wizard
    if (-not (Test-Path $configFile)) {
        $wizardConfig = Start-ConfigWizard -ConfigFile $configFile

        if ($wizardConfig) {
            New-ConfigFromWizard -ConfigFile $configFile -Config $wizardConfig
            Write-Host ""
            Write-Success "Configuration completed!"
        } else {
            # User skipped wizard, create default config
            Write-Info "Creating default configuration file..."
            $defaultConfig = @"
# Nudge Configuration
# Documentation: https://github.com/$GitHubRepo

model:
  endpoint: "http://localhost:11434/v1"
  model_name: "codellama:7b"
  # api_key_env: "OPENAI_API_KEY"  # Uncomment if using OpenAI
  timeout_ms: 5000

context:
  history_window: 20
  include_cwd_listing: true
  include_exit_code: true
  include_system_info: true
  similar_commands_enabled: true
  similar_commands_window: 200
  similar_commands_max: 5
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
  hotkey: "\C-e"

privacy:
  sanitize_enabled: true
  custom_patterns: []
  block_dangerous: true
  custom_blocked: []

log:
  level: "info"
  file_enabled: false
"@
            Set-Content -Path $configFile -Value $defaultConfig -Encoding UTF8
            Write-Success "Default configuration file created: $configFile"
            Write-Warning "Please edit the configuration file to set your LLM settings"
        }
    } else {
        Write-Info "Configuration file already exists: $configFile"
    }

    Write-Host ""
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host "    Next Steps" -ForegroundColor Cyan
    Write-Host "=========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  1. Restart your terminal (or run: . `$PROFILE)" -ForegroundColor White
    Write-Host "  2. Start Ollama if using local LLM: ollama serve" -ForegroundColor White
    Write-Host "  3. Press Ctrl+E in your terminal to trigger AI completion" -ForegroundColor White
    Write-Host ""
    Write-Info "Configuration file: $configFile"
    Write-Info "For more information, visit: https://github.com/$GitHubRepo"
    Write-Host ""
}

# Run main function
Main
