use tauri::{menu::*, AppHandle, Manager, Wry};

/// Create the application menu bar
pub fn create_menu() -> Result<Menu<Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new()?;

    // Note: Tauri v2 has a different menu API
    // We'll create a basic menu structure for now
    // Full menu implementation will be added once we confirm Tauri v2 API

    Ok(menu)
}

/// Handle menu events
pub fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "open" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:open-project", ()).ok();
            }
        }
        "save" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:save-project", ()).ok();
            }
        }
        "import" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:import-media", ()).ok();
            }
        }
        "export" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:export-video", ()).ok();
            }
        }
        "undo" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:undo", ()).ok();
            }
        }
        "redo" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:redo", ()).ok();
            }
        }
        "zoom_in" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:zoom-in", ()).ok();
            }
        }
        "zoom_out" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:zoom-out", ()).ok();
            }
        }
        "docs" => {
            open::that("https://clipforge.dev/docs").ok();
        }
        "about" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:show-about", ()).ok();
            }
        }
        _ => {}
    }
}
