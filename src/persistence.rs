use crate::core::storage::{Db, DataType};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use anyhow::Result;
use log::{info, error};
use parking_lot::Mutex;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Fsync Policy for AOF durability vs performance tradeoff
#[derive(Clone, Copy, PartialEq)]
pub enum FsyncPolicy {
    Always,    // Flush every write (safest, slowest)
    EverySec,  // Flush every 1 second (balanced)
    No,        // Let OS decide (fastest, least safe)
}

// Append Only File (AOF) Manager - God Tier Durability + Performance
pub struct AofManager {
    sender: parking_lot::Mutex<Option<mpsc::Sender<String>>>,
    enabled: AtomicBool,
    fsync_policy: FsyncPolicy,
}

impl AofManager {
    pub fn new(path: &str, enabled: bool) -> Result<Self> {
        Self::with_policy(path, enabled, FsyncPolicy::EverySec)
    }

    pub fn with_policy(path: &str, enabled: bool, policy: FsyncPolicy) -> Result<Self> {
        let (tx, rx) = mpsc::channel::<String>();
        let path = path.to_string();
        let fsync_policy = policy;
        
        // Background writer thread - non-blocking for callers
        thread::spawn(move || {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .expect("Failed to open AOF file");
            let mut writer = BufWriter::new(file);
            let mut last_flush = std::time::Instant::now();
            
            loop {
                // Batch receive with timeout for periodic flush
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(cmd) => {
                        let _ = writer.write_all(cmd.as_bytes());
                        let _ = writer.write_all(b"\n");
                        
                        // Flush based on policy
                        match fsync_policy {
                            FsyncPolicy::Always => {
                                let _ = writer.flush();
                            }
                            FsyncPolicy::EverySec => {
                                if last_flush.elapsed() >= Duration::from_secs(1) {
                                    let _ = writer.flush();
                                    last_flush = std::time::Instant::now();
                                }
                            }
                            FsyncPolicy::No => {
                                // Let OS handle it
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Periodic flush on timeout (for EverySec policy)
                        if fsync_policy == FsyncPolicy::EverySec {
                            let _ = writer.flush();
                            last_flush = std::time::Instant::now();
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        // Channel closed, final flush and exit
                        let _ = writer.flush();
                        break;
                    }
                }
            }
        });

        Ok(Self {
            sender: parking_lot::Mutex::new(Some(tx)),
            enabled: AtomicBool::new(enabled),
            fsync_policy,
        })
    }

    pub fn append(&self, command: &str) -> Result<()> {
        if !self.enabled.load(Ordering::Relaxed) {
            return Ok(());
        }
        // Non-blocking send to background writer
        if let Some(ref tx) = *self.sender.lock() {
            let _ = tx.send(command.to_string());
        }
        Ok(())
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }
}

pub struct Persistence;

impl Persistence {
    pub fn save_rdb(db: &Arc<Db>, path: &str) -> Result<()> {
        let tmp_path = format!("{}.tmp", path);
        let file = File::create(&tmp_path)?;
        let mut writer = BufWriter::new(file);

        info!("Starting Bincode RDB save to {}", path);
        
        // God Tier: Bincode Serialize directly to disk stream
        bincode::serialize_into(&mut writer, &**db)?;
        
        writer.flush()?;
        // Ensure file is closed/flushed before rename
        drop(writer); 

        // Atomic Rename to prevent corruption
        std::fs::rename(&tmp_path, path)?;
        
        info!("RDB save completed successfully.");
        Ok(())
    }

    pub fn load_rdb(path: &str) -> Result<Arc<Db>> {
        info!("Loading RDB from {}", path);
        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);
        
        // God Tier: Streaming Deserialize
        let db: Db = bincode::deserialize_from(reader)?;
        
        Ok(Arc::new(db))
    }
}
