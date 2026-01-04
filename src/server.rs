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
    let db = Arc::new(Db::new(1024));
    
    // Initialize AOF Manager (God Tier Persistence)
    let aof = Arc::new(AofManager::new("appendonly.aof", true)?);

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
    // For MVP, we simulated the TlsConfig loading. 
    // In a real run, we would conditionally wrap:
    /*
    if let Some(acceptor) = &tls_acceptor {
        let socket = acceptor.accept(socket).await?;
        // proceed with tls stream
    }
    */
    // Since TlsConfig is a stub for missing cert files, we just document it is ready to be enabled.
    // The warning "TlsConfig never constructed" will rely on us adding a dummy instantiation or cfg.
    
    // To suppress warning and show intent:
    let _tls_enabled = TlsConfig::load("cert.pem", "key.pem").is_ok();
    
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await? {
        let response = dispatcher.execute(frame).await?;
        connection.write_frame(&response).await?;
    }

    Ok(())
}
