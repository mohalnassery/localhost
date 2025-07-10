/*!
 * HTTP headers handling
 */

use std::collections::HashMap;

/// Common HTTP header names
pub struct HeaderNames;

impl HeaderNames {
    pub const ACCEPT: &'static str = "accept";
    pub const ACCEPT_ENCODING: &'static str = "accept-encoding";
    pub const ACCEPT_LANGUAGE: &'static str = "accept-language";
    pub const AUTHORIZATION: &'static str = "authorization";
    pub const CACHE_CONTROL: &'static str = "cache-control";
    pub const CONNECTION: &'static str = "connection";
    pub const CONTENT_ENCODING: &'static str = "content-encoding";
    pub const CONTENT_LENGTH: &'static str = "content-length";
    pub const CONTENT_TYPE: &'static str = "content-type";
    pub const COOKIE: &'static str = "cookie";
    pub const DATE: &'static str = "date";
    pub const ETAG: &'static str = "etag";
    pub const EXPIRES: &'static str = "expires";
    pub const HOST: &'static str = "host";
    pub const IF_MODIFIED_SINCE: &'static str = "if-modified-since";
    pub const IF_NONE_MATCH: &'static str = "if-none-match";
    pub const LAST_MODIFIED: &'static str = "last-modified";
    pub const LOCATION: &'static str = "location";
    pub const REFERER: &'static str = "referer";
    pub const SERVER: &'static str = "server";
    pub const SET_COOKIE: &'static str = "set-cookie";
    pub const TRANSFER_ENCODING: &'static str = "transfer-encoding";
    pub const USER_AGENT: &'static str = "user-agent";
    pub const WWW_AUTHENTICATE: &'static str = "www-authenticate";
}

/// HTTP headers collection with case-insensitive access
#[derive(Debug, Clone)]
pub struct Headers {
    headers: HashMap<String, String>,
}

impl Headers {
    /// Create a new headers collection
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    /// Add a header (case-insensitive)
    pub fn add(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_lowercase(), value.to_string());
    }

    /// Get a header value (case-insensitive)
    pub fn get(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    /// Remove a header (case-insensitive)
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.headers.remove(&name.to_lowercase())
    }

    /// Check if a header exists (case-insensitive)
    pub fn contains(&self, name: &str) -> bool {
        self.headers.contains_key(&name.to_lowercase())
    }

    /// Get all headers
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.headers.iter()
    }

    /// Get content length
    pub fn content_length(&self) -> Option<usize> {
        self.get(HeaderNames::CONTENT_LENGTH)
            .and_then(|v| v.parse().ok())
    }

    /// Check if connection should be kept alive
    pub fn keep_alive(&self) -> bool {
        self.get(HeaderNames::CONNECTION)
            .map(|v| v.to_lowercase() == "keep-alive")
            .unwrap_or(false)
    }

    /// Check if transfer encoding is chunked
    pub fn is_chunked(&self) -> bool {
        self.get(HeaderNames::TRANSFER_ENCODING)
            .map(|v| v.to_lowercase().contains("chunked"))
            .unwrap_or(false)
    }

    /// Get host header
    pub fn host(&self) -> Option<&String> {
        self.get(HeaderNames::HOST)
    }

    /// Get user agent
    pub fn user_agent(&self) -> Option<&String> {
        self.get(HeaderNames::USER_AGENT)
    }

    /// Get content type
    pub fn content_type(&self) -> Option<&String> {
        self.get(HeaderNames::CONTENT_TYPE)
    }

    /// Parse content type to get media type and charset
    pub fn parse_content_type(&self) -> Option<(String, Option<String>)> {
        self.content_type().map(|ct| {
            let parts: Vec<&str> = ct.split(';').collect();
            let media_type = parts[0].trim().to_string();

            let charset = parts.iter()
                .skip(1)
                .find_map(|part| {
                    let part = part.trim();
                    if part.starts_with("charset=") {
                        Some(part[8..].trim().to_string())
                    } else {
                        None
                    }
                });

            (media_type, charset)
        })
    }

    /// Get cookies as a map
    pub fn cookies(&self) -> HashMap<String, String> {
        let mut cookies = HashMap::new();

        if let Some(cookie_header) = self.get(HeaderNames::COOKIE) {
            for cookie in cookie_header.split(';') {
                let cookie = cookie.trim();
                if let Some(eq_pos) = cookie.find('=') {
                    let name = cookie[..eq_pos].trim().to_string();
                    let value = cookie[eq_pos + 1..].trim().to_string();
                    cookies.insert(name, value);
                }
            }
        }

        cookies
    }

    /// Clear all headers
    pub fn clear(&mut self) {
        self.headers.clear();
    }

    /// Get number of headers
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Check if headers collection is empty
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for Headers {
    fn from(headers: HashMap<String, String>) -> Self {
        let mut result = Headers::new();
        for (name, value) in headers {
            result.add(&name, &value);
        }
        result
    }
}
