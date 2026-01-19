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

    /// Show configuration paths and status
    Config {
        /// Show full configuration (not just paths)
        #[arg(long, default_value_t = false)]
        show: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Output only the suggestion text (for shell integration)
    Plain,
    /// Full JSON response (for debugging/advanced use)
    Json,
}
