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
$script:NudgeLastHistoryId = 0
$script:NudgeDiagnosisSuggestion = ""

# Interactive commands list (loaded from config, cached for performance)
$script:NudgeInteractiveCommands = @()
try {
    $interactiveCmds = nudge info --field interactive_commands 2>$null
    if ($interactiveCmds) {
        $script:NudgeInteractiveCommands = $interactiveCmds -split ','
    }
} catch {}
# Fallback if nudge info fails
if ($script:NudgeInteractiveCommands.Count -eq 0) {
    $script:NudgeInteractiveCommands = @(
        'vim', 'nvim', 'vi', 'nano', 'emacs', 'code',
        'ssh', 'telnet', 'mosh',
        'top', 'htop', 'btop', 'less', 'more', 'man',
        'fzf', 'sk',
        'tmux', 'screen',
        'python', 'python3', 'ipython', 'node', 'irb', 'psql', 'mysql', 'sqlite3',
        'watch', 'tail'
    )
}

# ============================================================================
# Helper Functions
# ============================================================================

# Check if running in interactive mode (to avoid breaking scp, rsync, etc.)
function Test-InteractiveSession {
    return [Environment]::UserInteractive -and $Host.UI.RawUI -and $Host.Name -ne 'Default Host'
}

# Check if command is interactive (should skip diagnosis)
function Test-InteractiveCommand {
    param([string]$Command)
    if (-not $Command) { return $false }

    # Extract the first word (command name)
    $firstWord = ($Command -split '\s+')[0]
    # Handle commands with path like C:\Program Files\vim\vim.exe
    $firstWord = Split-Path $firstWord -Leaf -ErrorAction SilentlyContinue
    if (-not $firstWord) { $firstWord = ($Command -split '\s+')[0] }
    # Remove .exe extension if present
    $firstWord = $firstWord -replace '\.exe$', ''

    return $script:NudgeInteractiveCommands -contains $firstWord
}

# ============================================================================
# Error Diagnosis Functions
# ============================================================================

function global:Invoke-NudgeDiagnosis {
    if (-not $script:NudgeDiagnosisEnabled) { return }

    # Clear previous suggestion
    $script:NudgeDiagnosisSuggestion = ""

    $currentErrorCount = $Global:Error.Count
    $currentExitCode = $script:NudgeLastExitCode

    # Get last command from history
    $lastHistory = Get-History -Count 1 -ErrorAction SilentlyContinue
    $lastHistoryId = if ($lastHistory) { $lastHistory.Id } else { -1 }
    $lastCommand = if ($lastHistory) { $lastHistory.CommandLine } else { "" }

    # Skip if same command (deduplication for interactive sessions)
    if ($lastHistoryId -gt 0 -and $lastHistoryId -eq $script:NudgeLastHistoryId) {
        return
    }
    if ($lastHistoryId -gt 0) {
        $script:NudgeLastHistoryId = $lastHistoryId
    }

    # Skip diagnosis for interactive commands
    if ($lastCommand -and (Test-InteractiveCommand $lastCommand)) {
        return
    }

    # Check for PowerShell errors (cmdlet exceptions)
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
                    --exit-code $(if ($currentExitCode) { $currentExitCode } else { 1 }) `
                    --command "$($lastError.InvocationInfo.Line)" `
                    --error-record $errorContext `
                    --cwd (Get-Location).Path `
                    --session "pwsh-$PID" `
                    --format plain 2>$null

                if ($LASTEXITCODE -eq 0 -and $diagnosis) {
                    Show-NudgeDiagnosis $diagnosis
                }
            } catch {
                # Silently ignore diagnosis errors
            }
        }

        $script:NudgeLastErrorCount = $currentErrorCount
        return
    }

    # Check for external command failures (non-zero exit code)
    if ($currentExitCode -ne 0 -and $currentExitCode -ne $null) {
        Start-NudgeDaemonIfNeeded

        # Use command from history if available, otherwise use a placeholder
        $cmdToSend = if ($lastCommand) { $lastCommand } else { "(unknown command)" }

        try {
            $diagnosis = & nudge diagnose `
                --exit-code $currentExitCode `
                --command $cmdToSend `
                --cwd (Get-Location).Path `
                --session "pwsh-$PID" `
                --format plain 2>$null

            if ($LASTEXITCODE -eq 0 -and $diagnosis) {
                Show-NudgeDiagnosis $diagnosis
            }
        } catch {
            # Silently ignore diagnosis errors
        }
    }

    $script:NudgeLastErrorCount = $currentErrorCount
}

