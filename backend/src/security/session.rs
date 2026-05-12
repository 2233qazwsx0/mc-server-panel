use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_timeout_mins: u64,
    pub absolute_timeout_mins: u64,
    pub max_sessions_per_user: usize,
    pub refresh_token_enabled: bool,
    pub refresh_token_expiry_days: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_timeout_mins: 30,
            absolute_timeout_mins: 480,
            max_sessions_per_user: 5,
            refresh_token_enabled: true,
            refresh_token_expiry_days: 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub expires_at: Instant,
    pub absolute_expires_at: Instant,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub permissions: Vec<String>,
    pub refresh_token: Option<String>,
}

impl Session {
    pub fn new(user_id: String, username: String, config: &SessionConfig) -> Self {
        let now = Instant::now();
        let refresh_token = if config.refresh_token_enabled {
            Some(uuid::Uuid::new_v4().to_string())
        } else {
            None
        };

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id,
            username,
            created_at: now,
            last_activity: now,
            expires_at: now + Duration::from_secs(config.session_timeout_mins * 60),
            absolute_expires_at: now + Duration::from_secs(config.absolute_timeout_mins * 60),
            ip_address: None,
            user_agent: None,
            is_active: true,
            permissions: Vec::new(),
            refresh_token,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at || Instant::now() > self.absolute_expires_at
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    pub fn touch(&mut self, config: &SessionConfig) {
        self.last_activity = Instant::now();
        self.expires_at = Instant::now() + Duration::from_secs(config.session_timeout_mins * 60);
    }

    pub fn invalidate(&mut self) {
        self.is_active = false;
    }

    pub fn update_context(&mut self, ip_address: Option<String>, user_agent: Option<String>) {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
    }

    pub fn add_permission(&mut self, permission: String) {
        if !self.permissions.contains(&permission) {
            self.permissions.push(permission);
        }
    }

    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.retain(|p| p != permission);
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission || p == "*")
    }

    pub fn remaining_ttl(&self) -> Duration {
        self.expires_at.saturating_duration_since(Instant::now())
    }
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    user_sessions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    config: Arc<RwLock<SessionConfig>>,
    listeners: Arc<RwLock<Vec<Box<dyn Fn(&Session, SessionEvent) + Send + Sync>>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(SessionConfig::default())),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn set_config(&self, config: SessionConfig) {
        *self.config.write() = config;
    }

    pub fn get_config(&self) -> SessionConfig {
        self.config.read().clone()
    }

    pub fn create_session(
        &self,
        user_id: &str,
        username: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<Session, String> {
        let config = self.config.read().clone();

        let user_session_ids = self
            .user_sessions
            .read()
            .get(user_id)
            .cloned()
            .unwrap_or_default();
        let active_user_sessions = user_session_ids
            .iter()
            .filter_map(|id| self.sessions.read().get(id))
            .filter(|s| s.is_valid())
            .count();

        if active_user_sessions >= config.max_sessions_per_user {
            if let Some(oldest) = user_session_ids
                .iter()
                .filter_map(|id| self.sessions.read().get(id))
                .filter(|s| s.is_valid())
                .min_by_key(|s| s.created_at)
            {
                self.invalidate_session(&oldest.id)?;
            }
        }

        let mut session = Session::new(user_id.to_string(), username.to_string(), &config);
        session.update_context(ip_address.clone(), user_agent.clone());

        self.sessions
            .write()
            .insert(session.id.clone(), session.clone());
        self.user_sessions
            .write()
            .entry(user_id.to_string())
            .or_insert_with(Vec::new)
            .push(session.id.clone());

        self.notify_listeners(&session, SessionEvent::Created);

        Ok(session)
    }

    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.read().get(session_id).cloned()
    }

    pub fn validate_session(&self, session_id: &str) -> SessionValidationResult {
        let sessions = self.sessions.read();
        let session = match sessions.get(session_id) {
            Some(s) => s,
            None => return SessionValidationResult::NotFound,
        };

        if !session.is_active {
            return SessionValidationResult::Invalid {
                reason: "session invalidated".to_string(),
            };
        }

        if session.is_expired() {
            return SessionValidationResult::Expired {
                session_id: session_id.to_string(),
            };
        }

        SessionValidationResult::Valid {
            session: session.clone(),
        }
    }

    pub fn refresh_session(&self, session_id: &str) -> Result<Session, String> {
        let config = self.config.read().clone();
        let mut sessions = self.sessions.write();

        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        if !session.is_active {
            return Err("Session invalidated".to_string());
        }

        if session.is_expired() {
            return Err("Session expired".to_string());
        }

        session.touch(&config);

        self.notify_listeners(session, SessionEvent::Refreshed);

        Ok(session.clone())
    }

    pub fn invalidate_session(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write();
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        session.is_active = false;

        let user_id = session.user_id.clone();
        drop(session);

        if let Some(ids) = self.user_sessions.write().get_mut(&user_id) {
            ids.retain(|id| id != session_id);
            if ids.is_empty() {
                self.user_sessions.write().remove(&user_id);
            }
        }

        let session_clone = sessions.get(session_id).cloned();
        if let Some(s) = session_clone {
            self.notify_listeners(&s, SessionEvent::Invalidated);
        }

        Ok(())
    }

    pub fn invalidate_all_user_sessions(&self, user_id: &str) -> usize {
        let session_ids: Vec<String> = self
            .user_sessions
            .read()
            .get(user_id)
            .cloned()
            .unwrap_or_default();

        let mut count = 0;
        for id in session_ids {
            if self.invalidate_session(&id).is_ok() {
                count += 1;
            }
        }

        count
    }

    pub fn get_user_sessions(&self, user_id: &str) -> Vec<Session> {
        self.user_sessions
            .read()
            .get(user_id)
            .iter()
            .flat_map(|ids| ids.iter())
            .filter_map(|id| self.sessions.read().get(id))
            .cloned()
            .collect()
    }

    pub fn get_active_sessions(&self) -> Vec<Session> {
        self.sessions
            .read()
            .values()
            .filter(|s| s.is_valid())
            .cloned()
            .collect()
    }

    pub fn cleanup_expired(&self) -> usize {
        let mut sessions = self.sessions.write();
        let mut removed = 0;

        sessions.retain(|id, session| {
            if session.is_valid() {
                true
            } else {
                removed += 1;
                false
            }
        });

        let mut user_sessions = self.user_sessions.write();
        user_sessions.retain(|_, ids| {
            ids.retain(|id| sessions.contains_key(id));
            !ids.is_empty()
        });

        removed
    }

    pub fn get_stats(&self) -> SessionStats {
        let sessions = self.sessions.read();
        let total = sessions.len();
        let active = sessions.values().filter(|s| s.is_valid()).count();

        let mut user_count = std::collections::HashMap::new();
        for session in sessions.values() {
            *user_count.entry(session.user_id.clone()).or_insert(0) += 1;
        }

        let mut ip_count = std::collections::HashMap::new();
        for session in sessions.values() {
            if let Some(ip) = &session.ip_address {
                *ip_count.entry(ip.clone()).or_insert(0) += 1;
            }
        }

        SessionStats {
            total_sessions: total,
            active_sessions: active,
            expired_sessions: total - active,
            unique_users: user_count.len(),
            sessions_per_user: user_count,
            sessions_per_ip: ip_count,
        }
    }

    pub fn update_session_context(
        &self,
        session_id: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.write();
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;

        session.update_context(ip_address, user_agent);
        Ok(())
    }

    pub fn update_permissions(
        &self,
        session_id: &str,
        permissions: Vec<String>,
    ) -> Result<(), String> {
        let mut sessions = self.sessions.write();
        let session = sessions.get_mut(session_id).ok_or("Session not found")?;
        session.permissions = permissions;
        Ok(())
    }

    pub fn add_listener<F>(&self, listener: F)
    where
        F: Fn(&Session, SessionEvent) + Send + Sync + 'static,
    {
        self.listeners.write().push(Box::new(listener));
    }

    fn notify_listeners(&self, session: &Session, event: SessionEvent) {
        let listeners = self.listeners.read();
        for listener in listeners.iter() {
            listener(session, event.clone());
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionValidationResult {
    Valid { session: Session },
    Expired { session_id: String },
    Invalid { reason: String },
    NotFound,
}

impl SessionValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, SessionValidationResult::Valid { .. })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    Created,
    Refreshed,
    Invalidated,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
    pub unique_users: usize,
    pub sessions_per_user: std::collections::HashMap<String, usize>,
    pub sessions_per_ip: std::collections::HashMap<String, usize>,
}
