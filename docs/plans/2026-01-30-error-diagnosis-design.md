# Error Diagnosis Feature Design

**Date**: 2026-01-30
**Status**: Draft
**Target Version**: v0.5.0

## Overview

When a command fails, Nudge automatically captures error context and provides intelligent fix suggestions using LLM analysis. This feature aims to reduce the friction of debugging command-line errors.

## Goals

1. **Zero-friction error recovery**: Automatically detect command failures and provide actionable suggestions
2. **Non-intrusive**: Disabled by default; users opt-in explicitly
3. **Shell-native integration**: Leverage each shell's native error handling mechanisms
4. **Single best suggestion**: Return only the most relevant fix to avoid decision fatigue

## User Experience

### Zsh

```
$ git pul origin main
❌ Command failed (exit 127)
💡 Typo detected: 'pul' → 'pull'

git pull origin main          ← gray text, Tab to accept
$ █
```

**Flow**:
1. User executes command that fails (non-zero exit code)
2. Nudge captures stderr (redirected to temp file during execution)
3. Diagnosis displayed in place of original stderr
4. Suggested fix shown as gray inline text (reusing auto-mode mechanism)
5. User presses Tab to accept, or ignores and types new command

### PowerShell

```
PS> git pul origin main
❌ Command failed (exit 127)
💡 Typo detected: 'pul' → 'pull'

PS> █                         ← user triggers Ctrl+E for suggested fix
```

**Flow**:
1. User executes command that fails
2. Nudge reads error from `$Error[0]` automatic variable
3. Diagnosis displayed before new prompt
4. User triggers manual completion (Ctrl+E) to get suggested fix

## Technical Design

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Shell Layer                                │
├──────────────────────────────┬──────────────────────────────────────┤
│            Zsh               │           PowerShell                  │
│  ┌────────────────────────┐  │  ┌────────────────────────────────┐  │
│  │ preexec():             │  │  │ prompt function:               │  │
│  │   - Save stderr fd     │  │  │   - Check $? and $Error[0]     │  │
│  │   - Redirect stderr    │  │  │   - Extract error details      │  │
│  │     to temp file       │  │  │   - Call nudge diagnose        │  │
│  │                        │  │  │   - Display diagnosis          │  │
│  │ precmd():              │  │  │                                │  │
│  │   - Restore stderr     │  │  └────────────────────────────────┘  │
│  │   - Check exit code    │  │                                      │
│  │   - Read captured err  │  │                                      │
│  │   - Call nudge diagnose│  │                                      │
│  │   - Display diagnosis  │  │                                      │
│  │   - Set gray suggestion│  │                                      │
│  └────────────────────────┘  │                                      │
└──────────────────────────────┴──────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Nudge CLI                                    │
│  nudge diagnose --exit-code <N> --stderr-file <path>                │
│                  --command <cmd> --cwd <path>                        │
│                  [--error-record <json>]  # PowerShell only          │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼ IPC
┌─────────────────────────────────────────────────────────────────────┐
│                         Nudge Daemon                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐  │
│  │ Context Engine  │  │   Sanitizer     │  │   LLM Connector     │  │
│  │ - Failed cmd    │  │ - Redact secrets│  │ - Build diagnosis   │  │
│  │ - Exit code     │  │   from stderr   │  │   prompt            │  │
│  │ - Stderr output │  │                 │  │ - Parse single fix  │  │
│  │ - CWD / Git     │  │                 │  │                     │  │
│  │ - History       │  │                 │  │                     │  │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Zsh Implementation

#### Error Capture Mechanism

