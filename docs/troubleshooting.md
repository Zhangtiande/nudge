# Troubleshooting Guide

This guide covers common issues and solutions for Nudge.

## Quick Diagnostics

Run these commands to gather diagnostic information:

```bash
# Check Nudge version
nudge --version

# Check daemon status
nudge status

# Show runtime information
nudge info

# Show detailed JSON info
nudge info --json
```

## Common Issues

### Daemon Issues

#### Daemon not starting

**Symptoms:** `nudge status` shows "Not running"

**Solutions:**

1. Start the daemon manually:
   ```bash
   nudge start
   ```

2. Check for port/socket conflicts:
   ```bash
   # Unix
   ls -la ~/.config/nudge/nudge.sock

   # If stale socket exists, remove it
   rm ~/.config/nudge/nudge.sock
   nudge start
   ```

3. Check logs for errors:
   ```bash
   # Enable file logging in config.yaml
   log:
     level: "debug"
     file_enabled: true

   # Then check logs
   # Linux: ~/.config/nudge/logs/
   # macOS: ~/Library/Application Support/nudge/logs/
   # Windows: %APPDATA%\nudge\logs\
   ```

#### Daemon crashes on startup

**Symptoms:** Daemon starts but immediately exits

**Solutions:**

1. Check configuration syntax:
   ```bash
   # Validate YAML syntax
   nudge info
   ```

2. Run daemon in foreground to see errors:
   ```bash
   nudge daemon --foreground
   ```

3. Check for missing dependencies (LLM endpoint):
   ```bash
   curl http://localhost:11434/v1/models  # For Ollama
   ```

### Completion Issues

#### No suggestions returned

**Symptoms:** Pressing Ctrl+E does nothing

**Solutions:**

1. Verify daemon is running:
   ```bash
   nudge status
   ```

2. Test completion directly:
   ```bash
   nudge complete --buffer "git sta" --cursor 7 --cwd . --session test
   ```

3. Check LLM endpoint configuration:
   ```yaml
   model:
     endpoint: "http://localhost:11434/v1"  # Verify this is correct
     model_name: "codellama:7b"
   ```

4. Verify LLM is accessible:
   ```bash
   # For Ollama
   curl http://localhost:11434/api/tags

   # For OpenAI
   curl -H "Authorization: Bearer $OPENAI_API_KEY" https://api.openai.com/v1/models
   ```

#### Slow completions

**Symptoms:** Completions take several seconds

**Solutions:**

1. Use a local LLM (Ollama) instead of remote APIs
2. Use a smaller/faster model:
   ```yaml
   model:
     model_name: "codellama:7b"  # Faster than larger models
   ```
3. Reduce context size:
   ```yaml
   context:
     history_window: 10          # Reduce from 20
     max_total_tokens: 2000      # Reduce from 4000
   ```
4. Check network latency to LLM endpoint

#### Irrelevant suggestions

**Symptoms:** Suggestions don't match context

**Solutions:**

1. Enable more context sources:
   ```yaml
   context:
     include_cwd_listing: true
     include_system_info: true
     similar_commands_enabled: true
   ```

2. Enable Git plugin for repository context:
   ```yaml
   plugins:
     git:
       enabled: true
       depth: standard
   ```

3. Try a different model (some models are better at code completion)

### Shell Integration Issues

#### Hotkey not working

**Symptoms:** Ctrl+E doesn't trigger completion

**Solutions:**

1. Verify shell integration is loaded:
   ```bash
   # Bash/Zsh - should see "Nudge loaded" on shell startup
   # Or check if function exists
   type _nudge_complete  # Bash
   which _nudge_complete  # Zsh
   ```

2. Re-run setup:
   ```bash
   nudge setup
   source ~/.bashrc  # or ~/.zshrc
   ```

3. Check for hotkey conflicts:
   ```bash
   # Bash
   bind -p | grep '\\C-e'

   # Zsh
   bindkey | grep '\^E'
   ```

4. Try a different hotkey:
   ```yaml
   trigger:
     hotkey: "\C-x"  # Use Ctrl+X instead
   ```

#### Shell integration not found

**Symptoms:** "source: file not found" error

**Solutions:**

1. Check integration script exists:
   ```bash
   # Linux
   ls ~/.config/nudge/shell/integration.bash

   # macOS
   ls ~/Library/Application\ Support/nudge/shell/integration.bash
   ```

2. Re-run setup to install scripts:
   ```bash
   nudge setup
   ```

### Auto Mode Issues

#### Auto mode not triggering

**Symptoms:** No ghost text appears while typing

**Solutions:**

1. Verify auto mode is enabled:
   ```bash
   nudge info --field trigger_mode
   # Should output: auto
   ```

