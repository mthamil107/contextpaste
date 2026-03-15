// ContextPaste — SQL Query Functions

use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

use super::database::DbPool;
use super::models::{AutoPasteEvent, ClipItem, ContentType, PasteEvent, PasteRule, WorkflowChain};

/// Insert a new clipboard item. Returns the item ID.
pub fn insert_clip_item(db: &DbPool, item: &ClipItem) -> Result<String, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    let tags_json = item
        .tags
        .as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default());

    conn.execute(
        "INSERT INTO clip_items (id, content, content_type, content_hash, content_length,
         is_credential, credential_type, source_app, source_window_title,
         is_pinned, is_starred, expires_at, created_at, last_pasted_at, paste_count, tags)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![
            item.id,
            item.content,
            item.content_type.as_str(),
            item.content_hash,
            item.content_length,
            item.is_credential,
            item.credential_type,
            item.source_app,
            item.source_window_title,
            item.is_pinned,
            item.is_starred,
            item.expires_at,
            item.created_at,
            item.last_pasted_at,
            item.paste_count,
            tags_json,
        ],
    )
    .map_err(|e| format!("Failed to insert clip item: {}", e))?;

    Ok(item.id.clone())
}

/// Get recent clipboard items, ordered by created_at DESC.
pub fn get_recent_items(db: &DbPool, limit: u32, offset: u32) -> Result<Vec<ClipItem>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, content, content_type, content_hash, content_length,
             is_credential, credential_type, source_app, source_window_title,
             is_pinned, is_starred, expires_at, created_at, last_pasted_at, paste_count, tags
             FROM clip_items
             ORDER BY is_pinned DESC, created_at DESC
             LIMIT ?1 OFFSET ?2",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let items = stmt
        .query_map(params![limit, offset], row_to_clip_item)
        .map_err(|e| format!("Failed to query recent items: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

/// Get a single item by ID.
pub fn get_item(db: &DbPool, id: &str) -> Result<ClipItem, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    conn.query_row(
        "SELECT id, content, content_type, content_hash, content_length,
         is_credential, credential_type, source_app, source_window_title,
         is_pinned, is_starred, expires_at, created_at, last_pasted_at, paste_count, tags
         FROM clip_items WHERE id = ?1",
        params![id],
        row_to_clip_item,
    )
    .map_err(|e| format!("Item not found: {}", e))
}

/// Full-text search over clip items.
pub fn search_items(db: &DbPool, query: &str, limit: u32) -> Result<Vec<ClipItem>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    // Use FTS5 match syntax
    let fts_query = format!("\"{}\"", query.replace('"', "\"\""));

    let mut stmt = conn
        .prepare(
            "SELECT c.id, c.content, c.content_type, c.content_hash, c.content_length,
             c.is_credential, c.credential_type, c.source_app, c.source_window_title,
             c.is_pinned, c.is_starred, c.expires_at, c.created_at, c.last_pasted_at,
             c.paste_count, c.tags
             FROM clip_items c
             JOIN clip_items_fts f ON c.rowid = f.rowid
             WHERE clip_items_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )
        .map_err(|e| format!("Failed to prepare search: {}", e))?;

    let items = stmt
        .query_map(params![fts_query, limit], row_to_clip_item)
        .map_err(|e| format!("Search failed: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

/// Delete a clip item by ID.
pub fn delete_item(db: &DbPool, id: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute("DELETE FROM clip_items WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete item: {}", e))?;
    Ok(())
}

/// Toggle pin status on an item.
pub fn toggle_pin(db: &DbPool, id: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "UPDATE clip_items SET is_pinned = NOT is_pinned WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("Failed to toggle pin: {}", e))?;
    Ok(())
}

/// Toggle star status on an item.
pub fn toggle_star(db: &DbPool, id: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "UPDATE clip_items SET is_starred = NOT is_starred WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("Failed to toggle star: {}", e))?;
    Ok(())
}

/// Record a paste event and update item stats.
pub fn record_paste(
    db: &DbPool,
    item_id: &str,
    target_app: Option<&str>,
    target_window_title: Option<&str>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    let paste_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO paste_events (id, item_id, target_app, target_window_title)
         VALUES (?1, ?2, ?3, ?4)",
        params![paste_id, item_id, target_app, target_window_title],
    )
    .map_err(|e| format!("Failed to record paste: {}", e))?;

    conn.execute(
        "UPDATE clip_items SET paste_count = paste_count + 1,
         last_pasted_at = datetime('now') WHERE id = ?1",
        params![item_id],
    )
    .map_err(|e| format!("Failed to update paste count: {}", e))?;

    Ok(())
}

/// Clear all non-pinned history.
pub fn clear_history(db: &DbPool) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute("DELETE FROM clip_items WHERE is_pinned = 0", [])
        .map_err(|e| format!("Failed to clear history: {}", e))?;
    Ok(())
}

