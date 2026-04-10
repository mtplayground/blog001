use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SessionStore {
    inner: Arc<RwLock<HashMap<String, SessionEntry>>>,
    ttl: Duration,
}

#[derive(Clone, Debug)]
struct SessionEntry {
    username: String,
    expires_at: SystemTime,
}

impl SessionStore {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub fn create_session(&self, username: String) -> Option<String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = SystemTime::now().checked_add(self.ttl)?;

        let mut guard = self.inner.write().ok()?;
        guard.insert(
            token.clone(),
            SessionEntry {
                username,
                expires_at,
            },
        );

        Some(token)
    }

    pub fn validate_session(&self, token: &str) -> Option<String> {
        let mut guard = self.inner.write().ok()?;

        let is_expired = guard
            .get(token)
            .and_then(|entry| entry.expires_at.duration_since(SystemTime::now()).ok())
            .is_none();

        if is_expired {
            guard.remove(token);
            return None;
        }

        guard.get(token).map(|entry| entry.username.clone())
    }

    pub fn invalidate_session(&self, token: &str) -> bool {
        self.inner
            .write()
            .ok()
            .and_then(|mut guard| guard.remove(token))
            .is_some()
    }
}
