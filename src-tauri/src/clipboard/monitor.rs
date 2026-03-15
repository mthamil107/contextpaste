// ContextPaste — Clipboard Monitor
// Watches for clipboard changes using arboard, classifies content,
// detects credentials, stores in DB, and emits events to the frontend.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use arboard::Clipboard;
use tauri::{AppHandle, Emitter};

use crate::ai::semantic_search::SemanticSearchEngine;
use crate::clipboard::classifier;
use crate::clipboard::credential_detector;
use crate::prediction::workflow;
use crate::storage::database::DbPool;
use crate::storage::models::ClipItem;
use crate::storage::queries;

/// Start the clipboard monitoring loop in a background thread.
pub fn start_monitoring(
    app_handle: AppHandle,
    db: DbPool,
    semantic_engine: Arc<SemanticSearchEngine>,
) {
    std::thread::spawn(move || {
        let clipboard = Arc::new(Mutex::new(
            Clipboard::new().expect("Failed to access clipboard"),
        ));
        let mut last_hash = String::new();

        log::info!("Clipboard monitor started");

        loop {
            std::thread::sleep(Duration::from_millis(500));

            let text = {
                let mut cb = match clipboard.lock() {
                    Ok(cb) => cb,
                    Err(_) => continue,
                };
                match cb.get_text() {
                    Ok(t) => t,
                    Err(_) => continue,
                }
            };

            if text.is_empty() {
                continue;
            }

            let hash = queries::compute_hash(&text);
            if hash == last_hash {
                continue;
            }
            last_hash = hash.clone();

            // Check dedup
            if let Ok(Some(_)) = queries::find_by_hash(&db, &hash) {
                log::debug!("Duplicate content detected, skipping");
                continue;
            }

            // Classify content (sync, fast)
            let content_type = classifier::classify(&text);

            // Detect credentials (sync, fast)
            let cred_match = credential_detector::detect(&text);
            let is_credential = cred_match.is_some();
            let credential_type = cred_match.as_ref().map(|m| m.credential_type.clone());

            // Credentials persist like normal items — no auto-expiry
            // They are shown masked in the UI but kept in history for re-use
            let expires_at: Option<String> = None;

            let item = ClipItem {
                id: uuid::Uuid::new_v4().to_string(),
                content: text.clone(),
                content_type: content_type.clone(),
                content_hash: hash,
                content_length: text.len() as i64,
                is_credential,
                credential_type: credential_type.clone(),
                source_app: None, // TODO: active window detection
                source_window_title: None,
                is_pinned: false,
                is_starred: false,
                expires_at,
                created_at: chrono::Utc::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                last_pasted_at: None,
                paste_count: 0,
                tags: None,
            };

            // Store in DB (async-like — we're in a bg thread)
            match queries::insert_clip_item(&db, &item) {
                Ok(_) => {
                    log::debug!(
                        "Captured: {} ({:?})",
                        &item.content[..item.content.len().min(50)],
                        item.content_type
                    );
                }
                Err(e) => {
                    log::error!("Failed to store clip item: {}", e);
                    continue;
                }
            }

            // Emit event to frontend
            if let Err(e) = app_handle.emit("clipboard:new-item", &item) {
                log::error!("Failed to emit clipboard event: {}", e);
            }

            // Emit credential warning
            if is_credential {
                let payload = serde_json::json!({
                    "itemId": item.id,
                    "credType": credential_type.unwrap_or_default(),
                });
                if let Err(e) = app_handle.emit("security:credential-detected", payload) {
                    log::error!("Failed to emit credential event: {}", e);
                }
            }

            // Track workflow chain detection (Phase 2)
            if let Some(chain_pattern) = workflow::process_copy_event(
                &db,
                item.content_type.as_str(),
                item.source_app.as_deref(),
            ) {
                let payload = serde_json::json!({
                    "pattern": chain_pattern,
                    "length": chain_pattern.len(),
                });
                if let Err(e) = app_handle.emit("workflow:chain-detected", payload) {
                    log::error!("Failed to emit workflow chain event: {}", e);
                }
            }

            // Phase 3: Generate embedding asynchronously
            if !item.is_credential {
                let embed_db = db.clone();
                let embed_semantic = semantic_engine.clone();
                let item_id = item.id.clone();
                let content = item.content.clone();
                std::thread::spawn(move || {
                    if let Ok(conn) = embed_db.lock() {
                        if let Err(e) =
                            embed_semantic.index_item(&conn, &item_id, &content, false)
                        {
                            log::debug!("Embedding skipped for {}: {}", item_id, e);
                        }
                    }
                });
            }

            // Enforce history limit
            if let Err(e) = queries::enforce_history_limit(&db, 5000) {
                log::warn!("Failed to enforce history limit: {}", e);
            }
        }
    });
}
