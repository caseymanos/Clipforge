use tauri::{AppHandle, Manager, Window};

// Module 2: File commands
pub mod file_commands;

// Re-export file commands for convenience
pub use file_commands::*;

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
        window.open_devtools();
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