2. Check configuration:
   ```yaml
   trigger:
     mode: auto
     auto_delay_ms: 500
   ```

3. Restart daemon after config change:
   ```bash
   nudge restart
   ```

4. Reload shell integration:
   ```bash
   source ~/.bashrc  # or ~/.zshrc
   ```

#### Ghost text not displaying (Unix)

**Symptoms:** Auto mode triggers but no preview shown

**Solutions:**

1. **Zsh**: Should work out of the box with POSTDISPLAY
2. **Bash**: Requires ANSI escape code support
   - Try a different terminal emulator
   - Consider switching to Zsh for better auto mode support

#### PowerShell predictor not working

**Symptoms:** No predictions in PowerShell 7.2+

**Solutions:**

1. Check PowerShell version:
   ```powershell
   $PSVersionTable.PSVersion
   # Must be 7.2 or later
   ```

2. Check PSReadLine version:
   ```powershell
   Get-Module PSReadLine | Select-Object Version
   # Must be 2.2.0 or later
   ```

3. Verify NudgePredictor module is loaded:
   ```powershell
   Get-Module NudgePredictor
   ```

4. Check predictor is registered:
   ```powershell
   Get-PSSubsystem -Kind CommandPredictor
   ```

5. Manually register predictor:
   ```powershell
   Import-Module NudgePredictor
   Register-NudgePredictor
   Set-NudgePredictionOptions
   ```

### Configuration Issues

#### Config file not found

**Symptoms:** Using default settings, custom config ignored

**Solutions:**

1. Check config file location:
   ```bash
   nudge info --field config_file
   ```

2. Verify file exists and is readable:
   ```bash
   cat "$(nudge info --field config_file)"
   ```

3. Check YAML syntax:
   ```bash
   # Install yq or use online YAML validator
   yq . ~/.config/nudge/config.yaml
   ```

#### Environment variable not working

**Symptoms:** `api_key_env` not picking up API key

**Solutions:**

1. Verify environment variable is set:
   ```bash
   echo $OPENAI_API_KEY
   ```

2. Ensure variable is exported:
   ```bash
   export OPENAI_API_KEY="sk-..."
   ```

3. Restart daemon to pick up new environment:
   ```bash
   nudge restart
   ```

### Privacy/Safety Issues

#### Sensitive data being sent to LLM

**Symptoms:** API keys or passwords appearing in completions

**Solutions:**

1. Verify sanitizer is enabled:
   ```yaml
   privacy:
     sanitize_enabled: true
   ```

2. Add custom patterns for your sensitive data:
   ```yaml
   privacy:
     custom_patterns:
       - "MY_SECRET_\\w+"
       - "internal-api-key-\\w+"
   ```

#### Dangerous command warnings not showing

**Symptoms:** No warnings for `rm -rf` etc.

**Solutions:**

1. Enable dangerous command blocking:
   ```yaml
   privacy:
     block_dangerous: true
   ```

2. Add custom blocked commands:
   ```yaml
   privacy:
     custom_blocked:
       - "drop database"
       - "format c:"
   ```

## Platform-Specific Issues

### Linux

#### Permission denied on socket

```bash
# Check socket permissions
ls -la ~/.config/nudge/nudge.sock

# Remove and restart
rm ~/.config/nudge/nudge.sock
nudge start
```

### macOS

#### "nudge" cannot be opened (Gatekeeper)

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/nudge
```

### Windows

#### Named pipe connection failed

```powershell
# Check if pipe exists
Get-ChildItem \\.\pipe\ | Where-Object { $_.Name -like "*nudge*" }

# Restart daemon
nudge stop
nudge start
```

#### PowerShell execution policy

```powershell
# Allow script execution
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

## Getting More Help

### Enable Debug Logging

```yaml
log:
  level: "debug"
  file_enabled: true
```

### Collect Diagnostic Information

```bash
# Create diagnostic report
echo "=== Nudge Version ===" > nudge-diag.txt
nudge --version >> nudge-diag.txt
echo "=== Nudge Info ===" >> nudge-diag.txt
nudge info --json >> nudge-diag.txt
echo "=== Daemon Status ===" >> nudge-diag.txt
nudge status >> nudge-diag.txt
echo "=== Config File ===" >> nudge-diag.txt
cat "$(nudge info --field config_file)" >> nudge-diag.txt 2>&1
```

### Report Issues

If you can't resolve your issue:

1. Search [existing issues](https://github.com/Zhangtiande/nudge/issues)
2. Create a new issue with:
   - Nudge version (`nudge --version`)
   - OS and shell version
   - Configuration (redact sensitive data)
   - Steps to reproduce
   - Error messages or logs
