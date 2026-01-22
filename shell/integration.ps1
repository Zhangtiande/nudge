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
    }
}

$script:NudgeLastExitCode = 0

# Capture exit codes
function global:Invoke-NudgeCaptureExitCode {
    $script:NudgeLastExitCode = $LASTEXITCODE
}

# Register prompt hook if not already registered
if (-not $global:NudgePromptHookRegistered) {
    $existingPrompt = Get-Content Function:\prompt -ErrorAction SilentlyContinue
    if ($existingPrompt) {
        $newPrompt = @"
Invoke-NudgeCaptureExitCode
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

# Main completion function
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

# Register key handler
Set-PSReadLineKeyHandler -Chord "Ctrl+e" -ScriptBlock { Invoke-NudgeComplete }

# Print success message
Write-Host "Nudge loaded. Press Ctrl+E to trigger completion." -ForegroundColor Green