/// Delete expired credential items.
pub fn clear_expired_credentials(db: &DbPool) -> Result<u64, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let deleted = conn
        .execute(
            "DELETE FROM clip_items WHERE is_credential = 1
             AND expires_at IS NOT NULL AND expires_at < datetime('now')",
            [],
        )
        .map_err(|e| format!("Failed to clear expired credentials: {}", e))?;
    Ok(deleted as u64)
}

/// Check if a content hash already exists (for dedup).
pub fn find_by_hash(db: &DbPool, hash: &str) -> Result<Option<String>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let result = conn.query_row(
        "SELECT id FROM clip_items WHERE content_hash = ?1 ORDER BY created_at DESC LIMIT 1",
        params![hash],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Hash lookup failed: {}", e)),
    }
}

/// Compute SHA256 hash of content for dedup.
pub fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get paste history for a specific item, ordered by most recent first.
pub fn get_paste_history(db: &DbPool, item_id: &str, limit: u32) -> Result<Vec<PasteEvent>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, item_id, target_app, target_window_title, pasted_at, session_id
             FROM paste_events
             WHERE item_id = ?1
             ORDER BY pasted_at DESC
             LIMIT ?2",
        )
        .map_err(|e| format!("Failed to prepare paste history query: {}", e))?;

    let events = stmt
        .query_map(params![item_id, limit], |row| {
            Ok(PasteEvent {
                id: row.get(0)?,
                item_id: row.get(1)?,
                target_app: row.get(2)?,
                target_window_title: row.get(3)?,
                pasted_at: row.get(4)?,
                session_id: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query paste history: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(events)
}

// --- Settings ---

/// Get all settings as key-value pairs.
pub fn get_all_settings(db: &DbPool) -> Result<std::collections::HashMap<String, String>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_settings")
        .map_err(|e| format!("Failed to prepare settings query: {}", e))?;

    let map = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| format!("Failed to query settings: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(map)
}

/// Upsert a setting.
pub fn update_setting(db: &DbPool, key: &str, value: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
        params![key, value],
    )
    .map_err(|e| format!("Failed to update setting: {}", e))?;
    Ok(())
}

/// Get ignored apps list from settings.
pub fn get_ignored_apps(db: &DbPool) -> Result<Vec<String>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let result = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'ignored_apps'",
        [],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(json) => {
            serde_json::from_str(&json).map_err(|e| format!("Failed to parse ignored apps: {}", e))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Vec::new()),
        Err(e) => Err(format!("Failed to get ignored apps: {}", e)),
    }
}

/// Update prediction stats for a content_type → target_app pair.
pub fn update_prediction_stat(
    db: &DbPool,
    content_type: &str,
    source_app: Option<&str>,
    target_app: &str,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    // Try to increment existing
    let updated = conn
        .execute(
            "UPDATE prediction_stats SET frequency = frequency + 1, last_used_at = datetime('now')
             WHERE content_type = ?1 AND target_app = ?2
             AND (source_app = ?3 OR (source_app IS NULL AND ?3 IS NULL))",
            params![content_type, target_app, source_app],
        )
        .map_err(|e| format!("Failed to update prediction stat: {}", e))?;

    if updated == 0 {
        conn.execute(
            "INSERT INTO prediction_stats (content_type, source_app, target_app)
             VALUES (?1, ?2, ?3)",
            params![content_type, source_app, target_app],
        )
        .map_err(|e| format!("Failed to insert prediction stat: {}", e))?;
    }

    Ok(())
}

/// Get prediction stats for a target app.
pub fn get_prediction_stats(
    db: &DbPool,
    target_app: &str,
) -> Result<Vec<(String, i64)>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT content_type, SUM(frequency) as total
             FROM prediction_stats WHERE target_app = ?1
             GROUP BY content_type ORDER BY total DESC",
        )
        .map_err(|e| format!("Failed to prepare prediction query: {}", e))?;

    let stats = stmt
        .query_map(params![target_app], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| format!("Failed to query predictions: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(stats)
}

/// Get normalized type match scores for a target app.
/// Returns a map of content_type -> score (0.0 to 100.0) based on how frequently
/// each content type is pasted into the given target app.
pub fn get_type_match_scores(
    db: &DbPool,
    target_app: &str,
) -> Result<std::collections::HashMap<String, f64>, String> {
    let stats = get_prediction_stats(db, target_app)?;

    if stats.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    // Find the max frequency for normalization
    let max_freq = stats.iter().map(|(_, f)| *f).max().unwrap_or(1) as f64;

    let scores = stats
        .into_iter()
        .map(|(content_type, freq)| {
            let normalized = (freq as f64 / max_freq) * 100.0;
            (content_type, normalized)
        })
        .collect();

    Ok(scores)
}

/// Get source affinity score (0.0 to 100.0) based on how often items
/// from source_app are pasted into target_app.
pub fn get_source_affinity(
    db: &DbPool,
    source_app: &str,
    target_app: &str,
) -> Result<f64, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    // Total pastes into target_app
    let total: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(frequency), 0) FROM prediction_stats WHERE target_app = ?1",
            params![target_app],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if total == 0 {
        return Ok(0.0);
    }

    // Pastes from source_app into target_app
    let from_source: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(frequency), 0) FROM prediction_stats
             WHERE target_app = ?1 AND source_app = ?2",
            params![target_app, source_app],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Normalize to 0-100 scale
    Ok((from_source as f64 / total as f64) * 100.0)
}

