// ContextPaste — AI IPC Commands (Phase 3)

use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::State;

use crate::ai::embeddings::EmbeddingEngine;
use crate::ai::semantic_search::SemanticSearchEngine;
use crate::storage::database::DbPool;
use crate::storage::queries;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiStatus {
    pub ready: bool,
    pub provider: String,
    pub model_name: String,
    pub embedded_count: i64,
    pub total_items: i64,
    pub dimension: usize,
}

#[tauri::command]
pub fn configure_ai_provider(
    db: State<'_, DbPool>,
    engine: State<'_, Arc<Mutex<EmbeddingEngine>>>,
    provider: String,
    api_key: Option<String>,
    base_url: Option<String>,
    model_name: Option<String>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    // Store API key if provided
    if let Some(ref key) = api_key {
        queries::store_api_key(&conn, &provider, key, base_url.as_deref(), model_name.as_deref())?;
    }

    let mut eng = engine.lock().map_err(|e| e.to_string())?;

    match provider.as_str() {
        "local" => {
            // Local model will be initialized when semantic search is used
            eng.set_provider_info("all-MiniLM-L6-v2", 384);
        }
        "openai" => {
            let _key = api_key.ok_or("API key required for OpenAI")?;
            eng.set_api_ready(
                model_name.as_deref().unwrap_or("text-embedding-3-small"),
                1536,
            );
        }
        "ollama" => {
            eng.set_api_ready(
                model_name.as_deref().unwrap_or("nomic-embed-text"),
                768,
            );
        }
        "anthropic" => {
            return Err(
                "Anthropic does not offer an embedding API. Use Local, OpenAI, or Ollama instead."
                    .to_string(),
            );
        }
        _ => return Err(format!("Unknown provider: {}", provider)),
    }

    // Clear old embeddings since dimension may have changed
    queries::clear_all_embeddings(&conn)?;

    Ok(())
}

#[tauri::command]
pub fn test_ai_connection(
    _db: State<'_, DbPool>,
    engine: State<'_, Arc<Mutex<EmbeddingEngine>>>,
) -> Result<String, String> {
    let eng = engine.lock().map_err(|e| e.to_string())?;
    if eng.is_ready() {
        Ok("AI engine is ready".to_string())
    } else {
        Err("AI engine not initialized".to_string())
    }
}

#[tauri::command]
pub fn get_ai_status(
    db: State<'_, DbPool>,
    engine: State<'_, Arc<Mutex<EmbeddingEngine>>>,
) -> Result<AiStatus, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let eng = engine.lock().map_err(|e| e.to_string())?;

    let embedded_count = queries::get_embedding_count(&conn)?;
    let total_items: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM clip_items WHERE is_credential = 0",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count items: {}", e))?;

    Ok(AiStatus {
        ready: eng.is_ready(),
        provider: if eng.is_ready() {
            "active".to_string()
        } else {
            "none".to_string()
        },
        model_name: eng.model_name().to_string(),
        embedded_count,
        total_items,
        dimension: eng.dimension(),
    })
}

#[tauri::command]
pub fn backfill_embeddings(
    db: State<'_, DbPool>,
    semantic: State<'_, Arc<SemanticSearchEngine>>,
) -> Result<u32, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    semantic.backfill(&conn, 500)
}
