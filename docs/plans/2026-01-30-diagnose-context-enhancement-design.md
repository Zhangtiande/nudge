# Diagnose Command Context Enhancement Design

## Overview

Enhance the `diagnose` command to have the same level of project awareness as the `complete` command by sharing the context gathering logic.

## Current State

| Context Type | `complete` | `diagnose` |
|-------------|------------|------------|
| System info (OS, arch) | ✅ | ❌ |
| Command history (session tracking) | ✅ Full | ❌ Only recent 5 |
| Similar command search | ✅ | ❌ |
| CWD file listing | ✅ | ✅ |
| Git plugin | ✅ | ❌ |
| Docker plugin | ✅ | ❌ |
| Node.js plugin | ✅ | ❌ |
| Rust plugin | ✅ | ❌ |
| Python plugin | ✅ | ❌ |

## Design Decisions

1. **Full context alignment** - diagnose will use the same context gathering as complete
2. **Plugin activation** - Use command prefix OR feature files (more lenient activation)
3. **No similar commands** - diagnose doesn't need similar command search

## Implementation

### Part 1: GatherParams Abstraction

Create a shared parameter struct to unify context gathering:

```rust
// src/daemon/context/mod.rs

/// Common parameters for context gathering
pub struct GatherParams {
    /// Session ID for history tracking
    pub session_id: String,
    /// Current working directory
    pub cwd: PathBuf,
    /// Command text (buffer for completion, failed command for diagnosis)
    pub command: String,
    /// Exit code of last command
    pub last_exit_code: Option<i32>,
    /// Whether to search for similar commands
    pub include_similar_commands: bool,
}

impl From<&CompletionRequest> for GatherParams { ... }
impl From<&DiagnosisRequest> for GatherParams { ... }
```

### Part 2: Refactor gather() Function

- Change signature from `gather(request: &CompletionRequest, ...)` to `gather(params: &GatherParams, ...)`
- Remove `gather_minimal()` function
- Use `params.include_similar_commands` to conditionally search similar commands

### Part 3: Update server.rs

```rust
// process_request() - completion
let context_data = context::gather(&GatherParams::from(&request), config).await?;

// process_diagnosis_request() - diagnosis
let context_data = context::gather(&GatherParams::from(&request), config).await?;
```

### Part 4: Enhance Diagnosis Prompt

Add new context sections to `build_diagnosis_prompt()`:

- System information (OS, version, arch)
- Git status (branch, staged files, recent commits)
- Plugin context (project type specific info)

## Files to Modify

1. `src/daemon/context/mod.rs` - Add GatherParams, refactor gather()
2. `src/daemon/server.rs` - Update call sites
3. `src/daemon/diagnosis.rs` - Enhance prompt building
4. `src/protocol.rs` - No changes needed

## Expected Outcome

After implementation, diagnose will have access to:
- Full system information
- Session-based command history
- All project-aware plugins (Git, Docker, Node, Rust, Python)
- Better error diagnosis with project context
