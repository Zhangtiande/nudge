# Nudge - PowerShell Integration
# Installed by: nudge setup powershell

# Get configuration from nudge CLI
$script:NudgeInfo = @{}
try {
    $infoJson = nudge info --json 2>$null | ConvertFrom-Json
    $script:NudgeInfo = $infoJson
} catch {
    # Fallback if nudge not in PATH
    $script:NudgeInfo = @{
        config_dir = Join-Path $env:APPDATA "nudge"
        socket_path = "\\.\pipe\nudge_$env:USERNAME"
        trigger_mode = "manual"
        auto_delay_ms = 500
    }
}

$script:NudgeLastExitCode = 0
$script:NudgeTriggerMode = if ($script:NudgeInfo.trigger_mode) { $script:NudgeInfo.trigger_mode } else { "manual" }

# Diagnosis state
$script:NudgeDiagnosisEnabled = $false
try {
    $diagEnabled = nudge info --field diagnosis_enabled 2>$null
    $script:NudgeDiagnosisEnabled = $diagEnabled -eq "true"
} catch {}

$script:NudgeLastErrorCount = 0

# Capture exit codes
function global:Invoke-NudgeCaptureExitCode {
    $script:NudgeLastExitCode = $LASTEXITCODE
}

# ============================================================================
# Error Diagnosis Functions
# ============================================================================

function global:Invoke-NudgeDiagnosis {
    if (-not $script:NudgeDiagnosisEnabled) { return }

    $currentErrorCount = $Global:Error.Count

    # Check if new error occurred
    if ($currentErrorCount -gt $script:NudgeLastErrorCount) {
        $lastError = $Global:Error[0]

        if ($lastError) {
            Start-NudgeDaemonIfNeeded

            # Build error record JSON
            $errorContext = @{
                message = $lastError.Exception.Message
                command = $lastError.InvocationInfo.Line
                scriptStackTrace = $lastError.ScriptStackTrace
                category = $lastError.CategoryInfo.ToString()
            } | ConvertTo-Json -Compress

            try {
                $diagnosis = & nudge diagnose `
                    --exit-code $(if ($LASTEXITCODE) { $LASTEXITCODE } else { 1 }) `
                    --command "$($lastError.InvocationInfo.Line)" `
                    --error-record $errorContext `
                    --cwd (Get-Location).Path `
                    --session "pwsh-$PID" `
                    --format plain 2>$null

                if ($LASTEXITCODE -eq 0 -and $diagnosis) {
                    Write-Host $diagnosis -ForegroundColor Yellow
                }
            } catch {
                # Silently ignore diagnosis errors
            }
        }
    }

    $script:NudgeLastErrorCount = $currentErrorCount
}

# Register prompt hook if not already registered
if (-not $global:NudgePromptHookRegistered) {
    $existingPrompt = Get-Content Function:\prompt -ErrorAction SilentlyContinue
    if ($existingPrompt) {
        $newPrompt = @"
Invoke-NudgeCaptureExitCode
Invoke-NudgeDiagnosis
$existingPrompt
"@
        Set-Content Function:\prompt -Value ([scriptblock]::Create($newPrompt))
    }
    $global:NudgePromptHookRegistered = $true
}

# Ensure daemon is running
function global:Start-NudgeDaemonIfNeeded {
    try {
        $status = nudge status 2>$null
        if ($LASTEXITCODE -ne 0) {
            Start-Process -FilePath "nudge" -ArgumentList "start" -WindowStyle Hidden -ErrorAction SilentlyContinue
            Start-Sleep -Milliseconds 100
        }
    } catch {
        # Silently ignore errors
    }
}

