#[cfg(unix)]
mod tests {
    use std::process::Command;

    fn has_zsh() -> bool {
        Command::new("zsh").arg("--version").output().is_ok()
    }

    #[test]
    fn clear_nudge_highlight_preserves_other_highlights() {
        if !has_zsh() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "auto" ;;
      auto_delay_ms) echo "500" ;;
      zsh_ghost_owner) echo "nudge" ;;
      zsh_overlay_backend) echo "message" ;;
      diagnosis_enabled) echo "false" ;;
      interactive_commands) echo "" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  fi
}

source shell/integration.zsh >/dev/null 2>&1
typeset -ga region_highlight
region_highlight=("0 3 fg=2" "3 8 fg=8")
_nudge_region_highlight_entry="3 8 fg=8"
_nudge_clear_own_highlight
print -r -- "${(j:|:)region_highlight}"
print -r -- "$_nudge_region_highlight_entry"
"#;

        let output = Command::new("zsh")
            .arg("-fc")
            .arg(script)
            .output()
            .expect("failed to run zsh");

        assert!(
            output.status.success(),
            "zsh script failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines();
        assert_eq!(lines.next(), Some("0 3 fg=2"));
        assert_eq!(lines.next(), Some(""));
    }

    #[test]
    fn diagnosis_uses_ctrl_g_when_autosuggestions_owns_ghost_text() {
        if !has_zsh() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "auto" ;;
      auto_delay_ms) echo "500" ;;
      zsh_ghost_owner) echo "autosuggestions" ;;
      zsh_overlay_backend) echo "message" ;;
      diagnosis_enabled) echo "true" ;;
      interactive_commands) echo "" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  fi
}

source shell/integration.zsh >/dev/null 2>&1
tab_binding=$(bindkey '^I' 2>/dev/null || true)
ctrl_g_binding=$(bindkey '^G' 2>/dev/null || true)
print -r -- "$tab_binding"
print -r -- "$ctrl_g_binding"
"#;

        let output = Command::new("zsh")
            .arg("-fc")
            .arg(script)
            .output()
            .expect("failed to run zsh");

        assert!(
            output.status.success(),
            "zsh script failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines();
        let tab_line = lines.next().unwrap_or_default();
        let ctrl_g_line = lines.next().unwrap_or_default();

        assert!(
            !tab_line.contains("_nudge_auto_accept"),
            "tab binding should not be owned by nudge when autosuggestions owns ghost text: {}",
            tab_line
        );
        assert!(
            ctrl_g_line.contains("_nudge_overlay_accept"),
            "ctrl+g should accept diagnosis suggestion in autosuggestions mode: {}",
            ctrl_g_line
        );
    }

    #[test]
    fn autosuggestions_overlay_binds_ctrl_g_without_diagnosis() {
        if !has_zsh() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "auto" ;;
      auto_delay_ms) echo "500" ;;
      zsh_ghost_owner) echo "autosuggestions" ;;
      zsh_overlay_backend) echo "message" ;;
      diagnosis_enabled) echo "false" ;;
      interactive_commands) echo "" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  fi
}

source shell/integration.zsh >/dev/null 2>&1
tab_binding=$(bindkey '^I' 2>/dev/null || true)
ctrl_g_binding=$(bindkey '^G' 2>/dev/null || true)
print -r -- "$tab_binding"
print -r -- "$ctrl_g_binding"
"#;

        let output = Command::new("zsh")
            .arg("-fc")
            .arg(script)
            .output()
            .expect("failed to run zsh");

        assert!(
            output.status.success(),
            "zsh script failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines();
        let tab_line = lines.next().unwrap_or_default();
        let ctrl_g_line = lines.next().unwrap_or_default();

        assert!(
            !tab_line.contains("_nudge_auto_accept"),
            "tab binding should not be owned by nudge in autosuggestions overlay mode: {}",
            tab_line
        );
        assert!(
            ctrl_g_line.contains("_nudge_overlay_accept"),
            "ctrl+g should accept overlay suggestion in autosuggestions mode: {}",
            ctrl_g_line
        );
    }

    #[test]
    fn auto_mode_binds_flagship_accept_and_explain_keys() {
        if !has_zsh() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "auto" ;;
      auto_delay_ms) echo "500" ;;
      zsh_ghost_owner) echo "nudge" ;;
      zsh_overlay_backend) echo "message" ;;
      diagnosis_enabled) echo "false" ;;
      interactive_commands) echo "" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  fi
}

source shell/integration.zsh >/dev/null 2>&1
right_binding=$(bindkey '^[[C' 2>/dev/null || true)
alt_right_binding=$(bindkey $'\e[1;3C' 2>/dev/null || true)
ctrl_right_binding=$(bindkey $'\e[1;5C' 2>/dev/null || true)
f1_binding=$(bindkey $'\eOP' 2>/dev/null || true)
print -r -- "$right_binding"
print -r -- "$alt_right_binding"
print -r -- "$ctrl_right_binding"
print -r -- "$f1_binding"
"#;

        let output = Command::new("zsh")
            .arg("-fc")
            .arg(script)
            .output()
            .expect("failed to run zsh");

        assert!(
            output.status.success(),
            "zsh script failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines();
        let right_line = lines.next().unwrap_or_default();
        let alt_right_line = lines.next().unwrap_or_default();
        let ctrl_right_line = lines.next().unwrap_or_default();
        let f1_line = lines.next().unwrap_or_default();

        assert!(
            right_line.contains("_nudge_auto_accept_word"),
            "right arrow should accept next word: {}",
            right_line
        );
        assert!(
            alt_right_line.contains("_nudge_auto_accept_argument"),
            "alt+right should accept next argument: {}",
            alt_right_line
        );
        assert!(
            ctrl_right_line.contains("_nudge_auto_accept_segment"),
            "ctrl+right should accept next segment: {}",
            ctrl_right_line
        );
        assert!(
            f1_line.contains("_nudge_toggle_explanation"),
            "f1 should toggle explanation: {}",
            f1_line
        );
    }

    #[test]
    fn rprompt_overlay_backend_restores_prompt_after_clear() {
        if !has_zsh() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "auto" ;;
      auto_delay_ms) echo "500" ;;
      zsh_ghost_owner) echo "autosuggestions" ;;
      zsh_overlay_backend) echo "rprompt" ;;
      diagnosis_enabled) echo "false" ;;
      interactive_commands) echo "" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  fi
}

source shell/integration.zsh >/dev/null 2>&1
RPS1="ORIGINAL_RPROMPT"
_nudge_overlay_set_message "hello"
print -r -- "$RPS1"
_nudge_overlay_clear_message
print -r -- "$RPS1"
"#;

        let output = Command::new("zsh")
            .arg("-fc")
            .arg(script)
            .output()
            .expect("failed to run zsh");

        assert!(
            output.status.success(),
            "zsh script failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines();
        let temporary = lines.next().unwrap_or_default();
        let restored = lines.next().unwrap_or_default();

        assert!(
            temporary.contains("hello"),
            "overlay message should be rendered in rprompt backend: {}",
            temporary
        );
        assert_eq!(restored, "ORIGINAL_RPROMPT");
    }
}
