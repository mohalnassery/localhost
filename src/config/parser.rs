/*!
 * Configuration file parser
 */

use crate::config::types::*;
use crate::error::{ServerError, ServerResult};

/// Parse configuration from string content
pub fn parse_config(content: &str) -> ServerResult<Config> {
    let mut config = Config {
        servers: Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line.starts_with("server") {
            let (server, consumed) = parse_server_block(&lines[i..])?;
            config.servers.push(server);
            i += consumed;
        } else {
            i += 1;
        }
    }

    // If no servers defined, use default
    if config.servers.is_empty() {
        config.servers.push(ServerConfig::default());
    }

    Ok(config)
}

fn parse_server_block(lines: &[&str]) -> ServerResult<(ServerConfig, usize)> {
    let mut server = ServerConfig::default();
    let mut i = 1; // Skip "server {" line

    // Find opening brace
    if !lines[0].contains('{') {
        return Err(ServerError::Config("Expected '{' after server".to_string()));
    }

    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line == "}" {
            i += 1;
            break;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            i += 1;
            continue;
        }

        match parts[0] {
            "host" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("host requires a value".to_string()));
                }
                server.host = parts[1].to_string();
            }
            "port" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("port requires a value".to_string()));
                }
                let port: u16 = parts[1].parse()
                    .map_err(|_| ServerError::Config(format!("Invalid port: {}", parts[1])))?;
                server.ports.push(port);
            }
            "server_name" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("server_name requires a value".to_string()));
                }
                server.server_name = Some(parts[1].to_string());
            }
            "error_page" => {
                if parts.len() < 3 {
                    return Err(ServerError::Config("error_page requires status code and path".to_string()));
                }
                let status: u16 = parts[1].parse()
                    .map_err(|_| ServerError::Config(format!("Invalid status code: {}", parts[1])))?;
                server.error_pages.insert(status, parts[2].to_string());
            }
            "max_body_size" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("max_body_size requires a value".to_string()));
                }
                server.max_body_size = parts[1].parse()
                    .map_err(|_| ServerError::Config(format!("Invalid max_body_size: {}", parts[1])))?;
            }
            "route" => {
                let (route, consumed) = parse_route_block(&lines[i..])?;
                server.routes.push(route);
                i += consumed - 1; // -1 because we'll increment at the end of the loop
            }
            _ => {
                return Err(ServerError::Config(format!("Unknown directive: {}", parts[0])));
            }
        }

        i += 1;
    }

    Ok((server, i))
}

fn parse_route_block(lines: &[&str]) -> ServerResult<(RouteConfig, usize)> {
    let mut route = RouteConfig::default();
    let mut i = 0;

    // Parse "route /path {" line
    let first_line = lines[0].trim();
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(ServerError::Config("route requires a path".to_string()));
    }
    route.path = parts[1].to_string();

    if !first_line.contains('{') {
        return Err(ServerError::Config("Expected '{' after route path".to_string()));
    }

    i += 1;

    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if line == "}" {
            i += 1;
            break;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            i += 1;
            continue;
        }

        match parts[0] {
            "methods" => {
                route.methods = parts[1..].iter().map(|s| s.to_uppercase()).collect();
            }
            "redirect" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("redirect requires a URL".to_string()));
                }
                route.redirect = Some(parts[1].to_string());
            }
            "root" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("root requires a path".to_string()));
                }
                route.root = Some(parts[1].to_string());
            }
            "index" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("index requires a filename".to_string()));
                }
                route.index = Some(parts[1].to_string());
            }
            "cgi" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("cgi requires an interpreter".to_string()));
                }
                route.cgi = Some(parts[1].to_string());
            }
            "directory_listing" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("directory_listing requires on/off".to_string()));
                }
                route.directory_listing = parts[1] == "on";
            }
            "upload_enabled" => {
                if parts.len() < 2 {
                    return Err(ServerError::Config("upload_enabled requires on/off".to_string()));
                }
                route.upload_enabled = parts[1] == "on";
            }
            _ => {
                return Err(ServerError::Config(format!("Unknown route directive: {}", parts[0])));
            }
        }

        i += 1;
    }

    Ok((route, i))
}
