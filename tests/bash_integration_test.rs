#[cfg(unix)]
mod tests {
    use std::process::Command;

    fn has_bash() -> bool {
        Command::new("bash").arg("--version").output().is_ok()
    }

    #[test]
    fn bash_popup_functions_load_with_builtin_backend() {
        if !has_bash() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "manual" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  fi
}

NUDGE_POPUP_BACKEND="builtin"
NUDGE_POPUP_CONFIRM_RISKY=0
source shell/integration.bash >/dev/null 2>&1
declare -F _nudge_complete >/dev/null && echo "complete=ok" || echo "complete=missing"
declare -F _nudge_popup_complete >/dev/null && echo "popup=ok" || echo "popup=missing"
backend=$(_nudge_resolve_popup_backend)
echo "backend=$backend"
"#;

        let output = Command::new("bash")
            .arg("-lc")
            .arg(script)
            .output()
            .expect("failed to run bash");

        assert!(
            output.status.success(),
            "bash script failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("complete=ok"));
        assert!(stdout.contains("popup=ok"));
        assert!(stdout.contains("backend=builtin"));
    }

    #[test]
    fn bash_popup_complete_applies_selected_candidate() {
        if !has_bash() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "manual" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  fi
}

NUDGE_POPUP_BACKEND="builtin"
NUDGE_POPUP_CONFIRM_RISKY=0
source shell/integration.bash >/dev/null 2>&1

_nudge_request_completion() {
  printf 'low\tgit status\t\tprefix completion\t+atus\n'
}

_nudge_select_candidate_builtin() {
  printf 'low\tgit status\t\tprefix completion\t+atus'
}

_nudge_show_warning() {
  echo "warn:$1"
}

READLINE_LINE="git st"
READLINE_POINT=${#READLINE_LINE}
_nudge_popup_complete
echo "line=$READLINE_LINE"
echo "point=$READLINE_POINT"
"#;

        let output = Command::new("bash")
            .arg("-lc")
            .arg(script)
            .output()
            .expect("failed to run bash");

        assert!(
            output.status.success(),
            "bash script failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("line=git status"));
        assert!(stdout.contains("point=10"));
        assert!(
            !stdout.contains("warn:"),
            "low-risk candidate with empty warning should not print warning: {}",
            stdout
        );
    }

    #[test]
    fn bash_popup_high_risk_candidate_requires_confirmation_by_default() {
        if !has_bash() {
            return;
        }

        let script = r#"
function nudge() {
  if [[ "$1" == "info" && "$2" == "--field" ]]; then
    case "$3" in
      config_dir) echo "/tmp" ;;
      socket_path) echo "/tmp/nudge.sock" ;;
      trigger_mode) echo "manual" ;;
      *) echo "" ;;
    esac
  elif [[ "$1" == "status" ]]; then
    return 0
  fi
}

NUDGE_POPUP_BACKEND="builtin"
source shell/integration.bash >/dev/null 2>&1

_nudge_request_completion() {
  printf 'high\trm -rf /\tDangerous command\tcontext rewrite\t~ rm -> rm -rf /\n'
}

_nudge_select_candidate_builtin() {
  printf 'high\trm -rf /\tDangerous command\tcontext rewrite\t~ rm -> rm -rf /'
}

READLINE_LINE="rm"
READLINE_POINT=${#READLINE_LINE}
_nudge_popup_complete
echo "line=$READLINE_LINE"
"#;

        let output = Command::new("bash")
            .arg("-lc")
            .arg(script)
            .output()
            .expect("failed to run bash");

        assert!(
            output.status.success(),
            "bash script failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("line=rm"));
    }
}
