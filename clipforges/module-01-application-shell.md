# Module 1: Application Shell

**Owner:** TBD  
**Dependencies:** None (foundation module)  
**Phase:** 1 (Weeks 1-2)  
**Estimated Effort:** 2-3 days

## Overview

The Application Shell module provides the foundation for ClipForge, handling:
- Tauri application lifecycle
- Window management and state persistence
- Menu bar integration
- Custom protocol registration for video streaming
- IPC command registration
- Application-wide configuration

This is the **entry point** for the entire application and must be completed before other modules can function.

## Responsibilities

### Core Functionality
- ✅ Initialize Tauri application
- ✅ Create and manage main window
- ✅ Register custom `stream://` protocol for video files
- ✅ Set up IPC command handlers
- ✅ Implement menu bar (File, Edit, View, Help)
- ✅ Handle app lifecycle events (startup, shutdown)
- ✅ Persist window state (size, position)

### Platform-Specific Tasks
- ✅ Configure macOS app bundle
- ✅ Set up Windows installer
- ✅ Create Linux .desktop file
- ✅ Handle platform-specific permissions

## File Structure

```
src-tauri/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── commands/
│   │   └── mod.rs             # Command registration
│   ├── menu.rs                # Menu bar configuration
│   ├── window_state.rs        # Window persistence
│   └── protocols.rs           # Custom protocols
├── tauri.conf.json            # Tauri configuration
├── Cargo.toml                 # Rust dependencies
└── icons/                     # App icons
```

## Implementation Details

### 1. Main Application Entry Point

```rust
// src-tauri/src/main.rs
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{Manager, Window};

mod commands;
mod menu;
mod protocols;
mod window_state;

fn main() {
    // Initialize logging
    env_logger::init();
    
    tauri::Builder::default()
        // Register custom protocols
        .setup(|app| {
            protocols::register_stream_protocol(app)?;
            window_state::restore_window_state(app)?;
            Ok(())
        })
        // Set up menu
        .menu(menu::create_menu())
        .on_menu_event(menu::handle_menu_event)
        // Register all commands
        .invoke_handler(tauri::generate_handler![
            commands::get_app_version,
            commands::open_devtools,
        ])
        // Handle window events
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { .. } => {
                window_state::save_window_state(event.window());
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Custom Protocol for Video Streaming

This enables efficient video file access without JSON serialization overhead.

```rust
// src-tauri/src/protocols.rs
use tauri::{App, Manager};
use std::fs;
use std::path::Path;

pub fn register_stream_protocol(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.app_handle();
    
    app_handle.register_uri_scheme_protocol("stream", move |_app, request| {
        let path = request.uri().path();
        
        // Security: Validate path is within allowed directories
        if !is_path_allowed(path) {
            return tauri::http::ResponseBuilder::new()
                .status(403)
                .body(vec![])
                .unwrap();
        }
        
        // Read video file
        match fs::read(path) {
            Ok(data) => {
                // Determine MIME type from extension
                let mime_type = get_mime_type(path);
                
                tauri::http::ResponseBuilder::new()
                    .status(200)
                    .header("Content-Type", mime_type)
                    .header("Accept-Ranges", "bytes")
                    .body(data)
            }
            Err(e) => {
                eprintln!("Failed to read file: {}", e);
                tauri::http::ResponseBuilder::new()
                    .status(404)
                    .body(vec![])
            }
        }
    });
    
    Ok(())
}

fn is_path_allowed(path: &str) -> bool {
    // Check if path is within user directories
    // Prevent access to system files
    let path = Path::new(path);
    
    // Only allow paths in user's home or app data directory
    if let Some(home_dir) = dirs::home_dir() {
        if path.starts_with(home_dir) {
            return true;
        }
    }
    
    if let Some(data_dir) = dirs::data_local_dir() {
        if path.starts_with(data_dir) {
            return true;
        }
    }
    
    false
}

fn get_mime_type(path: &str) -> &'static str {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("mp4") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("webm") => "video/webm",
        Some("avi") => "video/x-msvideo",
        Some("mkv") => "video/x-matroska",
        _ => "application/octet-stream",
    }
}
```

### 3. Menu Bar Implementation

```rust
// src-tauri/src/menu.rs
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, WindowMenuEvent};

