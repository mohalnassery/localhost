/*!
 * CGI script executor with process forking
 */

use crate::cgi::environment::CgiEnvironment;
use crate::config::{RouteConfig, ServerConfig};
use crate::error::{ServerError, ServerResult, HttpStatus};
use crate::http::{HttpRequest, HttpResponse};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// CGI script executor
pub struct CgiExecutor {
    timeout: Duration,
    max_output_size: usize,
}

impl CgiExecutor {
    /// Create a new CGI executor
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30), // 30 second timeout
            max_output_size: 1024 * 1024,    // 1MB max output
        }
    }

    /// Create CGI executor with custom settings
    pub fn with_settings(timeout_secs: u64, max_output_size: usize) -> Self {
        Self {
            timeout: Duration::from_secs(timeout_secs),
            max_output_size,
        }
    }

    /// Execute a CGI script and return HTTP response
    pub fn execute(
        &self,
        request: &HttpRequest,
        server_config: &ServerConfig,
        route_config: &RouteConfig,
        script_path: &str,
    ) -> ServerResult<HttpResponse> {
        // Validate script exists and is executable
        let script_file = Path::new(script_path);
        if !script_file.exists() {
            return Ok(HttpResponse::error(HttpStatus::NotFound, Some("CGI script not found")));
        }

        // Determine interpreter
        let interpreter = route_config.cgi.as_ref()
            .ok_or_else(|| ServerError::Cgi("No CGI interpreter configured".to_string()))?;

        // Build environment variables
        let path_info = self.extract_path_info(&request.path, &route_config.path);
        let environment = CgiEnvironment::from_request(request, server_config, script_path, &path_info);

        // Execute the script
        self.execute_script(interpreter, script_path, &environment, &request.body)
    }

    /// Execute the CGI script with the given interpreter
    fn execute_script(
        &self,
        interpreter: &str,
        script_path: &str,
        environment: &CgiEnvironment,
        input_data: &[u8],
    ) -> ServerResult<HttpResponse> {
        let start_time = Instant::now();

        // Create command
        let mut command = Command::new(interpreter);
        command
            .arg(script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(environment.to_env_vars());

        // Spawn the process
        let mut child = command.spawn()
            .map_err(|e| ServerError::Cgi(format!("Failed to spawn CGI process: {}", e)))?;

        // Write input data to stdin if present
        if !input_data.is_empty() {
            // Debug: Print first 100 bytes of input data
            let debug_data = if input_data.len() > 100 {
                &input_data[..100]
            } else {
                input_data
            };
            eprintln!("CGI Debug: Writing {} bytes to stdin. First 100 bytes: {:?}",
                     input_data.len(),
                     String::from_utf8_lossy(debug_data));

            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(input_data)
                    .map_err(|e| ServerError::Cgi(format!("Failed to write to CGI stdin: {}", e)))?;
            }
        }

        // Close stdin to signal end of input
        drop(child.stdin.take());

        // Read output with timeout
        let (stdout, _stderr) = self.read_output_with_timeout(&mut child, start_time)?;

        // Wait for process to complete
        let exit_status = child.wait()
            .map_err(|e| ServerError::Cgi(format!("Failed to wait for CGI process: {}", e)))?;

        if !exit_status.success() {
            return Ok(HttpResponse::error(
                HttpStatus::InternalServerError,
                Some("CGI script execution failed"),
            ));
        }

        // Parse CGI output
        self.parse_cgi_output(&stdout)
    }

    /// Read process output with timeout
    fn read_output_with_timeout(
        &self,
        child: &mut std::process::Child,
        start_time: Instant,
    ) -> ServerResult<(Vec<u8>, Vec<u8>)> {
        // Simple timeout implementation - in production, you'd want non-blocking I/O
        loop {
            if start_time.elapsed() > self.timeout {
                let _ = child.kill();
                return Err(ServerError::Cgi("CGI script timeout".to_string()));
            }

            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has finished
                    break;
                }
                Ok(None) => {
                    // Process still running
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    return Err(ServerError::Cgi(format!("Error waiting for CGI process: {}", e)));
                }
            }
        }

        // Read the output
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        if let Some(mut stdout_handle) = child.stdout.take() {
            stdout_handle.read_to_end(&mut stdout)
                .map_err(|e| ServerError::Cgi(format!("Failed to read CGI stdout: {}", e)))?;
        }

        if let Some(mut stderr_handle) = child.stderr.take() {
            stderr_handle.read_to_end(&mut stderr)
                .map_err(|e| ServerError::Cgi(format!("Failed to read CGI stderr: {}", e)))?;
        }

        // Check output size
        if stdout.len() > self.max_output_size {
            return Err(ServerError::Cgi("CGI output too large".to_string()));
        }

        Ok((stdout, stderr))
    }

    /// Parse CGI output into HTTP response
    fn parse_cgi_output(&self, output: &[u8]) -> ServerResult<HttpResponse> {
        let output_str = String::from_utf8_lossy(output);

        // Find the end of headers (double CRLF or double LF)
        let header_end = if let Some(pos) = output_str.find("\r\n\r\n") {
            pos + 4
        } else if let Some(pos) = output_str.find("\n\n") {
            pos + 2
        } else {
            // No headers found, treat entire output as body
            return Ok(HttpResponse::html(HttpStatus::Ok, &output_str));
        };

        let headers_str = &output_str[..header_end - 2]; // Remove the double newline
        let body_str = &output_str[header_end..];

        // Parse headers
        let mut response = HttpResponse::new(HttpStatus::Ok);
        let mut content_type_set = false;

        for line in headers_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim();
                let value = line[colon_pos + 1..].trim();

                match name.to_lowercase().as_str() {
                    "content-type" => {
                        response.set_content_type(value);
                        content_type_set = true;
                    }
                    "status" => {
                        // Parse status line (e.g., "200 OK" or "404 Not Found")
                        if let Some(space_pos) = value.find(' ') {
                            if let Ok(status_code) = value[..space_pos].parse::<u16>() {
                                // Map status code to HttpStatus (simplified)
                                let status = match status_code {
                                    200 => HttpStatus::Ok,
                                    201 => HttpStatus::Created,
                                    204 => HttpStatus::NoContent,
                                    301 => HttpStatus::MovedPermanently,
                                    302 => HttpStatus::Found,
                                    400 => HttpStatus::BadRequest,
                                    403 => HttpStatus::Forbidden,
                                    404 => HttpStatus::NotFound,
                                    405 => HttpStatus::MethodNotAllowed,
                                    413 => HttpStatus::RequestEntityTooLarge,
                                    500 => HttpStatus::InternalServerError,
                                    _ => HttpStatus::Ok, // Default to OK for unknown codes
                                };
                                response = HttpResponse::new(status);
                            }
                        }
                    }
                    "location" => {
                        response.add_header("Location", value);
                    }
                    _ => {
                        response.add_header(name, value);
                    }
                }
            }
        }

        // Set default content type if not specified
        if !content_type_set {
            response.set_content_type("text/html; charset=utf-8");
        }

        // Set body
        response.set_body_string(body_str.to_string());

        Ok(response)
    }

    /// Extract path info from request path and route path
    fn extract_path_info(&self, request_path: &str, route_path: &str) -> String {
        if request_path.starts_with(route_path) {
            let remaining = &request_path[route_path.len()..];
            remaining.strip_prefix('/').unwrap_or(remaining).to_string()
        } else {
            String::new()
        }
    }

    /// Set execution timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Set maximum output size
    pub fn set_max_output_size(&mut self, size: usize) {
        self.max_output_size = size;
    }

    /// Get current timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get maximum output size
    pub fn max_output_size(&self) -> usize {
        self.max_output_size
    }
}

impl Default for CgiExecutor {
    fn default() -> Self {
        Self::new()
    }
}
