# Nudge - Windows Shell Integration Setup Script
# This script sets up shell integration for Nudge.
# It assumes that the 'nudge' binary is already installed and in PATH.
#
# Usage:
#   .\setup-shell.ps1              # Install for PowerShell only
#   .\setup-shell.ps1 -Cmd         # Also install CMD integration
#   .\setup-shell.ps1 -Uninstall   # Remove Nudge integration

param(
    [switch]$Cmd,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

# Detect script location
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$SourceIntegrationPs1 = Join-Path $ScriptDir "integration.ps1"
$SourceIntegrationCmd = Join-Path $ScriptDir "integration.cmd"
$ConfigDir = Join-Path (Split-Path -Parent $ScriptDir) "config"
$TemplateDefaultConfig = Join-Path $ConfigDir "config.yaml.template"
$TemplateUserConfig = Join-Path $ConfigDir "config.user.yaml.template"

# Marker for profile modifications
$NudgeMarkerStart = "# >>> Nudge Integration >>>"
$NudgeMarkerEnd = "# <<< Nudge Integration <<<"

function Get-ProfilePath {
    # Prefer CurrentUserAllHosts profile
    if ($PROFILE.CurrentUserAllHosts) {
        return $PROFILE.CurrentUserAllHosts
    }
    return $PROFILE
}

function Get-NudgeDir {
    # Base nudge directory in APPDATA
    return Join-Path $env:APPDATA "nudge"
}

function Get-ConfigDir {
    # directories crate on Windows: config_dir() = {ROAMING_APPDATA}/{project}/config
    return Join-Path (Get-NudgeDir) "config"
}

# Get installed integration script paths (in APPDATA)
$NudgeDir = Get-NudgeDir
$IntegrationPs1 = Join-Path $NudgeDir "integration.ps1"
$IntegrationCmd = Join-Path $NudgeDir "integration.cmd"

function Install-PowerShellIntegration {
    $profilePath = Get-ProfilePath

    # Ensure profile directory exists
    $profileDir = Split-Path -Parent $profilePath
    if (-not (Test-Path $profileDir)) {
        New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
    }

    # Check if already installed
    if (Test-Path $profilePath) {
        $content = Get-Content $profilePath -Raw
        if ($content -and $content.Contains($NudgeMarkerStart)) {
            Write-Host "Nudge is already installed in PowerShell profile." -ForegroundColor Yellow
            Write-Host "Run with -Uninstall first to reinstall." -ForegroundColor Yellow
            return $false
        }
    }

    # Add integration to profile
    $integrationBlock = @"

$NudgeMarkerStart
# Load Nudge shell integration
if (Test-Path "$IntegrationPs1") {
    . "$IntegrationPs1"
}
$NudgeMarkerEnd
"@

    Add-Content -Path $profilePath -Value $integrationBlock
    Write-Host "PowerShell integration installed to: $profilePath" -ForegroundColor Green
    return $true
}

function Install-CmdIntegration {
    # Add to AutoRun registry key
    $regPath = "HKCU:\Software\Microsoft\Command Processor"
    $existingAutoRun = (Get-ItemProperty -Path $regPath -Name "AutoRun" -ErrorAction SilentlyContinue).AutoRun

    if ($existingAutoRun -and $existingAutoRun.Contains("nudge")) {
        Write-Host "Nudge is already installed in CMD AutoRun." -ForegroundColor Yellow
        return $false
    }

    if ($existingAutoRun) {
        # Append to existing AutoRun
        $newAutoRun = "$existingAutoRun & `"$IntegrationCmd`""
    } else {
        $newAutoRun = "`"$IntegrationCmd`""
    }

    Set-ItemProperty -Path $regPath -Name "AutoRun" -Value $newAutoRun
    Write-Host "CMD integration installed to registry AutoRun." -ForegroundColor Green
    return $true
}

function Uninstall-PowerShellIntegration {
    $profilePath = Get-ProfilePath

    if (-not (Test-Path $profilePath)) {
        Write-Host "No PowerShell profile found." -ForegroundColor Yellow
        return
    }

    $content = Get-Content $profilePath -Raw
    if (-not $content -or -not $content.Contains($NudgeMarkerStart)) {
        Write-Host "Nudge is not installed in PowerShell profile." -ForegroundColor Yellow
        return
    }

    # Remove the integration block
    $pattern = "(?s)$([regex]::Escape($NudgeMarkerStart)).*?$([regex]::Escape($NudgeMarkerEnd))\r?\n?"
    $newContent = $content -replace $pattern, ""

    Set-Content -Path $profilePath -Value $newContent.TrimEnd()
    Write-Host "PowerShell integration removed from: $profilePath" -ForegroundColor Green
}

function Uninstall-CmdIntegration {
    $regPath = "HKCU:\Software\Microsoft\Command Processor"
    $existingAutoRun = (Get-ItemProperty -Path $regPath -Name "AutoRun" -ErrorAction SilentlyContinue).AutoRun

    if (-not $existingAutoRun -or -not $existingAutoRun.Contains("nudge")) {
        Write-Host "Nudge is not installed in CMD AutoRun." -ForegroundColor Yellow
        return
    }

    # Remove nudge from AutoRun
    $newAutoRun = $existingAutoRun -replace '\s*&?\s*"[^"]*integration\.cmd"', ""
    $newAutoRun = $newAutoRun.Trim(" &")

    if ([string]::IsNullOrEmpty($newAutoRun)) {
        Remove-ItemProperty -Path $regPath -Name "AutoRun" -ErrorAction SilentlyContinue
    } else {
        Set-ItemProperty -Path $regPath -Name "AutoRun" -Value $newAutoRun
    }

    Write-Host "CMD integration removed from registry AutoRun." -ForegroundColor Green
}

function Setup-NudgeDir {
    $nudgeDir = Get-NudgeDir
    $configDir = Get-ConfigDir
    $defaultConfigFile = Join-Path $configDir "config.default.yaml"
    $userConfigFile = Join-Path $configDir "config.yaml"

    # Create nudge directory
    if (-not (Test-Path $nudgeDir)) {
        Write-Host "Creating nudge directory: $nudgeDir" -ForegroundColor Cyan
        New-Item -ItemType Directory -Path $nudgeDir -Force | Out-Null
    }

    # Create config directory
    if (-not (Test-Path $configDir)) {
        Write-Host "Creating config directory: $configDir" -ForegroundColor Cyan
        New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    }

    # Copy integration scripts to nudge directory
    if (Test-Path $SourceIntegrationPs1) {
        Write-Host "Installing integration script: $IntegrationPs1" -ForegroundColor Cyan
        Copy-Item -Path $SourceIntegrationPs1 -Destination $IntegrationPs1 -Force
    }

    if (Test-Path $SourceIntegrationCmd) {
        Write-Host "Installing integration script: $IntegrationCmd" -ForegroundColor Cyan
        Copy-Item -Path $SourceIntegrationCmd -Destination $IntegrationCmd -Force
    }

    # Always update config.default.yaml (ships with app, updated on upgrade)
    Write-Host "Updating default config: $defaultConfigFile" -ForegroundColor Cyan
    if (Test-Path $TemplateDefaultConfig) {
        Copy-Item -Path $TemplateDefaultConfig -Destination $defaultConfigFile -Force
        Write-Host "Default config updated from template" -ForegroundColor Green
    } else {
        # Fallback: create basic default config inline
        $defaultConfig = @"
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
        Set-Content -Path $defaultConfigFile -Value $defaultConfig
        Write-Host "Default config created with built-in settings" -ForegroundColor Green
    }

    # Create user config only if it doesn't exist (preserve user customizations)
    if (-not (Test-Path $userConfigFile)) {
        Write-Host "Creating user config: $userConfigFile" -ForegroundColor Cyan
        if (Test-Path $TemplateUserConfig) {
            Copy-Item -Path $TemplateUserConfig -Destination $userConfigFile
            Write-Host "User config created from template" -ForegroundColor Green
        } else {
            # Fallback: create minimal user config
            $userConfig = @"
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
"@
            Set-Content -Path $userConfigFile -Value $userConfig
            Write-Host "User config created with minimal template" -ForegroundColor Green
        }
    } else {
        Write-Host "User config preserved: $userConfigFile" -ForegroundColor Yellow
    }
}

# Main execution
Write-Host ""
Write-Host "Nudge Shell Integration Setup" -ForegroundColor Cyan
Write-Host "=============================" -ForegroundColor Cyan
Write-Host ""

if ($Uninstall) {
    Write-Host "Uninstalling Nudge shell integration..." -ForegroundColor Yellow
    Write-Host ""

    Uninstall-PowerShellIntegration
    Uninstall-CmdIntegration

    Write-Host ""
    Write-Host "Uninstallation complete!" -ForegroundColor Green
    Write-Host "Restart your shell for changes to take effect." -ForegroundColor Cyan
} else {
    Write-Host "Installing Nudge shell integration..." -ForegroundColor Yellow
    Write-Host ""

    # Check if nudge binary is in PATH
    $nudgePath = Get-Command "nudge" -ErrorAction SilentlyContinue
    if (-not $nudgePath) {
        Write-Host "Warning: 'nudge' command not found in PATH." -ForegroundColor Yellow
        Write-Host "Make sure to add the nudge binary to your PATH." -ForegroundColor Yellow
        Write-Host ""
    }

    Setup-NudgeDir
    $installed = Install-PowerShellIntegration

    if ($Cmd) {
        Install-CmdIntegration
    }

    Write-Host ""
    Write-Host "Shell integration complete!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Restart your shell (or run: . `$PROFILE)" -ForegroundColor White
    Write-Host "  2. Make sure Ollama is running (ollama serve)" -ForegroundColor White
    Write-Host "  3. Press Ctrl+E to trigger AI completion" -ForegroundColor White
    Write-Host ""

    if (-not $Cmd) {
        Write-Host "Tip: Run with -Cmd flag to also install CMD integration." -ForegroundColor Gray
    }
}
