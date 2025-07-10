/*!
 * Session management system
 */

use crate::session::cookie::{Cookie, CookieJar, SameSite};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Session data storage
pub type SessionData = HashMap<String, String>;

/// Individual session
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub data: SessionData,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub expires_at: Option<SystemTime>,
}

impl Session {
    /// Create a new session
    pub fn new(id: String) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            data: HashMap::new(),
            created_at: now,
            last_accessed: now,
            expires_at: None,
        }
    }

    /// Create a session with expiration
    pub fn with_expiration(id: String, expires_in: Duration) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            data: HashMap::new(),
            created_at: now,
            last_accessed: now,
            expires_at: Some(now + expires_in),
        }
    }

    /// Update last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
    }

    /// Check if session has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }

    /// Get a value from session data
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Set a value in session data
    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
        self.touch();
    }

    /// Remove a value from session data
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let result = self.data.remove(key);
        if result.is_some() {
            self.touch();
        }
        result
    }

    /// Clear all session data
    pub fn clear(&mut self) {
        self.data.clear();
        self.touch();
    }

    /// Check if session contains a key
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get all keys in the session
    pub fn keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    /// Get the age of the session
    pub fn age(&self) -> Duration {
        SystemTime::now().duration_since(self.created_at).unwrap_or(Duration::ZERO)
    }

    /// Get time since last access
    pub fn idle_time(&self) -> Duration {
        SystemTime::now().duration_since(self.last_accessed).unwrap_or(Duration::ZERO)
    }
}

/// Session manager configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub cookie_name: String,
    pub cookie_path: String,
    pub cookie_domain: Option<String>,
    pub cookie_secure: bool,
    pub cookie_http_only: bool,
    pub cookie_same_site: Option<SameSite>,
    pub session_timeout: Duration,
    pub cleanup_interval: Duration,
    pub max_sessions: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            cookie_name: "SESSIONID".to_string(),
            cookie_path: "/".to_string(),
            cookie_domain: None,
            cookie_secure: false,
            cookie_http_only: true,
            cookie_same_site: Some(SameSite::Lax),
            session_timeout: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(300),  // 5 minutes
            max_sessions: 10000,
        }
    }
}