/// Enforce max history limit by deleting oldest non-pinned items.
pub fn enforce_history_limit(db: &DbPool, max_items: u32) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "DELETE FROM clip_items WHERE id IN (
            SELECT id FROM clip_items WHERE is_pinned = 0
            ORDER BY created_at DESC LIMIT -1 OFFSET ?1
        )",
        params![max_items],
    )
    .map_err(|e| format!("Failed to enforce history limit: {}", e))?;
    Ok(())
}

// --- Workflow Chain Queries ---

/// Upsert a workflow chain: insert if new, increment frequency if exists.
pub fn upsert_workflow_chain(
    db: &DbPool,
    chain_hash: &str,
    items_json: &str,
    source_context: Option<&str>,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    // Try to increment existing chain
    let updated = conn
        .execute(
            "UPDATE workflow_chains SET frequency = frequency + 1,
             last_triggered_at = datetime('now')
             WHERE chain_hash = ?1",
            params![chain_hash],
        )
        .map_err(|e| format!("Failed to update workflow chain: {}", e))?;

    if updated == 0 {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO workflow_chains (id, chain_hash, items_json, frequency, last_triggered_at, source_context)
             VALUES (?1, ?2, ?3, 1, datetime('now'), ?4)",
            params![id, chain_hash, items_json, source_context],
        )
        .map_err(|e| format!("Failed to insert workflow chain: {}", e))?;
    }

    Ok(())
}

/// Get the most frequent workflow chains, ordered by frequency DESC.
pub fn get_top_chains(db: &DbPool, limit: u32) -> Result<Vec<WorkflowChain>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    let mut stmt = conn
        .prepare(
            "SELECT id, chain_hash, items_json, frequency, last_triggered_at, source_context
             FROM workflow_chains
             ORDER BY frequency DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("Failed to prepare chain query: {}", e))?;

    let chains = stmt
        .query_map(params![limit], |row| {
            Ok(WorkflowChain {
                id: row.get(0)?,
                chain_hash: row.get(1)?,
                items_json: row.get(2)?,
                frequency: row.get(3)?,
                last_triggered_at: row.get(4)?,
                source_context: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to query chains: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(chains)
}

/// Look up a workflow chain by its hash.
pub fn get_chain_by_hash(db: &DbPool, hash: &str) -> Result<Option<WorkflowChain>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    let result = conn.query_row(
        "SELECT id, chain_hash, items_json, frequency, last_triggered_at, source_context
         FROM workflow_chains WHERE chain_hash = ?1",
        params![hash],
        |row| {
            Ok(WorkflowChain {
                id: row.get(0)?,
                chain_hash: row.get(1)?,
                items_json: row.get(2)?,
                frequency: row.get(3)?,
                last_triggered_at: row.get(4)?,
                source_context: row.get(5)?,
            })
        },
    );

    match result {
        Ok(chain) => Ok(Some(chain)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Chain lookup failed: {}", e)),
    }
}

// --- AI Embedding Queries (Phase 3) ---

/// Store an embedding for a clip item.
pub fn store_embedding(
    conn: &Connection,
    item_id: &str,
    embedding: &[f32],
    model_name: &str,
    dimension: usize,
) -> Result<(), String> {
    let blob: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();
    conn.execute(
        "INSERT OR REPLACE INTO clip_embeddings (item_id, embedding, model_name, dimension) VALUES (?1, ?2, ?3, ?4)",
        params![item_id, blob, model_name, dimension as i64],
    )
    .map_err(|e| format!("Failed to store embedding: {}", e))?;
    Ok(())
}

/// Delete embedding for an item.
pub fn delete_embedding(conn: &Connection, item_id: &str) -> Result<(), String> {
    conn.execute(
        "DELETE FROM clip_embeddings WHERE item_id = ?1",
        params![item_id],
    )
    .map_err(|e| format!("Failed to delete embedding: {}", e))?;
    Ok(())
}

/// Get all embeddings for cosine similarity search.
/// Returns (item_id, embedding_blob) pairs.
pub fn get_all_embeddings(conn: &Connection) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut stmt = conn
        .prepare("SELECT item_id, embedding FROM clip_embeddings")
        .map_err(|e| format!("Failed to prepare embeddings query: {}", e))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(|e| format!("Failed to query embeddings: {}", e))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| format!("Failed to read embedding row: {}", e))?);
    }
    Ok(results)
}

