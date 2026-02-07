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
            !alt_right_line.contains("_nudge_auto_accept_argument"),
            "alt+right should not be bound to nudge argument-accept: {}",
            alt_right_line
        );
        assert!(
            !ctrl_right_line.contains("_nudge_auto_accept_segment"),
            "ctrl+right should not be bound to nudge segment-accept: {}",
            ctrl_right_line
        );
        assert!(
            f1_line.contains("_nudge_toggle_explanation"),
            "f1 should toggle explanation: {}",
            f1_line
        );
    }

    #[test]
    fn overlay_mode_preserves_existing_postdisplay() {
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
POSTDISPLAY="__AUTO__"
BUFFER="echo"
_nudge_auto_suggestion="echo hi"
_nudge_auto_display_preview
print -r -- "$POSTDISPLAY"
_nudge_auto_cancel
print -r -- "$POSTDISPLAY"
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
        let after_preview = lines.next().unwrap_or_default();
        let after_cancel = lines.next().unwrap_or_default();

        assert_eq!(
            after_preview, "__AUTO__",
            "overlay rendering should not clear autosuggestions preview"
        );
        assert_eq!(
            after_cancel, "__AUTO__",
            "overlay cancel should not clear autosuggestions preview"
        );
    }

    #[test]
    fn overlay_accept_clears_autosuggestions_preview() {
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
BUFFER="git st"
CURSOR=${#BUFFER}
POSTDISPLAY="atus"
_nudge_auto_suggestion="git status"
_nudge_overlay_accept
print -r -- "$BUFFER"
print -r -- "$POSTDISPLAY"
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
        let accepted_buffer = lines.next().unwrap_or_default();
        let postdisplay_after_accept = lines.next().unwrap_or_default();

        assert_eq!(
            accepted_buffer, "git status",
            "ctrl+g should accept overlay suggestion buffer"
        );
        assert_eq!(
            postdisplay_after_accept, "",
            "overlay accept should clear autosuggestions preview"
        );
    }

    #[test]
    fn overlay_marks_warning_as_high_risk() {
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
BUFFER="rm -rf /"
_nudge_auto_suggestion="rm -rf /tmp"
_nudge_auto_warning="daemon safety warning"
_nudge_overlay_render
print -r -- "$_nudge_overlay_last_message"
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
        let line = stdout.lines().next().unwrap_or_default();
        assert!(
            line.contains("risk:high"),
            "warning-backed suggestion should be marked as high risk: {}",
            line
        );
    }

    #[test]
    fn overlay_history_navigation_does_not_trigger_fetch() {
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
_nudge_overlay_mode_enabled="true"
_nudge_last_buffer=""
BUFFER="git status"
typeset -gi _nudge_fetch_calls=0
_nudge_fetch_async() { _nudge_fetch_calls=$((_nudge_fetch_calls + 1)); }

LASTWIDGET="up-line-or-history"
_nudge_overlay_line_pre_redraw
print -r -- "$_nudge_fetch_calls"

LASTWIDGET="self-insert"
BUFFER="git status -s"
_nudge_overlay_line_pre_redraw
print -r -- "$_nudge_fetch_calls"
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
        let after_history = lines.next().unwrap_or_default();
        let after_typing = lines.next().unwrap_or_default();

        assert_eq!(after_history, "0");
        assert_eq!(after_typing, "1");
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
RPROMPT="ORIGINAL_RPROMPT"
_nudge_overlay_set_message "hello"
print -r -- "$RPS1"
print -r -- "$RPROMPT"
_nudge_overlay_clear_message
print -r -- "$RPS1"
print -r -- "$RPROMPT"
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
        let temporary_rprompt = lines.next().unwrap_or_default();
        let restored = lines.next().unwrap_or_default();
        let restored_rprompt = lines.next().unwrap_or_default();

        assert!(
            temporary.contains("hello"),
            "overlay message should be rendered in rprompt backend: {}",
            temporary
        );
        assert!(
            temporary_rprompt.contains("hello"),
            "overlay message should be rendered in RPROMPT as well: {}",
            temporary_rprompt
        );
        assert_eq!(restored, "ORIGINAL_RPROMPT");
        assert_eq!(restored_rprompt, "ORIGINAL_RPROMPT");
    }

    #[test]
    fn message_overlay_keeps_full_fields() {
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
BUFFER="git st"
_nudge_auto_suggestion="git status"
_nudge_auto_warning=""
_nudge_overlay_render
print -r -- "$_nudge_overlay_last_message"
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
        let line = stdout.lines().next().unwrap_or_default();
        assert!(
            line.contains("why:"),
            "message overlay should include why field: {}",
            line
        );
        assert!(
            line.contains("risk:"),
            "message overlay should include risk field: {}",
            line
        );
        assert!(
            line.contains("diff:"),
            "message overlay should include diff field: {}",
            line
        );
    }

    #[test]
    fn manual_completion_clears_stale_postdisplay() {
        if !has_zsh() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "manual" ;;
      auto_delay_ms) echo "500" ;;
      zsh_ghost_owner) echo "auto" ;;
      zsh_overlay_backend) echo "message" ;;
      diagnosis_enabled) echo "false" ;;
      interactive_commands) echo "" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  elif [[ "$1" == "complete" ]]; then
    echo "git status"
  fi
}

source shell/integration.zsh >/dev/null 2>&1
BUFFER="git st"
CURSOR=${#BUFFER}
POSTDISPLAY="__AUTO__"
_nudge_complete
print -r -- "$BUFFER"
print -r -- "$POSTDISPLAY"
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
        let buffer = lines.next().unwrap_or_default();
        let postdisplay = lines.next().unwrap_or_default();

        assert_eq!(buffer, "git status");
        assert_eq!(postdisplay, "");
    }

    #[test]
    fn rprompt_overlay_reapplies_when_prompt_is_wiped() {
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
BUFFER="git st"
_nudge_auto_suggestion="git status"
_nudge_auto_warning=""
_nudge_overlay_render
RPS1=""
RPROMPT=""
_nudge_overlay_render
print -r -- "$RPS1"
print -r -- "$RPROMPT"
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
        let rps1 = lines.next().unwrap_or_default();
        let rprompt = lines.next().unwrap_or_default();

        assert!(
            rps1.contains("diff"),
            "RPS1 should render compact diff label: {}",
            rps1
        );
        assert!(
            rprompt.contains("diff"),
            "RPROMPT should render compact diff label: {}",
            rprompt
        );
        assert!(
            !rprompt.contains("why:"),
            "rprompt should not include full overlay fields: {}",
            rprompt
        );
    }
}
