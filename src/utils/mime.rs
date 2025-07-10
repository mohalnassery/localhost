/*!
 * MIME type detection
 */

use std::collections::HashMap;
use std::path::Path;

/// MIME type detector
pub struct MimeDetector {
    extensions: HashMap<String, String>,
}

impl MimeDetector {
    /// Create a new MIME detector with default mappings
    pub fn new() -> Self {
        let mut extensions = HashMap::new();

        // Text files
        extensions.insert("html".to_string(), "text/html; charset=utf-8".to_string());
        extensions.insert("htm".to_string(), "text/html; charset=utf-8".to_string());
        extensions.insert("css".to_string(), "text/css; charset=utf-8".to_string());
        extensions.insert("js".to_string(), "application/javascript; charset=utf-8".to_string());
        extensions.insert("json".to_string(), "application/json; charset=utf-8".to_string());
        extensions.insert("xml".to_string(), "application/xml; charset=utf-8".to_string());
        extensions.insert("txt".to_string(), "text/plain; charset=utf-8".to_string());
        extensions.insert("md".to_string(), "text/markdown; charset=utf-8".to_string());
        extensions.insert("csv".to_string(), "text/csv; charset=utf-8".to_string());

        // Images
        extensions.insert("png".to_string(), "image/png".to_string());
        extensions.insert("jpg".to_string(), "image/jpeg".to_string());
        extensions.insert("jpeg".to_string(), "image/jpeg".to_string());
        extensions.insert("gif".to_string(), "image/gif".to_string());
        extensions.insert("svg".to_string(), "image/svg+xml".to_string());
        extensions.insert("ico".to_string(), "image/x-icon".to_string());
        extensions.insert("webp".to_string(), "image/webp".to_string());
        extensions.insert("bmp".to_string(), "image/bmp".to_string());
        extensions.insert("tiff".to_string(), "image/tiff".to_string());

        // Audio
        extensions.insert("mp3".to_string(), "audio/mpeg".to_string());
        extensions.insert("wav".to_string(), "audio/wav".to_string());
        extensions.insert("ogg".to_string(), "audio/ogg".to_string());
        extensions.insert("m4a".to_string(), "audio/mp4".to_string());
        extensions.insert("flac".to_string(), "audio/flac".to_string());

        // Video
        extensions.insert("mp4".to_string(), "video/mp4".to_string());
        extensions.insert("avi".to_string(), "video/x-msvideo".to_string());
        extensions.insert("mov".to_string(), "video/quicktime".to_string());
        extensions.insert("wmv".to_string(), "video/x-ms-wmv".to_string());
        extensions.insert("webm".to_string(), "video/webm".to_string());

        // Documents
        extensions.insert("pdf".to_string(), "application/pdf".to_string());
        extensions.insert("doc".to_string(), "application/msword".to_string());
        extensions.insert("docx".to_string(), "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string());
        extensions.insert("xls".to_string(), "application/vnd.ms-excel".to_string());
        extensions.insert("xlsx".to_string(), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string());
        extensions.insert("ppt".to_string(), "application/vnd.ms-powerpoint".to_string());
        extensions.insert("pptx".to_string(), "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string());

        // Archives
        extensions.insert("zip".to_string(), "application/zip".to_string());
        extensions.insert("tar".to_string(), "application/x-tar".to_string());
        extensions.insert("gz".to_string(), "application/gzip".to_string());
        extensions.insert("bz2".to_string(), "application/x-bzip2".to_string());
        extensions.insert("7z".to_string(), "application/x-7z-compressed".to_string());
        extensions.insert("rar".to_string(), "application/vnd.rar".to_string());

        // Fonts
        extensions.insert("ttf".to_string(), "font/ttf".to_string());
        extensions.insert("otf".to_string(), "font/otf".to_string());
        extensions.insert("woff".to_string(), "font/woff".to_string());
        extensions.insert("woff2".to_string(), "font/woff2".to_string());

        // Programming languages
        extensions.insert("py".to_string(), "text/x-python".to_string());
        extensions.insert("rs".to_string(), "text/x-rust".to_string());
        extensions.insert("c".to_string(), "text/x-c".to_string());
        extensions.insert("cpp".to_string(), "text/x-c++".to_string());
        extensions.insert("h".to_string(), "text/x-c".to_string());
        extensions.insert("java".to_string(), "text/x-java".to_string());
        extensions.insert("php".to_string(), "text/x-php".to_string());
        extensions.insert("rb".to_string(), "text/x-ruby".to_string());
        extensions.insert("go".to_string(), "text/x-go".to_string());

        Self { extensions }
    }

    /// Detect MIME type from file path
    pub fn detect_from_path(&self, path: &Path) -> String {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                if let Some(mime_type) = self.extensions.get(&ext_lower) {
                    return mime_type.clone();
                }
            }
        }

        // Default to binary if no match
        "application/octet-stream".to_string()
    }

