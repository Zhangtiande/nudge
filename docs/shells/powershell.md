# PowerShell Guide

## Current Behavior

- Integration script: `shell/integration.ps1`
- Shell mode sent to daemon: `ps-inline`
- Primary trigger: `Ctrl+E` (manual completion)
- Diagnosis integration: enabled when `diagnosis.enabled: true`

## Mode Notes

- `ps-inline` is single-candidate mode.
- Predictor-based auto mode in PowerShell is optional and environment-dependent.
- Manual completion and diagnosis flows remain available regardless of predictor availability.

## Fast Path Guarantee

- `Ctrl+E` must stay available.
- `Ctrl+E` always uses `ps-inline` and returns one primary suggestion.
- This manual path is the baseline for latency and reliability.
