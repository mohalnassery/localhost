/*!
 * Error page generation and management
 */

use crate::config::ServerConfig;
use crate::error::HttpStatus;
use crate::http::HttpResponse;
use std::collections::HashMap;

/// Error page manager
pub struct ErrorPageManager {
    custom_pages: HashMap<u16, String>,
}

impl ErrorPageManager {
    /// Create a new error page manager
    pub fn new() -> Self {
        Self {
            custom_pages: HashMap::new(),
        }
    }

    /// Create error page manager from server configuration
    pub fn from_config(server_config: &ServerConfig) -> Self {
        Self {
            custom_pages: server_config.error_pages.clone(),
        }
    }

    /// Generate an HTTP error response
    pub fn generate_error_response(&self, status: HttpStatus, custom_message: Option<&str>) -> HttpResponse {
        let status_code = status.as_u16();
        let error_content = self.generate_error_page(status, custom_message);

        let mut response = HttpResponse::html(status, &error_content);

        // Add security headers for error pages
        response.add_header("X-Content-Type-Options", "nosniff");
        response.add_header("X-Frame-Options", "DENY");
        response.add_header("X-XSS-Protection", "1; mode=block");

        response
    }

    /// Generate HTML error page content
    pub fn generate_error_page(&self, status: HttpStatus, custom_message: Option<&str>) -> String {
        let status_code = status.as_u16();

        // Try to load custom error page
        if let Some(custom_path) = self.custom_pages.get(&status_code) {
            if let Ok(content) = std::fs::read_to_string(custom_path) {
                return content;
            }
        }

        // Generate default error page
        self.generate_default_error_page(status, custom_message)
    }

    /// Generate default error page when no custom page is available
    fn generate_default_error_page(&self, status: HttpStatus, custom_message: Option<&str>) -> String {
        let status_code = status.as_u16();
        let reason = status.reason_phrase();
        let message = custom_message.unwrap_or("The server encountered an error processing your request.");

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - {}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            text-align: center;
            padding: 50px;
            background-color: #f5f5f5;
            margin: 0;
        }}
        .error-container {{
            background: white;
            padding: 40px;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            max-width: 500px;
            margin: 0 auto;
        }}
        h1 {{
            color: #dc3545;
            font-size: 4em;
            margin: 0;
        }}
        h2 {{
            color: #333;
            margin: 20px 0;
        }}
        p {{
            color: #666;
            line-height: 1.6;
        }}
        a {{
            color: #007bff;
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        .server-info {{
            margin-top: 30px;
            padding-top: 20px;
            border-top: 1px solid #eee;
            font-size: 0.9em;
            color: #999;
        }}
    </style>
</head>
<body>
    <div class="error-container">
        <h1>{}</h1>
        <h2>{}</h2>
        <p>{}</p>
        <p><a href="/">← Return to Home</a></p>
        <div class="server-info">
            Localhost HTTP Server v0.1.0
        </div>
    </div>
</body>
</html>"#,
            status_code, reason, status_code, reason, message
        )
    }

    /// Add or update a custom error page
    pub fn set_custom_page(&mut self, status_code: u16, file_path: String) {
        self.custom_pages.insert(status_code, file_path);
    }

    /// Remove a custom error page
    pub fn remove_custom_page(&mut self, status_code: u16) {
        self.custom_pages.remove(&status_code);
    }

    /// Check if a custom error page exists for a status code
    pub fn has_custom_page(&self, status_code: u16) -> bool {
        self.custom_pages.contains_key(&status_code)
    }

    /// Get all configured custom error pages
    pub fn get_custom_pages(&self) -> &HashMap<u16, String> {
        &self.custom_pages
    }

    /// Validate that all custom error page files exist
    pub fn validate_custom_pages(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (status_code, file_path) in &self.custom_pages {
            if !std::path::Path::new(file_path).exists() {
                errors.push(format!("Error page file not found for status {}: {}", status_code, file_path));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for ErrorPageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy function for backward compatibility
pub fn generate_error_page(status: HttpStatus, custom_path: Option<&str>) -> String {
    let manager = ErrorPageManager::new();
    if let Some(path) = custom_path {
        if let Ok(content) = std::fs::read_to_string(path) {
            return content;
        }
    }
    manager.generate_default_error_page(status, None)
}
