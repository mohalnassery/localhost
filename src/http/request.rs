/*!
 * HTTP request parsing
 */

use crate::error::{ServerError, ServerResult};
use crate::session::CookieJar;
use std::collections::HashMap;
use std::str;

/// HTTP request method
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    DELETE,
    HEAD,
    PUT,
    OPTIONS,
    PATCH,
}

impl HttpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "DELETE" => Some(HttpMethod::DELETE),
            "HEAD" => Some(HttpMethod::HEAD),
            "PUT" => Some(HttpMethod::PUT),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            "PATCH" => Some(HttpMethod::PATCH),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::PUT => "PUT",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::PATCH => "PATCH",
        }
    }
}

/// HTTP version
#[derive(Debug, Clone, PartialEq)]
pub enum HttpVersion {
    Http10,
    Http11,
}

impl HttpVersion {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "HTTP/1.0" => Some(HttpVersion::Http10),
            "HTTP/1.1" => Some(HttpVersion::Http11),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HttpVersion::Http10 => "HTTP/1.0",
            HttpVersion::Http11 => "HTTP/1.1",
        }
    }
}

/// HTTP request structure
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub uri: String,
    pub version: HttpVersion,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub query_params: HashMap<String, String>,
    pub path: String,
    pub cookies: CookieJar,
}

impl HttpRequest {
    pub fn new() -> Self {
        Self {
            method: HttpMethod::GET,
            uri: "/".to_string(),
            version: HttpVersion::Http11,
            headers: HashMap::new(),
            body: Vec::new(),
            query_params: HashMap::new(),
            path: "/".to_string(),
            cookies: CookieJar::new(),
        }
    }

    /// Get header value (case-insensitive)
    pub fn get_header(&self, name: &str) -> Option<&String> {
        let name_lower = name.to_lowercase();
        self.headers.iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v)
    }

    /// Check if connection should be kept alive
    pub fn keep_alive(&self) -> bool {
        match self.version {
            HttpVersion::Http11 => {
                // HTTP/1.1 defaults to keep-alive unless explicitly closed
                self.get_header("connection")
                    .map(|v| v.to_lowercase() != "close")
                    .unwrap_or(true)
            }
            HttpVersion::Http10 => {
                // HTTP/1.0 defaults to close unless explicitly keep-alive
                self.get_header("connection")
                    .map(|v| v.to_lowercase() == "keep-alive")
                    .unwrap_or(false)
            }
        }
    }

    /// Get content length
    pub fn content_length(&self) -> Option<usize> {
        self.get_header("content-length")
            .and_then(|v| v.parse().ok())
    }

    /// Check if request has chunked transfer encoding
    pub fn is_chunked(&self) -> bool {
        self.get_header("transfer-encoding")
            .map(|v| v.to_lowercase().contains("chunked"))
            .unwrap_or(false)
    }
}

/// HTTP request parser state
#[derive(Debug, Clone, PartialEq)]
pub enum ParseState {
    RequestLine,
    Headers,
    Body,
    Complete,
}

/// HTTP request parser
pub struct HttpRequestParser {
    state: ParseState,
    request: HttpRequest,
    body_bytes_remaining: Option<usize>,
    buffer: Vec<u8>,
    headers_end_pos: Option<usize>,
}

impl HttpRequestParser {
    pub fn new() -> Self {
        Self {
            state: ParseState::RequestLine,
            request: HttpRequest::new(),
            body_bytes_remaining: None,
            buffer: Vec::new(),
            headers_end_pos: None,
        }
    }

