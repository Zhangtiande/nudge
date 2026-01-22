use crate::config::{Platform, ShellType};
use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

const MARKER_COMMENT: &str = "# Nudge shell integration";

/// Run the setup command
pub fn run_setup(shell: Option<String>, force: bool) -> Result<()> {
    let platform = Platform::detect()?;

    // Determine which shell to set up
    let target_shell = if let Some(shell_name) = shell {
        parse_shell_type(&shell_name)?
    } else {
        platform.shell
    };

    // Validate shell is supported
    if target_shell == ShellType::Unknown {
        anyhow::bail!("Cannot setup integration for unknown shell. Please specify shell type explicitly (bash, zsh, or powershell)");
    }

    if target_shell == ShellType::Cmd {
        anyhow::bail!(
            "CMD does not support automatic shell integration. Please use PowerShell instead."
        );
    }

    println!("Setting up Nudge for {}...", target_shell);
    println!();

    // Run shell-specific setup
    match target_shell {
        ShellType::Bash => setup_bash(&platform, force)?,
        ShellType::Zsh => setup_zsh(&platform, force)?,
        ShellType::PowerShell => setup_powershell(&platform, force)?,
        ShellType::Cmd | ShellType::Unknown => unreachable!(),
    }

    // Start daemon if needed
    start_daemon_if_needed()?;

    println!();
    println!("✓ Setup complete!");
    println!();
    println!("To start using Nudge:");
    match target_shell {
        ShellType::Bash => {
            println!("  1. Restart your terminal or run: source ~/.bashrc");
        }
        ShellType::Zsh => {
            println!("  1. Restart your terminal or run: source ~/.zshrc");
        }
        ShellType::PowerShell => {
            println!("  1. Restart PowerShell");
        }
        _ => {}
    }
    println!("  2. Press Ctrl+E to get AI-powered command suggestions");

    Ok(())
}

/// Setup Bash integration
fn setup_bash(platform: &Platform, force: bool) -> Result<()> {
    let profile_path = platform.shell_profile_path()?;
    let integration_script = platform.integration_script_path()?;

    // Check if already configured
    if !force && is_already_configured(&profile_path)? {
        println!(
            "✓ Bash integration is already configured in {}",
            profile_path.display()
        );
        println!("  Use --force to reinstall");
        return Ok(());
    }

    // Install integration script
    install_integration_script(platform, "bash")?;

    // Add source line to profile
    let source_line = format!(
        "\n{}\nsource \"{}\"\n",
        MARKER_COMMENT,
        integration_script.display()
    );

    if force {
        // Remove old integration if exists
        remove_old_integration(&profile_path)?;
    }

    append_to_file(&profile_path, &source_line)?;

    println!("✓ Added Nudge integration to {}", profile_path.display());

    Ok(())
}

/// Setup Zsh integration
fn setup_zsh(platform: &Platform, force: bool) -> Result<()> {
    let profile_path = platform.shell_profile_path()?;
    let integration_script = platform.integration_script_path()?;

    // Check if already configured
    if !force && is_already_configured(&profile_path)? {
        println!(
            "✓ Zsh integration is already configured in {}",
            profile_path.display()
        );
        println!("  Use --force to reinstall");
        return Ok(());
    }

    // Install integration script
    install_integration_script(platform, "zsh")?;

    // Add source line to profile
    let source_line = format!(
        "\n{}\nsource \"{}\"\n",
        MARKER_COMMENT,
        integration_script.display()
    );

    if force {
        // Remove old integration if exists
        remove_old_integration(&profile_path)?;
    }

    append_to_file(&profile_path, &source_line)?;

    println!("✓ Added Nudge integration to {}", profile_path.display());

    Ok(())
}

