use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "nudge")]
#[command(author, version, about = "Nudge - LLM-powered CLI auto-completion", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the daemon
    Daemon {
        /// Run in foreground (don't daemonize)
        #[arg(long, default_value_t = false)]
        foreground: bool,

        /// Fork and return immediately (for shell lazy-loading)
        #[arg(long, default_value_t = false)]
        fork: bool,
    },

    /// Request completion (called by shell)
    Complete {
        /// Current input buffer content
        #[arg(long)]
        buffer: String,

        /// Cursor position within buffer (0-indexed)
        #[arg(long)]
        cursor: usize,

        /// Current working directory
        #[arg(long)]
        cwd: PathBuf,

        /// Session identifier (e.g., "bash-12345")
        #[arg(long)]
        session: String,

        /// Exit code of the last executed command
        #[arg(long)]
        last_exit_code: Option<i32>,

        /// Git repository root (if available)
        #[arg(long)]
        git_root: Option<PathBuf>,

        /// Git state summary (repo_id|branch|dirty|staged)
        #[arg(long)]
        git_state: Option<String>,

        /// Shell mode (zsh-auto, zsh-inline, bash-inline, bash-popup, ps-inline, cmd-inline, etc.)
        #[arg(long)]
        shell_mode: Option<String>,

        /// Time bucket for auto mode (floor(now_ms / 2000))
        #[arg(long)]
        time_bucket: Option<u64>,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Plain)]
        format: OutputFormat,
    },

    /// Start daemon in background
    Start,

    /// Stop running daemon
    Stop,

    /// Restart daemon (stop + start)
    Restart,

    /// Check daemon status
    Status,

    /// Display runtime information (paths, status, configuration)
    Info {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Get specific field (config_dir, socket_path, shell_type, etc.)
        #[arg(long)]
        field: Option<String>,
    },

    /// Diagnose shell integration health
    Doctor {
        /// Shell target (currently: zsh, bash)
        shell: Option<String>,
    },

    /// Setup shell integration automatically
    Setup {
        /// Shell type (bash, zsh, powershell) - auto-detect if not specified
        shell: Option<String>,

        /// Force reinstall even if already configured
        #[arg(long)]
        force: bool,
    },

    /// Diagnose a failed command and suggest fixes
    Diagnose {
        /// Exit code of the failed command
        #[arg(long)]
        exit_code: i32,

        /// The failed command text
        #[arg(long)]
        command: String,

        /// Current working directory
        #[arg(long)]
        cwd: PathBuf,

        /// Session identifier (e.g., "zsh-12345")
        #[arg(long)]
        session: String,

        /// Path to file containing captured stderr (Zsh)
        #[arg(long)]
        stderr_file: Option<PathBuf>,

        /// PowerShell ErrorRecord as JSON string
        #[arg(long)]
        error_record: Option<String>,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Plain)]
        format: OutputFormat,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Output only the suggestion text (for shell integration)
    Plain,
    /// Output tab-separated candidates (risk, command, warning)
    List,
    /// Full JSON response (for debugging/advanced use)
    Json,
}
