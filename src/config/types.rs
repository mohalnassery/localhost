/*!
 * Configuration data structures
 */

use std::collections::HashMap;

/// Main configuration structure
#[derive(Debug, Clone)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
}

/// Individual server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub ports: Vec<u16>,
    pub server_name: Option<String>,
    pub error_pages: HashMap<u16, String>,
    pub max_body_size: usize,
    pub routes: Vec<RouteConfig>,
}

/// Route configuration
#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub path: String,
    pub methods: Vec<String>,
    pub redirect: Option<String>,
    pub root: Option<String>,
    pub index: Option<String>,
    pub cgi: Option<String>,
    pub directory_listing: bool,
    pub upload_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servers: vec![ServerConfig::default()],
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: crate::defaults::DEFAULT_HOST.to_string(),
            ports: Vec::new(), // Start with empty ports, they'll be added by config parser
            server_name: None,
            error_pages: HashMap::new(),
            max_body_size: crate::defaults::DEFAULT_MAX_BODY_SIZE,
            routes: vec![RouteConfig::default()],
        }
    }
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            path: "/".to_string(),
            methods: vec!["GET".to_string(), "POST".to_string(), "DELETE".to_string()],
            redirect: None,
            root: Some("www".to_string()),
            index: Some("index.html".to_string()),
            cgi: None,
            directory_listing: false,
            upload_enabled: false,
        }
    }
}