/// Get count of embedded items.
pub fn get_embedding_count(conn: &Connection) -> Result<i64, String> {
    conn.query_row("SELECT COUNT(*) FROM clip_embeddings", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count embeddings: {}", e))
}

/// Get items that don't have embeddings yet (for backfill).
/// Excludes credentials — they must NEVER be embedded.
pub fn get_unembedded_items(conn: &Connection, limit: u32) -> Result<Vec<ClipItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.content, i.content_type, i.content_hash, i.content_length,
             i.is_credential, i.credential_type, i.source_app, i.source_window_title,
             i.is_pinned, i.is_starred, i.expires_at, i.created_at, i.last_pasted_at,
             i.paste_count, i.tags
             FROM clip_items i
             LEFT JOIN clip_embeddings e ON i.id = e.item_id
             WHERE e.item_id IS NULL AND i.is_credential = 0
             ORDER BY i.created_at DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("Failed to prepare unembedded items query: {}", e))?;

    let items = stmt
        .query_map(params![limit], row_to_clip_item)
        .map_err(|e| format!("Failed to query unembedded items: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(items)
}

/// Get a single item by ID using a raw connection (for use in AI module).
pub fn get_item_by_conn(conn: &Connection, id: &str) -> Result<ClipItem, String> {
    conn.query_row(
        "SELECT id, content, content_type, content_hash, content_length,
         is_credential, credential_type, source_app, source_window_title,
         is_pinned, is_starred, expires_at, created_at, last_pasted_at, paste_count, tags
         FROM clip_items WHERE id = ?1",
        params![id],
        row_to_clip_item,
    )
    .map_err(|e| format!("Item not found: {}", e))
}

/// Store/update AI API key.
pub fn store_api_key(
    conn: &Connection,
    provider: &str,
    api_key: &str,
    base_url: Option<&str>,
    model_name: Option<&str>,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO ai_api_keys (provider, api_key_encrypted, base_url, model_name, updated_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now'))",
        params![provider, api_key, base_url, model_name],
    )
    .map_err(|e| format!("Failed to store API key: {}", e))?;
    Ok(())
}

/// Get AI API key for a provider.
/// Returns (api_key, base_url, model_name) if found.
pub fn get_api_key(
    conn: &Connection,
    provider: &str,
) -> Result<Option<(String, Option<String>, Option<String>)>, String> {
    let result = conn.query_row(
        "SELECT api_key_encrypted, base_url, model_name FROM ai_api_keys WHERE provider = ?1",
        params![provider],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        },
    );
    match result {
        Ok(r) => Ok(Some(r)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get API key: {}", e)),
    }
}

/// Clear all embeddings (for re-indexing after provider switch).
pub fn clear_all_embeddings(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM clip_embeddings", [])
        .map_err(|e| format!("Failed to clear embeddings: {}", e))?;
    Ok(())
}

// --- Paste Rules CRUD ---

/// Get all enabled paste rules, ordered by priority DESC.
pub fn get_enabled_paste_rules(db: &DbPool) -> Result<Vec<PasteRule>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, priority, enabled, app_pattern, window_title_pattern,
             context_pattern, content_type_filter, action_type, action_value,
             times_triggered, last_triggered_at, created_at, updated_at
             FROM paste_rules WHERE enabled = 1 ORDER BY priority DESC",
        )
        .map_err(|e| format!("Failed to prepare paste rules query: {}", e))?;

    let rules = stmt
        .query_map([], row_to_paste_rule)
        .map_err(|e| format!("Failed to query paste rules: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rules)
}

/// Get all paste rules (enabled and disabled), ordered by priority DESC.
pub fn get_all_paste_rules(db: &DbPool) -> Result<Vec<PasteRule>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, priority, enabled, app_pattern, window_title_pattern,
             context_pattern, content_type_filter, action_type, action_value,
             times_triggered, last_triggered_at, created_at, updated_at
             FROM paste_rules ORDER BY priority DESC",
        )
        .map_err(|e| format!("Failed to prepare paste rules query: {}", e))?;

    let rules = stmt
        .query_map([], row_to_paste_rule)
        .map_err(|e| format!("Failed to query paste rules: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(rules)
}

/// Create a new paste rule.
pub fn create_paste_rule(db: &DbPool, rule: &PasteRule) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "INSERT INTO paste_rules (id, name, priority, enabled, app_pattern, window_title_pattern,
         context_pattern, content_type_filter, action_type, action_value,
         times_triggered, last_triggered_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            rule.id,
            rule.name,
            rule.priority,
            rule.enabled,
            rule.app_pattern,
            rule.window_title_pattern,
            rule.context_pattern,
            rule.content_type_filter,
            rule.action_type,
            rule.action_value,
            rule.times_triggered,
            rule.last_triggered_at,
            rule.created_at,
            rule.updated_at,
        ],
    )
    .map_err(|e| format!("Failed to create paste rule: {}", e))?;
    Ok(())
}

