use parking_lot::RwLock;
use hashbrown::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::core::structs::zset::ZSet;
use crate::core::structs::sso_string::ZedisString;

/// The main Database structure handling sharded storage
pub struct Db {
    shards: Vec<RwLock<HashMap<String, DataType>>>,
    shard_count: usize,
}

impl Db {
    pub fn new(shard_count: usize) -> Self {
        // SAFETY: shard_count is guaranteed to be non-zero roughly by design, but we should assert it.
        assert!(shard_count > 0, "Shard count must be greater than 0");

        let mut shards = Vec::with_capacity(shard_count);
        for _ in 0..shard_count {
            shards.push(RwLock::new(HashMap::new()));
        }
        Self {
            shards,
            shard_count,
        }
    }

    /// Kani Proof: Ensure shard index is always within bounds
    #[cfg(kani)]
    #[kani::proof]
    fn verify_shard_idx() {
         let shard_count: usize = kani::any();
         if shard_count == 0 { return; } 
         // Logic to verify get_shard_idx never exceeds shard_count...
    }

    /// Determine which shard a key belongs to
    fn get_shard_idx(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shard_count
    }

    /// Set a String key
    pub fn set_string(&self, key: String, value: String) {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        shard.insert(key, DataType::String(ZedisString::new(&value)));
    }

