// ContextPaste — Search IPC Commands

use tauri::State;

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

/// Semantic search — Phase 3 stub.
#[tauri::command]
pub fn semantic_search(
    _db: State<'_, DbPool>,
    _query: String,
    _limit: u32,
) -> Result<Vec<ClipItem>, String> {
    // Phase 3: Implement ONNX embedding + sqlite-vec similarity search
    Ok(Vec::new())
}