pub fn create_menu() -> Menu {
    // File menu
    let open = CustomMenuItem::new("open".to_string(), "Open Project");
    let save = CustomMenuItem::new("save".to_string(), "Save Project");
    let save_as = CustomMenuItem::new("save_as".to_string(), "Save Project As...");
    let import = CustomMenuItem::new("import".to_string(), "Import Media...");
    let export = CustomMenuItem::new("export".to_string(), "Export Video...");
    
    let file_menu = Submenu::new(
        "File",
        Menu::new()
            .add_item(open)
            .add_native_item(MenuItem::Separator)
            .add_item(import)
            .add_native_item(MenuItem::Separator)
            .add_item(save)
            .add_item(save_as)
            .add_native_item(MenuItem::Separator)
            .add_item(export)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Quit),
    );
    
    // Edit menu
    let undo = CustomMenuItem::new("undo".to_string(), "Undo");
    let redo = CustomMenuItem::new("redo".to_string(), "Redo");
    
    let edit_menu = Submenu::new(
        "Edit",
        Menu::new()
            .add_item(undo)
            .add_item(redo)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::SelectAll),
    );
    
    // View menu
    let zoom_in = CustomMenuItem::new("zoom_in".to_string(), "Zoom In");
    let zoom_out = CustomMenuItem::new("zoom_out".to_string(), "Zoom Out");
    let zoom_reset = CustomMenuItem::new("zoom_reset".to_string(), "Actual Size");
    
    let view_menu = Submenu::new(
        "View",
        Menu::new()
            .add_item(zoom_in)
            .add_item(zoom_out)
            .add_item(zoom_reset)
            .add_native_item(MenuItem::Separator)
            .add_native_item(MenuItem::EnterFullScreen),
    );
    
    // Help menu
    let docs = CustomMenuItem::new("docs".to_string(), "Documentation");
    let about = CustomMenuItem::new("about".to_string(), "About ClipForge");
    
    let help_menu = Submenu::new(
        "Help",
        Menu::new()
            .add_item(docs)
            .add_native_item(MenuItem::Separator)
            .add_item(about),
    );
    
    Menu::new()
        .add_submenu(file_menu)
        .add_submenu(edit_menu)
        .add_submenu(view_menu)
        .add_submenu(help_menu)
}

pub fn handle_menu_event(event: WindowMenuEvent) {
    match event.menu_item_id() {
        "open" => {
            event.window().emit("menu:open-project", ()).unwrap();
        }
        "save" => {
            event.window().emit("menu:save-project", ()).unwrap();
        }
        "import" => {
            event.window().emit("menu:import-media", ()).unwrap();
        }
        "export" => {
            event.window().emit("menu:export-video", ()).unwrap();
        }
        "undo" => {
            event.window().emit("menu:undo", ()).unwrap();
        }
        "redo" => {
            event.window().emit("menu:redo", ()).unwrap();
        }
        "zoom_in" => {
            event.window().emit("menu:zoom-in", ()).unwrap();
        }
        "zoom_out" => {
            event.window().emit("menu:zoom-out", ()).unwrap();
        }
        "docs" => {
            open::that("https://clipforge.dev/docs").ok();
        }
        "about" => {
            event.window().emit("menu:show-about", ()).unwrap();
        }
        _ => {}
    }
}
```

### 4. Window State Persistence

```rust
// src-tauri/src/window_state.rs
use tauri::{App, Manager, PhysicalPosition, PhysicalSize, Window};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct WindowState {
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

pub fn restore_window_state(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_window("main").ok_or("Main window not found")?;
    
    if let Ok(state) = load_window_state() {
        window.set_position(PhysicalPosition::new(state.x, state.y))?;
        window.set_size(PhysicalSize::new(state.width, state.height))?;
        
        if state.maximized {
            window.maximize()?;
        }
    }
    
    Ok(())
}

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
            eprintln!("Failed to save window state: {}", e);
        }
    }
}

fn get_state_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("clipforge").join("window_state.json"))
}

fn load_window_state() -> Result<WindowState, Box<dyn std::error::Error>> {
    let path = get_state_file_path().ok_or("Config directory not found")?;
    let json = fs::read_to_string(path)?;
    let state = serde_json::from_str(&json)?;
    Ok(state)
}

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
```

### 5. Basic Commands

```rust
// src-tauri/src/commands/mod.rs
use tauri::Window;

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn open_devtools(window: Window) {
    #[cfg(debug_assertions)]
    window.open_devtools();
}

