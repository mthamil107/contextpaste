// ContextPaste — Prediction IPC Commands

use tauri::State;

use crate::prediction::engine;
use crate::prediction::workflow;
use crate::prediction::auto_paste;
use crate::storage::database::DbPool;
use crate::storage::models::{AutoPasteEvent, AutoPasteResult, PasteRule, RankedItem, WorkflowChain};
use crate::storage::queries;

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

// Auto-paste commands

#[tauri::command]
pub fn try_auto_paste_cmd(
    db: State<'_, DbPool>,
    threshold: f64,
) -> Result<AutoPasteResult, String> {
    auto_paste::try_auto_paste(&db, threshold)
}

#[tauri::command]
pub fn get_paste_rules(
    db: State<'_, DbPool>,
) -> Result<Vec<PasteRule>, String> {
    queries::get_all_paste_rules(&db)
}

#[tauri::command]
pub fn create_paste_rule(
    db: State<'_, DbPool>,
    rule: PasteRule,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let new_rule = PasteRule {
        id: id.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        times_triggered: 0,
        last_triggered_at: None,
        ..rule
    };
    queries::create_paste_rule(&db, &new_rule)?;
    Ok(id)
}

#[tauri::command]
pub fn update_paste_rule(
    db: State<'_, DbPool>,
    rule: PasteRule,
) -> Result<(), String> {
    queries::update_paste_rule(&db, &rule)
}

#[tauri::command]
pub fn delete_paste_rule(
    db: State<'_, DbPool>,
    id: String,
) -> Result<(), String> {
    queries::delete_paste_rule(&db, &id)
}

#[tauri::command]
pub fn toggle_paste_rule(
    db: State<'_, DbPool>,
    id: String,
) -> Result<(), String> {
    queries::toggle_paste_rule(&db, &id)
}

#[tauri::command]
pub fn get_auto_paste_history(
    db: State<'_, DbPool>,
    limit: u32,
) -> Result<Vec<AutoPasteEvent>, String> {
    queries::get_auto_paste_history(&db, limit)
}

#[tauri::command]
pub fn rate_auto_paste(
    db: State<'_, DbPool>,
    event_id: String,
    correct: bool,
) -> Result<(), String> {
    queries::rate_auto_paste(&db, &event_id, correct)
}
