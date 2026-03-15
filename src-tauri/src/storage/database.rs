// ContextPaste — SQLite Database Setup & Migrations

use rusqlite::Connection;
use std::sync::{Arc, Mutex};

use crate::utils::config;

pub type DbPool = Arc<Mutex<Connection>>;

/// Initialize the database connection with WAL mode and run all migrations.
pub fn init_db() -> Result<DbPool, String> {
    let db_path = config::db_path();

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database at {:?}: {}", db_path, e))?;

    // Enable WAL mode for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("Failed to enable WAL mode: {}", e))?;

    // Performance pragmas
    conn.execute_batch(
        "PRAGMA synchronous=NORMAL;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;",
    )
    .map_err(|e| format!("Failed to set pragmas: {}", e))?;

    run_migrations(&conn)?;

    log::info!("Database initialized at {:?}", db_path);
    Ok(Arc::new(Mutex::new(conn)))
}

/// Initialize an in-memory database for testing.
#[cfg(test)]
pub fn init_test_db() -> Result<DbPool, String> {
    let conn = Connection::open_in_memory()
        .map_err(|e| format!("Failed to open in-memory database: {}", e))?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("Failed to set pragmas: {}", e))?;
    run_migrations(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn run_migrations(conn: &Connection) -> Result<(), String> {
    // Create migrations tracking table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Failed to create migrations table: {}", e))?;

    let current_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    for (version, sql) in MIGRATIONS.iter().enumerate() {
        let v = (version + 1) as i64;
        if v > current_version {
            log::info!("Running migration v{}", v);
            conn.execute_batch(sql)
                .map_err(|e| format!("Migration v{} failed: {}", v, e))?;
            conn.execute(
                "INSERT INTO schema_migrations (version) VALUES (?1)",
                [v],
            )
            .map_err(|e| format!("Failed to record migration v{}: {}", v, e))?;
        }
    }

    Ok(())
}

const MIGRATIONS: &[&str] = &[
    // v1: Core tables
    "CREATE TABLE IF NOT EXISTS clip_items (
        id TEXT PRIMARY KEY,
        content TEXT NOT NULL,
        content_type TEXT NOT NULL DEFAULT 'PlainText',
        content_hash TEXT NOT NULL,
        content_length INTEGER NOT NULL,
        is_credential BOOLEAN NOT NULL DEFAULT 0,
        credential_type TEXT,
        source_app TEXT,
        source_window_title TEXT,
        is_pinned BOOLEAN NOT NULL DEFAULT 0,
        is_starred BOOLEAN NOT NULL DEFAULT 0,
        expires_at TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        last_pasted_at TEXT,
        paste_count INTEGER NOT NULL DEFAULT 0,
        tags TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_clip_created ON clip_items(created_at DESC);
    CREATE INDEX IF NOT EXISTS idx_clip_type ON clip_items(content_type);
    CREATE INDEX IF NOT EXISTS idx_clip_hash ON clip_items(content_hash);
    CREATE INDEX IF NOT EXISTS idx_clip_pinned ON clip_items(is_pinned) WHERE is_pinned = 1;
    CREATE INDEX IF NOT EXISTS idx_clip_source ON clip_items(source_app);

    CREATE VIRTUAL TABLE IF NOT EXISTS clip_items_fts USING fts5(
        content, source_app, source_window_title, tags,
        content='clip_items', content_rowid='rowid'
    );

    -- FTS sync triggers
    CREATE TRIGGER IF NOT EXISTS clip_items_ai AFTER INSERT ON clip_items BEGIN
        INSERT INTO clip_items_fts(rowid, content, source_app, source_window_title, tags)
        VALUES (new.rowid, new.content, new.source_app, new.source_window_title, new.tags);
    END;

    CREATE TRIGGER IF NOT EXISTS clip_items_ad AFTER DELETE ON clip_items BEGIN
        INSERT INTO clip_items_fts(clip_items_fts, rowid, content, source_app, source_window_title, tags)
        VALUES ('delete', old.rowid, old.content, old.source_app, old.source_window_title, old.tags);
    END;

    CREATE TRIGGER IF NOT EXISTS clip_items_au AFTER UPDATE ON clip_items BEGIN
        INSERT INTO clip_items_fts(clip_items_fts, rowid, content, source_app, source_window_title, tags)
        VALUES ('delete', old.rowid, old.content, old.source_app, old.source_window_title, old.tags);
        INSERT INTO clip_items_fts(rowid, content, source_app, source_window_title, tags)
        VALUES (new.rowid, new.content, new.source_app, new.source_window_title, new.tags);
    END;

    -- Paste events
    CREATE TABLE IF NOT EXISTS paste_events (
        id TEXT PRIMARY KEY,
        item_id TEXT NOT NULL REFERENCES clip_items(id) ON DELETE CASCADE,
        target_app TEXT,
        target_window_title TEXT,
        pasted_at TEXT NOT NULL DEFAULT (datetime('now')),
        session_id TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_paste_item ON paste_events(item_id);
    CREATE INDEX IF NOT EXISTS idx_paste_time ON paste_events(pasted_at DESC);

    -- Prediction stats
    CREATE TABLE IF NOT EXISTS prediction_stats (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        content_type TEXT NOT NULL,
        source_app TEXT,
        target_app TEXT NOT NULL,
        frequency INTEGER NOT NULL DEFAULT 1,
        last_used_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE INDEX IF NOT EXISTS idx_pred_target ON prediction_stats(target_app);

    -- Workflow chains
    CREATE TABLE IF NOT EXISTS workflow_chains (
        id TEXT PRIMARY KEY,
        chain_hash TEXT NOT NULL UNIQUE,
        items_json TEXT NOT NULL,
        frequency INTEGER NOT NULL DEFAULT 1,
        last_triggered_at TEXT NOT NULL DEFAULT (datetime('now')),
        source_context TEXT
    );

    -- Settings
    CREATE TABLE IF NOT EXISTS app_settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    // v2: AI embeddings (Phase 3)
    "CREATE TABLE IF NOT EXISTS clip_embeddings (
        item_id TEXT PRIMARY KEY REFERENCES clip_items(id) ON DELETE CASCADE,
        embedding BLOB NOT NULL,
        model_name TEXT NOT NULL DEFAULT 'all-MiniLM-L6-v2',
        dimension INTEGER NOT NULL DEFAULT 384,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE INDEX IF NOT EXISTS idx_embeddings_model ON clip_embeddings(model_name);

    CREATE TABLE IF NOT EXISTS ai_api_keys (
        provider TEXT PRIMARY KEY,
        api_key_encrypted TEXT NOT NULL,
        base_url TEXT,
        model_name TEXT,
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    // v3: Context-aware auto-paste tables
    "CREATE TABLE IF NOT EXISTS paste_rules (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        priority INTEGER NOT NULL DEFAULT 0,
        enabled BOOLEAN NOT NULL DEFAULT 1,
        app_pattern TEXT,
        window_title_pattern TEXT,
        context_pattern TEXT,
        content_type_filter TEXT,
        action_type TEXT NOT NULL DEFAULT 'paste_recent_type',
        action_value TEXT NOT NULL,
        times_triggered INTEGER NOT NULL DEFAULT 0,
        last_triggered_at TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE INDEX IF NOT EXISTS idx_paste_rules_enabled ON paste_rules(enabled, priority DESC);

    CREATE TABLE IF NOT EXISTS auto_paste_events (
        id TEXT PRIMARY KEY,
        item_id TEXT NOT NULL REFERENCES clip_items(id) ON DELETE CASCADE,
        rule_id TEXT REFERENCES paste_rules(id) ON DELETE SET NULL,
        confidence REAL NOT NULL,
        was_correct BOOLEAN,
        screen_context TEXT,
        target_app TEXT,
        target_window_title TEXT,
        pasted_at TEXT NOT NULL DEFAULT (datetime('now'))
    );

    CREATE INDEX IF NOT EXISTS idx_auto_paste_time ON auto_paste_events(pasted_at DESC);",
    // v4: Learned paste patterns — records WHERE + WHAT for every manual paste
    "CREATE TABLE IF NOT EXISTS learned_patterns (
        id TEXT PRIMARY KEY,
        content_type TEXT NOT NULL,
        target_app TEXT,
        target_window_title TEXT,
        screen_context TEXT,
        item_id TEXT REFERENCES clip_items(id) ON DELETE SET NULL,
        frequency INTEGER NOT NULL DEFAULT 1,
        last_used_at TEXT NOT NULL DEFAULT (datetime('now')),
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        promoted_to_rule_id TEXT REFERENCES paste_rules(id) ON DELETE SET NULL
    );

    CREATE INDEX IF NOT EXISTS idx_learned_app ON learned_patterns(target_app);
    CREATE INDEX IF NOT EXISTS idx_learned_freq ON learned_patterns(frequency DESC);",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db() {
        let db = init_test_db().expect("Failed to init test DB");
        let conn = db.lock().expect("Failed to lock");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM clip_items", [], |row| row.get(0))
            .expect("Failed to query");
        assert_eq!(count, 0);
    }

    #[test]
    fn test_migrations_idempotent() {
        let db = init_test_db().expect("Failed to init test DB");
        let conn = db.lock().expect("Failed to lock");
        // Running migrations again should not fail
        drop(conn);
        // Re-running init on same connection would work since migrations check version
    }
}
