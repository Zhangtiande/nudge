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
                return [SuggestionPackage]::new()
            }

            # Throttle requests
            $now = [DateTime]::Now
            if (($now - $this.LastPredictionTime).TotalMilliseconds -lt $this.ThrottleMs) {
                # Return cached result if available
                if ($this.Cache.ContainsKey($input)) {
                    $cached = $this.Cache[$input]
                    $suggestion = [PredictiveSuggestion]::new($cached)
                    return [SuggestionPackage]::new(@($suggestion))
                }
                return [SuggestionPackage]::new()
            }
            $this.LastPredictionTime = $now

            # Check cache first
            if ($this.Cache.ContainsKey($input)) {
                $cached = $this.Cache[$input]
                $suggestion = [PredictiveSuggestion]::new($cached)
                return [SuggestionPackage]::new(@($suggestion))
            }

            # Check for cancellation
            if ($cancellationToken.IsCancellationRequested) {
                return [SuggestionPackage]::new()
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
                    2>$null | ConvertFrom-Json
            } catch {
                return [SuggestionPackage]::new()
            }

            # Check if we got a valid suggestion
            if ($null -ne $result -and -not [string]::IsNullOrEmpty($result.suggestion)) {
                $suggestionText = $result.suggestion

                # Only return if suggestion is different from input
                if ($suggestionText -ne $input -and $suggestionText.StartsWith($input)) {
                    # Cache the result
                    $this.AddToCache($input, $suggestionText)

                    $suggestion = [PredictiveSuggestion]::new($suggestionText)
                    return [SuggestionPackage]::new(@($suggestion))
                }
            }
        } catch {
            # Silently fail - don't interrupt user's workflow
        }

        return [SuggestionPackage]::new()
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
