/*!
 * Error page generation
 */

use crate::error::HttpStatus;

/// Generate HTML error page
pub fn generate_error_page(status: HttpStatus, custom_path: Option<&str>) -> String {
    if let Some(path) = custom_path {
        if let Ok(content) = std::fs::read_to_string(path) {
            return content;
        }
    }

    // Default error page
    let status_code = status.as_u16();
    let reason = status.reason_phrase();
    
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{} - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; text-align: center; padding: 50px; }}
        h1 {{ color: #dc3545; font-size: 4em; margin: 0; }}
        h2 {{ color: #333; }}
        p {{ color: #666; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <h2>{}</h2>
    <p>The server encountered an error processing your request.</p>
</body>
</html>"#,
        status_code, reason, status_code, reason
    )
}
