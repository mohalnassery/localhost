/*!
 * Unit tests for localhost HTTP server components
 */

#[cfg(test)]
mod config_tests {
    use localhost_http_server::config::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_config_parsing() {
        let config_content = r#"
server {
    server_name localhost
    listen 8080
    max_body_size 1048576
    
    route / {
        methods GET POST
        root www
        index index.html
        directory_listing on
    }
    
    route /api {
        methods GET POST PUT DELETE
        root api
        directory_listing off
    }
}
"#;
        
        // Write test config to temporary file
        let test_config_path = "test_config.conf";
        fs::write(test_config_path, config_content).expect("Failed to write test config");
        
        // Parse config
        let config = Config::from_file(test_config_path).expect("Failed to parse config");
        
        // Verify config
        assert_eq!(config.servers.len(), 1);
        
        let server = &config.servers[0];
        assert_eq!(server.server_name, Some("localhost".to_string()));
        assert_eq!(server.ports, vec![8080]);
        assert_eq!(server.max_body_size, 1048576);
        assert_eq!(server.routes.len(), 2);
        
        // Verify first route
        let route1 = &server.routes[0];
        assert_eq!(route1.path, "/");
        assert_eq!(route1.methods, vec!["GET", "POST"]);
        assert_eq!(route1.root, Some("www".to_string()));
        assert_eq!(route1.index, Some("index.html".to_string()));
        assert_eq!(route1.directory_listing, true);
        
        // Verify second route
        let route2 = &server.routes[1];
        assert_eq!(route2.path, "/api");
        assert_eq!(route2.methods, vec!["GET", "POST", "PUT", "DELETE"]);
        assert_eq!(route2.root, Some("api".to_string()));
        assert_eq!(route2.directory_listing, false);
        
        // Cleanup
        fs::remove_file(test_config_path).ok();
    }

    #[test]
    fn test_invalid_config() {
        let invalid_config = r#"
invalid syntax here
server {
    missing closing brace
"#;
        
        let test_config_path = "invalid_config.conf";
        fs::write(test_config_path, invalid_config).expect("Failed to write test config");
        
        // Should fail to parse
        assert!(Config::from_file(test_config_path).is_err());
        
        // Cleanup
        fs::remove_file(test_config_path).ok();
    }
}

#[cfg(test)]
mod http_tests {
    use localhost_http_server::http::*;

    #[test]
    fn test_http_request_parsing() {
        let request_data = b"GET /test?param=value HTTP/1.1\r\nHost: localhost\r\nUser-Agent: test\r\n\r\n";
        
        let mut parser = HttpRequestParser::new();
        let request = parser.parse(request_data).expect("Failed to parse request").expect("Request not complete");
        
        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.path, "/test");
        assert_eq!(request.uri, "/test?param=value");
        assert_eq!(request.version, HttpVersion::Http11);
        assert_eq!(request.get_header("host"), Some(&"localhost".to_string()));
        assert_eq!(request.get_header("user-agent"), Some(&"test".to_string()));
        assert_eq!(request.query_params.get("param"), Some(&"value".to_string()));
    }

    #[test]
    fn test_http_response_generation() {
        let mut response = HttpResponse::new(HttpStatus::Ok);
        response.set_content_type("text/html");
        response.set_body_string("Hello, World!".to_string());
        
        let response_bytes = response.to_bytes();
        let response_str = String::from_utf8_lossy(&response_bytes);
        
        assert!(response_str.contains("HTTP/1.1 200 OK"));
        assert!(response_str.contains("Content-Type: text/html"));
        assert!(response_str.contains("Content-Length: 13"));
        assert!(response_str.contains("Hello, World!"));
    }

    #[test]
    fn test_http_status_codes() {
        assert_eq!(HttpStatus::Ok.as_u16(), 200);
        assert_eq!(HttpStatus::NotFound.as_u16(), 404);
        assert_eq!(HttpStatus::InternalServerError.as_u16(), 500);
        
        assert_eq!(HttpStatus::Ok.reason_phrase(), "OK");
        assert_eq!(HttpStatus::NotFound.reason_phrase(), "Not Found");
        assert_eq!(HttpStatus::InternalServerError.reason_phrase(), "Internal Server Error");
    }