/// Update an existing paste rule.
pub fn update_paste_rule(db: &DbPool, rule: &PasteRule) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "UPDATE paste_rules SET name = ?2, priority = ?3, enabled = ?4, app_pattern = ?5,
         window_title_pattern = ?6, context_pattern = ?7, content_type_filter = ?8,
         action_type = ?9, action_value = ?10, updated_at = datetime('now')
         WHERE id = ?1",
        params![
            rule.id,
            rule.name,
            rule.priority,
            rule.enabled,
            rule.app_pattern,
            rule.window_title_pattern,
            rule.context_pattern,
            rule.content_type_filter,
            rule.action_type,
            rule.action_value,
        ],
    )
    .map_err(|e| format!("Failed to update paste rule: {}", e))?;
    Ok(())
}

/// Delete a paste rule by ID.
pub fn delete_paste_rule(db: &DbPool, id: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute("DELETE FROM paste_rules WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete paste rule: {}", e))?;
    Ok(())
}

/// Toggle enabled status on a paste rule.
pub fn toggle_paste_rule(db: &DbPool, id: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "UPDATE paste_rules SET enabled = NOT enabled, updated_at = datetime('now') WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("Failed to toggle paste rule: {}", e))?;
    Ok(())
}

/// Increment trigger count and update last_triggered_at for a paste rule.
pub fn increment_rule_trigger(db: &DbPool, id: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "UPDATE paste_rules SET times_triggered = times_triggered + 1,
         last_triggered_at = datetime('now'), updated_at = datetime('now')
         WHERE id = ?1",
        params![id],
    )
    .map_err(|e| format!("Failed to increment rule trigger: {}", e))?;
    Ok(())
}

// --- Auto-paste event tracking ---

/// Record an auto-paste event.
pub fn record_auto_paste_event(db: &DbPool, event: &AutoPasteEvent) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "INSERT INTO auto_paste_events (id, item_id, rule_id, confidence, was_correct,
         screen_context, target_app, target_window_title, pasted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            event.id,
            event.item_id,
            event.rule_id,
            event.confidence,
            event.was_correct,
            event.screen_context,
            event.target_app,
            event.target_window_title,
            event.pasted_at,
        ],
    )
    .map_err(|e| format!("Failed to record auto-paste event: {}", e))?;
    Ok(())
}

/// Get auto-paste event history, ordered by most recent first.
pub fn get_auto_paste_history(db: &DbPool, limit: u32) -> Result<Vec<AutoPasteEvent>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, item_id, rule_id, confidence, was_correct,
             screen_context, target_app, target_window_title, pasted_at
             FROM auto_paste_events ORDER BY pasted_at DESC LIMIT ?1",
        )
        .map_err(|e| format!("Failed to prepare auto-paste history query: {}", e))?;

    let events = stmt
        .query_map(params![limit], |row| {
            Ok(AutoPasteEvent {
                id: row.get(0)?,
                item_id: row.get(1)?,
                rule_id: row.get(2)?,
                confidence: row.get(3)?,
                was_correct: row.get(4)?,
                screen_context: row.get(5)?,
                target_app: row.get(6)?,
                target_window_title: row.get(7)?,
                pasted_at: row.get(8)?,
            })
        })
        .map_err(|e| format!("Failed to query auto-paste history: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(events)
}

/// Rate an auto-paste event as correct or incorrect (user feedback).
pub fn rate_auto_paste(db: &DbPool, event_id: &str, correct: bool) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute(
        "UPDATE auto_paste_events SET was_correct = ?2 WHERE id = ?1",
        params![event_id, correct],
    )
    .map_err(|e| format!("Failed to rate auto-paste event: {}", e))?;
    Ok(())
}

/// Get the most recent clip item of a given content type.
pub fn get_most_recent_by_type(db: &DbPool, content_type: &str) -> Result<Option<ClipItem>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    get_most_recent_by_type_conn(&conn, content_type)
}

