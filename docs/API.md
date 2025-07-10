# API Documentation

This document provides comprehensive API documentation for the Localhost HTTP Server.

## Core Components

### Server

The main server struct that handles HTTP requests and manages connections.

```rust
pub struct Server {
    listener: TcpListener,
    epoll: Epoll,
    connection_manager: ConnectionManager,
    config: ServerConfig,
    router: Router,
    cgi_executor: CgiExecutor,
    session_manager: SessionManager,
    error_manager: ErrorManager,
}
```

#### Methods

##### `new(config: ServerConfig) -> Result<Self, ServerError>`

Creates a new server instance with the given configuration.

**Parameters:**
- `config`: Server configuration containing routes, error pages, and server settings

**Returns:**
- `Ok(Server)`: Successfully created server
- `Err(ServerError)`: Configuration or initialization error

##### `run(&mut self) -> Result<(), ServerError>`

Starts the server and begins processing requests. This method blocks until the server is shut down.

**Returns:**
- `Ok(())`: Server shut down gracefully
- `Err(ServerError)`: Server encountered a fatal error

### Configuration

#### ServerConfig

Main configuration structure for the server.

```rust
pub struct ServerConfig {
    pub server_name: Option<String>,
    pub ports: Vec<u16>,
    pub max_body_size: usize,
    pub routes: Vec<RouteConfig>,
    pub error_pages: HashMap<u16, String>,
}
```

#### RouteConfig

Configuration for individual routes.

```rust
pub struct RouteConfig {
    pub path: String,
    pub methods: Vec<String>,
    pub root: Option<String>,
    pub index: Option<String>,
    pub directory_listing: bool,
    pub cgi: Option<String>,
    pub upload_enabled: bool,
    pub redirect: Option<String>,
}
```

### HTTP Components

#### HttpRequest

Represents an HTTP request.

```rust
pub struct HttpRequest {
    pub method: HttpMethod,
    pub uri: String,
    pub path: String,
    pub version: HttpVersion,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub query_params: HashMap<String, String>,
}
```

##### Methods

- `get_header(&self, name: &str) -> Option<&String>`: Get header value
- `add_header(&mut self, name: &str, value: &str)`: Add header
- `keep_alive(&self) -> bool`: Check if connection should be kept alive
- `content_length(&self) -> Option<usize>`: Get content length

#### HttpResponse

Represents an HTTP response.

```rust
pub struct HttpResponse {
    pub status: HttpStatus,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}
```

##### Methods

- `new(status: HttpStatus) -> Self`: Create new response
- `set_content_type(&mut self, content_type: &str)`: Set content type
- `set_body_string(&mut self, body: String)`: Set body from string
- `set_body_bytes(&mut self, body: Vec<u8>)`: Set body from bytes
- `add_header(&mut self, name: &str, value: &str)`: Add header
- `to_bytes(&self) -> Vec<u8>`: Serialize to bytes

#### HttpStatus

HTTP status codes enumeration.

```rust
pub enum HttpStatus {
    Ok,                    // 200
    Created,               // 201
    NoContent,             // 204
    MovedPermanently,      // 301
    Found,                 // 302
    NotModified,           // 304
    BadRequest,            // 400
    Unauthorized,          // 401
    Forbidden,             // 403
    NotFound,              // 404
    MethodNotAllowed,      // 405
    RequestEntityTooLarge, // 413
    InternalServerError,   // 500
    NotImplemented,        // 501
    ServiceUnavailable,    // 503
}
```

### Session Management

#### SessionManager

Manages HTTP sessions and cookies.

```rust
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    config: SessionConfig,
}
```

##### Methods

- `create_session(&self) -> Result<String, SessionError>`: Create new session
- `get_session(&self, id: &str) -> Result<Option<Session>, SessionError>`: Get session
- `destroy_session(&self, id: &str) -> Result<bool, SessionError>`: Destroy session
- `cleanup_expired(&self) -> Result<usize, SessionError>`: Remove expired sessions

#### Session

Individual session data.

```rust
pub struct Session {
    pub id: String,
    pub data: HashMap<String, String>,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
}
```

##### Methods

- `new(id: String) -> Self`: Create new session
- `set(&mut self, key: String, value: String)`: Set session data
- `get(&self, key: &str) -> Option<&String>`: Get session data
- `remove(&mut self, key: &str) -> Option<String>`: Remove session data
- `is_expired(&self) -> bool`: Check if session is expired

