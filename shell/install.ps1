# Nudge - Windows Installation Script
# Run this script to install Nudge shell integration
#
# Usage:
#   .\install.ps1              # Install for PowerShell only
#   .\install.ps1 -Cmd         # Also install CMD integration
#   .\install.ps1 -Uninstall   # Remove Nudge integration

param(
    [switch]$Cmd,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

# Detect script location
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$IntegrationPs1 = Join-Path $ScriptDir "integration.ps1"
$IntegrationCmd = Join-Path $ScriptDir "integration.cmd"

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

# Main execution
Write-Host ""
Write-Host "Nudge Windows Installer" -ForegroundColor Cyan
Write-Host "=======================" -ForegroundColor Cyan
Write-Host ""

if ($Uninstall) {
    Write-Host "Uninstalling Nudge..." -ForegroundColor Yellow
    Write-Host ""
    
    Uninstall-PowerShellIntegration
    Uninstall-CmdIntegration
    
    Write-Host ""
    Write-Host "Uninstallation complete!" -ForegroundColor Green
    Write-Host "Restart your shell for changes to take effect." -ForegroundColor Cyan
} else {
    Write-Host "Installing Nudge..." -ForegroundColor Yellow
    Write-Host ""
    
    # Check if nudge binary is in PATH
    $nudgePath = Get-Command "nudge" -ErrorAction SilentlyContinue
    if (-not $nudgePath) {
        Write-Host "Warning: 'nudge' command not found in PATH." -ForegroundColor Yellow
        Write-Host "Make sure to add the nudge binary to your PATH." -ForegroundColor Yellow
        Write-Host ""
    }
    
    $installed = Install-PowerShellIntegration
    
    if ($Cmd) {
        Install-CmdIntegration
    }
    
    Write-Host ""
    Write-Host "Installation complete!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Restart your shell (or run: . `$PROFILE)" -ForegroundColor White
    Write-Host "  2. Press Ctrl+E to trigger AI completion" -ForegroundColor White
    Write-Host ""
    
    if (-not $Cmd) {
        Write-Host "Tip: Run with -Cmd flag to also install CMD integration." -ForegroundColor Gray
    }
}