```zsh
# State variables
typeset -g _nudge_stderr_file=""
typeset -g _nudge_stderr_fd=""
typeset -g _nudge_last_command=""

# preexec - Before command execution
_nudge_diagnosis_preexec() {
    # Only if diagnosis is enabled
    [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]] && return

    _nudge_last_command="$1"
    _nudge_stderr_file="/tmp/nudge_stderr_$$"

    # Save original stderr and redirect to file
    exec {_nudge_stderr_fd}>&2
    exec 2>"$_nudge_stderr_file"
}

# precmd - After command execution
_nudge_diagnosis_precmd() {
    local exit_code=$?

    # Restore stderr immediately
    if [[ -n "$_nudge_stderr_fd" ]]; then
        exec 2>&$_nudge_stderr_fd
        exec {_nudge_stderr_fd}>&-
        _nudge_stderr_fd=""
    fi

    # Only proceed if diagnosis enabled and command failed
    [[ "$NUDGE_DIAGNOSIS_ENABLED" != "true" ]] && return
    [[ $exit_code -eq 0 ]] && return
    [[ ! -s "$_nudge_stderr_file" ]] && return

    # Call nudge diagnose
    local diagnosis
    diagnosis=$(nudge diagnose \
        --exit-code "$exit_code" \
        --stderr-file "$_nudge_stderr_file" \
        --command "$_nudge_last_command" \
        --cwd "$PWD" \
        --session "zsh-$$" \
        --format json 2>/dev/null)

    if [[ $? -eq 0 && -n "$diagnosis" ]]; then
        # Extract and display diagnosis message
        local message=$(echo "$diagnosis" | jq -r '.message // empty')
        local suggestion=$(echo "$diagnosis" | jq -r '.suggestion // empty')

        if [[ -n "$message" ]]; then
            echo "$message"
        fi

        # Set suggestion as gray inline text (reuse auto-mode)
        if [[ -n "$suggestion" ]]; then
            _nudge_auto_suggestion="$suggestion"
        fi
    fi

    # Cleanup
    rm -f "$_nudge_stderr_file"
    _nudge_stderr_file=""
    _nudge_last_command=""
}

# Register hooks
if [[ "$NUDGE_DIAGNOSIS_ENABLED" == "true" ]]; then
    preexec_functions+=(_nudge_diagnosis_preexec)
    # Insert at beginning to capture exit code before other precmd functions
    precmd_functions=(_nudge_diagnosis_precmd "${precmd_functions[@]}")
fi
```

### PowerShell Implementation

#### Error Detection in Prompt

```powershell
# Track last error count to detect new errors
$Global:_NudgeLastErrorCount = 0

function _NudgeDiagnosisPrompt {
    $currentErrorCount = $Global:Error.Count

    # Check if diagnosis is enabled and new error occurred
    if ($env:NUDGE_DIAGNOSIS_ENABLED -eq "true" -and
        $currentErrorCount -gt $Global:_NudgeLastErrorCount) {

        $lastError = $Global:Error[0]

        if ($lastError) {
            # Build error context JSON
            $errorContext = @{
                message = $lastError.Exception.Message
                command = $lastError.InvocationInfo.Line
                scriptStackTrace = $lastError.ScriptStackTrace
                category = $lastError.CategoryInfo.ToString()
                exitCode = $LASTEXITCODE
            } | ConvertTo-Json -Compress

            # Call nudge diagnose
            $diagnosis = & nudge diagnose `
                --exit-code $LASTEXITCODE `
                --error-record $errorContext `
                --cwd (Get-Location).Path `
                --session "pwsh-$PID" `
                --format plain 2>$null

            if ($LASTEXITCODE -eq 0 -and $diagnosis) {
                Write-Host $diagnosis
            }
        }
    }

    $Global:_NudgeLastErrorCount = $currentErrorCount
}

# Inject into prompt (preserve existing prompt)
$Global:_NudgeOriginalPrompt = $function:prompt
function prompt {
    _NudgeDiagnosisPrompt
    & $Global:_NudgeOriginalPrompt
}
```

### Protocol Extension

#### New Request Type: DiagnosisRequest

```rust
/// Request for error diagnosis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisRequest {
    /// Unique identifier for the shell session
    pub session_id: String,
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// The failed command text
    pub command: String,
    /// Exit code of the failed command
    pub exit_code: i32,
    /// Captured stderr output (Zsh)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_output: Option<String>,
    /// PowerShell ErrorRecord as JSON (PowerShell)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_record: Option<serde_json::Value>,
    /// Current working directory
    pub cwd: PathBuf,
}