/// Session manager
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    config: SessionConfig,
    last_cleanup: Arc<Mutex<SystemTime>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            config,
            last_cleanup: Arc::new(Mutex::new(SystemTime::now())),
        }
    }

    /// Create a session manager with default configuration
    pub fn with_defaults() -> Self {
        Self::new(SessionConfig::default())
    }

    /// Generate a new session ID
    fn generate_session_id(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        SystemTime::now().hash(&mut hasher);
        std::process::id().hash(&mut hasher);

        // Add some randomness (simplified - in production use proper random number generator)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_nanos();
        timestamp.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }

    /// Create a new session
    pub fn create_session(&self) -> Result<String, String> {
        let session_id = self.generate_session_id();
        let session = Session::with_expiration(session_id.clone(), self.config.session_timeout);

        let mut sessions = self.sessions.lock().map_err(|_| "Failed to acquire session lock")?;

        // Check session limit
        if sessions.len() >= self.config.max_sessions {
            return Err("Maximum number of sessions reached".to_string());
        }

        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>, String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Failed to acquire session lock")?;

        if let Some(session) = sessions.get_mut(session_id) {
            if session.is_expired() {
                sessions.remove(session_id);
                Ok(None)
            } else {
                session.touch();
                Ok(Some(session.clone()))
            }
        } else {
            Ok(None)
        }
    }

    /// Update a session
    pub fn update_session(&self, session: Session) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Failed to acquire session lock")?;
        sessions.insert(session.id.clone(), session);
        Ok(())
    }

    /// Destroy a session
    pub fn destroy_session(&self, session_id: &str) -> Result<bool, String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Failed to acquire session lock")?;
        Ok(sessions.remove(session_id).is_some())
    }

    /// Get session from cookie jar
    pub fn get_session_from_cookies(&self, cookie_jar: &CookieJar) -> Result<Option<Session>, String> {
        if let Some(cookie) = cookie_jar.get(&self.config.cookie_name) {
            self.get_session(&cookie.value)
        } else {
            Ok(None)
        }
    }

    /// Create session cookie
    pub fn create_session_cookie(&self, session_id: &str) -> Cookie {
        let mut cookie = Cookie::new(self.config.cookie_name.clone(), session_id.to_string())
            .path(self.config.cookie_path.clone())
            .http_only(self.config.cookie_http_only)
            .secure(self.config.cookie_secure);

        if let Some(ref domain) = self.config.cookie_domain {
            cookie = cookie.domain(domain.clone());
        }

        if let Some(ref same_site) = self.config.cookie_same_site {
            cookie = cookie.same_site(same_site.clone());
        }

        // Set expiration based on session timeout
        let expires = SystemTime::now() + self.config.session_timeout;
        cookie.expires(expires)
    }

    /// Create session destruction cookie (expires immediately)
    pub fn create_destroy_cookie(&self) -> Cookie {
        let past_time = SystemTime::now() - Duration::from_secs(3600); // 1 hour ago

        let mut cookie = Cookie::new(self.config.cookie_name.clone(), "".to_string())
            .path(self.config.cookie_path.clone())
            .expires(past_time);

        if let Some(ref domain) = self.config.cookie_domain {
            cookie = cookie.domain(domain.clone());
        }

        cookie
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> Result<usize, String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Failed to acquire session lock")?;
        let initial_count = sessions.len();

        sessions.retain(|_, session| !session.is_expired());

        // Update last cleanup time
        if let Ok(mut last_cleanup) = self.last_cleanup.lock() {
            *last_cleanup = SystemTime::now();
        }

        Ok(initial_count - sessions.len())
    }

    /// Check if cleanup is needed and perform it
    pub fn maybe_cleanup(&self) -> Result<usize, String> {
        let should_cleanup = {
            if let Ok(last_cleanup) = self.last_cleanup.lock() {
                SystemTime::now().duration_since(*last_cleanup).unwrap_or(Duration::ZERO)
                    > self.config.cleanup_interval
            } else {
                false
            }
        };

        if should_cleanup {
            self.cleanup_expired_sessions()
        } else {
            Ok(0)
        }
    }

    /// Get session statistics
    pub fn get_stats(&self) -> Result<SessionStats, String> {
        let sessions = self.sessions.lock().map_err(|_| "Failed to acquire session lock")?;

        let total_sessions = sessions.len();
        let expired_sessions = sessions.values().filter(|s| s.is_expired()).count();
        let active_sessions = total_sessions - expired_sessions;

        Ok(SessionStats {
            total_sessions,
            active_sessions,
            expired_sessions,
            max_sessions: self.config.max_sessions,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
    pub max_sessions: usize,
}

impl SessionStats {
    /// Get memory usage percentage
    pub fn memory_usage_percent(&self) -> f64 {
        if self.max_sessions == 0 {
            0.0
        } else {
            (self.total_sessions as f64 / self.max_sessions as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("test_id".to_string());
        assert_eq!(session.id, "test_id");
        assert!(session.data.is_empty());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_data_operations() {
        let mut session = Session::new("test".to_string());

        session.set("key1".to_string(), "value1".to_string());
        assert_eq!(session.get("key1"), Some(&"value1".to_string()));
        assert!(session.contains_key("key1"));

        session.remove("key1");
        assert!(!session.contains_key("key1"));
    }

    #[test]
    fn test_session_manager() {
        let manager = SessionManager::with_defaults();

        let session_id = manager.create_session().unwrap();
        assert!(!session_id.is_empty());

        let session = manager.get_session(&session_id).unwrap();
        assert!(session.is_some());

        let destroyed = manager.destroy_session(&session_id).unwrap();
        assert!(destroyed);

        let session = manager.get_session(&session_id).unwrap();
        assert!(session.is_none());
    }
}