/// Setup PowerShell integration
fn setup_powershell(platform: &Platform, force: bool) -> Result<()> {
    let profile_path = platform.shell_profile_path()?;
    let integration_script = platform.integration_script_path()?;

    // Check if already configured
    if !force && is_already_configured(&profile_path)? {
        println!(
            "✓ PowerShell integration is already configured in {}",
            profile_path.display()
        );
        println!("  Use --force to reinstall");
        return Ok(());
    }

    // Install integration script
    install_integration_script(platform, "powershell")?;

    // Add source line to profile (PowerShell uses dot-sourcing)
    let source_line = format!(
        "\n{}\n. \"{}\"\n",
        MARKER_COMMENT,
        integration_script.display()
    );

    if force {
        // Remove old integration if exists
        remove_old_integration(&profile_path)?;
    }

    // Ensure profile directory exists
    if let Some(parent) = profile_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create profile directory: {}", parent.display()))?;
    }

    append_to_file(&profile_path, &source_line)?;

    println!("✓ Added Nudge integration to {}", profile_path.display());

    Ok(())
}

/// Install integration script to config directory
fn install_integration_script(platform: &Platform, shell: &str) -> Result<()> {
    let script_path = platform.integration_script_path()?;

    // Ensure parent directory exists
    if let Some(parent) = script_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Get embedded script content
    let script_content = match shell {
        "bash" => include_str!("../../shell/integration.bash"),
        "zsh" => include_str!("../../shell/integration.zsh"),
        "powershell" => include_str!("../../shell/integration.ps1"),
        _ => anyhow::bail!("Unsupported shell: {}", shell),
    };

    // Write script to file
    fs::write(&script_path, script_content).with_context(|| {
        format!(
            "Failed to write integration script: {}",
            script_path.display()
        )
    })?;

    // Set executable permission on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    println!(
        "✓ Installed integration script to {}",
        script_path.display()
    );

    Ok(())
}

/// Check if profile already has Nudge integration
fn is_already_configured(profile_path: &Path) -> Result<bool> {
    if !profile_path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(profile_path)
        .with_context(|| format!("Failed to read profile: {}", profile_path.display()))?;

    Ok(content.contains(MARKER_COMMENT))
}

/// Remove old integration from profile
fn remove_old_integration(profile_path: &Path) -> Result<()> {
    if !profile_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(profile_path)
        .with_context(|| format!("Failed to read profile: {}", profile_path.display()))?;

    // Remove lines containing marker comment and the following source line
    let mut new_lines = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut skip_next = false;

    for line in lines {
        if skip_next {
            skip_next = false;
            continue;
        }
        if line.contains(MARKER_COMMENT) {
            skip_next = true;
            continue;
        }
        new_lines.push(line);
    }

    let new_content = new_lines.join("\n");
    fs::write(profile_path, new_content)
        .with_context(|| format!("Failed to write profile: {}", profile_path.display()))?;

    Ok(())
}

/// Append content to file
fn append_to_file(path: &Path, content: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;

    file.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write to file: {}", path.display()))?;

    Ok(())
}

/// Parse shell type from string
fn parse_shell_type(shell: &str) -> Result<ShellType> {
    match shell.to_lowercase().as_str() {
        "bash" => Ok(ShellType::Bash),
        "zsh" => Ok(ShellType::Zsh),
        "powershell" | "pwsh" | "ps" => Ok(ShellType::PowerShell),
        "cmd" => Ok(ShellType::Cmd),
        _ => anyhow::bail!(
            "Unknown shell type: {}. Supported: bash, zsh, powershell",
            shell
        ),
    }
}

/// Start daemon if not already running
fn start_daemon_if_needed() -> Result<()> {
    // Check if daemon is running using synchronous check
    if crate::daemon::check_status().is_ok() {
        println!("✓ Daemon is already running");
        return Ok(());
    }

    println!("Starting Nudge daemon...");

    // Start daemon using async runtime
    let runtime = tokio::runtime::Runtime::new().context("Failed to create Tokio runtime")?;

    runtime.block_on(async { crate::daemon::start().await })?;

    Ok(())
}
