use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config};
use tokenizers::Tokenizer;
use std::path::Path;
use anyhow::{Result, Error};
use half::f16;

// BGE-M3 Wrapper
pub struct BgeM3 {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl BgeM3 {
    pub fn new(model_dir: &str) -> Result<Self> {
        let device = Device::Cpu; // Fallback to CPU for stability on Windows
        
        let config_path = Path::new(model_dir).join("config.json");
        let weights_path = Path::new(model_dir).join("model.safetensors");
        let tokenizer_path = Path::new(model_dir).join("tokenizer.json");

        let config = std::fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&config)?;

        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| Error::msg(e.to_string()))?;
        
        // Load Safetensors
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &device)? };
        let model = BertModel::load(vb, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    // Returns (Dense Vector, Sparse Bag-of-Words Weights)
    pub fn embed_hybrid(&self, text: &str) -> Result<(Vec<f16>, Vec<(u32, f32)>)> {
        let tokens = self.tokenizer.encode(text, true).map_err(|e| Error::msg(e.to_string()))?;
        let token_ids = Tensor::new(tokens.get_ids(), &self.device)?.unsqueeze(0)?;

        let embeddings = self.model.forward(&token_ids, &token_ids.zeros_like()?)?;
        
        // 1. Dense Embedding (CLS token - index 0)
        let cls_embedding = embeddings.get(0)?.get(0)?;
        let cls_vec: Vec<f32> = cls_embedding.to_vec1()?;
        
        // Normalize Dense
        let norm: f32 = cls_vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        let dense_normalized: Vec<f16> = cls_vec.iter()
            .map(|x| f16::from_f32(x / norm)) 
            .collect();

        // 2. Sparse Embedding (Simplified for BGE-M3 MVP)
        // Ideally BGE-M3 uses specific heads for sparse, but often pure BERT weights 
        // on the output layer map to lexical importance.
        // For MVP: We will use term frequency weighted by '1.0' as placeholder 
        // to prove the architectural pipeline works, as BGE-M3 specific sparse logic 
        // requires the custom head implementation which is complex for Step 1.
        // We will store actual Token IDs as dimensions.
        let ids = tokens.get_ids();
        let mut sparse_map: Vec<(u32, f32)> = Vec::new();
        for &id in ids {
            // Simple term frequency 1.0, effectively a "set" of words for now
            // Future: Use actual BGE-M3 sparse weights (Relu(Wx))
            sparse_map.push((id, 1.0));
        }
        
        Ok((dense_normalized, sparse_map))
    }
}
