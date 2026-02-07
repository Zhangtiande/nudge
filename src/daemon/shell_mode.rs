//! Shell mode parsing and capability flags.
//!
//! Keep shell-specific branching centralized here so daemon logic remains
//! extensible across platforms.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMode {
    ZshAuto,
    ZshInline,
    BashInline,
    BashPopup,
    PsInline,
    CmdInline,
    Unknown,
}

impl ShellMode {
    /// Resolve shell mode from explicit request field or session id fallback.
    pub fn resolve(explicit_mode: Option<&str>, session_id: &str) -> Self {
        if let Some(mode) = explicit_mode {
            let parsed = Self::parse(mode);
            if parsed != Self::Unknown {
                return parsed;
            }
        }
        Self::from_session(session_id)
    }

    /// Canonical mode string used in cache keys and logs.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ZshAuto => "zsh-auto",
            Self::ZshInline => "zsh-inline",
            Self::BashInline => "bash-inline",
            Self::BashPopup => "bash-popup",
            Self::PsInline => "ps-inline",
            Self::CmdInline => "cmd-inline",
            Self::Unknown => "unknown",
        }
    }

    /// Auto modes get shorter cache TTL due to high request frequency.
    pub fn is_auto(self) -> bool {
        matches!(self, Self::ZshAuto)
    }

    /// Popup modes benefit from multiple ranked suggestions.
    pub fn supports_multi_candidates(self) -> bool {
        matches!(self, Self::BashPopup)
    }

    fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "zsh-auto" => Self::ZshAuto,
            "zsh-inline" => Self::ZshInline,
            "bash-inline" => Self::BashInline,
            "bash-popup" => Self::BashPopup,
            "ps-inline" => Self::PsInline,
            "cmd-inline" => Self::CmdInline,
            _ => Self::Unknown,
        }
    }

    fn from_session(session_id: &str) -> Self {
        if session_id.starts_with("zsh-") {
            Self::ZshInline
        } else if session_id.starts_with("bash-") {
            // Keep session fallback on the fastest/safest baseline path.
            Self::BashInline
        } else if session_id.starts_with("pwsh-") || session_id.starts_with("powershell-") {
            Self::PsInline
        } else if session_id.starts_with("cmd-") {
            Self::CmdInline
        } else {
            Self::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ShellMode;

    #[test]
    fn resolve_prefers_known_explicit_mode() {
        let mode = ShellMode::resolve(Some("bash-popup"), "zsh-123");
        assert_eq!(mode, ShellMode::BashPopup);
    }

    #[test]
    fn resolve_falls_back_to_session_when_explicit_unknown() {
        let mode = ShellMode::resolve(Some("future-mode"), "pwsh-42");
        assert_eq!(mode, ShellMode::PsInline);
    }

    #[test]
    fn supports_multi_candidates_only_for_popup_modes() {
        assert!(ShellMode::BashPopup.supports_multi_candidates());
        assert!(!ShellMode::BashInline.supports_multi_candidates());
        assert!(!ShellMode::ZshInline.supports_multi_candidates());
        assert!(!ShellMode::PsInline.supports_multi_candidates());
    }

    #[test]
    fn session_fallback_prefers_bash_inline_mode() {
        let mode = ShellMode::resolve(None, "bash-123");
        assert_eq!(mode, ShellMode::BashInline);
    }
}