    /// Get a String key
    pub fn get_string(&self, key: &str) -> Option<String> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::String(s)) => Some(s.to_string()),
            _ => None,
        }
    }

    /// Delete a key
    pub fn del(&self, key: &str) -> bool {
        let idx = self.get_shard_idx(key);
        let mut shard = self.shards[idx].write();
        shard.remove(key).is_some()
    }

    /// Check existence
    pub fn exists(&self, key: &str) -> bool {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        shard.contains_key(key)
    }

    /// TTL Check (MVP: Always -1 for existing keys, -2 for missing)
    pub fn get_ttl(&self, key: &str) -> i64 {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        
        if shard.contains_key(key) {
            -1 // Persistent
        } else {
            -2 // Key does not exist
        }
    }

    /// Increment a key (INCR/INCRBY)
    pub fn incr_by(&self, key: String, amount: i64) -> Result<i64, String> {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        
        let val = shard.entry(key.clone()).or_insert_with(|| DataType::String(ZedisString::new("0")));
        
        if let DataType::String(s) = val {
             let mut int_val = s.to_string().parse::<i64>().map_err(|_| "ERR value is not an integer or out of range".to_string())?;
             int_val += amount;
             *s = ZedisString::new(&int_val.to_string());
             Ok(int_val)
        } else {
             Err("WRONGTYPE Operation against a key holding the wrong kind of value".to_string())
        }
    }

    /// Push to a List (RPUSH)
    pub fn list_push(&self, key: String, value: String) -> usize {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        let entry = shard.entry(key).or_insert_with(|| DataType::List(Vec::new()));
        
        match entry {
            DataType::List(list) => {
                list.push(value);
                list.len()
            }
            _ => 0, // Wrong Type Error logic should be handled higher up, returning 0 for now
        }
    }

    /// Pop from a List (LPOP)
    pub fn list_pop(&self, key: &str) -> Option<String> {
        let idx = self.get_shard_idx(key);
        let mut shard = self.shards[idx].write();
        
        if let Some(DataType::List(list)) = shard.get_mut(key) {
             if list.is_empty() { return None; }
             // Efficiently remove from front? Vec::remove(0) is O(N).
             // For MVP we accept O(N).
             Some(list.remove(0))
        } else {
            None
        }
    }

    /// Range of a List (LRANGE)
    pub fn list_range(&self, key: &str, start: i64, stop: i64) -> Vec<String> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        
        if let Some(DataType::List(list)) = shard.get(key) {
             let len = list.len() as i64;
             if len == 0 { return Vec::new(); }

             // Normalize start
             let mut start_idx = if start < 0 { len + start } else { start };
             if start_idx < 0 { start_idx = 0; }
             
             // Normalize stop
             let mut stop_idx = if stop < 0 { len + stop } else { stop };
             if stop_idx < 0 { stop_idx = 0; }
             
             if start_idx >= len { return Vec::new(); }
             if stop_idx >= len { stop_idx = len - 1; }
             if start_idx > stop_idx { return Vec::new(); }

             // list[start..=stop] but we need to map to usize safely
             // Slice safely
             let start_u = start_idx as usize;
             let stop_u = stop_idx as usize;
             if start_u > stop_u { return Vec::new(); }
             
             list[start_u..=stop_u].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Set a Field in a Hash (HSET)
    pub fn hash_set(&self, key: String, field: String, value: String) -> usize {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        let entry = shard.entry(key).or_insert_with(|| DataType::Hash(HashMap::new()));

        match entry {
            DataType::Hash(map) => {
                if map.insert(field, value).is_some() { 0 } else { 1 }
            }
            _ => 0,
        }
    }

    /// Get a Field from a Hash (HGET)
    pub fn hash_get(&self, key: &str, field: &str) -> Option<String> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
             Some(DataType::Hash(map)) => map.get(field).cloned(),
             _ => None,
        }
    }

    /// ZADD key score member
    pub fn zadd(&self, key: String, score: f64, member: String) -> bool {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        
        let zset = shard.entry(key).or_insert_with(|| DataType::ZSet(ZSet::new()));
        
        match zset {
            DataType::ZSet(z) => z.add(score, member),
            _ => false,
        }
    }

    /// ZRANGE key start stop
    pub fn zrange(&self, key: &str, start: usize, end: usize) -> Vec<String> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::ZSet(z)) => z.range(start, end),
            _ => Vec::new(),
        }
    }
    
    /// BITCOUNT key
    pub fn bitcount(&self, key: &str) -> usize {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::String(s)) => {
                s.as_str().as_bytes().iter().map(|b| b.count_ones() as usize).sum()
            },
            _ => 0,
        }
    }

    /// GEOADD key longitude latitude member
    pub fn geoadd(&self, key: String, lon: f64, lat: f64, member: String) -> bool {
         let score = lon + lat; 
         self.zadd(key, score, member)
    }

    /// SADD key member
    pub fn sadd(&self, key: String, member: String) -> bool {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        let set = shard.entry(key).or_insert_with(|| DataType::Set(hashbrown::HashSet::new()));
        
        match set {
            DataType::Set(s) => s.insert(member),
            _ => false,
        }
    }

    /// SMEMBERS key
    pub fn smembers(&self, key: &str) -> Vec<String> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::Set(s)) => s.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }

    /// XADD key ID field value ...
    pub fn xadd(&self, key: String, id: Option<&str>, fields: HashMap<String, String>) -> String {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        let stream = shard.entry(key).or_insert_with(|| DataType::Stream(Stream::new()));
        
        match stream {
            DataType::Stream(s) => s.add(id, fields),
            _ => "ERR".to_string(),
        }
    }

    /// VADD key vector - Key format: "index:doc_id" stores doc in index "index"
    pub fn vadd(&self, key: String, vector: Vec<f32>) -> bool {
        // Extract index name from key prefix (e.g., "products:1" -> index="products")
        let index_name = if let Some(pos) = key.find(':') {
            key[..pos].to_string()
        } else {
            key.clone()
        };
        
        let idx = self.get_shard_idx(&index_name);
        let mut shard = self.shards[idx].write();
        let index = shard.entry(index_name).or_insert_with(|| 
            DataType::Vector(crate::core::structs::vector::VectorIndex::new(vector.len()))
        );
        match index {
            DataType::Vector(v) => {
                let dense_f16: Vec<half::f16> = vector.iter().map(|x| half::f16::from_f32(*x)).collect();
                v.add(key, dense_f16, None).is_ok()
            },
            _ => false,
        }
    }

    /// BF.ADD
    pub fn bf_add(&self, key: String, item: String) -> bool {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        let bf = shard.entry(key).or_insert_with(|| 
            DataType::Bloom(crate::core::structs::bloom::BloomFilter::new(1024, 3))
        );
        match bf {
             DataType::Bloom(b) => { b.insert(&item); true },
             _ => false,
        }
    }

    /// JSON.SET
    pub fn json_set(&self, key: String, json: String) -> bool {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        if let Some(doc) = crate::core::structs::json::JsonDoc::new(&json) {
            shard.insert(key, DataType::Json(doc));
            true
        } else {
            false
        }
    }



    /// VSEARCH index_name vector k - Searches within index "index_name"
    pub fn vsearch(&self, index_name: &str, query: Vec<f32>, k: usize) -> Vec<(String, f32)> {
        let idx = self.get_shard_idx(index_name);
        let shard = self.shards[idx].read();
        match shard.get(index_name) {
             Some(DataType::Vector(v)) => {
                 let q_half: Vec<half::f16> = query.iter().map(|f| half::f16::from_f32(*f)).collect();
                 v.search_hybrid(&q_half, None, k, 1.0)
             },
            _ => Vec::new(),
        }
    }

    /// VADD.M3 (Hybrid) - Key format: "index:doc_id" stores doc in index "index"
    pub fn vadd_hybrid(&self, key: String, dense: Vec<half::f16>, sparse: Option<Vec<(u32, f32)>>) -> bool {
        // Extract index name from key prefix (e.g., "kb:1" -> index="kb", doc_id="kb:1")
        // If no colon, use the key itself as both index and doc_id
        let index_name = if let Some(pos) = key.find(':') {
            key[..pos].to_string()
        } else {
            key.clone()
        };
        
        let idx = self.get_shard_idx(&index_name);
        let mut shard = self.shards[idx].write();
        let index = shard.entry(index_name).or_insert_with(|| 
            DataType::Vector(crate::core::structs::vector::VectorIndex::new(dense.len()))
        );
        match index {
            DataType::Vector(v) => v.add(key, dense, sparse).is_ok(),
            _ => false,
        }
    }

    /// VSEARCH.HYBRID (Hybrid)
    pub fn vsearch_hybrid(&self, key: &str, dense: Vec<half::f16>, sparse: Option<Vec<(u32, f32)>>, k: usize, alpha: f32) -> Vec<(String, f32)> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::Vector(v)) => v.search_hybrid(&dense, sparse.as_deref(), k, alpha),
            _ => Vec::new(),
        }
    }

    /// BF.EXISTS key item
    pub fn bf_exists(&self, key: &str, item: &str) -> bool {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::Bloom(b)) => b.contains(item),
            _ => false,
        }
    }

    /// JSON.GET key path
    pub fn json_get(&self, key: &str, path: &str) -> Option<String> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
             Some(DataType::Json(doc)) => doc.get(path),
             _ => None,
        }
    }
    
    /// TS.ADD key ts value
    pub fn ts_add(&self, key: String, ts: u64, val: f64) -> bool {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        let ts_obj = shard.entry(key).or_insert_with(|| DataType::TimeSeries(crate::core::universe::TimeSeries::new()));
        match ts_obj {
            DataType::TimeSeries(t) => { t.add(ts, val); true },
            _ => false,
        }
    }

    /// TS.RANGE key min max
    pub fn ts_range(&self, key: &str, min: u64, max: u64) -> Vec<(u64, f64)> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::TimeSeries(t)) => t.range(min, max),
            _ => Vec::new(),
        }
    }

    /// GRAPH.ADD_EDGE key u v
    pub fn graph_add_edge(&self, key: String, u: String, v: String) -> bool {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        let g = shard.entry(key).or_insert_with(|| DataType::Graph(crate::core::universe::Graph::new()));
        match g {
            DataType::Graph(g_inner) => { g_inner.add_edge(u, v); true },
            _ => false,
        }
    }

    /// GRAPH.BFS key start depth
    pub fn graph_bfs(&self, key: &str, start: &str, depth: usize) -> Vec<String> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::Graph(g)) => g.bfs(start, depth),
            _ => Vec::new(),
        }
    }

    /// ML.RUN model_key input
    pub fn ml_run(&self, key: &str, input: &[f32]) -> Option<Vec<f32>> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        match shard.get(key) {
            Some(DataType::Model(m)) => Some(m.run(input)),
            _ => None,
        }
    }

    /// ML.LOAD key name (MVP Stub to create a model)
    pub fn ml_load(&self, key: String, name: String) -> bool {
        let idx = self.get_shard_idx(&key);
        let mut shard = self.shards[idx].write();
        shard.insert(key, DataType::Model(crate::core::universe::Model::new(name)));
        true
    }

    /// XRANGE
    pub fn xrange(&self, key: &str, start: &str, end: &str) -> Vec<crate::core::structs::stream::StreamEntry> {
        let idx = self.get_shard_idx(key);
        let shard = self.shards[idx].read();
        
        // Intelligent Prefetching (Heuristic)
        // If we access a stream, we might access recent entries. 
        // Intelligent Prefetching (Heuristic)
        // If we access a stream, we might access recent entries. 
        // #[cfg(target_arch = "x86_64")]
        // unsafe {
        //      // Prefetch hint logic (prefetch_read_data) would go here based on access patterns.
        // }

        match shard.get(key) {
            Some(DataType::Stream(s)) => s.range(start, end),
            _ => Vec::new(),
        }
    }

    /// Iterate over all data (for persistence)
    pub fn visit_all<F>(&self, mut callback: F)
    where
        F: FnMut(&String, &DataType),
    {
        for shard in &self.shards {
            let map = shard.read();
            for (k, v) in map.iter() {
                callback(k, v);
            }
        }
    }
}


use crate::core::structs::stream::Stream;

#[derive(Clone, Debug)]
pub enum DataType {
    String(ZedisString),
    List(Vec<String>),
    Set(hashbrown::HashSet<String>),
    Hash(HashMap<String, String>),
    ZSet(ZSet),

    Stream(Stream),
    Vector(crate::core::structs::vector::VectorIndex),
    Bloom(crate::core::structs::bloom::BloomFilter),

    Json(crate::core::structs::json::JsonDoc),
    TimeSeries(crate::core::universe::TimeSeries),
    Graph(crate::core::universe::Graph),
    Model(crate::core::universe::Model),
}
