// ContextPaste — Search IPC Commands

use std::sync::Arc;

use tauri::State;

use crate::ai::semantic_search::SemanticSearchEngine;
use crate::storage::database::DbPool;
use crate::storage::models::ClipItem;
use crate::storage::queries;

#[tauri::command]
pub fn search_items(
    db: State<'_, DbPool>,
    query: String,
    limit: u32,
) -> Result<Vec<ClipItem>, String> {
    if query.trim().is_empty() {
        return queries::get_recent_items(&db, limit, 0);
    }
    queries::search_items(&db, &query, limit)
}

#[tauri::command]
pub fn semantic_search(
    db: State<'_, DbPool>,
    semantic: State<'_, Arc<SemanticSearchEngine>>,
    query: String,
    limit: u32,
) -> Result<Vec<ClipItem>, String> {
    if query.trim().is_empty() {
        return queries::get_recent_items(&db, limit, 0);
    }

    if !semantic.is_ready() {
        // Fall back to FTS search if AI not ready
        return queries::search_items(&db, &query, limit);
    }

    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    semantic.search(&conn, &query, limit)
}
