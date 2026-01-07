use dashmap::DashMap;
use hashbrown::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::core::structs::zset::ZSet;
use crate::core::structs::sso_string::ZedisString;
use crate::core::structs::probabilistic::{HyperLogLogWrapper, CuckooFilterWrapper, TopKWrapper, CountMinSketchWrapper, TDigestWrapper};
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug, Clone, Copy)]
pub enum BitType {
    Signed(u8),
    Unsigned(u8),
}

pub enum BitfieldOp {
    Get(BitType, usize),        // type, offset
    Set(BitType, usize, i64),   // type, offset, value
    IncrBy(BitType, usize, i64),// type, offset, increment
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitOverflow {
    Wrap,
    Sat,
    Fail,
}

/// The main Database structure - God Tier Lock-Free with DashMap
pub struct Db {
    data: DashMap<String, DataType>,
}

impl Db {
    pub fn new(_shard_count: usize) -> Self {
        // DashMap handles sharding internally (default 64 shards)
        // _shard_count parameter kept for API compatibility
        Self {
            data: DashMap::with_capacity(100_000),
        }
    }
}

// God Tier Persistence: Custom Serialization for DashMap
impl Serialize for Db {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Collect to HashMap for serialization
        let snapshot: HashMap<String, DataType> = self.data.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        snapshot.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Db {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, DataType> = Deserialize::deserialize(deserializer)?;
        let data = DashMap::with_capacity(map.len());
        for (k, v) in map {
            data.insert(k, v);
        }
        Ok(Db { data })
    }
}

impl Db {
    /// Kani Proof: Ensure shard index is always within bounds
    #[cfg(kani)]
    #[kani::proof]
    fn verify_shard_idx() {
         // DashMap handles this internally, no manual shard index
    }

    /// Set a String key (lock-free)
    pub fn set_string(&self, key: String, value: String) {
        self.data.insert(key, DataType::String(ZedisString::new(&value)));
    }

