# Configuration Guide

This guide provides comprehensive documentation for configuring the Localhost HTTP Server.

## Configuration File Format

The server uses a configuration format inspired by nginx. Configuration files use a block-based syntax with nested directives.

## Basic Structure

```nginx
server {
    # Server-level directives
    server_name example.com
    listen 80 443
    max_body_size 1048576
    
    # Route definitions
    route / {
        # Route-level directives
        methods GET POST
        root /var/www
        index index.html
    }
    
    # Error page mappings
    error_page 404 /errors/404.html
    error_page 500 /errors/500.html
}
```

## Server Directives

### server_name

Specifies the server name (hostname) for this server block.

```nginx
server_name localhost
server_name example.com www.example.com
```

**Default:** None (accepts all hostnames)

### listen

Specifies the port(s) the server should listen on.

```nginx
listen 80
listen 80 443 8080
```

**Default:** 8080

### max_body_size

Maximum size of request body in bytes.

```nginx
max_body_size 1048576    # 1MB
max_body_size 10485760   # 10MB
```

**Default:** 1048576 (1MB)

### error_page

Maps HTTP status codes to custom error pages.

```nginx
error_page 404 /errors/404.html
error_page 500 /errors/500.html
error_page 403 /errors/forbidden.html
```

**Supported Status Codes:**
- 400 (Bad Request)
- 403 (Forbidden)
- 404 (Not Found)
- 405 (Method Not Allowed)
- 413 (Request Entity Too Large)
- 500 (Internal Server Error)

## Route Directives

Routes define how URLs are handled by the server. Routes are matched using longest-prefix matching.

### Basic Route

```nginx
route / {
    methods GET POST
    root www
    index index.html
    directory_listing on
}
```

### methods

Specifies which HTTP methods are allowed for this route.

```nginx
methods GET
methods GET POST
methods GET POST PUT DELETE HEAD
```

**Supported Methods:**
- GET
- POST
- PUT
- DELETE
- HEAD

**Default:** GET

### root

Specifies the document root directory for this route.

```nginx
root /var/www/html
root www
root ../content
```

**Default:** Current directory

### index

Specifies default files to serve when a directory is requested.

```nginx
index index.html
index index.html index.htm default.html
```

**Default:** index.html

### directory_listing

Enables or disables directory listing when no index file is found.

```nginx
directory_listing on
directory_listing off
```

**Default:** off

### cgi

Specifies the CGI interpreter for this route.

```nginx
cgi python3
cgi perl
cgi /usr/bin/python3
```

**Common Interpreters:**
- python3
- python
- perl
- sh
- bash
- php

### upload_enabled

Enables file uploads via POST requests.

```nginx
upload_enabled on
upload_enabled off
```

**Default:** off

### redirect

Redirects requests to another URL.

```nginx
redirect http://example.com/new-location
redirect /new-path
```

**Types:**
- Absolute URLs: `http://example.com/path`
- Relative paths: `/new-path`

## Configuration Examples

### Basic Static Website

```nginx
server {
    server_name localhost
    listen 8080
    max_body_size 1048576
    
    route / {
        methods GET
        root www
        index index.html
        directory_listing on
    }
    
    error_page 404 /errors/404.html
}
```

### Website with CGI Support

```nginx
server {
    server_name example.com
    listen 80
    max_body_size 10485760
    
    # Static files
    route / {
        methods GET
        root /var/www/html
        index index.html
        directory_listing off
    }
    
    # CGI scripts
    route /cgi-bin {
        methods GET POST
        root /var/www/cgi-bin
        cgi python3
    }
    
    # API endpoints
    route /api {
        methods GET POST PUT DELETE
        root /var/www/api
        cgi python3
    }
    
    error_page 404 /errors/404.html
    error_page 500 /errors/500.html
}
```

### Multiple Virtual Hosts

