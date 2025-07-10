/*!
 * HTTP response generation
 */

use crate::error::HttpStatus;
use crate::http::request::HttpVersion;
use std::collections::HashMap;
use std::fmt::Write;

/// HTTP response structure
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Create a new HTTP response
    pub fn new(status: HttpStatus) -> Self {
        let mut response = Self {
            version: HttpVersion::Http11,
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        };

        // Add default headers
        response.add_header("Server", "localhost-http-server/0.1.0");
        response.add_header("Date", &httpdate::fmt_http_date(std::time::SystemTime::now()));

        response
    }

    /// Add a header
    pub fn add_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    /// Set the response body
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
        self.add_header("Content-Length", &self.body.len().to_string());
    }

    /// Set the response body from string
    pub fn set_body_string(&mut self, body: String) {
        self.set_body(body.into_bytes());
    }

    /// Set content type
    pub fn set_content_type(&mut self, content_type: &str) {
        self.add_header("Content-Type", content_type);
    }

    /// Enable/disable keep-alive
    pub fn set_keep_alive(&mut self, keep_alive: bool) {
        if keep_alive {
            self.add_header("Connection", "keep-alive");
        } else {
            self.add_header("Connection", "close");
        }
    }

    /// Convert response to bytes for transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = String::new();

        // Status line
        write!(response, "{} {} {}\r\n",
               self.version.as_str(),
               self.status.as_u16(),
               self.status.reason_phrase()).unwrap();

        // Headers
        for (name, value) in &self.headers {
            write!(response, "{}: {}\r\n", name, value).unwrap();
        }

        // Empty line to separate headers from body
        response.push_str("\r\n");

        // Convert to bytes and append body
        let mut bytes = response.into_bytes();
        bytes.extend_from_slice(&self.body);

        bytes
    }

    /// Create a simple text response
    pub fn text(status: HttpStatus, text: &str) -> Self {
        let mut response = Self::new(status);
        response.set_content_type("text/plain; charset=utf-8");
        response.set_body_string(text.to_string());
        response
    }

    /// Create an HTML response
    pub fn html(status: HttpStatus, html: &str) -> Self {
        let mut response = Self::new(status);
        response.set_content_type("text/html; charset=utf-8");
        response.set_body_string(html.to_string());
        response
    }

    /// Create a JSON response
    pub fn json(status: HttpStatus, json: &str) -> Self {
        let mut response = Self::new(status);
        response.set_content_type("application/json; charset=utf-8");
        response.set_body_string(json.to_string());
        response
    }

    /// Create a file response
    pub fn file(status: HttpStatus, content: Vec<u8>, content_type: &str) -> Self {
        let mut response = Self::new(status);
        response.set_content_type(content_type);
        response.set_body(content);
        response
    }

    /// Create a redirect response
    pub fn redirect(location: &str, permanent: bool) -> Self {
        let status = if permanent {
            HttpStatus::MovedPermanently
        } else {
            HttpStatus::Found
        };

        let mut response = Self::new(status);
        response.add_header("Location", location);
        response.set_body_string(format!(
            "<!DOCTYPE html>\n<html><head><title>Redirect</title></head>\n\
             <body><h1>Redirect</h1><p>This page has moved to <a href=\"{}\">{}</a></p></body></html>",
            location, location
        ));
        response.set_content_type("text/html; charset=utf-8");
        response
    }

    /// Create an error response
    pub fn error(status: HttpStatus, message: Option<&str>) -> Self {
        let default_message = format!("{} {}", status.as_u16(), status.reason_phrase());
        let error_message = message.unwrap_or(&default_message);

        let html = format!(
            "<!DOCTYPE html>\n<html><head><title>{}</title></head>\n\
             <body><h1>{}</h1><p>{}</p></body></html>",
            default_message, default_message, error_message
        );

        Self::html(status, &html)
    }
}

/// Simple HTTP date formatting (basic implementation)
mod httpdate {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn fmt_http_date(time: SystemTime) -> String {
        let duration = time.duration_since(UNIX_EPOCH).unwrap();
        let timestamp = duration.as_secs();

        // This is a simplified implementation
        // In a production server, you'd want proper RFC 2822 formatting
        format!("Thu, 01 Jan 1970 00:00:{:02} GMT", timestamp % 60)
    }
}
