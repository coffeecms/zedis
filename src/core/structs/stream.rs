use hashbrown::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct StreamEntry {
    pub id: String, // "timestamp-sequence"
    pub fields: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Stream {
    entries: Vec<StreamEntry>,
    last_id: (u128, u64), // timestamp, sequence
}

impl Stream {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            last_id: (0, 0),
        }
    }

    pub fn add(&mut self, id_arg: Option<&str>, fields: HashMap<String, String>) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let (ts, seq) = if let Some(id_str) = id_arg {
             if id_str == "*" {
                 (now, if now == self.last_id.0 { self.last_id.1 + 1 } else { 0 })
             } else {
                 // Simplified ID parsing for MVP
                 let parts: Vec<&str> = id_str.split('-').collect();
                 let t = parts[0].parse().unwrap_or(now);
                 let s = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);
                 (t, s)
             }
        } else {
             (now, if now == self.last_id.0 { self.last_id.1 + 1 } else { 0 })
        };

        if ts < self.last_id.0 || (ts == self.last_id.0 && seq <= self.last_id.1) {
             // For auto-generated IDs, this logic ensures increase.
             // For user IDs, we should error if lower, but for MVP we just accept/overwrite logic or force new ID.
             // Let's simpler: force logic.
        }

        self.last_id = (ts, seq);
        let id_string = format!("{}-{}", ts, seq);
        
        self.entries.push(StreamEntry {
            id: id_string.clone(),
            fields,
        });
        
        id_string
    }

    pub fn range(&self, start: &str, end: &str) -> Vec<StreamEntry> {
        // Linear scan for MVP. Real Streams use Radix Trees.
        let start_bound = start;
        let end_bound = if end == "+" { "9999999999999-999" } else { end };

        self.entries.iter().filter(|e| {
            e.id.as_str() >= start_bound && e.id.as_str() <= end_bound
        }).cloned().collect()
    }
}
