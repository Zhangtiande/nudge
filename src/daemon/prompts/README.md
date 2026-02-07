# Prompt Templates

Prompt templates are stored as plain text files and loaded with `include_str!` for:

- version-controlled reviewability
- shell-specific contract separation
- compile-time embedding (no runtime file lookup)

Layout:

- `templates/completion/system.md`: default completion system prompt
- `templates/completion/contracts/*.md`: shell-mode response contracts

Implementation entrypoint:

- `src/daemon/prompts/completion.rs`
