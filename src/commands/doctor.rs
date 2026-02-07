use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::time::sleep;

use crate::client::ipc;
use crate::config::{Config, TriggerMode, ZshGhostOwner, ZshOverlayBackend};
use crate::paths::AppPaths;
use crate::protocol::CompletionRequest;

pub async fn run_doctor(shell: Option<String>) -> Result<()> {
    let target = shell.unwrap_or_else(|| "zsh".to_string()).to_lowercase();
    if target != "zsh" {
        anyhow::bail!(
            "Unsupported doctor target: {}. Currently supported: zsh",
            target
        );
    }

    run_zsh_doctor().await
}

async fn run_zsh_doctor() -> Result<()> {
    let config = Config::load().unwrap_or_default();
    let integration_script = AppPaths::shell_dir().join("integration.zsh");

    println!("Nudge Doctor (zsh)");
    println!("==================");
    println!();
    println!("Config");
    println!("------");
    let trigger_mode = match config.trigger.mode {
        TriggerMode::Manual => "manual",
        TriggerMode::Auto => "auto",
    };
    let ghost_owner = match config.trigger.zsh_ghost_owner {
        ZshGhostOwner::Auto => "auto",
        ZshGhostOwner::Nudge => "nudge",
        ZshGhostOwner::Autosuggestions => "autosuggestions",
    };
    let overlay_backend = match config.trigger.zsh_overlay_backend {
        ZshOverlayBackend::Message => "message",
        ZshOverlayBackend::Rprompt => "rprompt",
    };
    println!("trigger.mode: {}", trigger_mode);
    println!("trigger.zsh_ghost_owner: {}", ghost_owner);
    println!("trigger.zsh_overlay_backend: {}", overlay_backend);
    println!("diagnosis.enabled: {}", config.diagnosis.enabled);
    println!();

    println!("Checks");
    println!("------");
    let zsh_version =
        read_command_output("zsh", &["--version"]).unwrap_or_else(|_| "unavailable".to_string());
    println!("zsh: {}", zsh_version.trim());
    println!(
        "integration script: {} ({})",
        integration_script.display(),
        if integration_script.exists() {
            "exists"
        } else {
            "missing"
        }
    );

    if integration_script.exists() {
        let syntax_status = Command::new("zsh")
            .arg("-n")
            .arg(&integration_script)
            .status()
            .context("Failed to run zsh syntax check")?;
        println!(
            "integration syntax: {}",
            if syntax_status.success() {
                "ok"
            } else {
                "failed"
            }
        );
    }

    let hook_check = read_command_output(
        "zsh",
        &[
            "-fc",
            "autoload -Uz add-zle-hook-widget 2>/dev/null; whence -w add-zle-hook-widget 2>/dev/null || true",
        ],
    )
    .unwrap_or_default();
    println!(
        "add-zle-hook-widget: {}",
        if hook_check.contains("function") {
            "available"
        } else {
            "missing"
        }
    );

    let probe = probe_zsh_bindings(&integration_script).unwrap_or_default();
    println!();
    println!("Key Bindings");
    println!("------------");
    print_binding(&probe, "TAB");
    print_binding(&probe, "CTRL_G");
    print_binding(&probe, "RIGHT");
    print_binding(&probe, "F1");
    print_binding(&probe, "HOOK_LINE_PRE_REDRAW");
    print_binding(&probe, "HOOK_LINE_FINISH");

    println!();
    println!("Heuristics");
    println!("----------");
    if ghost_owner == "autosuggestions" {
        if let Some(tab) = probe.get("TAB") {
            if tab.contains("_nudge_") {
                println!("[warn] Tab is owned by nudge while ghost owner is autosuggestions");
            } else {
                println!("[ok] Tab ownership does not conflict with autosuggestions mode");
            }
        }
        if let Some(ctrl_g) = probe.get("CTRL_G") {
            if ctrl_g.contains("_nudge_overlay_accept") {
                println!("[ok] Ctrl+G accepts overlay suggestions");
            } else {
                println!("[warn] Ctrl+G is not bound to _nudge_overlay_accept");
            }
        }
    } else {
        println!(
            "[info] autosuggestions conflict checks skipped (ghost owner is {})",
            ghost_owner
        );
    }

    if let Some(f1) = probe.get("F1") {
        if f1.contains("_nudge_toggle_explanation") {
            println!("[ok] F1 explanation toggle is active");
        } else {
            println!("[warn] F1 is not bound to _nudge_toggle_explanation");
        }
    }

    println!();
    println!("Latency (daemon)");
    println!("----------------");
    match collect_latency_samples().await {
        Ok(samples) if !samples.is_empty() => {
            let p50 = percentile(&samples, 50);
            let p95 = percentile(&samples, 95);
            println!("samples: {:?}", samples);
            println!("p50: {} ms", p50);
            println!("p95: {} ms", p95);
        }
        Ok(_) => {
            println!("no successful latency samples");
        }
        Err(err) => {
            println!("latency sampling unavailable: {}", err);
        }
    }

    Ok(())
}