/// Response with diagnosis and suggested fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResponse {
    /// Unique identifier for this request
    pub request_id: String,
    /// Human-readable diagnosis message
    pub message: String,
    /// Suggested fix command (single best option)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    /// Confidence score (0.0 to 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Error if diagnosis failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}
```

### CLI Extension

```bash
# New subcommand
nudge diagnose \
    --exit-code <N> \
    --command <cmd> \
    --cwd <path> \
    --session <id> \
    [--stderr-file <path>] \      # Zsh: path to captured stderr
    [--error-record <json>] \     # PowerShell: ErrorRecord JSON
    [--format plain|json]         # Output format
```

### Configuration

```yaml
# config.yaml
diagnosis:
  # Enable/disable error diagnosis feature
  # Default: false (user must opt-in)
  enabled: false

  # Zsh: capture stderr to file during command execution
  # WARNING: stderr will not display in real-time when enabled
  # Default: true (when diagnosis.enabled is true)
  capture_stderr: true

  # Zsh: show suggested fix as gray inline text
  # Default: true
  auto_suggest: true

  # Maximum stderr size to send to LLM (bytes)
  # Default: 4096
  max_stderr_size: 4096

  # Timeout for diagnosis request (ms)
  # Default: 5000
  timeout_ms: 5000
```

### LLM Prompt Design

```
You are a CLI error diagnosis assistant. Analyze the failed command and provide a fix.

Rules:
1. Return ONLY a JSON object with "diagnosis" and "suggestion" fields
2. The "diagnosis" should be a brief (1-2 sentence) explanation of the error
3. The "suggestion" should be the single most likely correct command
4. If you cannot determine a fix, return null for "suggestion"
5. Do not explain or add commentary outside the JSON

Context:
- Failed command: {command}
- Exit code: {exit_code}
- Error output: {stderr_output}
- Current directory: {cwd}
- Recent history: {history}
- Git status: {git_status}

Respond with JSON only:
{"diagnosis": "...", "suggestion": "..."}
```

## Documentation Requirements

### README Warning

Add the following caution to README.md and README_zh.md:

```markdown
## Error Diagnosis Feature

> **⚠️ Caution**: When error diagnosis is enabled in Zsh, stderr output is
> temporarily redirected to a file during command execution. This means:
> - Error messages won't display in real-time
> - After command failure, Nudge will display the captured errors along with
>   diagnosis and suggested fixes
> - Some programs that check stderr's TTY status may behave differently
>
> This feature is disabled by default. Enable with:
> ```yaml
> diagnosis:
>   enabled: true
> ```
```

## Implementation Phases

### Phase 1: Core Infrastructure
- [ ] Add `DiagnosisRequest` and `DiagnosisResponse` to protocol
- [ ] Implement `nudge diagnose` CLI subcommand
- [ ] Add diagnosis configuration options
- [ ] Implement diagnosis handler in daemon

### Phase 2: Zsh Integration
- [ ] Implement stderr capture in `preexec`
- [ ] Implement diagnosis display in `precmd`
- [ ] Integrate with existing auto-mode for gray suggestions
- [ ] Add configuration reading in shell integration

### Phase 3: PowerShell Integration
- [ ] Implement `$Error` detection in prompt function
- [ ] Build ErrorRecord JSON serialization
- [ ] Integrate with existing completion flow

### Phase 4: Documentation & Polish
- [ ] Add caution notes to README.md / README_zh.md
- [ ] Add configuration examples
- [ ] Add user guide for error diagnosis
- [ ] Update ROADMAP.md

## Security Considerations

1. **Sensitive Data in Errors**: Stderr may contain sensitive information (passwords, tokens, paths). The existing sanitizer must be applied to stderr content before sending to LLM.

2. **Temp File Security**: The stderr temp file should use restrictive permissions (0600) and be deleted immediately after reading.

3. **Exit Code Spoofing**: A malicious script could set arbitrary exit codes. This is acceptable as it only affects the diagnosis feature, not system security.

## Future Enhancements (Out of Scope for v0.5.0)

- Interactive fix selection (multiple suggestions)
- Automatic fix execution with user confirmation
- Learning from user's fix patterns
- Integration with shell history for better context
