/*!
 * URL routing implementation
 */

use crate::config::{Config, RouteConfig, ServerConfig};
use crate::error::{ServerError, ServerResult};

/// Router for matching URLs to route configurations
pub struct Router {
    servers: Vec<ServerConfig>,
}

impl Router {
    /// Create a new router with server configurations
    pub fn new(config: &Config) -> Self {
        Self {
            servers: config.servers.clone(),
        }
    }

    /// Find the best matching route for a request
    pub fn find_route(&self, host: Option<&str>, path: &str) -> ServerResult<(&ServerConfig, &RouteConfig)> {
        // Find the appropriate server based on host header
        let server = self.find_server(host)?;

        // Find the best matching route within that server
        let route = self.find_best_route(server, path)?;

        Ok((server, route))
    }

    /// Find the appropriate server based on host header
    fn find_server(&self, host: Option<&str>) -> ServerResult<&ServerConfig> {
        if let Some(host_header) = host {
            // Extract hostname from host header (remove port if present)
            let hostname = host_header.split(':').next().unwrap_or(host_header);

            // Look for a server with matching server_name
            for server in &self.servers {
                if let Some(server_name) = &server.server_name {
                    if server_name == hostname {
                        return Ok(server);
                    }
                }
            }
        }

        // Fall back to the first server (default server)
        self.servers.first()
            .ok_or_else(|| ServerError::Config("No servers configured".to_string()))
    }

    /// Find the best matching route within a server
    fn find_best_route<'a>(&self, server: &'a ServerConfig, path: &str) -> ServerResult<&'a RouteConfig> {
        let mut best_match: Option<&RouteConfig> = None;
        let mut best_match_len = 0;

        // Find the longest matching route prefix
        for route in &server.routes {
            if self.path_matches_route(path, &route.path) {
                let match_len = route.path.len();
                if match_len > best_match_len {
                    best_match = Some(route);
                    best_match_len = match_len;
                }
            }
        }

        best_match.ok_or_else(|| ServerError::Http("No matching route found".to_string()))
    }

    /// Check if a path matches a route pattern
    fn path_matches_route(&self, path: &str, route_path: &str) -> bool {
        // Exact match
        if path == route_path {
            return true;
        }

        // Prefix match (path starts with route_path)
        if route_path.ends_with('/') {
            return path.starts_with(route_path);
        }

        // Prefix match with implicit trailing slash
        if path.starts_with(route_path) {
            let remaining = &path[route_path.len()..];
            return remaining.is_empty() || remaining.starts_with('/');
        }

        false
    }

    /// Get all servers
    pub fn servers(&self) -> &[ServerConfig] {
        &self.servers
    }

    /// Validate router configuration
    pub fn validate(&self) -> ServerResult<()> {
        if self.servers.is_empty() {
            return Err(ServerError::Config("No servers configured".to_string()));
        }

        for server in &self.servers {
            if server.routes.is_empty() {
                return Err(ServerError::Config("Server has no routes configured".to_string()));
            }

            for route in &server.routes {
                if route.path.is_empty() {
                    return Err(ServerError::Config("Route path cannot be empty".to_string()));
                }

                if !route.path.starts_with('/') {
                    return Err(ServerError::Config(format!(
                        "Route path must start with '/': {}", route.path
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, ServerConfig, RouteConfig};

    fn create_test_config() -> Config {
        Config {
            servers: vec![
                ServerConfig {
                    host: "127.0.0.1".to_string(),
                    ports: vec![8080],
                    server_name: Some("localhost".to_string()),
                    error_pages: std::collections::HashMap::new(),
                    max_body_size: 1024 * 1024,
                    routes: vec![
                        RouteConfig {
                            path: "/".to_string(),
                            methods: vec!["GET".to_string()],
                            redirect: None,
                            root: Some("www".to_string()),
                            index: Some("index.html".to_string()),
                            cgi: None,
                            directory_listing: false,
                            upload_enabled: false,
                        },
                        RouteConfig {
                            path: "/api/".to_string(),
                            methods: vec!["GET".to_string(), "POST".to_string()],
                            redirect: None,
                            root: None,
                            index: None,
                            cgi: Some("python3".to_string()),
                            directory_listing: false,
                            upload_enabled: false,
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_route_matching() {
        let config = create_test_config();
        let router = Router::new(&config);

        // Test exact match
        let (_, route) = router.find_route(Some("localhost"), "/").unwrap();
        assert_eq!(route.path, "/");

        // Test prefix match
        let (_, route) = router.find_route(Some("localhost"), "/api/test").unwrap();
        assert_eq!(route.path, "/api/");

        // Test longest match
        let (_, route) = router.find_route(Some("localhost"), "/index.html").unwrap();
        assert_eq!(route.path, "/");
    }
}
