# Nudge Predictor Module for PowerShell 7.2+
# Provides LLM-powered command prediction using PSReadLine's predictor API

using namespace System.Management.Automation
using namespace System.Management.Automation.Subsystem
using namespace System.Management.Automation.Subsystem.Prediction
using namespace System.Collections.Generic
using namespace System.Threading

# Check if we're running on PowerShell 7.2+ with predictor support
if ($PSVersionTable.PSVersion.Major -lt 7 -or
    ($PSVersionTable.PSVersion.Major -eq 7 -and $PSVersionTable.PSVersion.Minor -lt 2)) {
    Write-Warning "NudgePredictor requires PowerShell 7.2 or later. Current version: $($PSVersionTable.PSVersion)"
    return
}

# Helper to create suggestion package (workaround for namespace issues in classes)
function New-NudgeSuggestionPackage {
    param([string]$Suggestion)
    if ([string]::IsNullOrEmpty($Suggestion)) {
        # Return default struct value - equivalent to C# "return default;"
        return [Activator]::CreateInstance([System.Management.Automation.Subsystem.Prediction.SuggestionPackage])
    }
    $list = [System.Collections.Generic.List[System.Management.Automation.Subsystem.Prediction.PredictiveSuggestion]]::new()
    $list.Add([System.Management.Automation.Subsystem.Prediction.PredictiveSuggestion]::new($Suggestion))
    return [System.Management.Automation.Subsystem.Prediction.SuggestionPackage]::new($list)
}
$script:NewSuggestionPackageFunc = ${function:New-NudgeSuggestionPackage}

# NudgePredictor class implementing ICommandPredictor
class NudgePredictor : ICommandPredictor {
    # Unique identifier for this predictor
    [Guid] $Id = [Guid]::new("a1b2c3d4-e5f6-7890-abcd-ef1234567890")

    # Display name
    [string] $Name = "Nudge"

    # Description
    [string] $Description = "LLM-powered command prediction"

    # Throttle settings to avoid excessive API calls
    hidden [DateTime] $LastPredictionTime = [DateTime]::MinValue
    hidden [int] $ThrottleMs = 300

    # Cache for recent predictions
    hidden [hashtable] $Cache = @{}
    hidden [int] $MaxCacheSize = 50

    # Session ID for nudge
    hidden [string] $SessionId = "pwsh-$PID"

    # Constructor
    NudgePredictor() {
        # Initialize
    }

    # Main prediction method - called by PSReadLine
    # NOTE: PSReadLine has ~20ms timeout. For slow LLM backends, use manual mode (Ctrl+E) instead.
    [SuggestionPackage] GetSuggestion(
        [PredictionClient] $client,
        [PredictionContext] $context,
        [CancellationToken] $cancellationToken
    ) {
        try {
            # Get current input
            $input = $context.InputAst.Extent.Text

            # Skip if input is too short
            if ([string]::IsNullOrWhiteSpace($input) -or $input.Length -lt 2) {
                return (& $script:NewSuggestionPackageFunc)
            }

            # Check cache first
            if ($this.Cache.ContainsKey($input)) {
                return (& $script:NewSuggestionPackageFunc -Suggestion $this.Cache[$input])
            }

            # Check if any cached result starts with current input (prefix match)
            foreach ($key in $this.Cache.Keys) {
                $cachedValue = $this.Cache[$key]
                if ($cachedValue.StartsWith($input) -and $cachedValue -ne $input) {
                    return (& $script:NewSuggestionPackageFunc -Suggestion $cachedValue)
                }
            }

            # Throttle requests
            $now = [DateTime]::Now
            if (($now - $this.LastPredictionTime).TotalMilliseconds -lt $this.ThrottleMs) {
                return (& $script:NewSuggestionPackageFunc)
            }
            $this.LastPredictionTime = $now

            # Check for cancellation
            if ($cancellationToken.IsCancellationRequested) {
                return (& $script:NewSuggestionPackageFunc)
            }

            # Call nudge complete
            $cwd = (Get-Location).Path
            $cursor = $context.CursorPosition.Offset

            $result = $null
            try {
                $result = & nudge complete `
                    --format json `
                    --buffer $input `
                    --cursor $cursor `
                    --cwd $cwd `
                    --session $this.SessionId `
                    --shell-mode "ps-auto" `
                    --time-bucket ([Math]::Floor([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() / 2000)) `
                    2>$null | ConvertFrom-Json
            } catch {
                return (& $script:NewSuggestionPackageFunc)
            }

            # Check if we got a valid suggestion
            # API returns: { "suggestions": [{ "text": "..." }], ... }
            if ($null -ne $result -and $null -ne $result.suggestions -and $result.suggestions.Count -gt 0) {
                $suggestionText = $result.suggestions[0].text

                # Only return if suggestion starts with input
                if (-not [string]::IsNullOrEmpty($suggestionText) -and $suggestionText -ne $input -and $suggestionText.StartsWith($input)) {
                    $this.AddToCache($input, $suggestionText)
                    return (& $script:NewSuggestionPackageFunc -Suggestion $suggestionText)
                }
            }

        } catch {
            # Silently fail - don't interrupt user's workflow
        }

        return (& $script:NewSuggestionPackageFunc)
    }

