// ContextPaste — Workflow Chain Detection
// Detects sequential copy-paste patterns and tracks chains.
// Phase 2: Full chain detection implementation.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};

use crate::storage::database::DbPool;
use crate::storage::models::WorkflowChain;
use crate::storage::queries;

/// A single copy event tracked in the sliding window.
#[derive(Debug, Clone)]
pub struct CopyEvent {
    pub content_type: String,
    pub source_app: Option<String>,
    pub timestamp: DateTime<Utc>,
}

lazy_static! {
    /// Thread-safe sliding window of recent copy events (last 10 items).
    static ref COPY_WINDOW: Arc<Mutex<VecDeque<CopyEvent>>> =
        Arc::new(Mutex::new(VecDeque::with_capacity(10)));
}

const MAX_WINDOW_SIZE: usize = 10;
const MIN_CHAIN_LENGTH: usize = 3;
const MAX_CHAIN_LENGTH: usize = 5;

/// Push a new copy event into the sliding window and trim to 10.
pub fn track_copy_event(content_type: &str, source_app: Option<&str>) {
    let event = CopyEvent {
        content_type: content_type.to_string(),
        source_app: source_app.map(|s| s.to_string()),
        timestamp: Utc::now(),
    };

    let mut window = match COPY_WINDOW.lock() {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to lock copy window: {}", e);
            return;
        }
    };

    window.push_back(event);
    while window.len() > MAX_WINDOW_SIZE {
        window.pop_front();
    }
}

/// After each copy, check if the last N events (3-5) match a repeating chain pattern.
/// A chain is a repeating sequence of content types from the same source context.
/// Returns the chain pattern (as a Vec of content type strings) if detected.
pub fn detect_chain() -> Option<Vec<String>> {
    let window = match COPY_WINDOW.lock() {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to lock copy window for chain detection: {}", e);
            return None;
        }
    };

    let events: Vec<&CopyEvent> = window.iter().collect();
    if events.len() < MIN_CHAIN_LENGTH * 2 {
        return None;
    }

    // Try chain lengths from MAX down to MIN — prefer longer chains
    for chain_len in (MIN_CHAIN_LENGTH..=MAX_CHAIN_LENGTH).rev() {
        if events.len() < chain_len * 2 {
            continue;
        }

        // Extract the most recent `chain_len` events as the candidate pattern
        let candidate: Vec<&str> = events[events.len() - chain_len..]
            .iter()
            .map(|e| e.content_type.as_str())
            .collect();

        // Check if the preceding `chain_len` events match
        let previous: Vec<&str> = events[events.len() - chain_len * 2..events.len() - chain_len]
            .iter()
            .map(|e| e.content_type.as_str())
            .collect();

        if candidate == previous {
            // Also verify source context similarity: at least one common source app
            let candidate_sources: Vec<Option<&str>> = events[events.len() - chain_len..]
                .iter()
                .map(|e| e.source_app.as_deref())
                .collect();
            let previous_sources: Vec<Option<&str>> =
                events[events.len() - chain_len * 2..events.len() - chain_len]
                    .iter()
                    .map(|e| e.source_app.as_deref())
                    .collect();

            // Check if sources match (None matches None, or same app)
            let sources_match = candidate_sources
                .iter()
                .zip(previous_sources.iter())
                .all(|(a, b)| a == b);

            if sources_match {
                return Some(candidate.iter().map(|s| s.to_string()).collect());
            }
        }
    }

    None
}

