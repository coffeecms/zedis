use crate::core::storage::Db;
use crate::core::ai::BgeM3;
use crate::flow::config::{FlowItem, FlowTarget};
use std::sync::Arc;
use sqlx::{AnyPool, Row, Column};
use serde_json::{Value, Map};
use log::{info, error, warn, debug};
use std::time::Duration;

pub async fn run_flow(db: Arc<Db>, bge: Option<Arc<BgeM3>>, config: FlowItem) {
    info!("🌊 Z-Flow: Starting sync for '{}' (Source: {})", config.name, config.source);

    let pool = match AnyPool::connect(&config.source).await {
        Ok(p) => p,
        Err(e) => {
            error!("🌊 Z-Flow Error: Failed to connect to source '{}': {}", config.source, e);
            return;
        }
    };
    
    let mut interval = tokio::time::interval(Duration::from_secs(config.interval));

    loop {
        interval.tick().await;

        let query = format!("SELECT * FROM {}", config.table);
        let rows: Vec<sqlx::any::AnyRow> = match sqlx::query(&query).fetch_all(&pool).await {
            Ok(r) => r,
            Err(e) => {
                warn!("🌊 Z-Flow: Sync failed for '{}': {}", config.name, e);
                continue;
            }
        };

        if rows.is_empty() {
             continue;
        }
        
        debug!("🌊 Z-Flow: Process {} rows for '{}'", rows.len(), config.name);

        for row in &rows {
            // Build a simple map of column->value as strings
            let mut row_map: Map<String, Value> = Map::new();
            for col in row.columns() {
                let col_name = col.name();
                // Try extracting as string, fallback to debug format
                let val_str: String = row.try_get::<String, _>(col_name)
                    .or_else(|_| row.try_get::<i64, _>(col_name).map(|v| v.to_string()))
                    .or_else(|_| row.try_get::<f64, _>(col_name).map(|v| v.to_string()))
                    .unwrap_or_else(|_| "null".to_string());
                row_map.insert(col_name.to_string(), Value::String(val_str));
            }

            match config.target {
                FlowTarget::Json => {
                    let key = format_key(&config.key_format, &config.table, &row_map);
                    db.json_set(key, Value::Object(row_map.clone()).to_string());
                },
                FlowTarget::Vector => {
                    let mut text_parts = Vec::new();
                    for (k, v) in &row_map {
                        if k != "id" {
                            if let Some(s) = v.as_str() {
                                text_parts.push(s.to_string());
                            }
                        }
                    }
                    let text = text_parts.join(" ");
                    let key = format_key(&config.key_format, &config.table, &row_map);

                    if let Some(b) = &bge {
                         if let Ok((dense, sparse)) = b.embed_hybrid(&text) {
                              db.vadd_hybrid(key, dense, Some(sparse));
                         }
                    }
                },
                FlowTarget::Bloom => {
                    if let (Some(item_col), Some(bf_key)) = (&config.item, &config.key) {
                        if let Some(val) = row_map.get(item_col).and_then(|v| v.as_str()) {
                            if !val.is_empty() {
                                db.bf_add(bf_key.clone(), val.to_string());
                            }
                        }
                    }
                },
                FlowTarget::Graph => {
                    if let (Some(gkey), Some(src_col), Some(dst_col)) = (&config.graph_key, &config.source_node, &config.destination_node) {
                        let u = row_map.get(src_col).and_then(|v| v.as_str()).unwrap_or("");
                        let v = row_map.get(dst_col).and_then(|v| v.as_str()).unwrap_or("");
                        if !u.is_empty() && !v.is_empty() {
                            db.graph_add_edge(gkey.clone(), u.to_string(), v.to_string());
                        }
                    }
                },
                FlowTarget::TimeSeries => {
                    if let (Some(ts_col), Some(val_col)) = (&config.timestamp, &config.value) {
                        let ts: i64 = row_map.get(ts_col)
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);

                        let val: f64 = row_map.get(val_col)
                            .and_then(|v| v.as_str())
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0.0);
                        
                        if ts > 0 {
                            let key = format_key(&config.key_format, &config.table, &row_map);
                            db.ts_add(key, ts as u64, val);
                        }
                    }
                },
                FlowTarget::Geo => {
                    if let (Some(gkey), Some(lat_col), Some(lon_col), Some(mem_col)) = (&config.key, &config.lat, &config.lon, &config.member) {
                        let lat: f64 = row_map.get(lat_col).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        let lon: f64 = row_map.get(lon_col).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
                        let member = row_map.get(mem_col).and_then(|v| v.as_str()).unwrap_or("");
                        
                        if !member.is_empty() {
                            db.geoadd(gkey.clone(), lon, lat, member.to_string());
                        }
                    }
                }
            }
        }
    }
}

fn format_key(key_format: &Option<String>, table: &str, row_map: &Map<String, Value>) -> String {
    if let Some(fmt) = key_format {
        let mut result = fmt.clone();
        for (k, v) in row_map {
            let pattern = format!("{{{}}}", k);
            if result.contains(&pattern) {
                 let val_str = v.as_str().unwrap_or("");
                 result = result.replace(&pattern, val_str);
            }
        }
        result
    } else {
        let id = row_map.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
        format!("{}:{}", table, id)
    }
}