    # Add to cache with size limit
    hidden [void] AddToCache([string] $key, [string] $value) {
        if ($this.Cache.Count -ge $this.MaxCacheSize) {
            # Simple eviction: clear cache when full
            $this.Cache.Clear()
        }
        $this.Cache[$key] = $value
    }

    # Whether this predictor can accept feedback
    [bool] CanAcceptFeedback([PredictionClient] $client, [PredictorFeedbackKind] $feedback) {
        return $false
    }

    # Called when a suggestion is displayed (not used)
    [void] OnSuggestionDisplayed(
        [PredictionClient] $client,
        [uint64] $session,
        [int] $countOrIndex
    ) {
        # Not implemented
    }

    # Called when a suggestion is accepted (not used)
    [void] OnSuggestionAccepted(
        [PredictionClient] $client,
        [uint64] $session,
        [string] $acceptedSuggestion
    ) {
        # Not implemented
    }

    # Called when command line is accepted (not used)
    [void] OnCommandLineAccepted(
        [PredictionClient] $client,
        [IReadOnlyList[string]] $history
    ) {
        # Not implemented
    }

    # Called when command line is executed (not used)
    [void] OnCommandLineExecuted(
        [PredictionClient] $client,
        [string] $commandLine,
        [bool] $success
    ) {
        # Not implemented
    }
}

# Register the predictor
function Register-NudgePredictor {
    [CmdletBinding()]
    param()

    try {
        # Check if already registered
        $existing = [SubsystemManager]::GetSubsystemInfo([SubsystemKind]::CommandPredictor) |
            Where-Object { $_.Name -eq 'Nudge' }

        if ($existing) {
            Write-Verbose "NudgePredictor is already registered"
            return
        }

        # Create and register predictor
        $predictor = [NudgePredictor]::new()
        [SubsystemManager]::RegisterSubsystem([SubsystemKind]::CommandPredictor, $predictor)

        Write-Verbose "NudgePredictor registered successfully"
    } catch {
        Write-Warning "Failed to register NudgePredictor: $_"
    }
}

# Unregister the predictor
function Unregister-NudgePredictor {
    [CmdletBinding()]
    param()

    try {
        $existing = [SubsystemManager]::GetSubsystemInfo([SubsystemKind]::CommandPredictor) |
            Where-Object { $_.Name -eq 'Nudge' }

        if ($existing) {
            [SubsystemManager]::UnregisterSubsystem([SubsystemKind]::CommandPredictor, $existing.Id)
            Write-Verbose "NudgePredictor unregistered successfully"
        }
    } catch {
        Write-Warning "Failed to unregister NudgePredictor: $_"
    }
}

# Configure PSReadLine for optimal prediction experience
function Set-NudgePredictionOptions {
    [CmdletBinding()]
    param(
        [ValidateSet('InlineView', 'ListView')]
        [string] $ViewStyle = 'InlineView',

        [string] $PredictionColor = '#808080'
    )

    try {
        # Enable prediction from history and plugins
        Set-PSReadLineOption -PredictionSource HistoryAndPlugin

        # Set view style
        Set-PSReadLineOption -PredictionViewStyle $ViewStyle

        # Set prediction color
        Set-PSReadLineOption -Colors @{
            InlinePrediction = $PredictionColor
        }

        # Configure key bindings for accepting predictions
        Set-PSReadLineKeyHandler -Key Tab -Function AcceptSuggestion
        Set-PSReadLineKeyHandler -Key RightArrow -Function ForwardChar
        Set-PSReadLineKeyHandler -Key Ctrl+RightArrow -Function AcceptNextSuggestionWord

        Write-Verbose "PSReadLine prediction options configured"
    } catch {
        Write-Warning "Failed to configure PSReadLine options: $_"
    }
}

# Auto-register on module import
Register-NudgePredictor

# Export functions
Export-ModuleMember -Function @(
    'Register-NudgePredictor',
    'Unregister-NudgePredictor',
    'Set-NudgePredictionOptions'
)
