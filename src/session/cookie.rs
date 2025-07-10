/*!
 * HTTP Cookie handling and management
 */

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// HTTP Cookie representation
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<SystemTime>,
    pub max_age: Option<Duration>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>,
}

/// SameSite cookie attribute
#[derive(Debug, Clone, PartialEq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl fmt::Display for SameSite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SameSite::Strict => write!(f, "Strict"),
            SameSite::Lax => write!(f, "Lax"),
            SameSite::None => write!(f, "None"),
        }
    }
}

impl Cookie {
    /// Create a new cookie
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            domain: None,
            path: None,
            expires: None,
            max_age: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    /// Create a session cookie (expires when browser closes)
    pub fn session(name: String, value: String) -> Self {
        Self::new(name, value)
    }

    /// Create a persistent cookie with max-age
    pub fn persistent(name: String, value: String, max_age: Duration) -> Self {
        let mut cookie = Self::new(name, value);
        cookie.max_age = Some(max_age);
        cookie
    }

    /// Set the domain for the cookie
    pub fn domain(mut self, domain: String) -> Self {
        self.domain = Some(domain);
        self
    }

    /// Set the path for the cookie
    pub fn path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    /// Set the expires time for the cookie
    pub fn expires(mut self, expires: SystemTime) -> Self {
        self.expires = Some(expires);
        self
    }

    /// Set the max-age for the cookie
    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.max_age = Some(max_age);
        self
    }

    /// Mark the cookie as secure (HTTPS only)
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Mark the cookie as HTTP-only (not accessible via JavaScript)
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Set the SameSite attribute
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// Convert cookie to Set-Cookie header value
    pub fn to_header_value(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];

        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }

        if let Some(ref path) = self.path {
            parts.push(format!("Path={}", path));
        }

        if let Some(expires) = self.expires {
            if let Ok(duration) = expires.duration_since(UNIX_EPOCH) {
                // Format as HTTP date (RFC 7231)
                let timestamp = duration.as_secs();
                parts.push(format!("Expires={}", format_http_date(timestamp)));
            }
        }

        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age.as_secs()));
        }

        if self.secure {
            parts.push("Secure".to_string());
        }

        if self.http_only {
            parts.push("HttpOnly".to_string());
        }

        if let Some(ref same_site) = self.same_site {
            parts.push(format!("SameSite={}", same_site));
        }

        parts.join("; ")
    }

    /// Check if the cookie has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            return SystemTime::now() > expires;
        }
        false
    }

    /// Check if the cookie is valid for the given domain and path
    pub fn is_valid_for(&self, domain: &str, path: &str) -> bool {
        // Check domain
        if let Some(ref cookie_domain) = self.domain {
            if !domain.ends_with(cookie_domain) {
                return false;
            }
        }

        // Check path
        if let Some(ref cookie_path) = self.path {
            if !path.starts_with(cookie_path) {
                return false;
            }
        }

        // Check if expired
        !self.is_expired()
    }
}

/// Cookie jar for managing multiple cookies
#[derive(Debug, Default, Clone)]
pub struct CookieJar {
    cookies: HashMap<String, Cookie>,
}

impl CookieJar {
    /// Create a new cookie jar
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Add a cookie to the jar
    pub fn add(&mut self, cookie: Cookie) {
        self.cookies.insert(cookie.name.clone(), cookie);
    }

    /// Get a cookie by name
    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.cookies.get(name)
    }

    /// Remove a cookie by name
    pub fn remove(&mut self, name: &str) -> Option<Cookie> {
        self.cookies.remove(name)
    }

    /// Get all cookies
    pub fn cookies(&self) -> &HashMap<String, Cookie> {
        &self.cookies
    }

    /// Parse cookies from Cookie header value
    pub fn parse_cookie_header(&mut self, header_value: &str) {
        for pair in header_value.split(';') {
            let pair = pair.trim();
            if let Some(eq_pos) = pair.find('=') {
                let name = pair[..eq_pos].trim().to_string();
                let value = pair[eq_pos + 1..].trim().to_string();
                self.add(Cookie::new(name, value));
            }
        }
    }

    /// Generate Cookie header value for requests
    pub fn to_cookie_header(&self, domain: &str, path: &str) -> Option<String> {
        let valid_cookies: Vec<String> = self.cookies
            .values()
            .filter(|cookie| cookie.is_valid_for(domain, path))
            .map(|cookie| format!("{}={}", cookie.name, cookie.value))
            .collect();

        if valid_cookies.is_empty() {
            None
        } else {
            Some(valid_cookies.join("; "))
        }
    }

    /// Generate Set-Cookie headers for responses
    pub fn to_set_cookie_headers(&self) -> Vec<String> {
        self.cookies
            .values()
            .map(|cookie| cookie.to_header_value())
            .collect()
    }

    /// Clear expired cookies
    pub fn clear_expired(&mut self) {
        self.cookies.retain(|_, cookie| !cookie.is_expired());
    }

    /// Clear all cookies
    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    /// Get the number of cookies
    pub fn len(&self) -> usize {
        self.cookies.len()
    }

    /// Check if the jar is empty
    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }
}

/// Format timestamp as HTTP date (simplified)
fn format_http_date(timestamp: u64) -> String {
    // This is a simplified implementation
    // In production, you'd want to use a proper date formatting library
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                  "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

    let total_days = timestamp / 86400;
    let day_of_week = (total_days + 4) % 7; // Unix epoch was Thursday

    // Simplified date calculation (not accounting for leap years properly)
    let year = 1970 + total_days / 365;
    let day_of_year = total_days % 365;
    let month = day_of_year / 30; // Simplified
    let day = (day_of_year % 30) + 1;

    let hour = (timestamp % 86400) / 3600;
    let minute = (timestamp % 3600) / 60;
    let second = timestamp % 60;

    format!("{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
            days[day_of_week as usize],
            day,
            months[month.min(11) as usize],
            year,
            hour,
            minute,
            second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cookie_creation() {
        let cookie = Cookie::new("session_id".to_string(), "abc123".to_string());
        assert_eq!(cookie.name, "session_id");
        assert_eq!(cookie.value, "abc123");
        assert!(!cookie.secure);
        assert!(!cookie.http_only);
    }

    #[test]
    fn test_cookie_header_value() {
        let cookie = Cookie::new("test".to_string(), "value".to_string())
            .path("/".to_string())
            .http_only(true)
            .secure(true);

        let header = cookie.to_header_value();
        assert!(header.contains("test=value"));
        assert!(header.contains("Path=/"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("Secure"));
    }

    #[test]
    fn test_cookie_jar() {
        let mut jar = CookieJar::new();
        let cookie = Cookie::new("test".to_string(), "value".to_string());

        jar.add(cookie);
        assert_eq!(jar.len(), 1);
        assert!(jar.get("test").is_some());
        assert!(jar.get("nonexistent").is_none());
    }

    #[test]
    fn test_cookie_parsing() {
        let mut jar = CookieJar::new();
        jar.parse_cookie_header("session_id=abc123; user_pref=dark_mode");

        assert_eq!(jar.len(), 2);
        assert_eq!(jar.get("session_id").unwrap().value, "abc123");
        assert_eq!(jar.get("user_pref").unwrap().value, "dark_mode");
    }
}
