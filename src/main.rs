mod cli;
mod client;
mod config;
mod daemon;
mod protocol;

use anyhow::Result;
use clap::Parser;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::cli::{Cli, Command};
use crate::config::Config;

fn init_logging(config: &Config) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log.level));

    let registry = tracing_subscriber::registry().with(env_filter);

    if config.log.file_enabled {
        // Ensure log directory exists
        let log_dir = Config::log_dir();
        std::fs::create_dir_all(&log_dir).ok();

        // Daily rolling file appender
        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "nudge.log");

        // Both file and console output
        registry
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file_appender)
                    .with_ansi(false),
            )
            .init();
    } else {
        registry.with(tracing_subscriber::fmt::layer()).init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    init_logging(&config);

    let cli = Cli::parse();

    match cli.command {
        Command::Daemon { foreground, fork } => {
            daemon::run(foreground, fork).await?;
        }
        Command::Complete {
            buffer,
            cursor,
            cwd,
            session,
            last_exit_code,
            format,
        } => {
            client::complete(buffer, cursor, cwd, session, last_exit_code, format).await?;
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
        Command::Config { show } => {
            show_config(show)?;
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
