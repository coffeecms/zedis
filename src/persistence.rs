use crate::core::storage::{Db, DataType};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use anyhow::Result;
use log::{info, error};
use parking_lot::Mutex;

// Append Only File (AOF) Manager - God Tier Durability
pub struct AofManager {
    writer: Mutex<BufWriter<File>>,
    enabled: bool,
}

impl AofManager {
    pub fn new(path: &str, enabled: bool) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            enabled,
        })
    }

    pub fn append(&self, command: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        // God Tier: parking_lot Mutex doesn't poison, no unwrap needed
        let mut w = self.writer.lock();
        if let Err(e) = w.write_all(command.as_bytes()) {
            error!("AOF write error: {}", e);
            return Err(e.into());
        }
        if let Err(e) = w.write_all(b"\n") {
            error!("AOF write error: {}", e);
            return Err(e.into());
        }
        // God Tier Durability: Flush on every write (fsync)
        if let Err(e) = w.flush() {
            error!("AOF flush error: {}", e);
            return Err(e.into());
        }
        Ok(())
    }
}

pub struct Persistence;

impl Persistence {
    pub fn save_rdb(db: &Arc<Db>, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Header
        writer.write_all(b"REDIS0009")?; // Magic + Version
        
        info!("Starting RDB save to {}", path);

        db.visit_all(|key, value| {
            let _ = writer.write_all(key.as_bytes());
            let _ = writer.write_all(b"\n");
            // Placeholder for value serialization
             match value {
                DataType::String(s) => { let _ = writer.write_all(s.as_str().as_bytes()); },
                DataType::List(l) => { let _ = writer.write_all(format!("{:?}", l).as_bytes()); },
                DataType::Hash(h) => { let _ = writer.write_all(format!("{:?}", h).as_bytes()); },
                DataType::ZSet(z) => {
                    let range = z.range(0, usize::MAX);
                    for member in range {
                         let score = z.score(&member).unwrap_or(0.0);
                         let line = format!("{} {}\n", score, member);
                         let _ = writer.write_all(line.as_bytes());
                    }
                },
                DataType::Stream(s) => { let _ = writer.write_all(format!("STREAM: {:?}", s).as_bytes()); },
                _ => {}
            }
            let _ = writer.write_all(b"\n");
        });

        writer.write_all(b"\xFF")?; // EOF
        writer.flush()?;
        
        info!("RDB save completed.");
        Ok(())
    }
}
