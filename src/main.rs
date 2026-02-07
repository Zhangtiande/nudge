mod cli;
mod client;
mod commands;
mod config;
mod daemon;
mod paths;
mod protocol;

use anyhow::Result;
use clap::Parser;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::cli::{Cli, Command};
use crate::config::Config;

/// Initialize logging based on command type and configuration
///
/// By default, logs are only written to file (if enabled) and NOT to console.
/// Console logging is only enabled when:
/// 1. Running daemon in foreground mode (for debugging)
/// 2. RUST_LOG environment variable is explicitly set by user
fn init_logging(config: &Config, foreground_daemon: bool) {
    // Check if user explicitly set RUST_LOG environment variable
    let explicit_rust_log = std::env::var("RUST_LOG").is_ok();

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log.level));

    let registry = tracing_subscriber::registry().with(env_filter);

    // Enable console logging only for foreground daemon or when RUST_LOG is set
    let enable_console = foreground_daemon || explicit_rust_log;

    if config.log.file_enabled {
        // Ensure log directory exists before constructing file appender.
        let log_dir = Config::log_dir();
        let file_logging_ready = std::fs::create_dir_all(&log_dir).is_ok();
        let file_appender = if file_logging_ready {
            std::panic::catch_unwind(|| {
                RollingFileAppender::new(Rotation::DAILY, &log_dir, "nudge.log")
            })
            .ok()
        } else {
            None
        };

        if let Some(file_appender) = file_appender {
            if enable_console {
                // Both file and console output (for foreground daemon or debugging)
                registry
                    .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_writer(file_appender)
                            .with_ansi(false),
                    )
                    .init();
            } else {
                // File only (no console output) - normal CLI behavior
                registry
                    .with(
                        tracing_subscriber::fmt::layer()
                            .with_writer(file_appender)
                            .with_ansi(false),
                    )
                    .init();
            }
        } else if enable_console {
            eprintln!(
                "Warning: failed to initialize file logging at {}. Falling back to console logging.",
                log_dir.display()
            );
            registry.with(tracing_subscriber::fmt::layer()).init();
        } else {
            registry.init();
        }
    } else if enable_console {
        // Console only (when file logging is disabled but console is needed)
        registry.with(tracing_subscriber::fmt::layer()).init();
    } else {
        // No output at all - normal CLI behavior when file logging is disabled
        registry.init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let cli = Cli::parse();

    // Only enable console logging for foreground daemon (for debugging)
    // All other commands should NOT output logs to console (professional CLI behavior)
    let foreground_daemon = matches!(
        cli.command,
        Command::Daemon {
            foreground: true,
            ..
        }
    );
    init_logging(&config, foreground_daemon);

    match cli.command {
        Command::Daemon { foreground, fork } => {
            daemon::run(config, foreground, fork).await?;
        }
        Command::Complete {
            buffer,
            cursor,
            cwd,
            session,
            last_exit_code,
            git_root,
            git_state,
            shell_mode,
            time_bucket,
            format,
        } => {
            client::complete(
                buffer,
                cursor,
                cwd,
                session,
                last_exit_code,
                git_root,
                git_state,
                shell_mode,
                time_bucket,
                format,
            )
            .await?;
        }
        Command::Start => {
            daemon::start().await?;
        }
        Command::Stop => {
            daemon::stop().await?;
        }
        Command::Restart => {
            daemon::restart().await?;
        }
        Command::Status => {
            daemon::status().await?;
        }
        Command::Info { json, field } => {
            commands::info::run_info(json, field)?;
        }
        Command::Doctor { shell } => {
            commands::doctor::run_doctor(shell).await?;
        }
        Command::Setup { shell, force } => {
            commands::setup::run_setup(shell, force).await?;
        }
        Command::Diagnose {
            exit_code,
            command,
            cwd,
            session,
            stderr_file,
            error_record,
            format,
        } => {
            client::diagnose(
                exit_code,
                command,
                cwd,
                session,
                stderr_file,
                error_record,
                format,
            )
            .await?;
        }
    }

    Ok(())
}
