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
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Restart your terminal (or run: . `$PROFILE)" -ForegroundColor White
    Write-Host "  2. Start Ollama if using local LLM: ollama serve" -ForegroundColor White
    Write-Host "  3. Press Ctrl+E in your terminal to trigger AI completion" -ForegroundColor White
    Write-Host ""
    Write-Info "For more information, visit: https://github.com/$GitHubRepo"
    Write-Host ""
}

# Run main function
Main