# Display diagnosis and store suggestion for Tab completion
function global:Show-NudgeDiagnosis {
    param($DiagnosisOutput)

    # Handle array output (PowerShell captures multi-line output as array)
    if ($DiagnosisOutput -is [array]) {
        $message = $DiagnosisOutput[0]
        $suggestion = if ($DiagnosisOutput.Count -gt 1) { $DiagnosisOutput[1].Trim() } else { "" }
    } else {
        # Handle string output
        $lines = $DiagnosisOutput -split "`n"
        $message = $lines[0]
        $suggestion = if ($lines.Count -gt 1) { $lines[1].Trim() } else { "" }
    }

    # Display diagnosis message
    if ($message) {
        Write-Host $message -ForegroundColor Yellow
    }

    # Store and display suggestion with hint
    if ($suggestion) {
        $script:NudgeDiagnosisSuggestion = $suggestion
        Write-Host "  Suggested fix: " -ForegroundColor DarkGray -NoNewline
        Write-Host $suggestion -ForegroundColor Cyan -NoNewline
        Write-Host " (press Tab to accept)" -ForegroundColor DarkGray
    }
}

# Accept diagnosis suggestion
function global:Invoke-NudgeAcceptSuggestion {
    if ($script:NudgeDiagnosisSuggestion) {
        [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
        [Microsoft.PowerShell.PSConsoleReadLine]::Insert($script:NudgeDiagnosisSuggestion)
        $script:NudgeDiagnosisSuggestion = ""
    } else {
        # Fall back to default Tab behavior
        [Microsoft.PowerShell.PSConsoleReadLine]::TabCompleteNext()
    }
}

# Register prompt hook
# Always update to ensure latest version is used
function global:_NudgePromptHook {
    # Capture exit code FIRST before anything else runs
    $script:NudgeLastExitCode = $global:LASTEXITCODE
    Invoke-NudgeDiagnosis
}

if (-not $global:NudgePromptHookRegistered) {
    # First time registration - wrap existing prompt
    $script:NudgeOriginalPrompt = Get-Content Function:\prompt -ErrorAction SilentlyContinue
    if ($script:NudgeOriginalPrompt) {
        $newPrompt = @'
_NudgePromptHook
'@ + "`n" + $script:NudgeOriginalPrompt
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
            --shell-mode "ps-inline" `
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
        if (Test-InteractiveSession) {
            Write-Warning "NudgePredictor module not found. Auto mode disabled."
            Write-Warning "Install the module or use manual mode (Ctrl+E)."
        }
        return $false
    }

    try {
        # Import the module
        Import-Module $modulePath -Force -ErrorAction Stop

        # Configure PSReadLine for predictions
        Set-NudgePredictionOptions -ViewStyle InlineView

        return $true
    } catch {
        if (Test-InteractiveSession) {
            Write-Warning "Failed to initialize auto mode: $_"
        }
        return $false
    }
}

# ============================================================================
# Key Bindings
# ============================================================================

# Register manual mode key handler (always available)
Set-PSReadLineKeyHandler -Chord "Ctrl+e" -ScriptBlock { Invoke-NudgeComplete }

# Register Tab handler for diagnosis suggestion acceptance
Set-PSReadLineKeyHandler -Chord "Tab" -ScriptBlock { Invoke-NudgeAcceptSuggestion }

# ============================================================================
# Initialization
# ============================================================================

$autoModeEnabled = $false

if ($script:NudgeTriggerMode -eq "auto") {
    if (Test-NudgeAutoModeSupport) {
        $autoModeEnabled = Initialize-NudgeAutoMode
    } else {
        if (Test-InteractiveSession) {
            Write-Warning "Auto mode requires PowerShell 7.2+ with PSReadLine 2.2.0+"
            Write-Warning "Falling back to manual mode. Press Ctrl+E to trigger completion."
        }
    }
}

# Print success message (only in interactive sessions)
if (Test-InteractiveSession) {
    $modeMsg = if ($autoModeEnabled) { "auto mode" } else { "manual mode (Ctrl+E)" }
    if ($script:NudgeDiagnosisEnabled) {
        $modeMsg = "$modeMsg + error diagnosis"
    }
    Write-Host "Nudge loaded ($modeMsg)." -ForegroundColor Green
}