    #[test]
    fn test_http_methods() {
        assert_eq!(HttpMethod::GET.as_str(), "GET");
        assert_eq!(HttpMethod::POST.as_str(), "POST");
        assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
        
        assert_eq!(HttpMethod::from_str("GET"), Ok(HttpMethod::GET));
        assert_eq!(HttpMethod::from_str("POST"), Ok(HttpMethod::POST));
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    #[test]
    fn test_malformed_request() {
        let malformed_data = b"INVALID REQUEST FORMAT\r\n\r\n";
        
        let mut parser = HttpRequestParser::new();
        assert!(parser.parse(malformed_data).is_err());
    }

    #[test]
    fn test_post_request_with_body() {
        let request_data = b"POST /submit HTTP/1.1\r\nHost: localhost\r\nContent-Length: 13\r\n\r\nHello, World!";
        
        let mut parser = HttpRequestParser::new();
        let request = parser.parse(request_data).expect("Failed to parse request").expect("Request not complete");
        
        assert_eq!(request.method, HttpMethod::POST);
        assert_eq!(request.path, "/submit");
        assert_eq!(request.body, b"Hello, World!");
    }
}

#[cfg(test)]
mod session_tests {
    use localhost_http_server::session::*;
    use std::time::Duration;

    #[test]
    fn test_cookie_creation() {
        let cookie = Cookie::new("test".to_string(), "value".to_string())
            .path("/".to_string())
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax);
        
        let header_value = cookie.to_header_value();
        assert!(header_value.contains("test=value"));
        assert!(header_value.contains("Path=/"));
        assert!(header_value.contains("HttpOnly"));
        assert!(header_value.contains("Secure"));
        assert!(header_value.contains("SameSite=Lax"));
    }

    #[test]
    fn test_cookie_jar() {
        let mut jar = CookieJar::new();
        
        // Test adding cookies
        jar.add(Cookie::new("session".to_string(), "abc123".to_string()));
        jar.add(Cookie::new("user".to_string(), "john".to_string()));
        
        assert_eq!(jar.len(), 2);
        assert_eq!(jar.get("session").unwrap().value, "abc123");
        assert_eq!(jar.get("user").unwrap().value, "john");
        
        // Test parsing cookie header
        jar.parse_cookie_header("theme=dark; lang=en");
        assert_eq!(jar.len(), 4);
        assert_eq!(jar.get("theme").unwrap().value, "dark");
        assert_eq!(jar.get("lang").unwrap().value, "en");
    }

    #[test]
    fn test_session_manager() {
        let manager = SessionManager::with_defaults();
        
        // Create session
        let session_id = manager.create_session().expect("Failed to create session");
        assert!(!session_id.is_empty());
        
        // Get session
        let session = manager.get_session(&session_id).expect("Failed to get session");
        assert!(session.is_some());
        
        let session = session.unwrap();
        assert_eq!(session.id, session_id);
        assert!(session.data.is_empty());
        assert!(!session.is_expired());
        
        // Destroy session
        let destroyed = manager.destroy_session(&session_id).expect("Failed to destroy session");
        assert!(destroyed);
        
        // Session should no longer exist
        let session = manager.get_session(&session_id).expect("Failed to get session");
        assert!(session.is_none());
    }

    #[test]
    fn test_session_data() {
        let mut session = Session::new("test_id".to_string());
        
        // Test setting and getting data
        session.set("key1".to_string(), "value1".to_string());
        session.set("key2".to_string(), "value2".to_string());
        
        assert_eq!(session.get("key1"), Some(&"value1".to_string()));
        assert_eq!(session.get("key2"), Some(&"value2".to_string()));
        assert_eq!(session.get("nonexistent"), None);
        
        // Test removing data
        let removed = session.remove("key1");
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(session.get("key1"), None);
        
        // Test clearing data
        session.clear();
        assert!(session.data.is_empty());
    }

    #[test]
    fn test_session_expiration() {
        let session = Session::with_expiration("test".to_string(), Duration::from_millis(1));
        
        // Should not be expired immediately
        assert!(!session.is_expired());
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(10));
        assert!(session.is_expired());
    }
}

#[cfg(test)]
mod cgi_tests {
    use localhost_http_server::cgi::*;
    use localhost_http_server::config::*;
    use localhost_http_server::http::*;

