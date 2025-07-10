# Localhost HTTP Server

A production-ready HTTP/1.1 server implementation in Rust with epoll-based I/O multiplexing.

## Features

- **HTTP/1.1 Protocol Support**: Full compliance with HTTP/1.1 specification
- **High Performance**: Single-threaded epoll-based I/O multiplexing for maximum efficiency
- **Multi-Server Support**: Configure multiple servers with different ports and hostnames
- **Static File Serving**: Efficient static file serving with MIME type detection
- **CGI Support**: Execute CGI scripts with proper environment variable handling
- **Session Management**: Built-in session and cookie management system
- **File Uploads**: Support for file uploads with configurable size limits
- **Custom Error Pages**: Configurable error pages for different HTTP status codes
- **Robust Error Handling**: Comprehensive error handling that ensures the server never crashes
- **Memory Safe**: Written in Rust with careful memory management to prevent leaks

## Requirements

- Rust 1.70 or later
- Linux system with epoll support
- libc crate for system calls

## Building

```bash
cargo build --release
```

## Running

```bash
# Use default configuration
./target/release/localhost-server

# Use custom configuration file
./target/release/localhost-server config/custom.conf
```

## Configuration

The server uses a configuration file to define server settings, routes, and behavior. See `config/server.conf` for an example configuration.

### Configuration Options

- **Server Settings**: Host, ports, server name
- **Error Pages**: Custom error pages for different status codes
- **Body Size Limits**: Maximum request body size
- **Routes**: URL routing with method restrictions, redirects, and CGI support
- **Static Files**: Root directories and index files
- **Directory Listing**: Enable/disable directory browsing

## Testing

The server includes comprehensive tests and supports stress testing with siege:

```bash
# Run unit tests
cargo test

# Stress test (requires siege)
siege -b 127.0.0.1:8080
```

## Architecture

The server is built with a modular architecture:

- **Core Server**: Epoll-based event loop and connection management
- **HTTP Parser**: Streaming HTTP/1.1 request parser
- **Router**: URL-to-handler mapping with configuration-based routing
- **CGI Executor**: Process forking and CGI script execution
- **Session Store**: In-memory session storage with cookie management
- **Error Handler**: Comprehensive error handling and custom error pages

## License

MIT License - see LICENSE file for details.