#### Cookie

HTTP cookie representation.

```rust
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub expires: Option<SystemTime>,
    pub max_age: Option<Duration>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>,
}
```

### CGI Support

#### CgiExecutor

Executes CGI scripts with proper environment setup.

```rust
pub struct CgiExecutor {
    timeout: Duration,
}
```

##### Methods

- `execute(&self, script_path: &str, request: &HttpRequest, config: &ServerConfig) -> Result<HttpResponse, CgiError>`: Execute CGI script

#### CgiEnvironment

CGI environment variables.

```rust
pub struct CgiEnvironment {
    variables: HashMap<String, String>,
}
```

##### Methods

- `from_request(request: &HttpRequest, config: &ServerConfig, script_name: &str, path_info: &str) -> Self`: Create from request
- `set(&mut self, key: &str, value: &str)`: Set environment variable
- `get(&self, key: &str) -> Option<&String>`: Get environment variable
- `validate(&self) -> Result<(), CgiError>`: Validate required variables

### Error Handling

#### ErrorManager

Manages error responses and custom error pages.

```rust
pub struct ErrorManager {
    error_pages: HashMap<u16, String>,
}
```

##### Methods

- `generate_error_response(&self, status: HttpStatus, message: Option<&str>) -> HttpResponse`: Generate error response

### Timeout and Resource Management

#### TimeoutManager

Manages connection timeouts and resource cleanup.

```rust
pub struct TimeoutManager {
    connections: HashMap<RawFd, ConnectionInfo>,
    timeout_duration: Duration,
    max_connections: usize,
}
```

##### Methods

- `add_connection(&mut self, fd: RawFd) -> Result<(), TimeoutError>`: Add connection
- `remove_connection(&mut self, fd: RawFd) -> Option<ConnectionInfo>`: Remove connection
- `update_activity(&mut self, fd: RawFd, bytes: usize, is_read: bool)`: Update activity
- `cleanup_expired(&mut self) -> Vec<RawFd>`: Get expired connections

#### ResourceMonitor

Monitors server resource usage and performance.

```rust
pub struct ResourceMonitor {
    start_time: Instant,
    total_requests_served: u64,
    total_bytes_transferred: u64,
    error_count: u64,
    peak_connections: usize,
}
```

##### Methods

- `record_request(&mut self, bytes: usize)`: Record completed request
- `record_error(&mut self)`: Record error
- `get_stats(&self) -> ResourceStats`: Get performance statistics

## Error Types

### ServerError

Main server error type.

```rust
pub enum ServerError {
    IoError(std::io::Error),
    ConfigError(String),
    BindError(String),
    EpollError(String),
}
```

### HttpError

HTTP-related errors.

```rust
pub enum HttpError {
    ParseError(String),
    InvalidMethod(String),
    InvalidVersion(String),
    HeaderError(String),
    BodyTooLarge,
}
```

### CgiError

CGI execution errors.

```rust
pub enum CgiError {
    ExecutionError(String),
    TimeoutError,
    EnvironmentError(String),
    OutputParseError(String),
}
```

### SessionError

Session management errors.

```rust
pub enum SessionError {
    CreationError(String),
    NotFound,
    Expired,
    StorageError(String),
}
```

## Usage Examples

### Basic Server Setup

```rust
use localhost_http_server::*;

// Load configuration
let config = Config::from_file("server.conf")?;

// Create and start server
let mut server = Server::new(config.servers[0].clone())?;
server.run()?;
```

### Custom Route Handler

```rust
// Routes are configured via configuration file
// See configuration documentation for details
```

### Session Usage in CGI

```python
#!/usr/bin/env python3
import os

# Access session via environment variables
session_id = os.environ.get('HTTP_COOKIE', '').split('SESSIONID=')[1].split(';')[0]
print(f"Content-Type: text/html\n")
print(f"<h1>Session: {session_id}</h1>")
```

## Performance Considerations

- Use `cargo build --release` for production builds
- Configure appropriate `max_body_size` limits
- Monitor connection counts and resource usage
- Use keep-alive connections for better performance
- Consider reverse proxy for HTTPS termination

## Thread Safety

The server is designed as a single-threaded application using epoll for I/O multiplexing. Session management uses `Arc<Mutex<>>` for thread safety in case of future multi-threading support.
