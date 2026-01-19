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
$TemplateConfig = Join-Path (Split-Path -Parent $ScriptDir) "config\config.yaml.template"

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

# Interactive configuration wizard
function Start-ConfigWizard {
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
        Write-Host "Skipping configuration. You can configure manually later." -ForegroundColor Yellow
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
            Write-Host "Make sure to run 'ollama serve' before using Nudge!" -ForegroundColor Cyan
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
                Write-Host "Please set the OPENAI_API_KEY environment variable with your API key" -ForegroundColor Yellow
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
            $requiresKey = Read-Host "Does this API require an API key? (Y/N)"

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
                    Write-Host "Please set the $envVarName environment variable with your API key" -ForegroundColor Yellow
                }
            }
        }
        default {
            Write-Host "Invalid choice. Using default Ollama configuration." -ForegroundColor Yellow
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
# Documentation: https://github.com/Zhangtiande/nudge

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
    Write-Host "Configuration file created: $ConfigFile" -ForegroundColor Green
}

function Setup-NudgeDir {
    $nudgeDir = Get-NudgeDir
    $configDir = Get-ConfigDir
    $configFile = Join-Path $configDir "config.yaml"

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

    # Create config from interactive wizard or template if not exists
    if (-not (Test-Path $configFile)) {
        $wizardConfig = Start-ConfigWizard

        if ($wizardConfig) {
            New-ConfigFromWizard -ConfigFile $configFile -Config $wizardConfig
            Write-Host ""
            Write-Host "Configuration completed!" -ForegroundColor Green
        } else {
            # User skipped wizard, create default config
            Write-Host "Creating default config: $configFile" -ForegroundColor Cyan

            if (Test-Path $TemplateConfig) {
                Copy-Item -Path $TemplateConfig -Destination $configFile
                Write-Host "Config created from template" -ForegroundColor Green
            } else {
                # Fallback: create basic config inline
                $defaultConfig = @"
# Nudge Configuration
# Documentation: https://github.com/Zhangtiande/nudge

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
                Set-Content -Path $configFile -Value $defaultConfig
                Write-Host "Config created with default settings" -ForegroundColor Green
            }
            Write-Host "Please edit $configFile to configure your LLM settings" -ForegroundColor Yellow
        }
    } else {
        Write-Host "Config file already exists: $configFile" -ForegroundColor Yellow
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
