mod cli;
mod config;
mod protocol;
mod daemon;
mod client;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::cli::{Cli, Command};

fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

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
        Command::Status => {
            daemon::status().await?;
        }
    }

    Ok(())
}
