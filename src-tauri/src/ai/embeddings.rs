// ContextPaste — Local ONNX Embedding Engine (Phase 3)
//
// Uses all-MiniLM-L6-v2 for local 384-dimensional embeddings.
// Credentials are NEVER embedded.

use std::path::Path;

use ort::session::Session;
use ort::value::{Shape, Tensor};
use tokenizers::Tokenizer;

use super::AiError;

pub type EmbeddingResult<T> = Result<T, AiError>;

pub struct EmbeddingEngine {
    session: Option<Session>,
    tokenizer: Option<Tokenizer>,
    model_name: String,
    dimension: usize,
    ready: bool,
}

impl EmbeddingEngine {
    pub fn new() -> Self {
        Self {
            session: None,
            tokenizer: None,
            model_name: "all-MiniLM-L6-v2".to_string(),
            dimension: 384,
            ready: false,
        }
    }

    /// Initialize local ONNX model from file paths.
    pub fn init_local(&mut self, model_path: &Path, tokenizer_path: &Path) -> EmbeddingResult<()> {
        if !model_path.exists() {
            return Err(AiError::ModelNotLoaded(format!(
                "Model file not found: {}",
                model_path.display()
            )));
        }
        if !tokenizer_path.exists() {
            return Err(AiError::ModelNotLoaded(format!(
                "Tokenizer file not found: {}",
                tokenizer_path.display()
            )));
        }

        let session = Session::builder()
            .map_err(|e| AiError::ModelNotLoaded(e.to_string()))?
            .with_intra_threads(1)
            .map_err(|e| AiError::ModelNotLoaded(e.to_string()))?
            .commit_from_file(model_path)
            .map_err(|e| AiError::ModelNotLoaded(e.to_string()))?;

        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| AiError::TokenizationError(e.to_string()))?;

        self.session = Some(session);
        self.tokenizer = Some(tokenizer);
        self.dimension = 384;
        self.model_name = "all-MiniLM-L6-v2".to_string();
        self.ready = true;

        log::info!(
            "Local ONNX model loaded: {} ({}D)",
            self.model_name,
            self.dimension
        );
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn dimension(&self) -> usize {
        self.dimension
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Generate embedding for text. Returns None for credentials or empty text.
    /// Credentials are NEVER embedded per security policy.
    pub fn embed(&mut self, text: &str, is_credential: bool) -> EmbeddingResult<Option<Vec<f32>>> {
        // NEVER embed credentials
        if is_credential {
            return Ok(None);
        }

        if text.trim().is_empty() {
            return Ok(None);
        }

        let tokenizer = self
            .tokenizer
            .as_ref()
            .ok_or_else(|| AiError::ModelNotLoaded("Tokenizer not initialized".to_string()))?;

        // Tokenize — truncate to 128 tokens for short clipboard text
        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| AiError::TokenizationError(e.to_string()))?;

        let max_len = 128.min(encoding.get_ids().len());
        let input_ids: Vec<i64> = encoding.get_ids()[..max_len]
            .iter()
            .map(|&id| id as i64)
            .collect();
        let attention_mask: Vec<i64> = encoding.get_attention_mask()[..max_len]
            .iter()
            .map(|&m| m as i64)
            .collect();
        let token_type_ids: Vec<i64> = encoding.get_type_ids()[..max_len]
            .iter()
            .map(|&t| t as i64)
            .collect();

        let seq_len = input_ids.len();
        let shape = Shape::new([1i64, seq_len as i64]);

        // Create input tensors using (Shape, Vec<T>) tuples
        let ids_tensor = Tensor::from_array((shape.clone(), input_ids))
            .map_err(|e| AiError::EmbeddingFailed(e.to_string()))?;
        let mask_tensor = Tensor::from_array((shape.clone(), attention_mask.clone()))
            .map_err(|e| AiError::EmbeddingFailed(e.to_string()))?;
        let type_tensor = Tensor::from_array((shape, token_type_ids))
            .map_err(|e| AiError::EmbeddingFailed(e.to_string()))?;

        let inputs = ort::inputs![
            "input_ids" => ids_tensor,
            "attention_mask" => mask_tensor,
            "token_type_ids" => type_tensor,
        ];

        // Run inference (session.run requires &mut self)
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| AiError::ModelNotLoaded("ONNX session not initialized".to_string()))?;

        let outputs = session
            .run(inputs)
            .map_err(|e| AiError::EmbeddingFailed(e.to_string()))?;

        // Extract output tensor — shape [1, seq_len, 384]
        // try_extract_tensor returns (&Shape, &[T])
        let (output_shape, output_data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| AiError::EmbeddingFailed(e.to_string()))?;

        // Determine hidden size from shape (last dimension)
        let shape_dims: Vec<i64> = output_shape.iter().copied().collect();
        let hidden_size = if shape_dims.len() >= 3 {
            shape_dims[2] as usize
        } else if shape_dims.len() == 2 {
            shape_dims[1] as usize
        } else {
            return Err(AiError::EmbeddingFailed(format!(
                "Unexpected output shape: {:?}",
                shape_dims
            )));
        };

        // Mean pooling with attention mask
        let mut pooled = vec![0.0f32; hidden_size];
        let mut mask_sum = 0.0f32;

        for s in 0..seq_len {
            let mask_val = attention_mask[s] as f32;
            mask_sum += mask_val;
            for h in 0..hidden_size {
                let idx = s * hidden_size + h;
                if idx < output_data.len() {
                    pooled[h] += output_data[idx] * mask_val;
                }
            }
        }

        if mask_sum > 0.0 {
            for h in 0..hidden_size {
                pooled[h] /= mask_sum;
            }
        }

        // L2 normalize
        let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in pooled.iter_mut() {
                *v /= norm;
            }
        }

        Ok(Some(pooled))
    }

    /// Set dimension and model name (for API providers with different dimensions).
    pub fn set_provider_info(&mut self, model_name: &str, dimension: usize) {
        self.model_name = model_name.to_string();
        self.dimension = dimension;
    }

    /// Mark as ready for API-based providers that don't need a local model.
    pub fn set_api_ready(&mut self, model_name: &str, dimension: usize) {
        self.model_name = model_name.to_string();
        self.dimension = dimension;
        self.ready = true;
        self.session = None;
        self.tokenizer = None;
    }
}
