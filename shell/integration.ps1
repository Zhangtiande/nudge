# Nudge - PowerShell Integration
# Add this to your $PROFILE to enable Nudge completion
# . "path\to\integration.ps1"

# Configuration
$script:NudgeHotkey = if ($env:NUDGE_HOTKEY) { $env:NUDGE_HOTKEY } else { "Ctrl+e" }
$script:NudgeLastExitCode = 0

# Capture last exit code after each command
$script:NudgePromptHook = {
    $script:NudgeLastExitCode = $LASTEXITCODE
}

# Register prompt hook if not already registered
if (-not $global:NudgePromptHookRegistered) {
    $existingPrompt = Get-Content Function:\prompt -ErrorAction SilentlyContinue
    if ($existingPrompt) {
        $newPrompt = @"
`$script:NudgeLastExitCode = `$LASTEXITCODE
$existingPrompt
"@
        Set-Content Function:\prompt -Value $newPrompt
    }
    $global:NudgePromptHookRegistered = $true
}

# Ensure daemon is running
function Start-NudgeDaemon {
    $pipeName = "\\.\pipe\nudge_$env:USERNAME"
    
    # Check if Named Pipe exists (daemon is running)
    $pipeExists = [System.IO.Directory]::GetFiles("\\.\pipe\", "nudge_$env:USERNAME*").Count -gt 0
    
    if (-not $pipeExists) {
        # Start daemon in background
        Start-Process -FilePath "nudge" -ArgumentList "daemon", "--fork" -WindowStyle Hidden -ErrorAction SilentlyContinue
    }
}

# Main completion function
function Invoke-NudgeComplete {
    # Ensure daemon is running
    Start-NudgeDaemon
    
    # Get current buffer state
    $buffer = $null
    $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$buffer, [ref]$cursor)
    
    if ([string]::IsNullOrEmpty($buffer)) {
        return
    }
    
    # Call nudge CLI
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
            # Replace buffer with suggestion
            [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert($suggestion)
        }
    }
    catch {
        # Silently ignore errors
    }
}

# Register the key handler
Set-PSReadLineKeyHandler -Chord $script:NudgeHotkey -ScriptBlock {
    Invoke-NudgeComplete
}

# Print success message on first load
if (-not $global:NudgeLoaded) {
    $global:NudgeLoaded = $true
    Write-Host "Nudge loaded. Press $script:NudgeHotkey to trigger completion." -ForegroundColor Green
}
