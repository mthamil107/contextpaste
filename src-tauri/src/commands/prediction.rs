// ContextPaste — Prediction IPC Commands

use tauri::State;

use crate::prediction::engine;
use crate::prediction::workflow;
use crate::storage::database::DbPool;
use crate::storage::models::{RankedItem, WorkflowChain};

#[tauri::command]
pub fn get_predictions(
    db: State<'_, DbPool>,
    limit: u32,
    target_app: Option<String>,
) -> Result<Vec<RankedItem>, String> {
    engine::get_predictions(&db, limit, target_app.as_deref())
}

#[tauri::command]
pub fn get_workflow_chains(
    db: State<'_, DbPool>,
    limit: u32,
) -> Result<Vec<WorkflowChain>, String> {
    workflow::get_active_chains(&db, limit)
}
