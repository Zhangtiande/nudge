use std::path::PathBuf;

use directories::BaseDirs;

pub struct AppPaths;

impl AppPaths {
    pub fn root_dir() -> PathBuf {
        if let Some(base_dirs) = BaseDirs::new() {
            base_dirs.home_dir().join(".nudge")
        } else {
            std::env::temp_dir().join(".nudge")
        }
    }

    pub fn config_dir() -> PathBuf {
        Self::root_dir().join("config")
    }

    pub fn run_dir() -> PathBuf {
        Self::root_dir().join("run")
    }

    pub fn data_dir() -> PathBuf {
        Self::root_dir().join("data")
    }

    pub fn logs_dir() -> PathBuf {
        Self::root_dir().join("logs")
    }

    pub fn shell_dir() -> PathBuf {
        Self::root_dir().join("shell")
    }

    pub fn modules_dir() -> PathBuf {
        Self::root_dir().join("modules")
    }

    pub fn lib_dir() -> PathBuf {
        Self::root_dir().join("lib")
    }

    pub fn default_config_path() -> PathBuf {
        Self::config_dir().join("config.default.yaml")
    }

    pub fn user_config_path() -> PathBuf {
        Self::config_dir().join("config.yaml")
    }

    pub fn pid_path() -> PathBuf {
        Self::run_dir().join("nudge.pid")
    }

    #[cfg(unix)]
    pub fn socket_path() -> PathBuf {
        Self::run_dir().join("nudge.sock")
    }

    #[cfg(windows)]
    pub fn socket_path() -> PathBuf {
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "default".to_string());
        PathBuf::from(format!(r"\\.\pipe\nudge_{}", username))
    }
}
