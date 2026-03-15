// ContextPaste — Tauri application setup
// All modules and command registration happens here

mod ai;
mod clipboard;
mod commands;
mod prediction;
mod storage;
mod tray;
mod utils;

use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use ai::embeddings::EmbeddingEngine;
use ai::semantic_search::SemanticSearchEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus existing window when second instance is launched
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            // Clipboard commands
            commands::clipboard::get_recent_items,
            commands::clipboard::get_item,
            commands::clipboard::delete_item,
            commands::clipboard::toggle_pin,
            commands::clipboard::toggle_star,
            commands::clipboard::paste_item,
            commands::clipboard::clear_history,
            commands::clipboard::clear_expired_credentials,
            commands::clipboard::get_paste_history,
            // Search commands
            commands::search::search_items,
            commands::search::semantic_search,
            // Settings commands
            commands::settings::get_all_settings,
            commands::settings::update_setting,
            commands::settings::get_ignored_apps,
            commands::settings::add_ignored_app,
            commands::settings::remove_ignored_app,
            // Prediction commands
            commands::prediction::get_predictions,
            commands::prediction::get_workflow_chains,
            // Auto-paste commands
            commands::prediction::try_auto_paste_cmd,
            commands::prediction::get_paste_rules,
            commands::prediction::create_paste_rule,
            commands::prediction::update_paste_rule,
            commands::prediction::delete_paste_rule,
            commands::prediction::toggle_paste_rule,
            commands::prediction::get_auto_paste_history,
            commands::prediction::rate_auto_paste,
            // Learned patterns commands
            commands::prediction::get_learned_patterns,
            commands::prediction::promote_pattern_to_rule,
            commands::prediction::delete_learned_pattern,
            // AI commands
            commands::ai::configure_ai_provider,
            commands::ai::test_ai_connection,
            commands::ai::get_ai_status,
            commands::ai::backfill_embeddings,
        ])
        .setup(|app| {
            // Initialize database
            let db = storage::database::init_db()
                .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

            // Store DB pool in app state for Tauri commands
            app.manage(db.clone());

            // Initialize AI engines (Phase 3)
            let embedding_engine = Arc::new(Mutex::new(EmbeddingEngine::new()));
            let semantic_engine = Arc::new(SemanticSearchEngine::new(embedding_engine.clone()));

            app.manage(embedding_engine);
            app.manage(semantic_engine.clone());

            // Setup system tray
            if let Err(e) = tray::menu::setup_tray(app) {
                log::error!("Failed to setup tray: {}", e);
            }

            // Register global shortcuts
            let handle = app.handle();
            handle
                .global_shortcut()
                .on_shortcut("ctrl+shift+v", |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        // ALWAYS show window + overlay immediately (never block)
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        if let Err(e) = app.emit("shortcut:quick-paste", ()) {
                            log::error!("Failed to emit quick-paste shortcut event: {}", e);
                        }
                    }
                })
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to register Ctrl+Shift+V shortcut: {}", e),
                    ))
                })?;

            handle
                .global_shortcut()
                .on_shortcut("ctrl+shift+h", |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        if let Err(e) = app.emit("shortcut:history", ()) {
                            log::error!("Failed to emit history shortcut event: {}", e);
                        }
                    }
                })
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to register Ctrl+Shift+H shortcut: {}", e),
                    ))
                })?;

            log::info!("Global shortcuts registered");

            // Start clipboard monitoring
            let app_handle = app.handle().clone();
            clipboard::monitor::start_monitoring(app_handle, db.clone(), semantic_engine);

            // Spawn credential auto-expiry background timer
            {
                let expiry_db = db.clone();
                std::thread::spawn(move || {
                    log::info!("Credential auto-expiry timer started");
                    loop {
                        std::thread::sleep(std::time::Duration::from_secs(60));
                        match storage::queries::clear_expired_credentials(&expiry_db) {
                            Ok(count) => {
                                if count > 0 {
                                    log::info!(
                                        "Auto-expired {} credential(s)",
                                        count
                                    );
                                }
                            }
                            Err(e) => {
                                log::warn!("Credential auto-expiry failed: {}", e);
                            }
                        }
                    }
                });
            }

            log::info!("ContextPaste started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ContextPaste");
}