fn probe_zsh_bindings(integration_script: &Path) -> Result<HashMap<String, String>> {
    let exe = std::env::current_exe().context("Failed to resolve current executable path")?;
    let exe_q = shell_quote(exe.to_string_lossy().as_ref());
    let script_q = shell_quote(integration_script.to_string_lossy().as_ref());
    let script = format!(
        r#"
function nudge() {{
  {exe} "$@"
}}
autoload -Uz add-zle-hook-widget 2>/dev/null
source {script} >/dev/null 2>&1
print -r -- "TAB=$(bindkey '^I' 2>/dev/null || true)"
print -r -- "CTRL_G=$(bindkey '^G' 2>/dev/null || true)"
print -r -- "RIGHT=$(bindkey '^[[C' 2>/dev/null || true)"
print -r -- "F1=$(bindkey $'\eOP' 2>/dev/null || true)"
print -r -- "HOOK_LINE_PRE_REDRAW=$(( ${{+widgets[_nudge_overlay_line_pre_redraw]}} ))"
print -r -- "HOOK_LINE_FINISH=$(( ${{+widgets[_nudge_overlay_line_finish]}} ))"
"#,
        exe = exe_q,
        script = script_q
    );

    let output = Command::new("zsh")
        .arg("-fc")
        .arg(script)
        .output()
        .context("Failed to run zsh probe command")?;

    if !output.status.success() {
        anyhow::bail!(
            "zsh probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut result = HashMap::new();
    for line in stdout.lines() {
        if let Some((k, v)) = line.split_once('=') {
            result.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Ok(result)
}

async fn collect_latency_samples() -> Result<Vec<u64>> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let session = format!("doctor-{}", std::process::id());
    let mut samples = Vec::new();

    for i in 0..7 {
        let buffer = if i % 2 == 0 {
            "git st".to_string()
        } else {
            "ls -".to_string()
        };
        let mut request = CompletionRequest::new(
            session.clone(),
            buffer.clone(),
            buffer.len(),
            cwd.clone(),
            Some(0),
        );
        request.shell_mode = Some("zsh-auto".to_string());
        request.time_bucket = Some((chrono::Utc::now().timestamp() as u64) / 2);

        let response = ipc::send_request(&request).await?;
        if response.error.is_none() {
            samples.push(response.processing_time_ms);
        }

        sleep(Duration::from_millis(25)).await;
    }

    Ok(samples)
}

fn percentile(samples: &[u64], p: u32) -> u64 {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let len = sorted.len();
    let pos = ((len - 1) * p as usize + 50) / 100;
    sorted[pos.min(len - 1)]
}

fn print_binding(probe: &HashMap<String, String>, key: &str) {
    if let Some(value) = probe.get(key) {
        println!("{}: {}", key.to_lowercase(), value);
    } else {
        println!("{}: <unknown>", key.to_lowercase());
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn read_command_output(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run command: {} {:?}", cmd, args))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
