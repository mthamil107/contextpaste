// ContextPaste — Settings IPC Commands

use std::collections::HashMap;

use tauri::State;

use crate::storage::database::DbPool;
use crate::storage::queries;

#[tauri::command]
pub fn get_all_settings(db: State<'_, DbPool>) -> Result<HashMap<String, String>, String> {
    queries::get_all_settings(&db)
}

#[tauri::command]
pub fn update_setting(db: State<'_, DbPool>, key: String, value: String) -> Result<(), String> {
    queries::update_setting(&db, &key, &value)
}

#[tauri::command]
pub fn get_ignored_apps(db: State<'_, DbPool>) -> Result<Vec<String>, String> {
    queries::get_ignored_apps(&db)
}

#[tauri::command]
pub fn add_ignored_app(db: State<'_, DbPool>, app_name: String) -> Result<(), String> {
    let mut apps = queries::get_ignored_apps(&db)?;
    if !apps.contains(&app_name) {
        apps.push(app_name);
    }
    let json = serde_json::to_string(&apps)
        .map_err(|e| format!("Failed to serialize ignored apps: {}", e))?;
    queries::update_setting(&db, "ignored_apps", &json)
}

#[tauri::command]
pub fn remove_ignored_app(db: State<'_, DbPool>, app_name: String) -> Result<(), String> {
    let mut apps = queries::get_ignored_apps(&db)?;
    apps.retain(|a| a != &app_name);
    let json = serde_json::to_string(&apps)
        .map_err(|e| format!("Failed to serialize ignored apps: {}", e))?;
    queries::update_setting(&db, "ignored_apps", &json)
}
