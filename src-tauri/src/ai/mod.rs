pub mod api_client;
pub mod embeddings;
pub mod semantic_search;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("Model not loaded: {0}")]
    ModelNotLoaded(String),
    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Tokenization error: {0}")]
    TokenizationError(String),
}

impl From<AiError> for String {
    fn from(e: AiError) -> String {
        e.to_string()
    }
}
