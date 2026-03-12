// ContextPaste — AI API Client (Phase 3)
//
// Supports OpenAI-compatible and Ollama embedding APIs for BYOK mode.

use serde::{Deserialize, Serialize};

use super::AiError;

#[derive(Debug, Clone)]
pub struct ApiClient {
    provider: String,
    api_key: String,
    base_url: String,
    model_name: String,
    client: reqwest::blocking::Client,
    dimension: usize,
}

#[derive(Serialize)]
struct OpenAIEmbeddingRequest {
    input: String,
    model: String,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

impl ApiClient {
    /// Create a client for OpenAI-compatible embedding APIs.
    pub fn new_openai(api_key: &str, base_url: Option<&str>, model: Option<&str>) -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: api_key.to_string(),
            base_url: base_url.unwrap_or("https://api.openai.com").to_string(),
            model_name: model.unwrap_or("text-embedding-3-small").to_string(),
            client: reqwest::blocking::Client::new(),
            dimension: 1536,
        }
    }

    /// Create a client for Ollama embedding API.
    pub fn new_ollama(base_url: Option<&str>, model: Option<&str>) -> Self {
        Self {
            provider: "ollama".to_string(),
            api_key: String::new(),
            base_url: base_url.unwrap_or("http://localhost:11434").to_string(),
            model_name: model.unwrap_or("nomic-embed-text").to_string(),
            client: reqwest::blocking::Client::new(),
            dimension: 768,
        }
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// Generate embedding via API.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> {
        match self.provider.as_str() {
            "openai" => self.embed_openai(text),
            "ollama" => self.embed_ollama(text),
            _ => Err(AiError::ProviderNotConfigured(self.provider.clone())),
        }
    }

    fn embed_openai(&self, text: &str) -> Result<Vec<f32>, AiError> {
        let url = format!("{}/v1/embeddings", self.base_url);
        let req = OpenAIEmbeddingRequest {
            input: text.to_string(),
            model: self.model_name.clone(),
        };

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .map_err(|e: reqwest::Error| AiError::ApiError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            return Err(AiError::ApiError(format!(
                "OpenAI API error {}: {}",
                status, body
            )));
        }

        let result: OpenAIEmbeddingResponse = resp
            .json()
            .map_err(|e: reqwest::Error| AiError::ApiError(e.to_string()))?;

        result
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| AiError::ApiError("No embedding returned".to_string()))
    }

    fn embed_ollama(&self, text: &str) -> Result<Vec<f32>, AiError> {
        let url = format!("{}/api/embeddings", self.base_url);
        let req = OllamaEmbeddingRequest {
            model: self.model_name.clone(),
            prompt: text.to_string(),
        };

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .map_err(|e: reqwest::Error| AiError::ApiError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            return Err(AiError::ApiError(format!(
                "Ollama API error {}: {}",
                status, body
            )));
        }

        let result: OllamaEmbeddingResponse = resp
            .json()
            .map_err(|e: reqwest::Error| AiError::ApiError(e.to_string()))?;

        Ok(result.embedding)
    }

    /// Test API connection by embedding a short string.
    pub fn test_connection(&self) -> Result<(), AiError> {
        self.embed("test connection")?;
        Ok(())
    }
}