/// Get the most recent clip item of a given content type (using raw connection).
pub fn get_most_recent_by_type_conn(conn: &Connection, content_type: &str) -> Result<Option<ClipItem>, String> {
    let result = conn.query_row(
        "SELECT id, content, content_type, content_hash, content_length,
         is_credential, credential_type, source_app, source_window_title,
         is_pinned, is_starred, expires_at, created_at, last_pasted_at, paste_count, tags
         FROM clip_items WHERE content_type = ?1 ORDER BY created_at DESC LIMIT 1",
        params![content_type],
        row_to_clip_item,
    );
    match result {
        Ok(item) => Ok(Some(item)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to get most recent by type: {}", e)),
    }
}

// --- Row mapper ---

fn row_to_clip_item(row: &rusqlite::Row) -> rusqlite::Result<ClipItem> {
    let content_type_str: String = row.get(2)?;
    let tags_json: Option<String> = row.get(15)?;
    let tags: Option<Vec<String>> = tags_json
        .as_ref()
        .and_then(|j| serde_json::from_str(j).ok());

    Ok(ClipItem {
        id: row.get(0)?,
        content: row.get(1)?,
        content_type: ContentType::from_str(&content_type_str),
        content_hash: row.get(3)?,
        content_length: row.get(4)?,
        is_credential: row.get(5)?,
        credential_type: row.get(6)?,
        source_app: row.get(7)?,
        source_window_title: row.get(8)?,
        is_pinned: row.get(9)?,
        is_starred: row.get(10)?,
        expires_at: row.get(11)?,
        created_at: row.get(12)?,
        last_pasted_at: row.get(13)?,
        paste_count: row.get(14)?,
        tags,
    })
}

// ─── Learned Patterns ───

use crate::storage::models::LearnedPattern;

/// Record a learned pattern from a manual paste.
/// If a similar pattern exists (same content_type + target_app), increment frequency.
pub fn record_learned_pattern(
    db: &DbPool,
    content_type: &str,
    target_app: Option<&str>,
    target_window_title: Option<&str>,
    screen_context: Option<&str>,
    item_id: &str,
) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    // Check if a similar pattern already exists (same type + app)
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM learned_patterns WHERE content_type = ?1 AND target_app = ?2 LIMIT 1",
            params![content_type, target_app],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing_id {
        // Update existing: increment frequency, update context and item
        conn.execute(
            "UPDATE learned_patterns SET frequency = frequency + 1, last_used_at = datetime('now'), \
             screen_context = COALESCE(?2, screen_context), item_id = ?3 WHERE id = ?1",
            params![id, screen_context, item_id],
        )
        .map_err(|e| format!("Failed to update learned pattern: {}", e))?;
    } else {
        // Insert new pattern
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO learned_patterns (id, content_type, target_app, target_window_title, screen_context, item_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, content_type, target_app, target_window_title, screen_context, item_id],
        )
        .map_err(|e| format!("Failed to insert learned pattern: {}", e))?;
    }

    Ok(())
}

/// Get all learned patterns, sorted by frequency descending.
pub fn get_learned_patterns(db: &DbPool, limit: u32) -> Result<Vec<LearnedPattern>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, content_type, target_app, target_window_title, screen_context, \
             item_id, frequency, last_used_at, created_at, promoted_to_rule_id \
             FROM learned_patterns WHERE promoted_to_rule_id IS NULL \
             ORDER BY frequency DESC LIMIT ?1",
        )
        .map_err(|e| format!("Failed to prepare: {}", e))?;

    let patterns = stmt
        .query_map(params![limit], |row| {
            Ok(LearnedPattern {
                id: row.get(0)?,
                content_type: row.get(1)?,
                target_app: row.get(2)?,
                target_window_title: row.get(3)?,
                screen_context: row.get(4)?,
                item_id: row.get(5)?,
                frequency: row.get(6)?,
                last_used_at: row.get(7)?,
                created_at: row.get(8)?,
                promoted_to_rule_id: row.get(9)?,
            })
        })
        .map_err(|e| format!("Failed to query: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(patterns)
}

/// Promote a learned pattern to a paste rule.
pub fn promote_pattern_to_rule(db: &DbPool, pattern_id: &str) -> Result<String, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    // Get the pattern
    let pattern = conn.query_row(
        "SELECT content_type, target_app, screen_context FROM learned_patterns WHERE id = ?1",
        params![pattern_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<String>>(2)?)),
    ).map_err(|e| format!("Pattern not found: {}", e))?;

    let (content_type, target_app, screen_context) = pattern;

    // Create a rule from the pattern
    let rule_id = uuid::Uuid::new_v4().to_string();
    let rule_name = format!("Auto: {} in {}", content_type, target_app.as_deref().unwrap_or("any app"));

    // Build context pattern from the screen context (escape regex special chars, use keywords)
    let context_pattern = screen_context
        .as_deref()
        .map(|ctx| {
            // Extract significant words (>3 chars) and join with |
            ctx.split_whitespace()
                .filter(|w| w.len() > 3)
                .map(|w| regex::escape(w))
                .take(5)
                .collect::<Vec<_>>()
                .join("|")
        })
        .filter(|s| !s.is_empty());

    let app_pattern = target_app.as_deref().map(|a| regex::escape(a));

    conn.execute(
        "INSERT INTO paste_rules (id, name, priority, enabled, app_pattern, context_pattern, action_type, action_value) \
         VALUES (?1, ?2, 0, 1, ?3, ?4, 'paste_recent_type', ?5)",
        params![rule_id, rule_name, app_pattern, context_pattern, content_type],
    )
    .map_err(|e| format!("Failed to create rule: {}", e))?;

    // Mark pattern as promoted
    conn.execute(
        "UPDATE learned_patterns SET promoted_to_rule_id = ?2 WHERE id = ?1",
        params![pattern_id, rule_id],
    )
    .map_err(|e| format!("Failed to mark pattern: {}", e))?;

    Ok(rule_id)
}

