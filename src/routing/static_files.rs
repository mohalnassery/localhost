/*!
 * Static file serving implementation
 */

use crate::error::{ServerError, ServerResult, HttpStatus};
use crate::http::HttpResponse;
use crate::utils::mime::MimeDetector;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Static file server
pub struct StaticFileServer {
    mime_detector: MimeDetector,
}

impl StaticFileServer {
    /// Create a new static file server
    pub fn new() -> Self {
        Self {
            mime_detector: MimeDetector::new(),
        }
    }

    /// Serve a file from the filesystem
    pub fn serve_file(&self, file_path: &Path) -> ServerResult<HttpResponse> {
        // Check if file exists and is readable
        if !file_path.exists() {
            return Ok(HttpResponse::error(HttpStatus::NotFound, Some("File not found")));
        }

        if !file_path.is_file() {
            return Ok(HttpResponse::error(HttpStatus::Forbidden, Some("Not a file")));
        }

        // Read file content
        let content = fs::read(file_path)
            .map_err(|e| ServerError::Http(format!("Failed to read file: {}", e)))?;

        // Detect content type
        let content_type = self.mime_detector.detect_from_path(file_path);

        // Create response
        let mut response = HttpResponse::file(HttpStatus::Ok, content, &content_type);

        // Add caching headers
        self.add_caching_headers(&mut response, file_path)?;

        Ok(response)
    }

    /// Serve a directory (either index file or directory listing)
    pub fn serve_directory(
        &self,
        dir_path: &Path,
        index_file: Option<&str>,
        allow_listing: bool,
        url_path: &str,
    ) -> ServerResult<HttpResponse> {
        // Check if directory exists
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(HttpResponse::error(HttpStatus::NotFound, Some("Directory not found")));
        }

        // Try to serve index file if specified
        if let Some(index) = index_file {
            let index_path = dir_path.join(index);
            if index_path.exists() && index_path.is_file() {
                return self.serve_file(&index_path);
            }
        }

        // If directory listing is allowed, generate listing
        if allow_listing {
            return self.generate_directory_listing(dir_path, url_path);
        }

        // Otherwise, return forbidden
        Ok(HttpResponse::error(HttpStatus::Forbidden, Some("Directory listing disabled")))
    }

    /// Generate HTML directory listing
    fn generate_directory_listing(&self, dir_path: &Path, url_path: &str) -> ServerResult<HttpResponse> {
        let entries = fs::read_dir(dir_path)
            .map_err(|e| ServerError::Http(format!("Failed to read directory: {}", e)))?;

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html><head>\n");
        html.push_str("<title>Directory Listing</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 40px; }\n");
        html.push_str("h1 { color: #333; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { text-align: left; padding: 8px 12px; border-bottom: 1px solid #ddd; }\n");
        html.push_str("th { background-color: #f5f5f5; }\n");
        html.push_str("a { text-decoration: none; color: #0066cc; }\n");
        html.push_str("a:hover { text-decoration: underline; }\n");
        html.push_str(".size { text-align: right; }\n");
        html.push_str(".date { white-space: nowrap; }\n");
        html.push_str("</style>\n");
        html.push_str("</head><body>\n");
        html.push_str(&format!("<h1>Directory listing for {}</h1>\n", url_path));

        // Add parent directory link if not root
        if url_path != "/" {
            let parent_path = if url_path.ends_with('/') {
                format!("{}../", url_path)
            } else {
                format!("{}/", url_path.rsplitn(2, '/').nth(1).unwrap_or(""))
            };
            html.push_str(&format!("<p><a href=\"{}\">📁 Parent Directory</a></p>\n", parent_path));
        }

        html.push_str("<table>\n");
        html.push_str("<tr><th>Name</th><th>Size</th><th>Modified</th></tr>\n");

        // Collect and sort entries
        let mut entries_vec = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                entries_vec.push(entry);
            }
        }

        // Sort: directories first, then files, both alphabetically
        entries_vec.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        // Generate table rows
        for entry in entries_vec {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

            let display_name = if is_dir {
                format!("📁 {}/", name)
            } else {
                format!("📄 {}", name)
            };

            let href = if url_path.ends_with('/') {
                format!("{}{}", url_path, name)
            } else {
                format!("{}/{}", url_path, name)
            };

            // Get file size and modification time
            let (size_str, modified_str) = if let Ok(metadata) = entry.metadata() {
                let size = if is_dir {
                    "-".to_string()
                } else {
                    format_file_size(metadata.len())
                };

                let modified = metadata.modified()
                    .map(|time| format_time(time))
                    .unwrap_or_else(|_| "-".to_string());

                (size, modified)
            } else {
                ("-".to_string(), "-".to_string())
            };

            html.push_str(&format!(
                "<tr><td><a href=\"{}\">{}</a></td><td class=\"size\">{}</td><td class=\"date\">{}</td></tr>\n",
                href, display_name, size_str, modified_str
            ));
        }

        html.push_str("</table>\n");
        html.push_str("</body></html>");

        Ok(HttpResponse::html(HttpStatus::Ok, &html))
    }

    /// Add caching headers to response
    fn add_caching_headers(&self, response: &mut HttpResponse, file_path: &Path) -> ServerResult<()> {
        // Add Last-Modified header
        if let Ok(metadata) = fs::metadata(file_path) {
            if let Ok(modified) = metadata.modified() {
                let http_date = format_http_date(modified);
                response.add_header("Last-Modified", &http_date);
            }
        }

        // Add basic cache control
        response.add_header("Cache-Control", "public, max-age=3600");

        Ok(())
    }

    /// Resolve file path with security checks
    pub fn resolve_path(&self, root: &str, request_path: &str, route_path: &str) -> ServerResult<PathBuf> {
        // Remove route prefix from request path
        let relative_path = if request_path.starts_with(route_path) {
            &request_path[route_path.len()..]
        } else {
            request_path
        };

        // Remove leading slash
        let relative_path = relative_path.strip_prefix('/').unwrap_or(relative_path);

        // Construct full path
        let mut full_path = PathBuf::from(root);
        if !relative_path.is_empty() {
            full_path.push(relative_path);
        }

        // Security check: ensure path doesn't escape root directory
        let canonical_root = fs::canonicalize(root)
            .map_err(|_| ServerError::Config(format!("Invalid root directory: {}", root)))?;

        if let Ok(canonical_path) = fs::canonicalize(&full_path) {
            if !canonical_path.starts_with(&canonical_root) {
                return Err(ServerError::Http("Path traversal attempt detected".to_string()));
            }
        }

        Ok(full_path)
    }
}

impl Default for StaticFileServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Format file size in human-readable format
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format system time as HTTP date
fn format_http_date(time: SystemTime) -> String {
    // This is a simplified implementation
    // In production, you'd want proper RFC 2822 formatting
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_secs();
            format!("Thu, 01 Jan 1970 00:00:{:02} GMT", timestamp % 60)
        }
        Err(_) => "Thu, 01 Jan 1970 00:00:00 GMT".to_string(),
    }
}

/// Format system time for directory listing
fn format_time(time: SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => {
            let timestamp = duration.as_secs();
            // Simplified time formatting
            format!("1970-01-01 00:00:{:02}", timestamp % 60)
        }
        Err(_) => "1970-01-01 00:00:00".to_string(),
    }
}
