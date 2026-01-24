// Utility functions for logging and path handling

use std::path::Path;

/// Extract just the filename from a path
pub fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Extract just the directory name from a path (for logging)
pub fn dirname(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or(".")
        .to_string()
}

/// Check if we're in dev mode (for conditional logging)
pub fn is_dev_mode() -> bool {
    // In release builds, this will be false
    cfg!(debug_assertions)
}
