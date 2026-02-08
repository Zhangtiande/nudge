# Nudge - PowerShell Integration
# Installed by: nudge setup powershell

# Get configuration from nudge CLI
$script:NudgeInfo = @{}
try {
    $infoJson = nudge info --json 2>$null | ConvertFrom-Json
    $script:NudgeInfo = $infoJson
} catch {
    # Fallback if nudge not in PATH
    $homeDir = if ($env:HOME) { $env:HOME } else { $env:USERPROFILE }
    $script:NudgeInfo = @{
        config_dir = Join-Path $homeDir ".nudge"
        socket_path = "\\.\pipe\nudge_$env:USERNAME"
        trigger_mode = "manual"
        auto_delay_ms = 500
    }
}

$script:NudgeScriptDir = $PSScriptRoot
$script:NudgeLastExitCode = 0
$script:NudgeTriggerMode = if ($script:NudgeInfo.trigger_mode) { $script:NudgeInfo.trigger_mode } else { "manual" }
$script:NudgeWarningPrefix = "NUDGE_WARNING:"
$script:NudgePopupKey = if ($env:NUDGE_PS_POPUP_KEY) {
    $env:NUDGE_PS_POPUP_KEY
} elseif ($env:NUDGE_POPUP_KEY) {
    $env:NUDGE_POPUP_KEY
} else {
    "Alt+/"
}
$script:NudgePopupBackend = if ($env:NUDGE_PS_POPUP_BACKEND) {
    $env:NUDGE_PS_POPUP_BACKEND
} elseif ($env:NUDGE_POPUP_BACKEND) {
    $env:NUDGE_POPUP_BACKEND
} else {
    "auto"
}
$script:NudgePopupHeight = if ($env:NUDGE_PS_POPUP_HEIGHT) {
    $env:NUDGE_PS_POPUP_HEIGHT
} elseif ($env:NUDGE_POPUP_HEIGHT) {
    $env:NUDGE_POPUP_HEIGHT
} else {
    "70%"
}
$script:NudgePopupEnabled = if ($env:NUDGE_PS_POPUP_ENABLED) {
    $env:NUDGE_PS_POPUP_ENABLED
} elseif ($env:NUDGE_POPUP_ENABLED) {
    $env:NUDGE_POPUP_ENABLED
} else {
    "1"
}
$script:NudgePopupConfirmRisky = if ($env:NUDGE_PS_POPUP_CONFIRM_RISKY) {
    $env:NUDGE_PS_POPUP_CONFIRM_RISKY
} elseif ($env:NUDGE_POPUP_CONFIRM_RISKY) {
    $env:NUDGE_POPUP_CONFIRM_RISKY
} else {
    "1"
}
$script:NudgePopupKeyEffective = ""
$script:NudgePopupRegistered = $false

# Diagnosis state
$script:NudgeDiagnosisEnabled = $false
try {
    $diagEnabled = nudge info --field diagnosis_enabled 2>$null
    $script:NudgeDiagnosisEnabled = $diagEnabled -eq "true"
} catch {}

# NudgeLastErrorCount is initialized at the end of the script to avoid
# false positives from errors generated during loading (e.g. Get-Command,
# nudge info try/catch blocks).
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

# Parse env-style feature flags.
function Test-NudgeFlag {
    param(
        [string]$Value,
        [bool]$Default = $true
    )

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return $Default
    }

    switch ($Value.Trim().ToLowerInvariant()) {
        "0" { return $false }
        "false" { return $false }
        "off" { return $false }
        "no" { return $false }
        default { return $true }
    }
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

# Display warning message for dangerous suggestions
function global:Show-NudgeWarning {
    param([string]$Message)
    if ($Message) {
        Write-Host "Warning: $Message" -ForegroundColor Yellow
    }
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
    if ($currentExitCode -ne 0 -and $null -ne $currentExitCode) {
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
        $null = nudge status 2>$null
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
            if ($suggestion.StartsWith($script:NudgeWarningPrefix)) {
                $warningMessage = $suggestion.Substring($script:NudgeWarningPrefix.Length).TrimStart()
                Show-NudgeWarning $warningMessage
                return
            }
            [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert($suggestion)
        }
    } catch {
        # Silently ignore errors
    }
}

# ============================================================================
# Popup Selector (manual multi-candidate mode)
# ============================================================================

function global:Set-NudgeBufferText {
    param([string]$Text)

    if ([string]::IsNullOrEmpty($Text)) { return }

    # Use RevertLine + Insert for maximum compatibility
    # SetBufferState may not exist in all PSReadLine versions
    [Microsoft.PowerShell.PSConsoleReadLine]::RevertLine()
    [Microsoft.PowerShell.PSConsoleReadLine]::Insert($Text)
}

