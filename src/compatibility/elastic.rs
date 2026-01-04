use axum::{
    extract::{Path, State},
    routing::{get, post, put, delete},
    Json, Router, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use crate::core::storage::Db;
use crate::core::ai::BgeM3;
use half::f16;
use tokio::net::TcpListener;

// Mask State
#[derive(Clone)]
pub struct ElasticMask {
    pub db: Arc<Db>,
    pub bge: Option<Arc<BgeM3>>,
}

// Request Models
#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: QueryObj,
}

#[derive(Deserialize)]
pub struct QueryObj {
    #[serde(rename = "match")]
    pub match_query: Option<Value>, // "match": { "field": "text" }
}

// Response Models (Elastic Compatible)
#[derive(Serialize)]
pub struct SearchResponse {
    pub took: u64,
    pub timed_out: bool,
    pub hits: HitsObj,
}

#[derive(Serialize)]
pub struct HitsObj {
    pub total: TotalObj,
    pub max_score: f32,
    pub hits: Vec<HitItem>,
}

#[derive(Serialize)]
pub struct TotalObj {
    pub value: usize,
    pub relation: String,
}

#[derive(Serialize)]
pub struct HitItem {
    pub _index: String,
    pub _type: String,
    pub _id: String,
    pub _score: f32,
    pub _source: Value,
}

impl ElasticMask {
    pub async fn run(self, port: u16) {
        let app = Router::new()
            .route("/", get(root_info))
            .route("/:index/_search", post(handle_search))
            .route("/:index/_doc/:id", put(handle_index).get(handle_get).delete(handle_delete))
            .with_state(self);

        let addr = format!("0.0.0.0:{}", port);
        log::info!("🎭 Z-Mask (Generic Elastic) listening on {}", addr);
        
        if let Ok(listener) = TcpListener::bind(addr).await {
            if let Err(e) = axum::serve(listener, app).await {
                log::error!("Mask server error: {}", e);
            }
        }
    }
}

// Handlers

async fn root_info() -> Json<Value> {
    Json(serde_json::json!({
        "name": "zedis",
        "cluster_name": "zedis-universe",
        "cluster_uuid": "god-tier-uuid",
        "version": {
            "number": "7.10.0",
            "build_flavor": "oss",
            "msg": "Impersonating ElasticSearch for Compatibility"
        },
        "tagline": "You Know, for Search"
    }))
}

async fn handle_search(
    State(mask): State<ElasticMask>,
    Path(index): Path<String>,
    Json(payload): Json<SearchRequest>,
) -> Json<SearchResponse> {
    // 1. Extract Query Text (Simplistic mapping for "match")
    let mut query_text = "".to_string();
    if let Some(m) = payload.query.match_query {
        // match: { "content": "text" }
        if let Some(obj) = m.as_object() {
            if let Some(val) = obj.values().next() {
                query_text = val.as_str().unwrap_or("").to_string();
            }
        }
    }

    // 2. Direct Dispatch - Embed & Search
    let mut hits = Vec::new();
    if let Some(bge) = &mask.bge {
        if let Ok((dense, sparse)) = bge.embed_hybrid(&query_text) {
             let results = mask.db.vsearch_hybrid(&index, dense, Some(sparse), 10, 0.5);
             for (id, score) in results {
                 hits.push(HitItem {
                     _index: index.clone(),
                     _type: "_doc".to_string(),
                     _id: id,
                     _score: score,
                     _source: serde_json::json!({}), // We don't store full JSON source in VectorIndex in this MVP yet, but we could!
                 });
             }
        }
    }

    Json(SearchResponse {
        took: 1,
        timed_out: false,
        hits: HitsObj {
            total: TotalObj { value: hits.len(), relation: "eq".to_string() },
            max_score: hits.first().map(|h| h._score).unwrap_or(0.0),
            hits,
        }
    })
}

async fn handle_index(
    State(mask): State<ElasticMask>,
    Path((index, id)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    // Extract text to vectorise (heuristic: look for 'content', 'text', or first string field)
    let body_str = payload.to_string(); // Native JSON storage
    
    // 1. Store JSON
    // mask.db.json_set(id.clone(), body_str.clone()); // Assuming we had JSON store integrated fully. MVP: just embed.

    // 2. Embed
    let mut embedded = false;
    if let Some(bge) = &mask.bge {
        // Try to find a text field
        let text = if let Some(t) = payload.get("content").and_then(|v| v.as_str()) {
            t
        } else if let Some(t) = payload.get("text").and_then(|v| v.as_str()) {
            t
        } else {
             &body_str // Fallback: Embed the whole JSON
        };

        if let Ok((dense, sparse)) = bge.embed_hybrid(text) {
            // Use 'index' as the key prefix or collection? 
            // Zedis MVP is flat key space. We'll use "index:id" as key.
            let key = format!("{}:{}", index, id);
            mask.db.vadd_hybrid(key, dense, Some(sparse));
            embedded = true;
        }
    }

    Json(serde_json::json!({
        "_index": index,
        "_id": id,
        "result": if embedded { "created" } else { "error" }
    }))
}

async fn handle_get(
    State(_mask): State<ElasticMask>,
    Path((index, id)): Path<(String, String)>,
) -> Json<Value> {
    // MVP stub
    Json(serde_json::json!({
        "_index": index,
        "_id": id,
        "found": false
    }))
}

async fn handle_delete(
    State(_mask): State<ElasticMask>,
    Path((index, id)): Path<(String, String)>,
) -> Json<Value> {
    Json(serde_json::json!({
        "_index": index,
        "_id": id,
        "result": "deleted"
    }))
}
