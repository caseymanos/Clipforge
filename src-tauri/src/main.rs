#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::Manager;
use std::sync::Arc;
use tokio::sync::Mutex;

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
mod error_handler;

// Module 3: FFmpeg Integration
mod ffmpeg;

// Module 4: Screen Recording
mod recording;

// Module 5: Timeline Engine
mod timeline;
mod timeline_commands;

// Module 6: Export & Rendering
mod export;
mod export_commands;

// Module 8: Video Preview
mod preview_cache;
mod preview_service;

use database::Database;
use thumbnail::ThumbnailGenerator;
use file_service::FileService;
use ffmpeg::FFmpegService;
use commands::recording_commands::RecordingService;
use timeline::TimelineService;
use timeline_commands::TimelineServiceState;
use export::ExportService;
use export_commands::ExportServiceState;
use preview_service::PreviewService;
use models::Resolution;

fn main() {
    // Initialize logging
    env_logger::init();

    tauri::Builder::default()
        // Register plugins
        .plugin(tauri_plugin_dialog::init())
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

            // Module 3: Initialize FFmpeg service
            let ffmpeg_service = FFmpegService::new()
                .expect("Failed to initialize FFmpeg service");

            // Module 4: Initialize Recording service
            let recording_service = RecordingService::new();

            // Module 5: Initialize Timeline service with default timeline
            let mut timeline_service = TimelineService::new();

            // Create default timeline so app starts with a ready-to-use timeline
            let default_resolution = Resolution {
                width: 1920,
                height: 1080,
            };

            if let Err(e) = timeline_service.create_timeline(
                "Default Timeline".to_string(),
                30.0,
                default_resolution,
            ) {
                eprintln!("Warning: Failed to create default timeline: {}", e);
            }

            let timeline_state = TimelineServiceState {
                service: Arc::new(Mutex::new(timeline_service)),
            };

            // Module 6: Initialize Export service
            let export_service = ExportService::new()
                .expect("Failed to initialize export service");
            let export_state = ExportServiceState {
                service: Arc::new(Mutex::new(export_service)),
            };

            // Module 8: Initialize Preview service
            let preview_service = Arc::new(Mutex::new(PreviewService::new()));

            app.manage(file_service);
            app.manage(ffmpeg_service);
            app.manage(recording_service);
            app.manage(timeline_state);
            app.manage(export_state);
            app.manage(preview_service);

            log::info!("ClipForge initialized successfully");

            Ok(())
        })
        // Set up menu
        .menu(menu::create_menu)
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
            // Module 3 commands
            commands::trim_video_clip,
            commands::concatenate_clips,
            commands::extract_video_frame,
            commands::apply_video_filter,
            // Module 4 commands
            commands::list_recording_sources,
            commands::check_recording_permissions,
            commands::request_recording_permissions,
            commands::start_recording,
            commands::stop_recording,
            commands::get_recording_state,
            commands::get_recording_duration,
            // Module 5 commands
            timeline_commands::create_timeline,
            timeline_commands::get_current_timeline,
            timeline_commands::add_track,
            timeline_commands::remove_track,
            timeline_commands::add_clip_to_timeline,
            timeline_commands::remove_clip_from_timeline,
            timeline_commands::move_clip_on_timeline,
            timeline_commands::trim_clip_on_timeline,
            timeline_commands::split_clip_at_time,
            timeline_commands::get_clips_at_playhead,
            timeline_commands::save_timeline_project,
            timeline_commands::load_timeline_project,
            // Module 6 commands
            export_commands::export_timeline,
            export_commands::cancel_export,
            export_commands::get_export_presets,
            export_commands::validate_timeline_for_export,
            // Module 8 commands
            commands::render_preview_frame,
            commands::clear_preview_cache,
            commands::get_cache_stats,
        ])
        // Handle window events
        .on_window_event(|window, event| if let tauri::WindowEvent::CloseRequested { .. } = event {
            window_state::save_window_state(window);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
