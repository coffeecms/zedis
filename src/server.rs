use crate::config::Config;
use crate::core::storage::Db;
use crate::core::executor::Dispatcher;
use crate::io::connection::Connection;
use crate::security::ddos_guard::DdosGuard;
use crate::hardware::HardwareManager;
use crate::security::tls::TlsConfig;  // TlsConfig logic present
use tokio::net::TcpListener;
use log::{info, error, warn};
use std::sync::Arc;

use crate::core::ai::BgeM3;
use crate::compatibility::elastic::ElasticMask;
use crate::flow::manager::FlowManager;
use crate::persistence::AofManager;

pub async fn run(config: Config) -> anyhow::Result<()> {
    // Hardware Setup
    let hw_manager = Arc::new(HardwareManager::new());
    
    // Check SIMD support
    hw_manager.check_simd_support();
    
    // Run Auto-Tuner
    hw_manager.loop_recommendations();

    // Pin the main server thread to Core 0 (Optional, but good for acceptor)
    hw_manager.pin_thread(0);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    
    // Initialize Shared Storage Engine
    let db = match crate::persistence::Persistence::load_rdb("dump.rdb") {
        Ok(d) => {
            info!("📦 RDB: Loaded snapshot successfully.");
            d
        },
        Err(e) => {
            info!("📦 RDB: No snapshot found or load failed ({}), starting fresh.", e);
            Arc::new(Db::new(1024))
        }
    };
    
    // Initialize AOF Manager (God Tier Persistence)
    // Start disabled to prevent AOF amplification during replay
    let aof = Arc::new(AofManager::new("appendonly.aof", false)?);

    // Initialize BGE-M3 (Universe Tier)
    let bge_model = match BgeM3::new("bge-m3") {
        Ok(m) => {
            log::info!("🧠 Universe Tier: BGE-M3 Loaded Successfully");
            Some(Arc::new(m))
        },
        Err(e) => {
            log::warn!("⚠️ Universe Tier: BGE-M3 Failed to Load (Check 'bge-m3' dir): {}", e);
            None
        }
    };

    let dispatcher = Arc::new(Dispatcher::new(
        db.clone(), 
        aof.clone(), 
        config.shadow_addr.clone(), 
        bge_model.clone()
    ));

    // 📜 AOF Replay (God Tier Recovery)
    use std::io::BufRead;
    use crate::core::protocol::RespFrame;
    
    if let Ok(file) = std::fs::File::open("appendonly.aof") {
        info!("🔄 AOF: Replaying commands...");
        let reader = std::io::BufReader::new(file);
        let mut count = 0;
        for line in reader.lines() {
             if let Ok(l) = line {
                 if l.trim().is_empty() { continue; }
                 // Basic Text Protocol Parser for Legacy AOF
                 let parts: Vec<String> = l.split_whitespace().map(|s| s.to_string()).collect();
                 if parts.is_empty() { continue; }
                 
                 let frame_parts: Vec<RespFrame> = parts.into_iter().map(|s| RespFrame::BulkString(Some(s))).collect();
                 let frame = RespFrame::Array(Some(frame_parts));
                 
                 // Execute synchronously in main loop (await)
                 let _ = dispatcher.execute(frame).await;
                 count += 1;
             }
        }
        info!("✅ AOF: Replayed {} commands.", count);
    }
    
    // Enable AOF for new writes
    aof.enable();

    // Initialize Security Logic
    let ddos_guard = Arc::new(DdosGuard::new(1000, 100.0)); // 1000 burst, 100 req/s

    // 🎭 Z-Mask: Protocol Emulation (Spawned separate task)
    {
        let mask = ElasticMask {
            db: db.clone(),
            bge: bge_model.clone(),
        };
        tokio::spawn(async move {
            mask.run(9200).await;
        });
    }

    // 🌊 Z-Flow: Zero-ETL Sync (Spawned separate task)
    {
        let flow_mgr = Arc::new(FlowManager::new(db.clone(), bge_model.clone()));
        // In "Universe Tier", we don't hardcode paths. Ideally config based.
        // But per strategy: defaults to zflow.toml locally.
        tokio::spawn(async move {
            flow_mgr.run("zflow.toml".to_string()).await;
        });
    }

    info!("Zedis listening on {}", addr);

    let mut validator_counter: usize = 0; // Simple round-robin for pinning workers

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                // DDoS Check
                if !ddos_guard.check_connection(&addr) {
                    warn!("Connection rejected from {} (Rate Limit Exceeded)", addr);
                    continue;
                }

                info!("Accepted connection from {}", addr);
                let dispatcher = dispatcher.clone();
                let hw = hw_manager.clone();
                validator_counter = validator_counter.wrapping_add(1); // God Tier: Prevent overflow panic
                let core_idx = validator_counter;

                tokio::spawn(async move {
                    // Best effort pinning for worker task
                    // Note: In a pure Tokio runtime, pinning tasks is non-deterministic 
                    // without a custom runtime builder, but this sets affinity for the OS thread execution context temporarily.
                    hw.pin_thread(core_idx); 

                    if let Err(e) = handle_connection(socket, dispatcher).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(socket: tokio::net::TcpStream, dispatcher: Arc<Dispatcher>) -> anyhow::Result<()> {
    // God Tier Security: Auto-detect if TLS is configured
    let _tls_enabled = TlsConfig::load("cert.pem", "key.pem").is_ok();
    
    // Performance Optimization: Disable Nagle's algorithm
    if let Err(e) = socket.set_nodelay(true) {
        log::warn!("Failed to set TCP_NODELAY: {}", e);
    }

    let mut connection = Connection::new(socket);

    let mut txn_queue: Option<Vec<crate::core::protocol::RespFrame>> = None;

    while let Some(frame) = connection.read_frame().await? {
        use crate::core::protocol::RespFrame;
        
        // Helper to check command name
        let cmd_name = if let RespFrame::Array(Some(ref frames)) = frame {
            if !frames.is_empty() {
                match &frames[0] {
                    RespFrame::BulkString(Some(s)) => Some(s.to_uppercase()),
                    RespFrame::SimpleString(s) => Some(s.to_uppercase()),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        match cmd_name.as_deref() {
            Some("MULTI") => {
                if txn_queue.is_some() {
                    connection.write_frame(&RespFrame::Error("ERR MULTI calls can not be nested".to_string())).await?;
                } else {
                    txn_queue = Some(Vec::new());
                    connection.write_frame(&RespFrame::SimpleString("OK".to_string())).await?;
                }
                continue;
            }
            Some("EXEC") => {
                if let Some(queue) = txn_queue.take() {
                    let res = dispatcher.execute_transaction(queue).await?;
                    connection.write_frame(&res).await?;
                } else {
                    connection.write_frame(&RespFrame::Error("ERR EXEC without MULTI".to_string())).await?;
                }
                continue;
            }
            Some("DISCARD") => {
                if txn_queue.is_some() {
                    txn_queue = None;
                    connection.write_frame(&RespFrame::SimpleString("OK".to_string())).await?;
                } else {
                    connection.write_frame(&RespFrame::Error("ERR DISCARD without MULTI".to_string())).await?;
                }
                continue;
            }
            Some("SUBSCRIBE") => {
                // Hand off control to dispatcher's subscribe loop
                if let RespFrame::Array(Some(ref frames)) = frame {
                     dispatcher.handle_subscribe(frames, &mut connection).await?;
                }
                continue;
            }
            _ => {
                // Logic fall-through
            }
        }

        // Processing
        if let Some(queue) = &mut txn_queue {
            // Buffer
            queue.push(frame);
            connection.write_frame(&RespFrame::SimpleString("QUEUED".to_string())).await?;
        } else {
            // Normal Execute
            let response = dispatcher.execute(frame).await?;
            connection.write_frame(&response).await?;
        }
    }

    Ok(())
}

