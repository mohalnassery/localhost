/*!
 * Localhost HTTP Server Library
 * 
 * Core library modules for the HTTP server implementation
 */

pub mod config;
pub mod server;
pub mod http;
pub mod routing;
pub mod cgi;
pub mod session;
pub mod error;
pub mod utils;

// Re-export commonly used types
pub use config::Config;
pub use server::Server;
pub use error::{ServerError, ServerResult};

/// Server version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Default configuration values
pub mod defaults {
    pub const DEFAULT_PORT: u16 = 8080;
    pub const DEFAULT_HOST: &str = "127.0.0.1";
    pub const DEFAULT_TIMEOUT: u64 = 30; // seconds
    pub const DEFAULT_MAX_BODY_SIZE: usize = 1024 * 1024; // 1MB
    pub const DEFAULT_BUFFER_SIZE: usize = 8192; // 8KB
    pub const MAX_CONNECTIONS: usize = 1024;
    pub const MAX_EVENTS: usize = 1024;
}