#[tauri::command]
pub fn close_splashscreen(window: Window) {
    // Close splashscreen, show main window
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    window.get_window("main").unwrap().show().unwrap();
}
```

### 6. Tauri Configuration

```json
// src-tauri/tauri.conf.json
{
  "$schema": "../node_modules/@tauri-apps/cli/schema.json",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:5173",
    "distDir": "../dist"
  },
  "package": {
    "productName": "ClipForge",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "fs": {
        "all": true,
        "scope": ["$APPDATA/*", "$HOME/*", "$RESOURCE/*"]
      },
      "dialog": {
        "all": true,
        "open": true,
        "save": true
      },
      "shell": {
        "all": false,
        "open": true
      },
      "window": {
        "all": false,
        "close": true,
        "hide": true,
        "show": true,
        "maximize": true,
        "minimize": true,
        "unmaximize": true,
        "unminimize": true,
        "startDragging": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.clipforge.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "resources": [],
      "externalBin": [],
      "copyright": "",
      "category": "Video",
      "shortDescription": "Desktop video editor",
      "longDescription": "A modern, performant desktop video editor built with Tauri and Rust",
      "macOS": {
        "frameworks": [],
        "minimumSystemVersion": "11.0",
        "exceptionDomain": ""
      },
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      },
      "linux": {
        "deb": {
          "depends": []
        }
      }
    },
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: stream: https:; media-src 'self' stream:; connect-src 'self'"
    },
    "windows": [
      {
        "fullscreen": false,
        "height": 720,
        "resizable": true,
        "title": "ClipForge",
        "width": 1280,
        "minWidth": 800,
        "minHeight": 600,
        "center": true
      }
    ]
  }
}
```

## Dependencies

```toml
# src-tauri/Cargo.toml
[package]
name = "clipforge"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2.0", features = ["macos-private-api"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "5.0"
env_logger = "0.11"
log = "0.4"
open = "5.0"  # For opening URLs

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mime_type_detection() {
        assert_eq!(get_mime_type("video.mp4"), "video/mp4");
        assert_eq!(get_mime_type("clip.mov"), "video/quicktime");
        assert_eq!(get_mime_type("unknown.xyz"), "application/octet-stream");
    }
    
    #[test]
    fn test_path_validation() {
        // Test path security checks
        assert!(!is_path_allowed("/etc/passwd"));
        assert!(!is_path_allowed("/System/Library/"));
    }
}
```

### Manual Testing Checklist

- [ ] Application launches successfully
- [ ] Window opens at saved position/size
- [ ] Window maximized state persists
- [ ] Menu items appear correctly on all platforms
- [ ] Menu keyboard shortcuts work
- [ ] DevTools open in debug mode
- [ ] Custom protocol serves video files
- [ ] CSP doesn't block legitimate resources
- [ ] App icon displays correctly in taskbar/dock

## Platform-Specific Considerations

### macOS
- **Permissions:** Request Screen Recording permission in Info.plist
- **Notarization:** Required for distribution outside App Store
- **Code Signing:** Configure signing identity in tauri.conf.json

### Windows
- **Installer:** MSI installer generated automatically
- **Start Menu:** Shortcut created automatically
- **Antivirus:** May flag unsigned executables

### Linux
- **Desktop File:** Create .desktop entry for launcher
- **Dependencies:** GTK3 and WebKit2GTK required
- **Packaging:** Both .deb and .AppImage supported

## Acceptance Criteria

- [x] Application launches in under 3 seconds
- [x] Window state persists between sessions
- [x] Menu bar displays on all platforms
- [x] Custom protocol streams video without JSON serialization
- [x] All IPC commands registered and functional
- [x] App can be built for macOS, Windows, Linux
- [x] Debug mode opens DevTools automatically
- [x] No console errors on startup
- [x] App icon displays correctly

## Next Steps

Once this module is complete:
1. **Module 2 (File System)** can begin using the custom protocol
2. **Module 7 (Timeline UI)** can integrate with menu events
3. **All modules** can register their Tauri commands

## Resources

- [Tauri Documentation](https://tauri.app/)
- [Tauri Custom Protocols](https://tauri.app/v1/guides/features/custom-protocols)
- [Tauri Window State Management](https://tauri.app/v1/guides/features/window-customization)

---

**Status:** Not Started  
**Assignee:** TBD  
**Start Date:** Week 1, Day 1  
**Target Completion:** Week 1, Day 3
