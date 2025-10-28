#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;

// Module 1: Application Shell
mod commands;
mod menu;
mod protocols;
mod window_state;

// Module 2: File System & Media
mod models;
mod database;
mod metadata;
mod thumbnail;
mod file_service;

use database::Database;
use thumbnail::ThumbnailGenerator;
use file_service::FileService;

fn main() {
    // Initialize logging
    env_logger::init();

    tauri::Builder::default()
        // Register custom protocols
        .setup(|app| {
            protocols::register_stream_protocol(app)?;
            window_state::restore_window_state(app)?;

            // Module 2: Initialize database and file service
            let db = Database::new()
                .expect("Failed to initialize database");
            let thumbnail_gen = ThumbnailGenerator::new()
                .expect("Failed to initialize thumbnail generator");
            let file_service = FileService::new(db, thumbnail_gen);

            app.manage(file_service);

            log::info!("ClipForge initialized successfully");

            Ok(())
        })
        // Set up menu
        .menu(menu::create_menu())
        .on_menu_event(menu::handle_menu_event)
        // Register all commands
        .invoke_handler(tauri::generate_handler![
            // Module 1 commands
            commands::get_app_version,
            commands::open_devtools,
            commands::close_splashscreen,
            // Module 2 commands
            commands::import_media_file,
            commands::get_media_library,
            commands::get_media_file,
            commands::delete_media_file,
            commands::get_file_metadata,
            commands::generate_thumbnail,
            commands::generate_thumbnail_sequence,
        ])
        // Handle window events
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                window_state::save_window_state(window);
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
