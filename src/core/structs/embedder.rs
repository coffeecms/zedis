use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Simulates an Embedding Model (e.g., BERT/OpenAI) 
// In production, this calls a sidecar or ONNX model.
pub struct Embedder;

impl Embedder {
    pub fn embed(text: &str, dim: usize) -> Vec<f32> {
        // Deterministic pseudo-random vector based on text hash
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();
        
        let mut vec = Vec::with_capacity(dim);
        for i in 0..dim {
            // Very naive LCG for testing stability
            let val = ((seed.wrapping_add(i as u64)).wrapping_mul(6364136223846793005) % 100) as f32 / 100.0;
            vec.push(val);
        }
        vec
    }
}