# Main completion function (manual mode)
function global:Invoke-NudgeComplete {
    Start-NudgeDaemonIfNeeded

    $buffer = $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$buffer, [ref]$cursor)

    if ([string]::IsNullOrEmpty($buffer)) { return }

    try {
        $suggestion = & nudge complete `
            --format plain `
            --buffer $buffer `
            --cursor $cursor `
            --cwd (Get-Location).Path `
            --session "pwsh-$PID" `
            --last-exit-code $script:NudgeLastExitCode `
            2>$null

        if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrEmpty($suggestion)) {
            [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert($suggestion)
        }
    } catch {
        # Silently ignore errors
    }
}

# ============================================================================
# Auto Mode Setup (PowerShell 7.2+ only)
# ============================================================================

function Test-NudgeAutoModeSupport {
    # Check PowerShell version
    if ($PSVersionTable.PSVersion.Major -lt 7) {
        return $false
    }
    if ($PSVersionTable.PSVersion.Major -eq 7 -and $PSVersionTable.PSVersion.Minor -lt 2) {
        return $false
    }

    # Check PSReadLine version
    $psrl = Get-Module PSReadLine -ErrorAction SilentlyContinue
    if (-not $psrl) {
        $psrl = Get-Module PSReadLine -ListAvailable | Select-Object -First 1
    }
    if (-not $psrl -or $psrl.Version -lt [Version]'2.2.0') {
        return $false
    }

    return $true
}

function Initialize-NudgeAutoMode {
    [CmdletBinding()]
    param()

    # Find NudgePredictor module
    $modulePath = $null

    # Check in nudge config directory
    $configDir = if ($script:NudgeInfo.config_dir) { $script:NudgeInfo.config_dir } else { Join-Path $env:APPDATA "nudge" }
    $nudgePredictorPath = Join-Path $configDir "modules\NudgePredictor"
    if (Test-Path (Join-Path $nudgePredictorPath "NudgePredictor.psd1")) {
        $modulePath = $nudgePredictorPath
    }

    # Check in PSModulePath
    if (-not $modulePath) {
        $existingModule = Get-Module NudgePredictor -ListAvailable -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($existingModule) {
            $modulePath = Split-Path $existingModule.Path -Parent
        }
    }

    # Check in script directory (for development)
    if (-not $modulePath) {
        $scriptDir = Split-Path $MyInvocation.MyCommand.Path -Parent -ErrorAction SilentlyContinue
        if ($scriptDir) {
            $devPath = Join-Path $scriptDir "NudgePredictor"
            if (Test-Path (Join-Path $devPath "NudgePredictor.psd1")) {
                $modulePath = $devPath
            }
        }
    }

    if (-not $modulePath) {
        Write-Warning "NudgePredictor module not found. Auto mode disabled."
        Write-Warning "Install the module or use manual mode (Ctrl+E)."
        return $false
    }

    try {
        # Import the module
        Import-Module $modulePath -Force -ErrorAction Stop

        # Configure PSReadLine for predictions
        Set-NudgePredictionOptions -ViewStyle InlineView

        return $true
    } catch {
        Write-Warning "Failed to initialize auto mode: $_"
        return $false
    }
}

# ============================================================================
# Key Bindings
# ============================================================================

# Register manual mode key handler (always available)
Set-PSReadLineKeyHandler -Chord "Ctrl+e" -ScriptBlock { Invoke-NudgeComplete }

# ============================================================================
# Initialization
# ============================================================================

$autoModeEnabled = $false

if ($script:NudgeTriggerMode -eq "auto") {
    if (Test-NudgeAutoModeSupport) {
        $autoModeEnabled = Initialize-NudgeAutoMode
    } else {
        Write-Warning "Auto mode requires PowerShell 7.2+ with PSReadLine 2.2.0+"
        Write-Warning "Falling back to manual mode. Press Ctrl+E to trigger completion."
    }
}

# Print success message
$modeMsg = if ($autoModeEnabled) { "auto mode" } else { "manual mode (Ctrl+E)" }
if ($script:NudgeDiagnosisEnabled) {
    $modeMsg = "$modeMsg + error diagnosis"
}
Write-Host "Nudge loaded ($modeMsg)." -ForegroundColor Green
