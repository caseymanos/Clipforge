use tauri::{App, AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Window};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowState {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 100,
            y: 100,
            width: 1280,
            height: 720,
            maximized: false,
        }
    }
}

/// Restore window state from saved configuration
pub fn restore_window_state(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(state) = load_window_state() {
            window.set_position(PhysicalPosition::new(state.x, state.y))?;
            window.set_size(PhysicalSize::new(state.width, state.height))?;

            if state.maximized {
                window.maximize()?;
            }
        }
    }

    Ok(())
}

/// Save window state to disk
pub fn save_window_state(window: &Window) {
    let position = window.outer_position().ok();
    let size = window.outer_size().ok();
    let maximized = window.is_maximized().unwrap_or(false);

    if let (Some(pos), Some(size)) = (position, size) {
        let state = WindowState {
            x: pos.x,
            y: pos.y,
            width: size.width,
            height: size.height,
            maximized,
        };

        if let Err(e) = persist_window_state(&state) {
            log::error!("Failed to save window state: {}", e);
        }
    }
}

/// Get the path to the window state file
fn get_state_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("clipforge").join("window_state.json"))
}

/// Load window state from disk
fn load_window_state() -> Result<WindowState, Box<dyn std::error::Error>> {
    let path = get_state_file_path().ok_or("Config directory not found")?;
    let json = fs::read_to_string(path)?;
    let state = serde_json::from_str(&json)?;
    Ok(state)
}

/// Persist window state to disk
fn persist_window_state(state: &WindowState) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_state_file_path().ok_or("Config directory not found")?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json)?;

    Ok(())
}
