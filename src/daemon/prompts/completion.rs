use crate::daemon::shell_mode::ShellMode;

const DEFAULT_SYSTEM_PROMPT: &str = include_str!("templates/completion/system.md");
const CONTRACT_BASH_POPUP: &str = include_str!("templates/completion/contracts/bash-popup.md");
const CONTRACT_ZSH: &str = include_str!("templates/completion/contracts/zsh.md");
const CONTRACT_INLINE: &str = include_str!("templates/completion/contracts/inline.md");

pub fn default_system_prompt() -> &'static str {
    DEFAULT_SYSTEM_PROMPT
}

pub fn response_contract(shell_mode: ShellMode) -> &'static str {
    match shell_mode {
        ShellMode::BashPopup => CONTRACT_BASH_POPUP,
        ShellMode::ZshAuto | ShellMode::ZshInline => CONTRACT_ZSH,
        _ => CONTRACT_INLINE,
    }
}

#[cfg(test)]
mod tests {
    use super::{default_system_prompt, response_contract};
    use crate::daemon::shell_mode::ShellMode;

    #[test]
    fn system_prompt_template_is_non_empty() {
        assert!(default_system_prompt().contains("CLI command completion assistant"));
    }

    #[test]
    fn shell_mode_contract_switching_works() {
        assert!(response_contract(ShellMode::BashPopup).contains("bash-popup"));
        assert!(response_contract(ShellMode::ZshAuto).contains("Shell mode: zsh"));
        assert!(response_contract(ShellMode::PsInline).contains("Shell mode: inline"));
    }
}
