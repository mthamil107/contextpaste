// ContextPaste — Semantic Search Engine (Phase 3)
//
// Performs vector similarity search over clip item embeddings.
// Credentials are NEVER included in the vector index.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use super::embeddings::EmbeddingEngine;
use crate::storage::models::ClipItem;
use crate::storage::queries;

pub struct SemanticSearchEngine {
    embedder: Arc<Mutex<EmbeddingEngine>>,
}

impl SemanticSearchEngine {
    pub fn new(embedder: Arc<Mutex<EmbeddingEngine>>) -> Self {
        Self { embedder }
    }

    /// Index a single clip item by generating and storing its embedding.
    /// Credentials are NEVER indexed.
    pub fn index_item(
        &self,
        conn: &Connection,
        item_id: &str,
        content: &str,
        is_credential: bool,
    ) -> Result<(), String> {
        if is_credential {
            return Ok(()); // Never index credentials
        }

        let mut engine = self.embedder.lock().map_err(|e| e.to_string())?;
        if !engine.is_ready() {
            return Ok(()); // Silently skip if AI not initialized
        }

        let embedding = engine.embed(content, false).map_err(|e| e.to_string())?;
        if let Some(emb) = embedding {
            let model_name = engine.model_name().to_string();
            let dimension = engine.dimension();
            drop(engine); // Release lock before DB operation
            queries::store_embedding(conn, item_id, &emb, &model_name, dimension)?;
        }
        Ok(())
    }

    /// Remove an item from the vector index.
    pub fn remove_item(&self, conn: &Connection, item_id: &str) -> Result<(), String> {
        queries::delete_embedding(conn, item_id)
    }

    /// Semantic search using cosine similarity.
    /// Embeds the query, then computes similarity against all stored embeddings.
    pub fn search(
        &self,
        conn: &Connection,
        query: &str,
        limit: u32,
    ) -> Result<Vec<ClipItem>, String> {
        let mut engine = self.embedder.lock().map_err(|e| e.to_string())?;
        if !engine.is_ready() {
            return Err("AI engine not initialized".to_string());
        }

        let query_embedding = engine
            .embed(query, false)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Failed to embed query".to_string())?;
        drop(engine); // Release lock before DB operation

        // Get all embeddings and compute cosine similarity in Rust
        let all_embeddings = queries::get_all_embeddings(conn)?;

        let mut scored: Vec<(String, f32)> = all_embeddings
            .iter()
            .filter_map(|(item_id, blob)| {
                let emb = blob_to_f32_vec(blob);
                if emb.len() != query_embedding.len() {
                    return None;
                }
                let sim = cosine_similarity(&query_embedding, &emb);
                Some((item_id.clone(), sim))
            })
            .collect();

        // Sort by similarity descending
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit as usize);

        // Fetch full ClipItems
        let mut results = Vec::new();
        for (item_id, _score) in scored {
            match queries::get_item_by_conn(conn, &item_id) {
                Ok(item) => {
                    if !item.is_credential {
                        results.push(item);
                    }
                }
                Err(_) => continue,
            }
        }

        Ok(results)
    }

    /// Backfill embeddings for items that don't have them yet.
    pub fn backfill(&self, conn: &Connection, batch_size: u32) -> Result<u32, String> {
        let items = queries::get_unembedded_items(conn, batch_size)?;
        let mut count = 0u32;
        for item in &items {
            if item.is_credential {
                continue;
            }
            match self.index_item(conn, &item.id, &item.content, item.is_credential) {
                Ok(()) => count += 1,
                Err(e) => log::warn!("Failed to embed item {}: {}", item.id, e),
            }
        }
        log::info!("Backfill complete: embedded {} items", count);
        Ok(count)
    }

    /// Check if the underlying embedding engine is ready.
    pub fn is_ready(&self) -> bool {
        self.embedder
            .lock()
            .map(|e| e.is_ready())
            .unwrap_or(false)
    }
}

/// Convert BLOB to Vec<f32> (little-endian).
fn blob_to_f32_vec(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

/// Cosine similarity between two vectors.
/// Both vectors should be L2-normalized, so this is just the dot product.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}
