# CLI Reference

This document is a practical command reference for day-to-day Nudge operation.

## Quick Examples

```bash
nudge status
nudge info
nudge doctor zsh
nudge restart
```

## Command List

```text
nudge daemon [--foreground|--fork]
nudge complete --buffer ... --cursor ... --cwd ... --session ... [--shell-mode ...] [--format plain|list|json]
nudge start
nudge stop
nudge restart
nudge status
nudge info [--json] [--field <name>]
nudge doctor [zsh|bash]
nudge setup [bash|zsh|powershell] [--force]
nudge diagnose --exit-code ... --command ... --cwd ... --session ... [--stderr-file ...] [--error-record ...]
```

## `nudge info --field` Common Keys

- `config_dir`
- `config_file`
- `default_config_file`
- `socket_path`
- `integration_script`
- `daemon_status`
- `shell_type`
- `trigger_mode`
- `trigger_hotkey`
- `zsh_ghost_owner`
- `zsh_overlay_backend`
- `diagnosis_enabled`
- `interactive_commands`

## Typical Workflows

Initial setup check:

```bash
nudge setup zsh --force
nudge restart
nudge status
nudge info
```

Debug integration behavior:

```bash
nudge doctor zsh
nudge doctor bash
```

## Boundaries

- `nudge complete` is an integration/internal path; prefer shell key bindings in normal use
- `--shell-mode` is a hint from integration scripts; do not rely on undocumented values
