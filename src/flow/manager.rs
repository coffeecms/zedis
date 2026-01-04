use crate::core::storage::Db;
use crate::core::ai::BgeM3;
use crate::flow::config::ZFlowConfig;
use crate::flow::etl::run_flow;
use std::sync::Arc;
use std::path::Path;
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config};
use tokio::sync::Mutex;
use log::{info, error};
use std::collections::HashMap;
use tokio::task::JoinHandle;

pub struct FlowManager {
    db: Arc<Db>,
    bge: Option<Arc<BgeM3>>,
    tasks: Mutex<HashMap<String, JoinHandle<()>>>,
}

impl FlowManager {
    pub fn new(db: Arc<Db>, bge: Option<Arc<BgeM3>>) -> Self {
        Self {
            db,
            bge,
            tasks: Mutex::new(HashMap::new()),
        }
    }

    pub async fn run(self: Arc<Self>, path_str: String) {
        let path = Path::new(&path_str);
        if !path.exists() {
             return;
        }

        info!("🌊 Z-Flow: Monitoring config at '{}'", path_str);

        self.reload_config(&path_str).await;

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        
        let watcher = RecommendedWatcher::new(move |res| {
             if let Ok(event) = res {
                 let _ = tx.blocking_send(event);
             }
        }, Config::default()).ok();

        if let Some(mut w) = watcher {
             if let Err(e) = w.watch(path, RecursiveMode::NonRecursive) {
                 error!("🌊 Z-Flow: Watch failed: {}", e);
             }
             
             let self_clone = self.clone();
             let path_string = path_str.clone();
             tokio::spawn(async move {
                 let _w = w; // Keep watcher alive
                 while rx.recv().await.is_some() {
                      tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                      self_clone.reload_config(&path_string).await;
                 }
             });
        }
    }

    async fn reload_config(&self, path: &str) {
        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => {
                error!("🌊 Z-Flow: Read failed: {}", e);
                return;
            }
        };

        let config: ZFlowConfig = match toml::from_str(&content) {
             Ok(c) => c,
             Err(e) => {
                 error!("🌊 Z-Flow: TOML Parse Error: {}", e);
                 return;
             }
        };

        info!("🌊 Z-Flow: Reloading configuration... found {} flows.", config.flow.len());
        let mut tasks = self.tasks.lock().await;

        let mut current_names = Vec::new();
        for item in config.flow {
             let name = item.name.clone();
             current_names.push(name.clone());
             
             // Always restart for simplicity in MVP
             if let Some(old) = tasks.remove(&name) {
                  old.abort();
             }
             
             let db = self.db.clone();
             let bge = self.bge.clone();
             let task = tokio::spawn(async move {
                  run_flow(db, bge, item).await;
             });
             tasks.insert(name, task);
        }

        // Remove deleted flows
        let keys: Vec<String> = tasks.keys().cloned().collect();
        for k in keys {
             if !current_names.contains(&k) {
                 if let Some(handle) = tasks.remove(&k) {
                     info!("🌊 Z-Flow: Stopping '{}'", k);
                     handle.abort();
                 }
             }
        }
    }
}