/// Compute a deterministic hash for a chain pattern.
pub fn compute_chain_hash(pattern: &[String]) -> String {
    let joined = pattern.join("|");
    let mut hasher = Sha256::new();
    hasher.update(joined.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// When a new chain is detected, compute a hash and upsert into workflow_chains table.
pub fn store_chain(
    db: &DbPool,
    pattern: &[String],
    source_context: Option<&str>,
) -> Result<(), String> {
    let chain_hash = compute_chain_hash(pattern);

    // Build items_json from the pattern
    let items: Vec<serde_json::Value> = pattern
        .iter()
        .enumerate()
        .map(|(i, ct)| {
            serde_json::json!({
                "contentType": ct,
                "position": i,
                "preview": ct,
            })
        })
        .collect();

    let items_json =
        serde_json::to_string(&items).map_err(|e| format!("Failed to serialize chain: {}", e))?;

    queries::upsert_workflow_chain(db, &chain_hash, &items_json, source_context)
}

/// Query top workflow chains ordered by frequency DESC.
pub fn get_active_chains(db: &DbPool, limit: u32) -> Result<Vec<WorkflowChain>, String> {
    queries::get_top_chains(db, limit)
}

/// Full workflow tracking pipeline: track event, detect chain, store if found.
/// Returns Some(WorkflowChain pattern) if a chain was detected and stored.
pub fn process_copy_event(
    db: &DbPool,
    content_type: &str,
    source_app: Option<&str>,
) -> Option<Vec<String>> {
    track_copy_event(content_type, source_app);

    if let Some(pattern) = detect_chain() {
        log::info!("Workflow chain detected: {:?}", pattern);
        if let Err(e) = store_chain(db, &pattern, source_app) {
            log::error!("Failed to store workflow chain: {}", e);
        }
        Some(pattern)
    } else {
        None
    }
}

/// Clear the sliding window (useful for testing).
#[allow(dead_code)]
pub fn clear_window() {
    if let Ok(mut window) = COPY_WINDOW.lock() {
        window.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::database::init_test_db;

    #[test]
    fn test_track_copy_event_adds_to_window() {
        clear_window();

        track_copy_event("Url", Some("Chrome"));
        track_copy_event("Code", Some("VSCode"));
        track_copy_event("Json", None);

        let window = COPY_WINDOW.lock().unwrap();
        assert_eq!(window.len(), 3);
        assert_eq!(window[0].content_type, "Url");
        assert_eq!(window[1].content_type, "Code");
        assert_eq!(window[2].content_type, "Json");
    }

    #[test]
    fn test_track_copy_event_trims_to_max() {
        clear_window();

        for i in 0..15 {
            track_copy_event(&format!("Type{}", i), None);
        }

        let window = COPY_WINDOW.lock().unwrap();
        assert_eq!(window.len(), MAX_WINDOW_SIZE);
        // Should keep the last 10 (Type5..Type14)
        assert_eq!(window[0].content_type, "Type5");
        assert_eq!(window[9].content_type, "Type14");
    }

    #[test]
    fn test_detect_chain_finds_repeating_pattern() {
        clear_window();

        // Create a repeating pattern of length 3: Url, Code, Sql repeated twice
        let pattern = vec!["Url", "Code", "Sql"];
        for ct in &pattern {
            track_copy_event(ct, Some("TestApp"));
        }
        for ct in &pattern {
            track_copy_event(ct, Some("TestApp"));
        }

        let detected = detect_chain();
        assert!(detected.is_some());
        let chain = detected.unwrap();
        assert_eq!(chain, vec!["Url", "Code", "Sql"]);
    }

    #[test]
    fn test_detect_chain_no_match_without_repetition() {
        clear_window();

        track_copy_event("Url", Some("Chrome"));
        track_copy_event("Code", Some("VSCode"));
        track_copy_event("Json", Some("Postman"));
        track_copy_event("Sql", Some("DBeaver"));
        track_copy_event("PlainText", Some("Notepad"));
        track_copy_event("Email", Some("Outlook"));

        let detected = detect_chain();
        assert!(detected.is_none());
    }

    #[test]
    fn test_detect_chain_requires_source_match() {
        clear_window();

        // Same content types but different sources — should NOT match
        track_copy_event("Url", Some("Chrome"));
        track_copy_event("Code", Some("VSCode"));
        track_copy_event("Sql", Some("DBeaver"));
        track_copy_event("Url", Some("Firefox")); // different source
        track_copy_event("Code", Some("VSCode"));
        track_copy_event("Sql", Some("DBeaver"));

        let detected = detect_chain();
        assert!(detected.is_none());
    }

    #[test]
    fn test_compute_chain_hash_deterministic() {
        let pattern = vec!["Url".to_string(), "Code".to_string(), "Sql".to_string()];
        let hash1 = compute_chain_hash(&pattern);
        let hash2 = compute_chain_hash(&pattern);
        assert_eq!(hash1, hash2);

        let different = vec!["Code".to_string(), "Url".to_string(), "Sql".to_string()];
        let hash3 = compute_chain_hash(&different);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_store_chain_and_upsert() {
        let db = init_test_db().expect("DB init failed");
        let pattern = vec!["Url".to_string(), "Code".to_string(), "Sql".to_string()];

        // Store once
        store_chain(&db, &pattern, Some("Chrome")).expect("Store failed");

        let chains = get_active_chains(&db, 10).expect("Get chains failed");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].frequency, 1);

        // Store again — should increment frequency
        store_chain(&db, &pattern, Some("Chrome")).expect("Store failed");

        let chains = get_active_chains(&db, 10).expect("Get chains failed");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].frequency, 2);
    }

    #[test]
    fn test_get_top_chains_ordered_by_frequency() {
        let db = init_test_db().expect("DB init failed");

        let pattern_a = vec!["Url".to_string(), "Code".to_string(), "Sql".to_string()];
        let pattern_b = vec![
            "Json".to_string(),
            "ShellCommand".to_string(),
            "FilePath".to_string(),
        ];

        // pattern_a stored 3 times
        for _ in 0..3 {
            store_chain(&db, &pattern_a, None).expect("Store failed");
        }
        // pattern_b stored 1 time
        store_chain(&db, &pattern_b, None).expect("Store failed");

        let chains = get_active_chains(&db, 10).expect("Get chains failed");
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].frequency, 3); // pattern_a first (higher freq)
        assert_eq!(chains[1].frequency, 1); // pattern_b second
    }
}