/// Delete a learned pattern.
pub fn delete_learned_pattern(db: &DbPool, id: &str) -> Result<(), String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    conn.execute("DELETE FROM learned_patterns WHERE id = ?1", params![id])
        .map_err(|e| format!("Failed to delete: {}", e))?;
    Ok(())
}

/// Find a matching learned pattern for auto-paste scoring.
pub fn find_matching_pattern(db: &DbPool, content_type: &str, target_app: Option<&str>) -> Result<Option<LearnedPattern>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let result = conn.query_row(
        "SELECT id, content_type, target_app, target_window_title, screen_context, \
         item_id, frequency, last_used_at, created_at, promoted_to_rule_id \
         FROM learned_patterns \
         WHERE content_type = ?1 AND target_app = ?2 AND promoted_to_rule_id IS NULL \
         ORDER BY frequency DESC LIMIT 1",
        params![content_type, target_app],
        |row| {
            Ok(LearnedPattern {
                id: row.get(0)?,
                content_type: row.get(1)?,
                target_app: row.get(2)?,
                target_window_title: row.get(3)?,
                screen_context: row.get(4)?,
                item_id: row.get(5)?,
                frequency: row.get(6)?,
                last_used_at: row.get(7)?,
                created_at: row.get(8)?,
                promoted_to_rule_id: row.get(9)?,
            })
        },
    );

    match result {
        Ok(p) => Ok(Some(p)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Failed to find pattern: {}", e)),
    }
}

