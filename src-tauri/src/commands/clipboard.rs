// ContextPaste — Clipboard IPC Commands

use tauri::State;

use crate::prediction::context;
use crate::storage::database::DbPool;
use crate::storage::models::{ClipItem, PasteEvent};
use crate::storage::queries;

#[tauri::command]
pub fn get_recent_items(
    db: State<'_, DbPool>,
    limit: u32,
    offset: u32,
) -> Result<Vec<ClipItem>, String> {
    queries::get_recent_items(&db, limit, offset)
}

#[tauri::command]
pub fn get_item(db: State<'_, DbPool>, id: String) -> Result<ClipItem, String> {
    queries::get_item(&db, &id)
}

#[tauri::command]
pub fn delete_item(db: State<'_, DbPool>, id: String) -> Result<(), String> {
    queries::delete_item(&db, &id)
}

#[tauri::command]
pub fn toggle_pin(db: State<'_, DbPool>, id: String) -> Result<(), String> {
    queries::toggle_pin(&db, &id)
}

#[tauri::command]
pub fn toggle_star(db: State<'_, DbPool>, id: String) -> Result<(), String> {
    queries::toggle_star(&db, &id)
}

#[tauri::command]
pub fn paste_item(db: State<'_, DbPool>, id: String) -> Result<(), String> {
    // Get the item content
    let item = queries::get_item(&db, &id)?;

    // Capture target window context before pasting
    let window_ctx = context::get_active_window();

    // Write to system clipboard
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Clipboard error: {}", e))?;
    clipboard
        .set_text(&item.content)
        .map_err(|e| format!("Failed to set clipboard: {}", e))?;

    // Record the paste event with target context
    queries::record_paste(
        &db,
        &id,
        window_ctx.app_name.as_deref(),
        window_ctx.window_title.as_deref(),
    )?;

    // Update prediction stats for this content_type → target_app pair
    if let Some(ref target_app) = window_ctx.app_name {
        queries::update_prediction_stat(
            &db,
            item.content_type.as_str(),
            item.source_app.as_deref(),
            target_app,
        )?;
    }

    Ok(())
}

#[tauri::command]
pub fn clear_history(db: State<'_, DbPool>) -> Result<(), String> {
    queries::clear_history(&db)
}

#[tauri::command]
pub fn clear_expired_credentials(db: State<'_, DbPool>) -> Result<(), String> {
    queries::clear_expired_credentials(&db)?;
    Ok(())
}

#[tauri::command]
pub fn get_paste_history(
    db: State<'_, DbPool>,
    item_id: String,
    limit: u32,
) -> Result<Vec<PasteEvent>, String> {
    queries::get_paste_history(&db, &item_id, limit)
}
