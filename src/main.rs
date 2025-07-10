/*!
 * Localhost HTTP Server
 * 
 * A production-ready HTTP/1.1 server implementation in Rust
 * Features:
 * - Single-threaded epoll-based I/O multiplexing
 * - HTTP/1.1 protocol support with GET, POST, DELETE methods
 * - Static file serving and CGI support
 * - Session and cookie management
 * - Multi-server and multi-port configuration
 * - Comprehensive error handling and custom error pages
 */

use std::env;
use std::process;

use localhost_http_server::config::Config;
use localhost_http_server::server::Server;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "config/server.conf"
    };

    // Load configuration
    let config = match Config::from_file(config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Error loading configuration from '{}': {}", config_path, e);
            process::exit(1);
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        process::exit(1);
    }

    println!("Starting localhost HTTP server...");
    println!("Configuration loaded from: {}", config_path);
    
    // Create and start the server
    let mut server = match Server::new(config) {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Failed to create server: {}", e);
            process::exit(1);
        }
    };

    // Run the server (this will block until shutdown)
    if let Err(e) = server.run() {
        eprintln!("Server error: {}", e);
        process::exit(1);
    }

    println!("Server shutdown complete.");
}
