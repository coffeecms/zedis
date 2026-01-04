use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use half::f16;
use crate::core::ai::BgeM3; // Assumed we might need this trait later, or just access structure

// God Tier: Hybrid Vector Storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZVector {
    pub dense: Vec<f16>,             // 1024 dims * 2 bytes = 2KB
    pub sparse: Option<Vec<(u32, f32)>>, // Lexical weights
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorIndex {
    vectors: HashMap<String, ZVector>,
    dim: usize,
}

impl VectorIndex {
    pub fn new(dim: usize) -> Self {
        Self {
            vectors: HashMap::new(),
            dim,
        }
    }

    pub fn add(&mut self, key: String, dense: Vec<f16>, sparse: Option<Vec<(u32, f32)>>) -> Result<(), String> {
        if dense.len() != self.dim {
            return Err(format!("Vector dimension mismatch. Expected {}, got {}", self.dim, dense.len()));
        }
        self.vectors.insert(key, ZVector { dense, sparse });
        Ok(())
    }

    // Hybrid Search: Alpha * Dense + (1-Alpha) * Sparse
    pub fn search_hybrid(&self, query_dense: &[f16], query_sparse: Option<&[(u32, f32)]>, k: usize, alpha: f32) -> Vec<(String, f32)> {
        let mut scores: Vec<(String, f32)> = self.vectors.iter()
            .map(|(key, doc_vec)| {
                // 1. Dense Score (Cosine)
                let dense_score = cosine_similarity_f16(query_dense, &doc_vec.dense);
                
                // 2. Sparse Score (Dot Product / Overlap)
                let sparse_score = if let (Some(q_sp), Some(d_sp)) = (query_sparse, &doc_vec.sparse) {
                     sparse_dot_product(q_sp, d_sp)
                } else {
                    0.0
                };

                // 3. Weighted Sum
                let hybrid_score = (alpha * dense_score) + ((1.0 - alpha) * sparse_score);
                (key.clone(), hybrid_score)
            })
            .collect();

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }
}

// Optimized f16 Cosine Similarity
fn cosine_similarity_f16(a: &[f16], b: &[f16]) -> f32 {
    let mut dot: f32 = 0.0;
    let mut norm_a: f32 = 0.0;
    let mut norm_b: f32 = 0.0;
    
    // TODO: SIMD Optimize this loop for AVX-512
    for (x, y) in a.iter().zip(b) {
        let fx = x.to_f32();
        let fy = y.to_f32();
        dot += fx * fy;
        norm_a += fx * fx;
        norm_b += fy * fy;
    }
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

fn sparse_dot_product(a: &[(u32, f32)], b: &[(u32, f32)]) -> f32 {
    // A and B should strictly be sorted by Index for O(N) merge, 
    // assuming they are for now or using Hash lookup for one.
    // Simple naive loop for MVP:
    let mut dot = 0.0;
    for (idx_a, val_a) in a {
        for (idx_b, val_b) in b {
            if idx_a == idx_b {
                dot += val_a * val_b;
                break; // Found match, since unique
            }
        }
    }
    dot
}
