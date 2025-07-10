/*!
 * HTTP methods implementation
 */

use crate::config::{Config, RouteConfig};
use crate::error::{ServerError, ServerResult, HttpStatus};
use crate::http::{HttpRequest, HttpResponse};
use crate::routing::{Router, StaticFileServer};
use std::fs;
use std::path::{Path, PathBuf};

/// HTTP method handler
pub struct MethodHandler {
    router: Router,
    static_server: StaticFileServer,
}

impl MethodHandler {
    pub fn new(config: Config) -> Self {
        Self {
            router: Router::new(&config),
            static_server: StaticFileServer::new(),
        }
    }

    /// Handle an HTTP request and generate a response
    pub fn handle_request(&self, request: &HttpRequest) -> ServerResult<HttpResponse> {
        // Find matching route using the router
        let host = request.get_header("host").map(|s| s.as_str());
        let (server, route) = self.router.find_route(host, &request.path)?;

        // Check if method is allowed
        if !route.methods.contains(&request.method.as_str().to_string()) {
            return Ok(HttpResponse::error(HttpStatus::MethodNotAllowed,
                Some(&format!("Method {} not allowed for this route", request.method.as_str()))));
        }

        // Handle based on method
        match request.method {
            crate::http::HttpMethod::GET => self.handle_get(request, server, route),
            crate::http::HttpMethod::POST => self.handle_post(request, server, route),
            crate::http::HttpMethod::DELETE => self.handle_delete(request, server, route),
            crate::http::HttpMethod::HEAD => self.handle_head(request, server, route),
            _ => Ok(HttpResponse::error(HttpStatus::MethodNotAllowed,
                Some(&format!("Method {} not implemented", request.method.as_str())))),
        }
    }

    /// Handle GET requests
    fn handle_get(&self, request: &HttpRequest, _server: &crate::config::ServerConfig, route: &RouteConfig) -> ServerResult<HttpResponse> {
        // Handle redirects
        if let Some(redirect_url) = &route.redirect {
            return Ok(HttpResponse::redirect(redirect_url, false));
        }

        // Get root directory
        let root = route.root.as_ref()
            .ok_or_else(|| ServerError::Config("Route has no root directory".to_string()))?;

        // Resolve file path using static file server
        let file_path = self.static_server.resolve_path(root, &request.path, &route.path)?;

        // Check if path exists
        if !file_path.exists() {
            return Ok(HttpResponse::error(HttpStatus::NotFound, Some("File not found")));
        }

        // Handle directories
        if file_path.is_dir() {
            return self.static_server.serve_directory(
                &file_path,
                route.index.as_deref(),
                route.directory_listing,
                &request.path,
            );
        }

        // Serve file using static file server
        self.static_server.serve_file(&file_path)
    }

    /// Handle POST requests
    fn handle_post(&self, request: &HttpRequest, _server: &crate::config::ServerConfig, route: &RouteConfig) -> ServerResult<HttpResponse> {
        // Handle file uploads
        if route.upload_enabled {
            return self.handle_file_upload(request, route);
        }

        // Handle CGI
        if let Some(_cgi_interpreter) = &route.cgi {
            return self.handle_cgi(request, route);
        }

        // Default POST handling
        Ok(HttpResponse::text(HttpStatus::Ok, "POST request received"))
    }

    /// Handle DELETE requests
    fn handle_delete(&self, request: &HttpRequest, _server: &crate::config::ServerConfig, route: &RouteConfig) -> ServerResult<HttpResponse> {
        let root = route.root.as_ref()
            .ok_or_else(|| ServerError::Config("Route has no root directory".to_string()))?;
        let file_path = self.static_server.resolve_path(root, &request.path, &route.path)?;

        if !file_path.exists() {
            return Ok(HttpResponse::error(HttpStatus::NotFound, Some("File not found")));
        }

        // Only allow deletion in upload directories for security
        if !route.upload_enabled {
            return Ok(HttpResponse::error(HttpStatus::Forbidden,
                Some("Deletion not allowed in this directory")));
        }

        match fs::remove_file(&file_path) {
            Ok(_) => Ok(HttpResponse::text(HttpStatus::NoContent, "")),
            Err(_) => Ok(HttpResponse::error(HttpStatus::InternalServerError,
                Some("Failed to delete file"))),
        }
    }

    /// Handle HEAD requests (like GET but without body)
    fn handle_head(&self, request: &HttpRequest, server: &crate::config::ServerConfig, route: &RouteConfig) -> ServerResult<HttpResponse> {
        let mut response = self.handle_get(request, server, route)?;
        response.body.clear(); // Remove body for HEAD requests
        response.add_header("Content-Length", "0");
        Ok(response)
    }



    /// Handle file upload
    fn handle_file_upload(&self, request: &HttpRequest, route: &RouteConfig) -> ServerResult<HttpResponse> {
        // Simple file upload implementation
        // In a real implementation, you'd parse multipart/form-data
        let upload_path = route.root.as_ref()
            .ok_or_else(|| ServerError::Config("Upload route has no root directory".to_string()))?;

        // For now, just save the raw body as a file
        let filename = format!("upload_{}.bin", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());

        let file_path = Path::new(upload_path).join(filename);

        fs::write(&file_path, &request.body)
            .map_err(|_| ServerError::Http("Failed to save uploaded file".to_string()))?;

        Ok(HttpResponse::text(HttpStatus::Created,
            &format!("File uploaded successfully: {}", file_path.display())))
    }

    /// Handle CGI requests
    fn handle_cgi(&self, _request: &HttpRequest, _route: &RouteConfig) -> ServerResult<HttpResponse> {
        // TODO: Implement CGI execution
        Ok(HttpResponse::text(HttpStatus::Ok, "CGI not yet implemented"))
    }
}
