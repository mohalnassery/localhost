/*!
 * CGI environment variable handling
 */

use crate::config::ServerConfig;
use crate::http::HttpRequest;
use std::collections::HashMap;
use std::ffi::OsString;

/// CGI environment variable builder
pub struct CgiEnvironment {
    variables: HashMap<String, String>,
}

impl CgiEnvironment {
    /// Create a new CGI environment
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Build CGI environment from HTTP request and server configuration
    pub fn from_request(
        request: &HttpRequest,
        server_config: &ServerConfig,
        script_path: &str,
        path_info: &str,
    ) -> Self {
        let mut env = Self::new();

        // Required CGI variables
        env.set("GATEWAY_INTERFACE", "CGI/1.1");
        env.set("SERVER_SOFTWARE", "localhost-http-server/0.1.0");
        env.set("SERVER_PROTOCOL", request.version.as_str());
        env.set("REQUEST_METHOD", request.method.as_str());
        env.set("REQUEST_URI", &request.uri);
        env.set("SCRIPT_NAME", script_path);
        env.set("PATH_INFO", path_info);

        // Server information
        env.set("SERVER_NAME", server_config.server_name.as_deref().unwrap_or("localhost"));
        env.set("SERVER_PORT", &server_config.ports.first().unwrap_or(&80).to_string());

        // Request information
        if let Some(query) = request.uri.split('?').nth(1) {
            env.set("QUERY_STRING", query);
        } else {
            env.set("QUERY_STRING", "");
        }

        // Content information
        if !request.body.is_empty() {
            env.set("CONTENT_LENGTH", &request.body.len().to_string());
        }

        if let Some(content_type) = request.get_header("content-type") {
            env.set("CONTENT_TYPE", content_type);
        }

        // HTTP headers (convert to CGI format)
        for (name, value) in &request.headers {
            let cgi_name = format!("HTTP_{}", name.to_uppercase().replace('-', "_"));
            env.set(&cgi_name, value);
        }

        // Remote information (simplified for localhost)
        env.set("REMOTE_ADDR", "127.0.0.1");
        env.set("REMOTE_HOST", "localhost");

        // Authentication (if present)
        if let Some(_auth) = request.get_header("authorization") {
            env.set("AUTH_TYPE", "Basic"); // Simplified
            env.set("REMOTE_USER", ""); // Would need to parse auth header
        }

        // Path translation
        env.set("PATH_TRANSLATED", script_path);

        env
    }

    /// Set an environment variable
    pub fn set(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }

    /// Get an environment variable
    pub fn get(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Convert to vector of OsString for process execution
    pub fn to_env_vars(&self) -> Vec<(OsString, OsString)> {
        self.variables
            .iter()
            .map(|(k, v)| (OsString::from(k), OsString::from(v)))
            .collect()
    }

    /// Get all variables as a HashMap
    pub fn variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Add custom environment variable
    pub fn add_custom(&mut self, name: &str, value: &str) {
        self.set(name, value);
    }

    /// Remove an environment variable
    pub fn remove(&mut self, name: &str) {
        self.variables.remove(name);
    }

    /// Clear all environment variables
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Get the number of environment variables
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Check if environment is empty
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Validate required CGI environment variables
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let required_vars = [
            "GATEWAY_INTERFACE",
            "SERVER_SOFTWARE",
            "SERVER_PROTOCOL",
            "REQUEST_METHOD",
            "SCRIPT_NAME",
        ];

        let mut missing = Vec::new();
        for var in &required_vars {
            if !self.variables.contains_key(*var) {
                missing.push(var.to_string());
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

impl Default for CgiEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{HttpMethod, HttpVersion};

    #[test]
    fn test_cgi_environment_creation() {
        let mut request = HttpRequest::new();
        request.method = HttpMethod::GET;
        request.version = HttpVersion::Http11;
        request.uri = "/cgi-bin/test.py?param=value".to_string();
        request.path = "/cgi-bin/test.py".to_string();

        let server_config = ServerConfig::default();
        let env = CgiEnvironment::from_request(&request, &server_config, "/cgi-bin/test.py", "");

        assert_eq!(env.get("REQUEST_METHOD"), Some(&"GET".to_string()));
        assert_eq!(env.get("QUERY_STRING"), Some(&"param=value".to_string()));
        assert_eq!(env.get("SCRIPT_NAME"), Some(&"/cgi-bin/test.py".to_string()));
        assert!(env.validate().is_ok());
    }

    #[test]
    fn test_environment_validation() {
        let mut env = CgiEnvironment::new();
        assert!(env.validate().is_err());

        env.set("GATEWAY_INTERFACE", "CGI/1.1");
        env.set("SERVER_SOFTWARE", "test");
        env.set("SERVER_PROTOCOL", "HTTP/1.1");
        env.set("REQUEST_METHOD", "GET");
        env.set("SCRIPT_NAME", "/test.py");

        assert!(env.validate().is_ok());
    }
}