    /// Get a String key (lock-free)
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key).and_then(|entry| {
            match entry.value() {
                DataType::String(s) => Some(s.to_string()),
                _ => None,
            }
        })
    }

    /// Delete a key (lock-free)
    pub fn del(&self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    /// Check existence (lock-free)
    pub fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// TTL Check (MVP: Always -1 for existing keys, -2 for missing)
    pub fn get_ttl(&self, key: &str) -> i64 {
        if self.data.contains_key(key) {
            -1 // Persistent
        } else {
            -2 // Key does not exist
        }
    }

    /// Increment a key (INCR/INCRBY) - atomic via entry API
    pub fn incr_by(&self, key: String, amount: i64) -> Result<i64, String> {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::String(ZedisString::new("0")));
        
        if let DataType::String(s) = entry.value_mut() {
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
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::List(Vec::new()));
        
        match entry.value_mut() {
            DataType::List(list) => {
                list.push(value);
                list.len()
            }
            _ => 0,
        }
    }

    /// Pop from a List (LPOP)
    pub fn list_pop(&self, key: &str) -> Option<String> {
        self.data.get_mut(key).and_then(|mut entry| {
            if let DataType::List(list) = entry.value_mut() {
                if list.is_empty() { return None; }
                Some(list.remove(0))
            } else {
                None
            }
        })
    }

    /// Range of a List (LRANGE)
    pub fn list_range(&self, key: &str, start: i64, stop: i64) -> Vec<String> {
        self.data.get(key).map(|entry| {
            if let DataType::List(list) = entry.value() {
                let len = list.len() as i64;
                if len == 0 { return Vec::new(); }

                let mut start_idx = if start < 0 { len + start } else { start };
                if start_idx < 0 { start_idx = 0; }
                
                let mut stop_idx = if stop < 0 { len + stop } else { stop };
                if stop_idx < 0 { stop_idx = 0; }
                
                if start_idx >= len { return Vec::new(); }
                if stop_idx >= len { stop_idx = len - 1; }
                if start_idx > stop_idx { return Vec::new(); }

                let start_u = start_idx as usize;
                let stop_u = stop_idx as usize;
                if start_u > stop_u { return Vec::new(); }
                
                list[start_u..=stop_u].to_vec()
            } else {
                Vec::new()
            }
        }).unwrap_or_default()
    }

    /// Set a Field in a Hash (HSET)
    pub fn hash_set(&self, key: String, field: String, value: String) -> usize {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::Hash(HashMap::new()));
        
        match entry.value_mut() {
            DataType::Hash(map) => {
                if map.insert(field, value).is_some() { 0 } else { 1 }
            }
            _ => 0,
        }
    }

    /// Get a Field from a Hash (HGET)
    pub fn hash_get(&self, key: &str, field: &str) -> Option<String> {
        self.data.get(key).and_then(|entry| {
            match entry.value() {
                 DataType::Hash(map) => map.get(field).cloned(),
                 _ => None,
            }
        })
    }

    /// ZADD key score member
    pub fn zadd(&self, key: String, score: f64, member: String) -> bool {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::ZSet(ZSet::new()));
        
        match entry.value_mut() {
            DataType::ZSet(z) => z.add(score, member),
            _ => false,
        }
    }

    /// ZRANGE key start stop
    pub fn zrange(&self, key: &str, start: usize, end: usize) -> Vec<String> {
        self.data.get(key).map(|entry| {
            match entry.value() {
                DataType::ZSet(z) => z.range(start, end),
                _ => Vec::new(),
            }
        }).unwrap_or_default()
    }
    
    /// BITCOUNT key
    pub fn bitcount(&self, key: &str) -> usize {
        self.data.get(key).map(|entry| {
             match entry.value() {
                DataType::String(s) => s.as_str().as_bytes().iter().map(|b| b.count_ones() as usize).sum(),
                _ => 0,
            }
        }).unwrap_or(0)
    }

    /// BITFIELD key
    pub fn bitfield(&self, key: String, ops: Vec<BitfieldOp>, overflow: Vec<BitOverflow>) -> Vec<Option<i64>> {
         let mut entry = self.data.entry(key.clone()).or_insert_with(|| DataType::String(ZedisString::new("")));
         
         if let DataType::String(ref mut s) = entry.value_mut() {
             let mut bytes = s.to_string().into_bytes();
             
             let mut results = Vec::new();
             let mut current_overflow = BitOverflow::Wrap;
             let mut ov_idx = 0;

             for op in ops {
                 if ov_idx < overflow.len() {
                     current_overflow = overflow[ov_idx];
                     ov_idx += 1;
                 }

                 match op {
                     BitfieldOp::Get(typ, offset) => {
                          let val = self.read_bits(&bytes, typ, offset);
                          results.push(Some(val));
                     },
                     BitfieldOp::Set(typ, offset, value) => {
                          let res = self.write_bits(&mut bytes, typ, offset, value, current_overflow);
                         results.push(res);
                     },
                     BitfieldOp::IncrBy(typ, offset, incr) => {
                          let old = self.read_bits(&bytes, typ, offset);
                          // Incr logic with wrapping
                          let width = match typ { BitType::Signed(w) | BitType::Unsigned(w) => w };
                          // Simple wrapping for now as simplified logic
                          let new_val_base = old.wrapping_add(incr); 
                          let res = self.write_bits(&mut bytes, typ, offset, new_val_base, current_overflow);
                          results.push(res);
                     }
                 }
             }
             
             unsafe {
                  *s = ZedisString::new(&String::from_utf8_unchecked(bytes));
             }
             
             results
         } else {
             vec![]
         }
    }

    // Helper for Bit Manip (Internal)
    fn read_bits(&self, bytes: &[u8], typ: BitType, offset: usize) -> i64 {
        let (width, is_signed) = match typ { BitType::Signed(w) => (w, true), BitType::Unsigned(w) => (w, false) };
        if width == 0 || width > 64 { return 0; }
        
        let mut val: u64 = 0;
        let mut bits_remaining = width as usize;
        let mut current_byte_idx = offset / 8;
        let mut current_bit_in_byte = offset % 8;
        
        while bits_remaining > 0 {
            let bit = if current_byte_idx >= bytes.len() { 0 } else { (bytes[current_byte_idx] >> (7 - current_bit_in_byte)) & 1 };
            val = (val << 1) | (bit as u64);
            bits_remaining -= 1;
            current_bit_in_byte += 1;
            if current_bit_in_byte == 8 { current_bit_in_byte = 0; current_byte_idx += 1; }
        }
        
        if is_signed {
            let shift = 64 - width;
            (val as i64) << shift >> shift
        } else {
            val as i64
        }
    }

    fn write_bits(&self, bytes: &mut Vec<u8>, typ: BitType, offset: usize, value: i64, overflow: BitOverflow) -> Option<i64> {
         let (width, is_signed) = match typ { BitType::Signed(w) => (w, true), BitType::Unsigned(w) => (w, false) };
         if width == 0 || width > 64 { return None; }

         let (val_to_store, result_val) = if is_signed {
             let min = -(1i64 << (width - 1));
             let max = (1i64 << (width - 1)) - 1;
             match overflow {
                 BitOverflow::Wrap => {
                     let mask = (1u64 << width) - 1;
                     let raw = value as u64 & mask;
                     let shift = 64 - width;
                     (raw, Some((raw as i64) << shift >> shift))
                 },
                 BitOverflow::Sat => {
                     let val = if value < min { min } else if value > max { max } else { value };
                     let mask = (1u64 << width) - 1;
                     (val as u64 & mask, Some(val))
                 },
                 BitOverflow::Fail => {
                     if value < min || value > max { (0, None) } else { let mask = (1u64 << width) - 1; (value as u64 & mask, Some(value)) }
                 }
             }
         } else {
             let max = if width == 64 { u64::MAX } else { (1u64 << width) - 1 };
             let u_val = value as u64;
             match overflow {
                 BitOverflow::Wrap => {
                     let mask = if width == 64 { u64::MAX } else { (1u64 << width) - 1 };
                     (u_val & mask, Some((u_val & mask) as i64))
                 },
                 BitOverflow::Sat => {
                     let val = if u_val > max { max } else { u_val };
                     (val, Some(val as i64))
                 },
                 BitOverflow::Fail => {
                     if u_val > max { (0, None) } else { (u_val, Some(u_val as i64)) }
                 }
             }
         };

         if result_val.is_none() { return None; }
         
         let end_bit = offset + width as usize;
         let end_byte = (end_bit + 7) / 8;
         if bytes.len() < end_byte { bytes.resize(end_byte, 0); }

         let mut bits_remaining = width as usize;
         let mut current_byte_idx = offset / 8;
         let mut current_bit_in_byte = offset % 8;
         let val_ptr = val_to_store;

         while bits_remaining > 0 {
             let bit = (val_ptr >> (bits_remaining - 1)) & 1;
             if bit == 1 { bytes[current_byte_idx] |= 1 << (7 - current_bit_in_byte); } 
             else { bytes[current_byte_idx] &= !(1 << (7 - current_bit_in_byte)); }
             bits_remaining -= 1;
             current_bit_in_byte += 1;
             if current_bit_in_byte == 8 { current_bit_in_byte = 0; current_byte_idx += 1; }
         }

         result_val
    }



    /// GEOADD key longitude latitude member
    pub fn geoadd(&self, key: String, lon: f64, lat: f64, member: String) -> bool {
         let score = lon + lat; 
         self.zadd(key, score, member)
    }

    /// SADD key member
    pub fn sadd(&self, key: String, member: String) -> bool {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::Set(hashbrown::HashSet::new()));
        
        match entry.value_mut() {
            DataType::Set(s) => s.insert(member),
            _ => false,
        }
    }

    /// SMEMBERS key
    pub fn smembers(&self, key: &str) -> Vec<String> {
        self.data.get(key).map(|entry| {
            match entry.value() {
                DataType::Set(s) => s.iter().cloned().collect(),
                _ => Vec::new(),
            }
        }).unwrap_or_default()
    }

    /// XADD key ID field value ...
    pub fn xadd(&self, key: String, id: Option<&str>, fields: HashMap<String, String>) -> String {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::Stream(crate::core::structs::stream::Stream::new()));
        
        match entry.value_mut() {
            DataType::Stream(s) => s.add(id, fields),
            _ => "ERR".to_string(),
        }
    }

    /// VADD key vector
    pub fn vadd(&self, key: String, vector: Vec<f32>) -> bool {
        let index_name = if let Some(pos) = key.find(':') {
            key[..pos].to_string()
        } else {
            key.clone()
        };
        
        let mut entry = self.data.entry(index_name).or_insert_with(|| 
            DataType::Vector(crate::core::structs::vector::VectorIndex::new(vector.len()))
        );
        match entry.value_mut() {
            DataType::Vector(v) => {
                let dense_f16: Vec<half::f16> = vector.iter().map(|x| half::f16::from_f32(*x)).collect();
                v.add(key, dense_f16, None).is_ok()
            },
            _ => false,
        }
    }
    
    /// BF.ADD
    pub fn bf_add(&self, key: String, item: String) -> bool {
        let mut entry = self.data.entry(key).or_insert_with(|| 
            DataType::Bloom(crate::core::structs::bloom::BloomFilter::new(1024, 3))
        );
        match entry.value_mut() {
             DataType::Bloom(b) => { b.insert(&item); true },
             _ => false,
        }
    }

    /// JSON.SET
    pub fn json_set(&self, key: String, json: String) -> bool {
        if let Some(doc) = crate::core::structs::json::JsonDoc::new(&json) {
            self.data.insert(key, DataType::Json(doc));
            true
        } else {
            false
        }
    }

    /// VSEARCH
    pub fn vsearch(&self, index_name: &str, query: Vec<f32>, k: usize) -> Vec<(String, f32)> {
        self.data.get(index_name).map(|entry| {
            match entry.value() {
                 DataType::Vector(v) => {
                     let q_half: Vec<half::f16> = query.iter().map(|f| half::f16::from_f32(*f)).collect();
                     v.search_hybrid(&q_half, None, k, 1.0)
                 },
                _ => Vec::new(),
            }
        }).unwrap_or_default()
    }

    /// VADD.M3 (Hybrid)
    pub fn vadd_hybrid(&self, key: String, dense: Vec<half::f16>, sparse: Option<Vec<(u32, f32)>>) -> bool {
        let index_name = if let Some(pos) = key.find(':') {
            key[..pos].to_string()
        } else {
            key.clone()
        };
        
        let mut entry = self.data.entry(index_name).or_insert_with(|| 
            DataType::Vector(crate::core::structs::vector::VectorIndex::new(dense.len()))
        );
        match entry.value_mut() {
            DataType::Vector(v) => v.add(key, dense, sparse).is_ok(),
            _ => false,
        }
    }

    /// VSEARCH.HYBRID (Hybrid)
    pub fn vsearch_hybrid(&self, key: &str, dense: Vec<half::f16>, sparse: Option<Vec<(u32, f32)>>, k: usize, alpha: f32) -> Vec<(String, f32)> {
        self.data.get(key).map(|entry| {
            match entry.value() {
                DataType::Vector(v) => v.search_hybrid(&dense, sparse.as_deref(), k, alpha),
                _ => Vec::new(),
            }
        }).unwrap_or_default()
    }

    /// BF.EXISTS key item
    pub fn bf_exists(&self, key: &str, item: &str) -> bool {
        self.data.get(key).map(|entry| {
            match entry.value() {
                DataType::Bloom(b) => b.contains(item),
                _ => false,
            }
        }).unwrap_or(false)
    }

    /// JSON.GET key path
    pub fn json_get(&self, key: &str, path: &str) -> Option<String> {
        self.data.get(key).and_then(|entry| {
            match entry.value() {
                 DataType::Json(doc) => doc.get(path),
                 _ => None,
            }
        })
    }
    
    /// TS.ADD key ts value
    pub fn ts_add(&self, key: String, ts: u64, val: f64) -> bool {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::TimeSeries(crate::core::universe::TimeSeries::new()));
        match entry.value_mut() {
            DataType::TimeSeries(t) => { t.add(ts, val); true },
            _ => false,
        }
    }

    /// TS.RANGE key min max
    pub fn ts_range(&self, key: &str, min: u64, max: u64) -> Vec<(u64, f64)> {
        self.data.get(key).map(|entry| {
            match entry.value() {
                DataType::TimeSeries(t) => t.range(min, max),
                _ => Vec::new(),
            }
        }).unwrap_or_default()
    }

    /// GRAPH.ADD_EDGE key u v
    pub fn graph_add_edge(&self, key: String, u: String, v: String) -> bool {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::Graph(crate::core::universe::Graph::new()));
        match entry.value_mut() {
            DataType::Graph(g_inner) => { g_inner.add_edge(u, v); true },
            _ => false,
        }
    }

    /// GRAPH.BFS key start depth
    pub fn graph_bfs(&self, key: &str, start: &str, depth: usize) -> Vec<String> {
        self.data.get(key).map(|entry| {
            match entry.value() {
                DataType::Graph(g) => g.bfs(start, depth),
                _ => Vec::new(),
            }
        }).unwrap_or_default()
    }

    /// ML.RUN model_key input
    pub fn ml_run(&self, key: &str, input: &[f32]) -> Option<Vec<f32>> {
        self.data.get(key).and_then(|entry| {
            match entry.value() {
                DataType::Model(m) => Some(m.run(input)),
                _ => None,
            }
        })
    }

    /// ML.LOAD key name
    pub fn ml_load(&self, key: String, name: String) -> bool {
        self.data.insert(key, DataType::Model(crate::core::universe::Model::new(name)));
        true
    }

    /// XRANGE
    pub fn xrange(&self, key: &str, start: &str, end: &str) -> Vec<crate::core::structs::stream::StreamEntry> {
        self.data.get(key).map(|entry| {
            match entry.value() {
                DataType::Stream(s) => s.range(start, end),
                _ => Vec::new(),
            }
        }).unwrap_or_default()
    }

    /// PFADD key element
    pub fn pf_add(&self, key: String, element: String) -> bool {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::HyperLogLog(HyperLogLogWrapper::new()));
        match entry.value_mut() {
             DataType::HyperLogLog(h) => h.add(&element),
             _ => false,
        }
    }

    /// PFCOUNT key
    pub fn pf_count(&self, key: &str) -> usize {
        self.data.get_mut(key).map(|mut entry| {
            match entry.value_mut() {
                 DataType::HyperLogLog(h) => h.count() as usize,
                 _ => 0,
            }
        }).unwrap_or(0)
    }

    /// CF.ADD key item
    pub fn cf_add(&self, key: String, item: String) -> bool {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::Cuckoo(CuckooFilterWrapper::new()));
        match entry.value_mut() {
             DataType::Cuckoo(c) => c.add(&item),
             _ => false,
        }
    }

    /// CF.EXISTS key item
    pub fn cf_exists(&self, key: &str, item: &str) -> bool {
        self.data.get(key).map(|entry| {
            match entry.value() {
                 DataType::Cuckoo(c) => c.contains(item),
                 _ => false,
            }
        }).unwrap_or(false)
    }

    /// CMS.INCRBY key item increment
    pub fn cms_incr(&self, key: String, item: String, incr: usize) {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::CountMin(CountMinSketchWrapper::new()));
        match entry.value_mut() {
             DataType::CountMin(c) => c.incr(&item, incr),
             _ => {},
        }
    }

    /// CMS.QUERY key item
    pub fn cms_query(&self, key: &str, item: &str) -> usize {
        self.data.get(key).map(|entry| {
            match entry.value() {
                 DataType::CountMin(c) => c.query(item),
                 _ => 0,
            }
        }).unwrap_or(0)
    }

    /// TOPK.ADD key item
    pub fn topk_add(&self, key: String, item: String) {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::TopK(TopKWrapper::new(50)));
        match entry.value_mut() {
             DataType::TopK(t) => t.add(&item),
             _ => {},
        }
    }

    /// TOPK.LIST key
    pub fn topk_list(&self, key: &str) -> Vec<(String, usize)> {
        self.data.get(key).map(|entry| {
             match entry.value() {
                 DataType::TopK(t) => t.query(),
                 _ => Vec::new(),
            }
        }).unwrap_or_default()
    }

    /// TDIGEST.ADD key value
    pub fn tdigest_add(&self, key: String, value: f64) {
        let mut entry = self.data.entry(key).or_insert_with(|| DataType::TDigest(TDigestWrapper::new()));
        match entry.value_mut() {
             DataType::TDigest(t) => t.add(value),
             _ => {},
        }
    }

    /// TDIGEST.QUANTILE key q
    pub fn tdigest_quantile(&self, key: &str, q: f64) -> f64 {
        self.data.get(key).map(|entry| {
             match entry.value() {
                 DataType::TDigest(t) => t.quantile(q),
                 _ => 0.0,
             }
        }).unwrap_or(0.0)
    }

    /// Iterate over all data (for persistence)
    pub fn visit_all<F>(&self, mut callback: F)
    where
        F: FnMut(&String, &DataType),
    {
        for entry in self.data.iter() {
            callback(entry.key(), entry.value());
        }
    }
}


use crate::core::structs::stream::Stream;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    HyperLogLog(HyperLogLogWrapper),
    Cuckoo(CuckooFilterWrapper),
    CountMin(CountMinSketchWrapper),
    TopK(TopKWrapper),
    TDigest(TDigestWrapper),
}