    /// Detect MIME type from filename
    pub fn detect_from_filename(&self, filename: &str) -> String {
        if let Some(dot_pos) = filename.rfind('.') {
            let extension = &filename[dot_pos + 1..].to_lowercase();
            if let Some(mime_type) = self.extensions.get(extension) {
                return mime_type.clone();
            }
        }

        "application/octet-stream".to_string()
    }

    /// Add or update a MIME type mapping
    pub fn add_mapping(&mut self, extension: &str, mime_type: &str) {
        self.extensions.insert(extension.to_lowercase(), mime_type.to_string());
    }

    /// Check if a MIME type is text-based
    pub fn is_text_type(&self, mime_type: &str) -> bool {
        mime_type.starts_with("text/") ||
        mime_type.starts_with("application/json") ||
        mime_type.starts_with("application/xml") ||
        mime_type.starts_with("application/javascript")
    }

    /// Check if a MIME type is an image
    pub fn is_image_type(&self, mime_type: &str) -> bool {
        mime_type.starts_with("image/")
    }

    /// Check if a MIME type is compressible
    pub fn is_compressible(&self, mime_type: &str) -> bool {
        self.is_text_type(mime_type) ||
        mime_type.starts_with("image/svg") ||
        mime_type.starts_with("application/json") ||
        mime_type.starts_with("application/xml")
    }

    /// Get all supported extensions
    pub fn supported_extensions(&self) -> Vec<&String> {
        self.extensions.keys().collect()
    }
}

impl Default for MimeDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mime_detection() {
        let detector = MimeDetector::new();

        // Test common file types
        assert_eq!(detector.detect_from_filename("test.html"), "text/html; charset=utf-8");
        assert_eq!(detector.detect_from_filename("style.css"), "text/css; charset=utf-8");
        assert_eq!(detector.detect_from_filename("script.js"), "application/javascript; charset=utf-8");
        assert_eq!(detector.detect_from_filename("image.png"), "image/png");
        assert_eq!(detector.detect_from_filename("document.pdf"), "application/pdf");

        // Test case insensitivity
        assert_eq!(detector.detect_from_filename("TEST.HTML"), "text/html; charset=utf-8");
        assert_eq!(detector.detect_from_filename("Image.PNG"), "image/png");

        // Test unknown extension
        assert_eq!(detector.detect_from_filename("unknown.xyz"), "application/octet-stream");

        // Test no extension
        assert_eq!(detector.detect_from_filename("noextension"), "application/octet-stream");
    }

    #[test]
    fn test_path_detection() {
        let detector = MimeDetector::new();
        let path = PathBuf::from("/var/www/test.html");

        assert_eq!(detector.detect_from_path(&path), "text/html; charset=utf-8");
    }

    #[test]
    fn test_type_checking() {
        let detector = MimeDetector::new();

        assert!(detector.is_text_type("text/html"));
        assert!(detector.is_text_type("application/json"));
        assert!(!detector.is_text_type("image/png"));

        assert!(detector.is_image_type("image/png"));
        assert!(detector.is_image_type("image/jpeg"));
        assert!(!detector.is_image_type("text/html"));

        assert!(detector.is_compressible("text/html"));
        assert!(detector.is_compressible("application/json"));
        assert!(!detector.is_compressible("image/png"));
    }
}
