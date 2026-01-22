use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use chrono::{DateTime, Utc};

/// A shell session
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Session {
    pub id: String,
    pub shell_type: ShellType,
    pub started_at: DateTime<Utc>,
    pub cwd: PathBuf,
    pub last_activity: DateTime<Utc>,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellType {
    Bash,
    Zsh,
    Unknown,
}

impl Session {
    pub fn new(id: String, cwd: PathBuf) -> Self {
        let shell_type = if id.starts_with("bash-") {
            ShellType::Bash
        } else if id.starts_with("zsh-") {
            ShellType::Zsh
        } else {
            ShellType::Unknown
        };

        let now = Utc::now();
        Self {
            id,
            shell_type,
            started_at: now,
            cwd,
            last_activity: now,
            active: true,
        }
    }

    pub fn update(&mut self, cwd: &Path) {
        self.cwd = cwd.to_path_buf();
        self.last_activity = Utc::now();
        self.active = true;
    }
}

/// Thread-safe session store
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a session
    #[allow(dead_code)]
    pub fn get_session(&self, id: &str) -> Option<Session> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(id).cloned()
    }

    /// Update session state
    pub fn update_session(&self, id: &str, cwd: &Path) {
        let mut sessions = self.sessions.write().unwrap();

        if let Some(session) = sessions.get_mut(id) {
            session.update(cwd);
        } else {
            let session = Session::new(id.to_string(), cwd.to_path_buf());
            sessions.insert(id.to_string(), session);
        }
    }

    /// Remove inactive sessions older than the given duration
    #[allow(dead_code)]
    pub fn cleanup(&self, max_age: chrono::Duration) {
        let mut sessions = self.sessions.write().unwrap();
        let cutoff = Utc::now() - max_age;

        sessions.retain(|_, session| session.last_activity > cutoff);
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
