# Release Notes - v0.3.0

**Release Date**: 2026-01-23

This release introduces **Auto Mode** - ghost text suggestions that appear as you type, similar to GitHub Copilot. It also includes significant architectural improvements for better performance and cross-platform support.

## Highlights

### Auto Mode (Ghost Text Suggestions)

Nudge now supports automatic suggestions that appear as gray "ghost text" after your cursor while you type. No need to press a hotkey - suggestions appear automatically after a brief pause.

**Enable auto mode:**
```yaml
trigger:
  mode: auto
  auto_delay_ms: 500
```

**Key bindings:**
- `Tab`: Accept full suggestion
- `Right Arrow`: Accept next word (Zsh/PowerShell)
- `Ctrl+E`: Force trigger (bypasses debounce)

### Platform Support

| Shell | Auto Mode Support |
|-------|-------------------|
| Zsh | Full support (POSTDISPLAY) |
| Bash | ANSI escape codes |
| PowerShell 7.2+ | PSReadLine predictor |
| PowerShell 5.1 | Manual mode only |
| CMD | Manual mode only |

## New Features

### FFI Layer (Unix)

- New dynamic library (`libnudge.so`/`libnudge.dylib`) for lower latency
- Direct function calls instead of CLI invocation
- Embedded Tokio runtime for async operations
- Automatic fallback to CLI mode when library unavailable

### PowerShell Predictor

- Native PSReadLine predictor integration for PowerShell 7.2+
- `NudgePredictor` module automatically installed
- Throttling and caching for optimal performance
- Graceful fallback for older PowerShell versions

### Improved Shell Integration

- Zsh: Native `POSTDISPLAY` for inline preview
- Bash: ANSI escape code support for ghost text
- Better buffer change detection
- Request cancellation on new input

## Improvements

### Performance

- Debounce timer prevents excessive API calls
- Suggestion caching reduces redundant requests
- Background thread for non-blocking completion

### Documentation

- New [Auto Mode Guide](docs/auto-mode.md)
- New [Migration Guide](docs/migration-guide.md)
- New [Troubleshooting Guide](docs/troubleshooting.md)
- Updated README with auto mode documentation

## Configuration Changes

### New Options

```yaml
trigger:
  mode: "manual"          # NEW: "manual" or "auto"
  hotkey: "\C-e"          # Existing
  auto_delay_ms: 500      # NEW: Debounce delay for auto mode
```

### Backward Compatibility

- All existing configuration files remain valid
- The `trigger` section is optional (defaults to manual mode)
- Old shell integration scripts continue to work

## Breaking Changes

None. This release is fully backward compatible.

## Bug Fixes

- Fixed flaky test in FFI module due to parallel test execution
- Improved error handling in shell integration scripts

## Dependencies

No new runtime dependencies. Build dependencies:
- `libc` (optional, for FFI feature)

## Upgrade Instructions

1. Update Nudge binary:
   ```bash
   # Unix/Linux/macOS
   curl -fsSL https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.sh | bash

   # Windows
   irm https://raw.githubusercontent.com/Zhangtiande/nudge/main/scripts/install.ps1 | iex
   ```

2. Update shell integration:
   ```bash
   nudge setup
   source ~/.bashrc  # or ~/.zshrc
   ```

3. (Optional) Enable auto mode in config.yaml

See [Migration Guide](docs/migration-guide.md) for detailed instructions.

## Contributors

Thanks to all contributors who made this release possible!

## Full Changelog

### Phase 1: Foundation Refactoring
- Centralized platform logic in `src/config.rs`
- Implemented `nudge info` command with JSON output
- Implemented `nudge setup` command for shell integration
- Updated shell integration scripts to use `nudge info`

### Phase 2: Unix FFI Layer
- Created FFI module structure (`src/ffi/`)
- Implemented C ABI interface (`nudge_init`, `nudge_complete`, `nudge_free`)
- Added panic catching at FFI boundaries
- Created C header file (`include/nudge.h`)

### Phase 3: Unix Auto Mode
- Implemented `nudge_auto_start` FFI function
- Added debounce timer with request cancellation
- Implemented Zsh POSTDISPLAY integration
- Implemented Bash ANSI escape code preview

### Phase 4: Windows Auto Mode
- Created `NudgePredictor` PowerShell module
- Implemented PSReadLine predictor API integration
- Added throttling and caching
- Updated installation scripts

### Phase 5: Polish and Documentation
- Updated README.md and README_zh.md
- Created migration guide
- Created troubleshooting guide
- Version bump to 0.3.0
