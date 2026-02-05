mod cli;
mod client;
mod commands;
mod config;
mod daemon;
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
        // Ensure log directory exists
        let log_dir = Config::log_dir();
        std::fs::create_dir_all(&log_dir).ok();

        // Daily rolling file appender
        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "nudge.log");

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
        #[allow(deprecated)]
        Command::Config { show } => {
            show_config(show)?;
        }
        Command::Info { json, field } => {
            commands::info::run_info(json, field)?;
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

fn show_config(show_full: bool) -> Result<()> {
    println!("Nudge Configuration\n");

    // Show file locations
    if let Some(base_path) = Config::base_config_path() {
        println!("Default config: {}", base_path.display());
        println!("  Exists: {}", base_path.exists());
    }

    if let Some(user_path) = Config::default_config_path() {
        println!("User config:    {}", user_path.display());
        println!("  Exists: {}", user_path.exists());
    }

    println!("\nSocket path:    {}", Config::socket_path().display());
    println!("PID file:       {}", Config::pid_path().display());
    println!("Log directory:  {}", Config::log_dir().display());

    if show_full {
        println!("\n{}", "=".repeat(50));
        println!("Loading Configuration (with layered merge)...\n");

        match Config::load() {
            Ok(config) => {
                println!("✓ Configuration loaded successfully\n");
                println!("{}", config.llm_config_summary());
                println!("\nContext Settings:");
                println!("  History window: {}", config.context.history_window);
                println!(
                    "  Include CWD listing: {}",
                    config.context.include_cwd_listing
                );
                println!(
                    "  Include system info: {}",
                    config.context.include_system_info
                );
                println!("  Max total tokens: {}", config.context.max_total_tokens);
                println!("\nPlugin Settings:");
                println!("  Git enabled: {}", config.plugins.git.enabled);
                println!("\nPrivacy Settings:");
                println!(
                    "  Sanitization enabled: {}",
                    config.privacy.sanitize_enabled
                );
                println!(
                    "  Block dangerous commands: {}",
                    config.privacy.block_dangerous
                );
            }
            Err(e) => {
                println!("✗ Failed to load configuration: {}", e);
                return Err(e);
            }
        }
    } else {
        println!("\nTip: Use 'nudge config --show' to see full configuration");
    }

    Ok(())
}