fn row_to_paste_rule(row: &rusqlite::Row) -> rusqlite::Result<PasteRule> {
    Ok(PasteRule {
        id: row.get(0)?,
        name: row.get(1)?,
        priority: row.get(2)?,
        enabled: row.get(3)?,
        app_pattern: row.get(4)?,
        window_title_pattern: row.get(5)?,
        context_pattern: row.get(6)?,
        content_type_filter: row.get(7)?,
        action_type: row.get(8)?,
        action_value: row.get(9)?,
        times_triggered: row.get(10)?,
        last_triggered_at: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::init_test_db;

    fn make_test_item(content: &str) -> ClipItem {
        let hash = compute_hash(content);
        ClipItem {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            content_type: ContentType::PlainText,
            content_hash: hash,
            content_length: content.len() as i64,
            is_credential: false,
            credential_type: None,
            source_app: Some("TestApp".to_string()),
            source_window_title: None,
            is_pinned: false,
            is_starred: false,
            expires_at: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_pasted_at: None,
            paste_count: 0,
            tags: None,
        }
    }

    #[test]
    fn test_insert_and_get() {
        let db = init_test_db().expect("DB init failed");
        let item = make_test_item("Hello, world!");
        let id = insert_clip_item(&db, &item).expect("Insert failed");
        let fetched = get_item(&db, &id).expect("Get failed");
        assert_eq!(fetched.content, "Hello, world!");
    }

    #[test]
    fn test_delete_item() {
        let db = init_test_db().expect("DB init failed");
        let item = make_test_item("delete me");
        let id = insert_clip_item(&db, &item).expect("Insert failed");
        delete_item(&db, &id).expect("Delete failed");
        assert!(get_item(&db, &id).is_err());
    }

    #[test]
    fn test_toggle_pin() {
        let db = init_test_db().expect("DB init failed");
        let item = make_test_item("pin me");
        let id = insert_clip_item(&db, &item).expect("Insert failed");
        toggle_pin(&db, &id).expect("Toggle pin failed");
        let fetched = get_item(&db, &id).expect("Get failed");
        assert!(fetched.is_pinned);
    }

    #[test]
    fn test_settings() {
        let db = init_test_db().expect("DB init failed");
        update_setting(&db, "theme", "dark").expect("Set failed");
        let settings = get_all_settings(&db).expect("Get failed");
        assert_eq!(settings.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_dedup_hash() {
        let db = init_test_db().expect("DB init failed");
        let item = make_test_item("duplicate check");
        insert_clip_item(&db, &item).expect("Insert failed");
        let found = find_by_hash(&db, &item.content_hash).expect("Hash lookup failed");
        assert!(found.is_some());
    }

    #[test]
    fn test_record_paste_increments_count() {
        let db = init_test_db().expect("DB init failed");
        let item = make_test_item("paste me");
        let id = insert_clip_item(&db, &item).expect("Insert failed");

        // Initial paste_count should be 0
        let fetched = get_item(&db, &id).expect("Get failed");
        assert_eq!(fetched.paste_count, 0);

        // Record two paste events
        record_paste(&db, &id, Some("VSCode"), Some("main.rs - contextpaste")).expect("Paste 1 failed");
        record_paste(&db, &id, Some("Chrome"), None).expect("Paste 2 failed");

        // paste_count should be 2, last_pasted_at should be set
        let fetched = get_item(&db, &id).expect("Get failed");
        assert_eq!(fetched.paste_count, 2);
        assert!(fetched.last_pasted_at.is_some());
    }

    #[test]
    fn test_update_prediction_stat_increments_frequency() {
        let db = init_test_db().expect("DB init failed");

        // First call inserts
        update_prediction_stat(&db, "Url", Some("Chrome"), "VSCode").expect("First update failed");
        let stats = get_prediction_stats(&db, "VSCode").expect("Get stats failed");
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0], ("Url".to_string(), 1));

        // Second call increments
        update_prediction_stat(&db, "Url", Some("Chrome"), "VSCode").expect("Second update failed");
        let stats = get_prediction_stats(&db, "VSCode").expect("Get stats failed");
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0], ("Url".to_string(), 2));
    }

    #[test]
    fn test_get_paste_history() {
        let db = init_test_db().expect("DB init failed");
        let item = make_test_item("history item");
        let id = insert_clip_item(&db, &item).expect("Insert failed");

        // Record several paste events
        record_paste(&db, &id, Some("VSCode"), Some("editor")).expect("Paste 1 failed");
        record_paste(&db, &id, Some("Chrome"), Some("browser")).expect("Paste 2 failed");
        record_paste(&db, &id, None, None).expect("Paste 3 failed");

        // Get all paste history
        let events = get_paste_history(&db, &id, 10).expect("Get history failed");
        assert_eq!(events.len(), 3);

        // All events should reference our item
        for event in &events {
            assert_eq!(event.item_id, id);
        }

        // With limit=2, should only get 2
        let limited = get_paste_history(&db, &id, 2).expect("Get limited history failed");
        assert_eq!(limited.len(), 2);

        // No history for a random item
        let empty = get_paste_history(&db, "nonexistent-id", 10).expect("Get empty history failed");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_get_type_match_scores() {
        let db = init_test_db().expect("DB init failed");

        // Seed prediction stats: Url pasted 10 times, Code pasted 5 times into VSCode
        for _ in 0..10 {
            update_prediction_stat(&db, "Url", Some("Chrome"), "VSCode").expect("stat failed");
        }
        for _ in 0..5 {
            update_prediction_stat(&db, "Code", Some("Terminal"), "VSCode").expect("stat failed");
        }

        let scores = get_type_match_scores(&db, "VSCode").expect("scores failed");
        assert_eq!(scores.len(), 2);
        // Url should be 100.0 (max), Code should be 50.0 (half of max)
        assert!((scores["Url"] - 100.0).abs() < 0.01);
        assert!((scores["Code"] - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_get_type_match_scores_empty() {
        let db = init_test_db().expect("DB init failed");
        let scores = get_type_match_scores(&db, "NoSuchApp").expect("scores failed");
        assert!(scores.is_empty());
    }

    #[test]
    fn test_get_source_affinity() {
        let db = init_test_db().expect("DB init failed");

        // Chrome->VSCode: 8 pastes, Terminal->VSCode: 2 pastes
        for _ in 0..8 {
            update_prediction_stat(&db, "Url", Some("Chrome"), "VSCode").expect("stat failed");
        }
        for _ in 0..2 {
            update_prediction_stat(&db, "Code", Some("Terminal"), "VSCode").expect("stat failed");
        }

        let chrome_affinity = get_source_affinity(&db, "Chrome", "VSCode").expect("affinity failed");
        let terminal_affinity = get_source_affinity(&db, "Terminal", "VSCode").expect("affinity failed");

        // Chrome: 8/10 = 80%, Terminal: 2/10 = 20%
        assert!((chrome_affinity - 80.0).abs() < 0.01);
        assert!((terminal_affinity - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_get_source_affinity_no_data() {
        let db = init_test_db().expect("DB init failed");
        let affinity = get_source_affinity(&db, "Chrome", "NoSuchApp").expect("affinity failed");
        assert!((affinity - 0.0).abs() < 0.01);
    }
}