function global:Get-NudgePopupCandidates {
    param(
        [string]$Buffer,
        [int]$Cursor
    )

    try {
        $rows = & nudge complete `
            --format list `
            --buffer $Buffer `
            --cursor $Cursor `
            --cwd (Get-Location).Path `
            --session "pwsh-$PID" `
            --shell-mode "ps-popup" `
            --last-exit-code $script:NudgeLastExitCode `
            2>$null
    } catch {
        return @()
    }

    if ($LASTEXITCODE -ne 0 -or -not $rows) {
        return @()
    }

    if ($rows -is [string]) {
        $rows = $rows -split "`r?`n"
    }

    return @($rows | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
}

function global:ConvertFrom-NudgeListRow {
    param([string]$Row)

    $parts = @($Row -split "`t", 5)
    while ($parts.Count -lt 5) {
        $parts += ""
    }

    return [PSCustomObject]@{
        Risk    = $parts[0]
        Command = $parts[1]
        Warning = $parts[2]
        Why     = $parts[3]
        Diff    = $parts[4]
    }
}

function global:Get-NudgeNumberedRows {
    param([string[]]$Rows)

    $numbered = New-Object System.Collections.Generic.List[string]
    for ($i = 0; $i -lt $Rows.Count; $i++) {
        $numbered.Add(("{0}`t{1}" -f ($i + 1), $Rows[$i]))
    }
    return $numbered
}

function global:Resolve-NudgePopupKey {
    $resolved = "$script:NudgePopupKey".Trim()
    if ([string]::IsNullOrWhiteSpace($resolved)) {
        return "Alt+/"
    }
    return $resolved
}

function global:Read-NudgePopupChoice {
    param([int]$MaxIndex)

    if ($MaxIndex -lt 1) { return $null }

    while ($true) {
        try {
            $keyInfo = [Console]::ReadKey($true)
        } catch {
            return $null
        }

        if ($keyInfo.Key -eq [ConsoleKey]::Enter -or $keyInfo.Key -eq [ConsoleKey]::Escape) {
            Write-Host ""
            return $null
        }

        $keyChar = $keyInfo.KeyChar
        if ($keyChar -eq 'q' -or $keyChar -eq 'Q') {
            Write-Host ""
            return $null
        }

        if ($keyChar -ge '1' -and $keyChar -le '9') {
            [int]$index = [int]([string]$keyChar)
            Write-Host $keyChar
            if ($index -ge 1 -and $index -le $MaxIndex) {
                return $index
            }
            return $null
        }
    }
}

function global:Read-NudgePopupConfirmation {
    param([string]$Prompt = "Apply high-risk suggestion? [y/N]")

    Write-Host $Prompt -NoNewline
    try {
        $keyInfo = [Console]::ReadKey($true)
    } catch {
        Write-Host ""
        return $false
    }

    if ($keyInfo.Key -eq [ConsoleKey]::Enter -or $keyInfo.Key -eq [ConsoleKey]::Escape) {
        Write-Host ""
        return $false
    }

    Write-Host $keyInfo.KeyChar
    return $keyInfo.KeyChar -eq 'y' -or $keyInfo.KeyChar -eq 'Y'
}

function global:Resolve-NudgePopupBackend {
    $preferred = "$script:NudgePopupBackend".Trim().ToLowerInvariant()
    if ([string]::IsNullOrWhiteSpace($preferred)) {
        $preferred = "auto"
    }

    # Use -ErrorAction Ignore (not SilentlyContinue) to prevent adding
    # error records to $Error, which would trigger false diagnosis.
    $hasFzf = $null -ne (Get-Command fzf -ErrorAction Ignore)
    $hasSk = $null -ne (Get-Command sk -ErrorAction Ignore)
    $hasPeco = $null -ne (Get-Command peco -ErrorAction Ignore)
    $hasConsoleGrid = $null -ne (Get-Command Out-ConsoleGridView -ErrorAction Ignore)
    $hasGridView = $null -ne (Get-Command Out-GridView -ErrorAction Ignore)

    switch ($preferred) {
        "fzf" { if ($hasFzf) { return "fzf" } }
        "sk" { if ($hasSk) { return "sk" } }
        "peco" { if ($hasPeco) { return "peco" } }
        "consolegrid" { if ($hasConsoleGrid) { return "consolegrid" } }
        "gridview" { if ($hasGridView) { return "gridview" } }
        "builtin" { return "builtin" }
        default {}
    }

    if ($hasFzf) { return "fzf" }
    if ($hasSk) { return "sk" }
    if ($hasPeco) { return "peco" }
    if ($hasConsoleGrid) { return "consolegrid" }
    if ($hasGridView) { return "gridview" }
    return "builtin"
}

function global:Select-NudgeCandidateBuiltin {
    param([string[]]$CandidateRows)

    if (-not $CandidateRows -or $CandidateRows.Count -eq 0) { return $null }

    Write-Host ""
    for ($i = 0; $i -lt $CandidateRows.Count; $i++) {
        $entry = ConvertFrom-NudgeListRow $CandidateRows[$i]
        Write-Host ("  {0}) [{1}] {2}" -f ($i + 1), $entry.Risk, $entry.Command)
        if ($entry.Why) { Write-Host ("     why: {0}" -f $entry.Why) }
        if ($entry.Diff) { Write-Host ("     diff: {0}" -f $entry.Diff) }
        if ($entry.Warning) { Write-Host ("     warn: {0}" -f $entry.Warning) }
    }

    Write-Host ("Select candidate [1-{0}, Enter/q/Esc to cancel]: " -f $CandidateRows.Count) -NoNewline
    $index = Read-NudgePopupChoice -MaxIndex $CandidateRows.Count
    if (-not $index) { return $null }

    return (ConvertFrom-NudgeListRow $CandidateRows[$index - 1])
}

function global:Select-NudgeCandidateFzf {
    param([string[]]$CandidateRows)

    if (-not $CandidateRows -or $CandidateRows.Count -eq 0) { return $null }

    $numbered = Get-NudgeNumberedRows -Rows $CandidateRows
    $selected = $numbered | & fzf `
        --height $script:NudgePopupHeight `
        --layout reverse `
        --border `
        --header "enter: apply  esc: cancel" `
        --prompt "nudge> " `
        --delimiter "`t" `
        --with-nth "1,2,3,5" 2>$null

    if (-not $selected) { return $null }
    if ($selected -is [array]) { $selected = $selected[0] }

    [int]$index = 0
    $first = ($selected -split "`t", 2)[0]
    if (-not [int]::TryParse($first, [ref]$index)) { return $null }
    if ($index -lt 1 -or $index -gt $CandidateRows.Count) { return $null }

    return (ConvertFrom-NudgeListRow $CandidateRows[$index - 1])
}

function global:Select-NudgeCandidateSk {
    param([string[]]$CandidateRows)

    if (-not $CandidateRows -or $CandidateRows.Count -eq 0) { return $null }

    $numbered = Get-NudgeNumberedRows -Rows $CandidateRows
    $selected = $numbered | & sk `
        --height $script:NudgePopupHeight `
        --prompt "nudge> " `
        --delimiter "`t" `
        --with-nth "1,2,3,5" 2>$null

    if (-not $selected) { return $null }
    if ($selected -is [array]) { $selected = $selected[0] }

    [int]$index = 0
    $first = ($selected -split "`t", 2)[0]
    if (-not [int]::TryParse($first, [ref]$index)) { return $null }
    if ($index -lt 1 -or $index -gt $CandidateRows.Count) { return $null }

    return (ConvertFrom-NudgeListRow $CandidateRows[$index - 1])
}

function global:Select-NudgeCandidatePeco {
    param([string[]]$CandidateRows)

    if (-not $CandidateRows -or $CandidateRows.Count -eq 0) { return $null }

    $numbered = Get-NudgeNumberedRows -Rows $CandidateRows
    $selected = $numbered | & peco --prompt "nudge> " 2>$null

    if (-not $selected) { return $null }
    if ($selected -is [array]) { $selected = $selected[0] }

    [int]$index = 0
    $first = ($selected -split "`t", 2)[0]
    if (-not [int]::TryParse($first, [ref]$index)) { return $null }
    if ($index -lt 1 -or $index -gt $CandidateRows.Count) { return $null }

    return (ConvertFrom-NudgeListRow $CandidateRows[$index - 1])
}

function global:Select-NudgeCandidateConsoleGrid {
    param([string[]]$CandidateRows)

    if (-not $CandidateRows -or $CandidateRows.Count -eq 0) { return $null }

    $items = for ($i = 0; $i -lt $CandidateRows.Count; $i++) {
        $entry = ConvertFrom-NudgeListRow $CandidateRows[$i]
        [PSCustomObject]@{
            Index   = $i + 1
            Risk    = $entry.Risk
            Command = $entry.Command
            Why     = $entry.Why
            Diff    = $entry.Diff
            Warning = $entry.Warning
        }
    }

    try {
        $selected = $items | Out-ConsoleGridView -Title "nudge popup" -OutputMode Single
    } catch {
        return $null
    }

    if (-not $selected) { return $null }

    [int]$index = [int]$selected.Index
    if ($index -lt 1 -or $index -gt $CandidateRows.Count) { return $null }

    return (ConvertFrom-NudgeListRow $CandidateRows[$index - 1])
}

function global:Select-NudgeCandidateGridView {
    param([string[]]$CandidateRows)

    if (-not $CandidateRows -or $CandidateRows.Count -eq 0) { return $null }

    $items = for ($i = 0; $i -lt $CandidateRows.Count; $i++) {
        $entry = ConvertFrom-NudgeListRow $CandidateRows[$i]
        [PSCustomObject]@{
            Index   = $i + 1
            Risk    = $entry.Risk
            Command = $entry.Command
            Why     = $entry.Why
            Diff    = $entry.Diff
            Warning = $entry.Warning
        }
    }

    try {
        $selected = $items | Out-GridView -Title "nudge popup" -PassThru
    } catch {
        return $null
    }

    if (-not $selected) { return $null }
    if ($selected -is [array]) { $selected = $selected[0] }

    [int]$index = [int]$selected.Index
    if ($index -lt 1 -or $index -gt $CandidateRows.Count) { return $null }

    return (ConvertFrom-NudgeListRow $CandidateRows[$index - 1])
}

function global:Invoke-NudgePopupComplete {
    Start-NudgeDaemonIfNeeded

    $buffer = $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$buffer, [ref]$cursor)

    if ([string]::IsNullOrEmpty($buffer)) { return }

    $candidateRows = Get-NudgePopupCandidates -Buffer $buffer -Cursor $cursor
    if (-not $candidateRows -or $candidateRows.Count -eq 0) {
        Invoke-NudgeComplete
        return
    }

    $backend = Resolve-NudgePopupBackend
    $selected = $null
    switch ($backend) {
        "fzf" { $selected = Select-NudgeCandidateFzf -CandidateRows $candidateRows }
        "sk" { $selected = Select-NudgeCandidateSk -CandidateRows $candidateRows }
        "peco" { $selected = Select-NudgeCandidatePeco -CandidateRows $candidateRows }
        "consolegrid" { $selected = Select-NudgeCandidateConsoleGrid -CandidateRows $candidateRows }
        "gridview" { $selected = Select-NudgeCandidateGridView -CandidateRows $candidateRows }
        default { $selected = Select-NudgeCandidateBuiltin -CandidateRows $candidateRows }
    }

    if (-not $selected -or [string]::IsNullOrWhiteSpace($selected.Command)) {
        return
    }

    $alreadyWarned = $false
    if ($selected.Risk -eq "high") {
        $warningMessage = if ($selected.Warning) { $selected.Warning } else { "This suggestion is marked high risk." }
        Show-NudgeWarning $warningMessage
        $alreadyWarned = $true

        if (Test-NudgeFlag -Value $script:NudgePopupConfirmRisky -Default $true) {
            if (-not (Read-NudgePopupConfirmation)) {
                return
            }
        }
    }

    Set-NudgeBufferText -Text $selected.Command

    if (-not $alreadyWarned -and -not [string]::IsNullOrWhiteSpace($selected.Warning)) {
        Show-NudgeWarning $selected.Warning
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
    $homeDir = if ($env:HOME) { $env:HOME } else { $env:USERPROFILE }
    $configDir = if ($script:NudgeInfo.config_dir) { $script:NudgeInfo.config_dir } else { Join-Path $homeDir ".nudge" }
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
    # Use $script:NudgeScriptDir captured at load time; $MyInvocation.MyCommand.Path
    # is empty inside functions when the script was dot-sourced.
    if (-not $modulePath) {
        $scriptDir = $script:NudgeScriptDir
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

function global:Register-NudgePopupKeyHandler {
    if (-not (Test-NudgeFlag -Value $script:NudgePopupEnabled -Default $true)) {
        $script:NudgePopupKeyEffective = ""
        return $false
    }

    $popupKey = Resolve-NudgePopupKey
    try {
        Set-PSReadLineKeyHandler -Chord $popupKey -ScriptBlock { Invoke-NudgePopupComplete }
        $script:NudgePopupKeyEffective = $popupKey
        return $true
    } catch {
        $script:NudgePopupKeyEffective = ""
        if (Test-InteractiveSession) {
            Write-Warning "Failed to register popup key '$popupKey'. Popup selector disabled."
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
$script:NudgePopupRegistered = Register-NudgePopupKeyHandler

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
    if ($script:NudgePopupRegistered -and $script:NudgePopupKeyEffective) {
        $modeMsg = "$modeMsg + popup ($($script:NudgePopupKeyEffective))"
    }
    if ($script:NudgeDiagnosisEnabled) {
        $modeMsg = "$modeMsg + error diagnosis"
    }
    Write-Host "Nudge loaded ($modeMsg)." -ForegroundColor Green
}

# Synchronize error count AFTER all initialization is done so that errors
# generated during loading (e.g. nudge info try/catch, module imports) are
# not mistaken for user command failures by the diagnosis hook.
$script:NudgeLastErrorCount = $Global:Error.Count
