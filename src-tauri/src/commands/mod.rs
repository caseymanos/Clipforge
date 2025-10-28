use tauri::{AppHandle, Manager, Window};

// Module 2: File commands
pub mod file_commands;

// Module 3: FFmpeg commands
pub mod ffmpeg_commands;

// Module 4: Recording commands
pub mod recording_commands;

// Module 8: Preview commands
pub mod preview_commands;

// Re-export commands for convenience
pub use file_commands::*;
pub use ffmpeg_commands::*;
pub use recording_commands::*;
pub use preview_commands::*;

/// Get the application version from Cargo.toml
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Open DevTools window (debug only)
#[tauri::command]
pub fn open_devtools(window: Window) {
    #[cfg(debug_assertions)]
    {
        // Note: In Tauri v2, devtools are opened via WebviewWindow methods
        // This is a no-op for now - devtools can be opened via right-click context menu
        log::info!("DevTools requested - use right-click menu to open");
        let _ = window; // Suppress unused variable warning
    }
}

/// Close splashscreen and show main window
#[tauri::command]
pub fn close_splashscreen(app: AppHandle) {
    // Get windows
    if let Some(splashscreen) = app.get_webview_window("splashscreen") {
        splashscreen.close().ok();
    }
    if let Some(main) = app.get_webview_window("main") {
        main.show().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_version() {
        let version = get_app_version();
        assert!(!version.is_empty());
        assert_eq!(version, "0.1.0");
    }
}
