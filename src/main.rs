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
    }

    Ok(())
}