    #[test]
    fn test_cgi_environment() {
        let mut request = HttpRequest::new();
        request.method = HttpMethod::GET;
        request.uri = "/cgi-bin/test.py?param=value".to_string();
        request.path = "/cgi-bin/test.py".to_string();
        request.add_header("user-agent", "test-agent");
        request.add_header("host", "localhost:8080");
        
        let server_config = ServerConfig::default();
        let env = CgiEnvironment::from_request(&request, &server_config, "/cgi-bin/test.py", "");
        
        // Check required CGI variables
        assert_eq!(env.get("REQUEST_METHOD"), Some(&"GET".to_string()));
        assert_eq!(env.get("SCRIPT_NAME"), Some(&"/cgi-bin/test.py".to_string()));
        assert_eq!(env.get("QUERY_STRING"), Some(&"param=value".to_string()));
        assert_eq!(env.get("SERVER_SOFTWARE"), Some(&"localhost-http-server/0.1.0".to_string()));
        assert_eq!(env.get("GATEWAY_INTERFACE"), Some(&"CGI/1.1".to_string()));
        
        // Check HTTP headers
        assert_eq!(env.get("HTTP_USER_AGENT"), Some(&"test-agent".to_string()));
        assert_eq!(env.get("HTTP_HOST"), Some(&"localhost:8080".to_string()));
        
        // Validate environment
        assert!(env.validate().is_ok());
    }

    #[test]
    fn test_cgi_environment_post() {
        let mut request = HttpRequest::new();
        request.method = HttpMethod::POST;
        request.uri = "/cgi-bin/form.py".to_string();
        request.path = "/cgi-bin/form.py".to_string();
        request.body = b"name=John&email=john@example.com".to_vec();
        request.add_header("content-type", "application/x-www-form-urlencoded");
        request.add_header("content-length", "30");
        
        let server_config = ServerConfig::default();
        let env = CgiEnvironment::from_request(&request, &server_config, "/cgi-bin/form.py", "");
        
        assert_eq!(env.get("REQUEST_METHOD"), Some(&"POST".to_string()));
        assert_eq!(env.get("CONTENT_TYPE"), Some(&"application/x-www-form-urlencoded".to_string()));
        assert_eq!(env.get("CONTENT_LENGTH"), Some(&"30".to_string()));
    }

    #[test]
    fn test_cgi_environment_validation() {
        let mut env = CgiEnvironment::new();
        
        // Should fail validation without required variables
        assert!(env.validate().is_err());
        
        // Add required variables
        env.set("GATEWAY_INTERFACE", "CGI/1.1");
        env.set("SERVER_SOFTWARE", "test");
        env.set("SERVER_PROTOCOL", "HTTP/1.1");
        env.set("REQUEST_METHOD", "GET");
        env.set("SCRIPT_NAME", "/test.py");
        
        // Should pass validation
        assert!(env.validate().is_ok());
    }
}

#[cfg(test)]
mod timeout_tests {
    use localhost_http_server::utils::*;
    use std::time::Duration;

    #[test]
    fn test_timeout_manager() {
        let mut manager = TimeoutManager::with_defaults();
        
        // Add connection
        assert!(manager.add_connection(1).is_ok());
        assert_eq!(manager.connection_count(), 1);
        
        // Update activity
        manager.update_activity(1, 100, true);
        let info = manager.get_connection(1).unwrap();
        assert_eq!(info.bytes_read, 100);
        assert_eq!(info.bytes_written, 0);
        
        // Update state
        manager.update_state(1, ConnectionState::Processing);
        let info = manager.get_connection(1).unwrap();
        assert_eq!(info.state, ConnectionState::Processing);
        
        // Remove connection
        assert!(manager.remove_connection(1).is_some());
        assert_eq!(manager.connection_count(), 0);
    }

    #[test]
    fn test_resource_monitor() {
        let mut monitor = ResourceMonitor::new();
        
        // Record some activity
        monitor.record_request(1024);
        monitor.record_request(2048);
        monitor.record_error();
        monitor.update_peak_connections(5);
        
        let stats = monitor.get_stats();
        assert_eq!(stats.total_requests_served, 2);
        assert_eq!(stats.total_bytes_transferred, 3072);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.peak_connections, 5);
        
        // Test calculated metrics
        assert_eq!(stats.error_rate_percent(), 50.0); // 1 error out of 2 requests
        assert!(stats.requests_per_second() > 0.0);
        assert!(stats.bytes_per_second() > 0.0);
    }

    #[test]
    fn test_timeout_stats() {
        let manager = TimeoutManager::with_defaults();
        let stats = manager.get_stats();
        
        assert_eq!(stats.total_connections, 0);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.utilization_percent(), 0.0);
        assert_eq!(stats.avg_requests_per_connection(), 0.0);
    }
}