```nginx
# Main website
server {
    server_name example.com www.example.com
    listen 80
    
    route / {
        methods GET POST
        root /var/www/example.com
        index index.html
    }
    
    route /api {
        methods GET POST PUT DELETE
        root /var/www/example.com/api
        cgi python3
    }
}

# Blog subdomain
server {
    server_name blog.example.com
    listen 80
    
    route / {
        methods GET POST
        root /var/www/blog
        index index.html
        directory_listing on
    }
    
    route /admin {
        methods GET POST
        root /var/www/blog/admin
        cgi python3
    }
}

# API server
server {
    server_name api.example.com
    listen 80
    
    route / {
        methods GET POST PUT DELETE
        root /var/www/api
        cgi python3
    }
}
```

### File Upload Server

```nginx
server {
    server_name upload.example.com
    listen 80
    max_body_size 104857600  # 100MB
    
    # Upload endpoint
    route /upload {
        methods POST
        root /var/www/uploads
        cgi python3
        upload_enabled on
    }
    
    # Download files
    route /files {
        methods GET
        root /var/www/uploads/files
        directory_listing on
    }
    
    error_page 413 /errors/file-too-large.html
}
```

### Development Server

```nginx
server {
    server_name localhost
    listen 3000
    max_body_size 1048576
    
    # Static assets
    route /static {
        methods GET
        root assets
        directory_listing on
    }
    
    # Development API
    route /api {
        methods GET POST PUT DELETE
        root api
        cgi python3
    }
    
    # Catch-all for SPA
    route / {
        methods GET
        root dist
        index index.html
    }
}
```

## Advanced Configuration

### Route Priority

Routes are matched using longest-prefix matching. More specific routes should be defined before general ones:

```nginx
# Specific routes first
route /api/v2 {
    root api/v2
    cgi python3
}

route /api {
    root api/v1
    cgi python3
}

# General routes last
route / {
    root www
}
```

### Method-Specific Handling

Different methods can be handled by different routes:

```nginx
# GET requests for browsing
route /files {
    methods GET
    root uploads
    directory_listing on
}

# POST requests for uploading
route /files {
    methods POST
    root uploads
    cgi python3
    upload_enabled on
}
```

### Security Considerations

```nginx
server {
    server_name secure.example.com
    listen 443
    max_body_size 1048576
    
    # Restrict admin access
    route /admin {
        methods GET POST
        root admin
        cgi python3
        # Note: Add authentication in CGI scripts
    }
    
    # Public content
    route / {
        methods GET
        root public
        directory_listing off
    }
    
    # Custom error pages (don't reveal server info)
    error_page 404 /errors/not-found.html
    error_page 500 /errors/server-error.html
}
```

## Configuration Validation

The server validates configuration on startup. Common errors include:

- **Syntax Errors**: Missing braces, invalid directives
- **Invalid Paths**: Non-existent root directories
- **Invalid Methods**: Unsupported HTTP methods
- **Invalid Ports**: Ports outside valid range (1-65535)
- **Duplicate Routes**: Conflicting route definitions

## Environment Variables

Configuration files can reference environment variables:

```nginx
server {
    server_name ${SERVER_NAME}
    listen ${PORT}
    
    route / {
        root ${DOCUMENT_ROOT}
    }
}
```

## Best Practices

1. **Use specific routes before general ones**
2. **Set appropriate max_body_size limits**
3. **Disable directory_listing in production**
4. **Use custom error pages**
5. **Validate configuration before deployment**
6. **Use absolute paths for production**
7. **Implement proper CGI security**
8. **Monitor server logs**

## Troubleshooting

### Common Issues

1. **Server won't start**: Check port availability and permissions
2. **404 errors**: Verify root paths and file permissions
3. **CGI errors**: Check interpreter paths and script permissions
4. **Upload failures**: Verify max_body_size and upload_enabled
5. **Permission denied**: Check file and directory permissions

### Debug Mode

Start the server with verbose logging:

```bash
RUST_LOG=debug ./target/release/localhost-server config.conf
```

### Configuration Testing

Test configuration without starting the server:

```bash
./target/release/localhost-server --test config.conf
```
