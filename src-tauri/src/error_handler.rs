/// Error handling utilities for Tauri commands
///
/// Provides helpers to preserve error context in logs while returning
/// user-friendly messages to the frontend

use std::fmt::Display;

/// Handle an error by logging the full context and returning a user-friendly message
///
/// # Arguments
/// * `error` - The error to handle (must implement std::error::Error)
/// * `user_message` - User-friendly message prefix
///
/// # Returns
/// A string suitable for returning to the frontend
pub fn handle_command_error<E: std::error::Error>(error: E, user_message: &str) -> String {
    // Log full error with debug formatting to preserve context and error chain
    log::error!("{}: {:?}", user_message, error);

    // Return user-friendly message with error details
    format!("{}: {}", user_message, error)
}

/// Handle an error with additional context information
///
/// # Arguments
/// * `error` - The error to handle
/// * `user_message` - User-friendly message prefix
/// * `context` - Additional context to log (e.g., file path, operation details)
///
/// # Returns
/// A string suitable for returning to the frontend
pub fn handle_command_error_with_context<E, C>(
    error: E,
    user_message: &str,
    context: C,
) -> String
where
    E: std::error::Error,
    C: Display,
{
    // Log full error with context and debug formatting
    log::error!("{} [context: {}]: {:?}", user_message, context, error);

    // Return user-friendly message (don't expose internal details)
    format!("{}: {}", user_message, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_handle_command_error() {
        let error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let result = handle_command_error(error, "Failed to read file");

        assert!(result.contains("Failed to read file"));
        assert!(result.contains("file not found"));
    }

    #[test]
    fn test_handle_command_error_with_context() {
        let error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let result = handle_command_error_with_context(
            error,
            "Failed to write",
            "/path/to/file.txt"
        );

        assert!(result.contains("Failed to write"));
        assert!(result.contains("access denied"));
    }
}
