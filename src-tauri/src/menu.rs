use tauri::{menu::*, AppHandle, Emitter, Manager, Wry};

/// Create the application menu bar
pub fn create_menu(app: &AppHandle<Wry>) -> tauri::Result<Menu<Wry>> {
    // File Menu
    let open_item = MenuItem::with_id(app, "open", "Open Project", true, Some("CmdOrCtrl+O"))?;
    let import_item = MenuItem::with_id(app, "import", "Import Media", true, Some("CmdOrCtrl+I"))?;
    let save_item = MenuItem::with_id(app, "save", "Save Project", true, Some("CmdOrCtrl+S"))?;
    let export_item = MenuItem::with_id(app, "export", "Export Video", true, Some("CmdOrCtrl+E"))?;
    let close_item = MenuItem::with_id(app, "close", "Close Window", true, Some("CmdOrCtrl+W"))?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&open_item)
        .item(&import_item)
        .separator()
        .item(&save_item)
        .item(&export_item)
        .separator()
        .item(&close_item)
        .item(&quit_item)
        .build()?;

    // Edit Menu
    let undo_item = MenuItem::with_id(app, "undo", "Undo", true, Some("CmdOrCtrl+Z"))?;
    let redo_item = MenuItem::with_id(app, "redo", "Redo", true, Some("CmdOrCtrl+Shift+Z"))?;
    let cut_item = MenuItem::with_id(app, "cut", "Cut", true, Some("CmdOrCtrl+X"))?;
    let copy_item = MenuItem::with_id(app, "copy", "Copy", true, Some("CmdOrCtrl+C"))?;
    let paste_item = MenuItem::with_id(app, "paste", "Paste", true, Some("CmdOrCtrl+V"))?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&undo_item)
        .item(&redo_item)
        .separator()
        .item(&cut_item)
        .item(&copy_item)
        .item(&paste_item)
        .build()?;

    // View Menu
    let zoom_in_item = MenuItem::with_id(app, "zoom_in", "Zoom In", true, Some("CmdOrCtrl+Plus"))?;
    let zoom_out_item = MenuItem::with_id(app, "zoom_out", "Zoom Out", true, Some("CmdOrCtrl+Minus"))?;
    let fullscreen_item = MenuItem::with_id(app, "fullscreen", "Toggle Fullscreen", true, Some("F11"))?;

    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&zoom_in_item)
        .item(&zoom_out_item)
        .separator()
        .item(&fullscreen_item)
        .build()?;

    // Help Menu
    let docs_item = MenuItem::with_id(app, "docs", "Documentation", true, None::<&str>)?;
    let about_item = MenuItem::with_id(app, "about", "About ClipForge", true, None::<&str>)?;

    let help_menu = SubmenuBuilder::new(app, "Help")
        .item(&docs_item)
        .separator()
        .item(&about_item)
        .build()?;

    // Build the complete menu using MenuBuilder
    let menu = MenuBuilder::new(app)
        .item(&file_menu)
        .item(&edit_menu)
        .item(&view_menu)
        .item(&help_menu)
        .build()?;

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
        "close" => {
            if let Some(window) = app.get_webview_window("main") {
                window.close().ok();
            }
        }
        "quit" => {
            app.exit(0);
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
        "cut" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:cut", ()).ok();
            }
        }
        "copy" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:copy", ()).ok();
            }
        }
        "paste" => {
            if let Some(window) = app.get_webview_window("main") {
                window.emit("menu:paste", ()).ok();
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
        "fullscreen" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.is_fullscreen().map(|is_full| {
                    window.set_fullscreen(!is_full).ok();
                });
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