    /// Parse HTTP request from buffer data
    /// Returns (Option<HttpRequest>, bytes_consumed)
    pub fn parse(&mut self, data: &[u8]) -> ServerResult<(Option<HttpRequest>, usize)> {
        let initial_buffer_len = self.buffer.len();
        // Append new data to buffer
        self.buffer.extend_from_slice(data);

        loop {
            match self.state {
                ParseState::RequestLine => {
                    if let Some(line_end) = self.find_sequence(&self.buffer, b"\r\n") {
                        let line_bytes = &self.buffer[..line_end];
                        let line = str::from_utf8(line_bytes)
                            .map_err(|_| ServerError::Http("Invalid UTF-8 in request line".to_string()))?
                            .to_string();
                        self.buffer.drain(..line_end + 2);
                        self.parse_request_line(&line)?;
                        self.state = ParseState::Headers;
                    } else {
                        break; // Need more data
                    }
                }
                ParseState::Headers => {
                    if let Some(headers_end) = self.find_sequence(&self.buffer, b"\r\n\r\n") {
                        let headers_bytes = &self.buffer[..headers_end];
                        let headers_str = str::from_utf8(headers_bytes)
                            .map_err(|_| ServerError::Http("Invalid UTF-8 in headers".to_string()))?
                            .to_string();
                        self.buffer.drain(..headers_end + 4);
                        self.parse_headers(&headers_str)?;

                        // Determine if we need to read body
                        if let Some(content_length) = self.request.content_length() {
                            if content_length > 0 {
                                self.body_bytes_remaining = Some(content_length);
                                self.state = ParseState::Body;

                                // Debug: Print buffer state after headers
                                eprintln!("HTTP Parser Debug: After headers, buffer has {} bytes, expecting {} body bytes",
                                         self.buffer.len(), content_length);
                                let debug_remaining = if self.buffer.len() > 50 { &self.buffer[..50] } else { &self.buffer };
                                eprintln!("HTTP Parser Debug: Remaining buffer: {:?}",
                                         String::from_utf8_lossy(debug_remaining));
                            } else {
                                self.state = ParseState::Complete;
                            }
                        } else if self.request.is_chunked() {
                            // TODO: Implement chunked encoding
                            return Err(ServerError::Http("Chunked encoding not yet implemented".to_string()));
                        } else {
                            self.state = ParseState::Complete;
                        }
                    } else {
                        break; // Need more data
                    }
                }
                ParseState::Body => {
                    if let Some(remaining) = self.body_bytes_remaining {
                        let available = self.buffer.len();
                        if available >= remaining {
                            // We have all the body data - keep it as binary
                            self.request.body = self.buffer[..remaining].to_vec();

                            // Debug: Print what we're extracting as body
                            let debug_body = if remaining > 100 { &self.buffer[..100] } else { &self.buffer[..remaining] };
                            eprintln!("HTTP Parser Debug: Extracting {} bytes as body. First 100 bytes: {:?}",
                                     remaining,
                                     String::from_utf8_lossy(debug_body));

                            self.buffer.drain(..remaining);
                            self.state = ParseState::Complete;
                        } else {
                            // Need more data
                            break;
                        }
                    } else {
                        self.state = ParseState::Complete;
                    }
                }
                ParseState::Complete => {
                    let consumed = initial_buffer_len + data.len() - self.buffer.len();
                    return Ok((Some(self.request.clone()), consumed));
                }
            }
        }

        let consumed = initial_buffer_len + data.len() - self.buffer.len();
        Ok((None, consumed)) // Need more data
    }

    /// Find a byte sequence in a buffer
    fn find_sequence(&self, buffer: &[u8], pattern: &[u8]) -> Option<usize> {
        buffer.windows(pattern.len()).position(|window| window == pattern)
    }

    /// Parse the HTTP request line
    fn parse_request_line(&mut self, line: &str) -> ServerResult<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(ServerError::Http("Invalid request line".to_string()));
        }

        // Parse method
        self.request.method = HttpMethod::from_str(parts[0])
            .ok_or_else(|| ServerError::Http(format!("Unknown HTTP method: {}", parts[0])))?;

        // Parse URI and extract path and query parameters
        self.request.uri = parts[1].to_string();
        self.parse_uri(parts[1])?;

        // Parse version
        self.request.version = HttpVersion::from_str(parts[2])
            .ok_or_else(|| ServerError::Http(format!("Unsupported HTTP version: {}", parts[2])))?;

        Ok(())
    }

    /// Parse URI to extract path and query parameters
    fn parse_uri(&mut self, uri: &str) -> ServerResult<()> {
        if let Some(query_start) = uri.find('?') {
            self.request.path = uri[..query_start].to_string();
            let query_string = &uri[query_start + 1..];
            self.parse_query_params(query_string)?;
        } else {
            self.request.path = uri.to_string();
        }

        // URL decode the path
        self.request.path = url_decode(&self.request.path)?;

        Ok(())
    }

    /// Parse query parameters
    fn parse_query_params(&mut self, query: &str) -> ServerResult<()> {
        for pair in query.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = url_decode(&pair[..eq_pos])?;
                let value = url_decode(&pair[eq_pos + 1..])?;
                self.request.query_params.insert(key, value);
            } else {
                let key = url_decode(pair)?;
                self.request.query_params.insert(key, String::new());
            }
        }
        Ok(())
    }

    /// Parse HTTP headers
    fn parse_headers(&mut self, headers_str: &str) -> ServerResult<()> {
        for line in headers_str.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();

                // Parse cookies if this is a Cookie header
                if name == "cookie" {
                    self.request.cookies.parse_cookie_header(&value);
                }

                self.request.headers.insert(name, value);
            } else {
                return Err(ServerError::Http(format!("Invalid header line: {}", line)));
            }
        }
        Ok(())
    }

    /// Reset parser for reuse
    pub fn reset(&mut self) {
        self.state = ParseState::RequestLine;
        self.request = HttpRequest::new();
        self.body_bytes_remaining = None;
        self.buffer.clear();
        self.headers_end_pos = None;
    }

    /// Check if parsing is complete
    pub fn is_complete(&self) -> bool {
        self.state == ParseState::Complete
    }
}

/// URL decode a string
fn url_decode(s: &str) -> ServerResult<String> {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        match ch {
            '%' => {
                let hex1 = chars.next()
                    .ok_or_else(|| ServerError::Http("Invalid URL encoding".to_string()))?;
                let hex2 = chars.next()
                    .ok_or_else(|| ServerError::Http("Invalid URL encoding".to_string()))?;

                let hex_str = format!("{}{}", hex1, hex2);
                let byte = u8::from_str_radix(&hex_str, 16)
                    .map_err(|_| ServerError::Http("Invalid URL encoding".to_string()))?;

                result.push(byte as char);
            }
            '+' => result.push(' '),
            _ => result.push(ch),
        }
    }

    Ok(result)
}
