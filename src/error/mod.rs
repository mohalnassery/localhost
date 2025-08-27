/*!
 * Error handling module
 */

pub mod pages;
pub mod types;

pub use pages::*;

use std::fmt;

/// Main error type for the server
#[derive(Debug)]
pub enum ServerError {
    /// Configuration errors
    Config(String),
    /// I/O errors
    Io(std::io::Error),
    /// HTTP parsing errors
    Http(String),
    /// CGI execution errors
    Cgi(String),
    /// Internal server errors
    Internal(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ServerError::Io(err) => write!(f, "I/O error: {}", err),
            ServerError::Http(msg) => write!(f, "HTTP error: {}", msg),
            ServerError::Cgi(msg) => write!(f, "CGI error: {}", msg),
            ServerError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ServerError {}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        ServerError::Io(err)
    }
}

/// Result type alias for server operations
pub type ServerResult<T> = Result<T, ServerError>;

/// HTTP status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    Ok = 200,
    Created = 201,
    NoContent = 204,
    MovedPermanently = 301,
    Found = 302,
    BadRequest = 400,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    RequestEntityTooLarge = 413,
    InternalServerError = 500,
}

impl HttpStatus {
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn reason_phrase(self) -> &'static str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::Created => "Created",
            HttpStatus::NoContent => "No Content",
            HttpStatus::MovedPermanently => "Moved Permanently",
            HttpStatus::Found => "Found",
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::MethodNotAllowed => "Method Not Allowed",
            HttpStatus::RequestEntityTooLarge => "Request Entity Too Large",
            HttpStatus::InternalServerError => "Internal Server Error",
        }
    }
}
