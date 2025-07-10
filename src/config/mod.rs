/*!
 * Configuration module
 * 
 * Handles parsing and validation of server configuration files
 */

pub mod parser;
pub mod types;

pub use types::*;
pub use parser::*;

use crate::error::{ServerError, ServerResult};
use std::path::Path;

impl Config {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> ServerResult<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ServerError::Config(format!("Failed to read config file: {}", e)))?;
        
        parser::parse_config(&content)
    }

    /// Validate the configuration
    pub fn validate(&self) -> ServerResult<()> {
        // Check for duplicate ports
        let mut used_ports = std::collections::HashSet::new();
        
        for server in &self.servers {
            for port in &server.ports {
                let key = (server.host.clone(), *port);
                if used_ports.contains(&key) {
                    return Err(ServerError::Config(format!(
                        "Duplicate port {} for host {}", port, server.host
                    )));
                }
                used_ports.insert(key);
            }
        }

        // Validate routes
        for server in &self.servers {
            for route in &server.routes {
                if route.path.is_empty() {
                    return Err(ServerError::Config("Route path cannot be empty".to_string()));
                }
            }
        }

        Ok(())
    }
}
