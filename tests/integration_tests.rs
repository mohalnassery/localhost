/*!
 * Integration tests for localhost HTTP server
 */

use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use std::fs;
use std::path::Path;

/// Test configuration
const TEST_HOST: &str = "127.0.0.1";
const TEST_PORT: u16 = 8889;
const TEST_CONFIG: &str = "config/test-listing.conf";

/// Helper function to start the server
fn start_test_server() -> std::process::Child {
    Command::new("./target/release/localhost-server")
        .arg(TEST_CONFIG)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start test server")
}

/// Helper function to wait for server to be ready
fn wait_for_server() {
    thread::sleep(Duration::from_millis(500));
}

/// Helper function to make HTTP request
fn make_request(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://{}:{}{}", TEST_HOST, TEST_PORT, path);
    let output = Command::new("curl")
        .args(&["-s", &url])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Helper function to make HTTP request with headers
fn make_request_with_headers(path: &str, headers: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://{}:{}{}", TEST_HOST, TEST_PORT, path);
    let mut cmd = Command::new("curl");
    cmd.arg("-s");
    
    for header in headers {
        cmd.args(&["-H", header]);
    }
    
    cmd.arg(&url);
    let output = cmd.output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Helper function to make POST request
fn make_post_request(path: &str, data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("http://{}:{}{}", TEST_HOST, TEST_PORT, path);
    let output = Command::new("curl")
        .args(&["-s", "-X", "POST", "-d", data, &url])
        .output()?;
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Helper function to get HTTP status code
fn get_status_code(path: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let url = format!("http://{}:{}{}", TEST_HOST, TEST_PORT, path);
    let output = Command::new("curl")
        .args(&["-s", "-o", "/dev/null", "-w", "%{http_code}", &url])
        .output()?;
    
    let status_str = String::from_utf8_lossy(&output.stdout);
    Ok(status_str.parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_file_serving() {
        let mut server = start_test_server();
        wait_for_server();

        // Test serving index.html
        let response = make_request("/static/").expect("Failed to make request");
        assert!(response.contains("Directory Listing"));
        assert!(response.contains("test.txt"));

        // Test serving specific file
        let response = make_request("/static/test.txt").expect("Failed to make request");
        assert_eq!(response.trim(), "Hello, World!");

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_directory_listing() {
        let mut server = start_test_server();
        wait_for_server();

        let response = make_request("/static/").expect("Failed to make request");
        
        // Check for directory listing elements
        assert!(response.contains("Directory Listing"));
        assert!(response.contains("test.txt"));
        assert!(response.contains("24 B")); // File size
        assert!(response.contains("📄")); // File icon

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_error_pages() {
        let mut server = start_test_server();
        wait_for_server();

        // Test 404 error
        let status = get_status_code("/nonexistent").expect("Failed to get status");
        assert_eq!(status, 404);

        let response = make_request("/nonexistent").expect("Failed to make request");
        assert!(response.contains("404"));
        assert!(response.contains("Not Found"));

        // Test 405 error (method not allowed)
        let url = format!("http://{}:{}/static/", TEST_HOST, TEST_PORT);
        let output = Command::new("curl")
            .args(&["-s", "-X", "PATCH", &url])
            .output()
            .expect("Failed to make PATCH request");
        
        let response = String::from_utf8_lossy(&output.stdout);
        assert!(response.contains("405") || response.contains("Method Not Allowed"));

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_cgi_functionality() {
        let mut server = start_test_server();
        wait_for_server();

        // Test CGI script execution
        let response = make_request("/cgi-bin/hello.py").expect("Failed to make request");
        assert!(response.contains("CGI Test Script"));
        assert!(response.contains("REQUEST_METHOD"));
        assert!(response.contains("GET"));

        // Test CGI with query parameters
        let response = make_request("/cgi-bin/hello.py?name=test&value=123").expect("Failed to make request");
        assert!(response.contains("name=test"));
        assert!(response.contains("value=123"));

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_cgi_post_request() {
        let mut server = start_test_server();
        wait_for_server();

        // Test CGI with POST data
        let response = make_post_request("/cgi-bin/hello.py", "name=John&email=john@example.com")
            .expect("Failed to make POST request");
        
        assert!(response.contains("POST"));
        assert!(response.contains("name"));
        assert!(response.contains("John"));
        assert!(response.contains("email"));

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_session_cookies() {
        let mut server = start_test_server();
        wait_for_server();

        // Test session creation
        let url = format!("http://{}:{}/cgi-bin/session_test.py", TEST_HOST, TEST_PORT);
        let output = Command::new("curl")
            .args(&["-v", &url])
            .output()
            .expect("Failed to make request");
        
        let response = String::from_utf8_lossy(&output.stderr);
        assert!(response.contains("Set-Cookie: SESSIONID="));
        assert!(response.contains("HttpOnly"));
        assert!(response.contains("Max-Age=3600"));

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_mime_types() {
        let mut server = start_test_server();
        wait_for_server();

        // Create test files with different extensions
        fs::write("www/static/test.json", r#"{"test": "data"}"#).expect("Failed to create test.json");
        fs::write("www/static/test.css", "body { color: red; }").expect("Failed to create test.css");

        // Test JSON MIME type
        let url = format!("http://{}:{}/static/test.json", TEST_HOST, TEST_PORT);
        let output = Command::new("curl")
            .args(&["-v", &url])
            .output()
            .expect("Failed to make request");
        
        let response = String::from_utf8_lossy(&output.stderr);
        assert!(response.contains("Content-Type: application/json"));

        // Test CSS MIME type
        let url = format!("http://{}:{}/static/test.css", TEST_HOST, TEST_PORT);
        let output = Command::new("curl")
            .args(&["-v", &url])
            .output()
            .expect("Failed to make request");
        
        let response = String::from_utf8_lossy(&output.stderr);
        assert!(response.contains("Content-Type: text/css"));

        // Cleanup
        let _ = fs::remove_file("www/static/test.json");
        let _ = fs::remove_file("www/static/test.css");

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_concurrent_requests() {
        let mut server = start_test_server();
        wait_for_server();

        let handles: Vec<_> = (0..10).map(|i| {
            thread::spawn(move || {
                let path = format!("/cgi-bin/hello.py?request={}", i);
                make_request(&path)
            })
        }).collect();

        let mut success_count = 0;
        for handle in handles {
            if let Ok(Ok(response)) = handle.join() {
                if response.contains("CGI Test Script") {
                    success_count += 1;
                }
            }
        }

        assert!(success_count >= 8, "At least 8 out of 10 concurrent requests should succeed");

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_large_request_body() {
        let mut server = start_test_server();
        wait_for_server();

        // Test with large POST data (should be rejected due to size limit)
        let large_data = "a".repeat(2 * 1024 * 1024); // 2MB
        let url = format!("http://{}:{}/cgi-bin/hello.py", TEST_HOST, TEST_PORT);
        let output = Command::new("curl")
            .args(&["-s", "-o", "/dev/null", "-w", "%{http_code}", "-X", "POST", "-d", &large_data, &url])
            .output()
            .expect("Failed to make request");
        
        let status_str = String::from_utf8_lossy(&output.stdout);
        let status: u16 = status_str.parse().expect("Failed to parse status code");
        
        // Should return 413 (Request Entity Too Large)
        assert_eq!(status, 413);

        server.kill().expect("Failed to kill server");
    }

    #[test]
    fn test_http_headers() {
        let mut server = start_test_server();
        wait_for_server();

        let url = format!("http://{}:{}/static/test.txt", TEST_HOST, TEST_PORT);
        let output = Command::new("curl")
            .args(&["-v", &url])
            .output()
            .expect("Failed to make request");
        
        let response = String::from_utf8_lossy(&output.stderr);
        
        // Check for required headers
        assert!(response.contains("Server: localhost-http-server"));
        assert!(response.contains("Content-Type: text/plain"));
        assert!(response.contains("Content-Length:"));
        assert!(response.contains("Date:"));

        server.kill().expect("Failed to kill server");
    }
}
