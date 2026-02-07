# CMD Guide

## Current Behavior

- Integration script: `shell/integration.cmd`
- Shell mode sent to daemon: `cmd-inline`
- Usage surface: `nudge-complete <partial command>` macro

## Mode Notes

- `cmd-inline` is single-candidate mode.
- CMD integration does not provide native popup/overlay UX.

## Fast Path Guarantee

- `cmd-inline` remains the baseline path for low-latency manual completion.
- This path should stay single-candidate and avoid extra ranking overhead.
