use crate::core::protocol::RespFrame;
use crate::core::storage::Db;
use crate::security::acl::AclEngine;
use crate::persistence::AofManager;
use std::sync::Arc;
use anyhow::Result;

use crate::scripting::ScriptEngine;
use crate::core::ai::BgeM3;


pub struct Dispatcher {
    db: Arc<Db>,
    acl: Arc<AclEngine>,
    aof: Arc<AofManager>,
    shadow_addr: Option<String>,
    script_engine: ScriptEngine,
    bge_model: Option<Arc<BgeM3>>,
}


impl Dispatcher {
    pub fn new(db: Arc<Db>, aof: Arc<AofManager>, shadow_addr: Option<String>, bge_model: Option<Arc<BgeM3>>) -> Self {
        Self { 
            db,
            acl: Arc::new(AclEngine::new()),
            aof,
            shadow_addr,
            script_engine: ScriptEngine::new(),
            bge_model,
        }

    }

    pub async fn execute(&self, frame: RespFrame) -> Result<RespFrame> {
        match frame {
            RespFrame::Array(Some(frames)) => {
                if frames.is_empty() {
                    return Ok(RespFrame::Error("ERR empty command".to_string()));
                }

                // Parse command name
                let cmd_name = match &frames[0] {
                    RespFrame::BulkString(Some(s)) => s.to_uppercase(),
                    RespFrame::SimpleString(s) => s.to_uppercase(),
                    _ => return Ok(RespFrame::Error("ERR invalid command format".to_string())),
                };

                // ACL Check
                if !self.acl.check_permission("default", &cmd_name) {
                     return Ok(RespFrame::Error(format!("NOPERM this user has no permissions to run the '{}' command", cmd_name)));
                }

use crate::persistence::Persistence;

                match cmd_name.as_str() {
                    "GET" => self.handle_get(&frames).await,
                    "DEL" => self.handle_del(&frames).await,
                    "EXISTS" => self.handle_exists(&frames).await,
                    "TTL" => self.handle_ttl(&frames).await,
                    "INCR" => self.handle_incr(&frames).await,
                    "INCRBY" => self.handle_incrby(&frames).await,
                    "SET" => {
                         // Shadow Mode: Fire and Forget
                         if let Some(addr) = &self.shadow_addr {
                             let addr = addr.clone();
                             // Quick & dirty serialization re-use is hard without 'encode' returning bytes, 
                             // but we can just forward valid frames in a real impl. 
                             // For MVP, we spawn a task to just log it or connect.
                             // tokio::spawn(async move { ... });
                             // Currently just a placeholder log to prove architectural capability.
                             log::info!("Shadow Mode: Mirroring SET to {}", addr);
                         }
                         self.handle_set(&frames).await
                    },
                    "SETEX" => self.handle_setex(&frames).await,
                    "RPUSH" => self.handle_rpush(&frames).await,
                    "LPOP" => self.handle_lpop(&frames).await,
                    "LRANGE" => self.handle_lrange(&frames).await,
                    "HSET" => self.handle_hset(&frames).await,
                    "HGET" => self.handle_hget(&frames).await,
                    "ZADD" => self.handle_zadd(&frames).await,
                    "ZRANGE" => self.handle_zrange(&frames).await,
                    "BITCOUNT" => self.handle_bitcount(&frames).await,
                    "GEOADD" => self.handle_geoadd(&frames).await,
                    "XADD" => self.handle_xadd(&frames).await,
                    "XRANGE" => self.handle_xrange(&frames).await,
                    "SADD" => self.handle_sadd(&frames).await,
                    "SMEMBERS" => self.handle_smembers(&frames).await,
                    "VADD" => self.handle_vadd(&frames).await,
                    "VADD.TEXT" => self.handle_vadd_text(&frames).await,
                    "VADD.M3" => self.handle_vadd_m3(&frames).await,
                    "VSEARCH" => self.handle_vsearch(&frames).await,
                    "VSEARCH.TEXT" => self.handle_vsearch_text(&frames).await,
                    "VSEARCH.HYBRID" => self.handle_vsearch_hybrid(&frames).await,
                    "BF.ADD" => self.handle_bfadd(&frames).await,
                    "BF.EXISTS" => self.handle_bfexists(&frames).await,
                    "JSON.SET" => self.handle_jsonset(&frames).await,
                    "JSON.GET" => self.handle_jsonget(&frames).await,
                    "TS.ADD" => self.handle_tsadd(&frames).await,
                    "TS.RANGE" => self.handle_tsrange(&frames).await,
                    "GRAPH.ADD" => self.handle_graphadd(&frames).await,
                    "GRAPH.BFS" => self.handle_graphbfs(&frames).await,
                    "ML.LOAD" => self.handle_mlload(&frames).await,
                    "ML.RUN" => self.handle_mlrun(&frames).await,
                    "EVAL" => self.handle_eval(&frames).await,
                    "SAVE" => {
                        // Blocking save for now
                        match Persistence::save_rdb(&self.db, "dump.rdb") {
                            Ok(_) => Ok(RespFrame::SimpleString("OK".to_string())),
                            Err(e) => Ok(RespFrame::Error(format!("ERR save failed: {}", e))),
                        }
                    }
                    "PING" => Ok(RespFrame::SimpleString("PONG".to_string())),
                    _ => Ok(RespFrame::Error(format!("ERR unknown command '{}'", cmd_name))),
                }
            }
            _ => Ok(RespFrame::Error("ERR request must be an array".to_string())),
        }
    }

    async fn handle_get(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 2 {
            return Ok(RespFrame::Error("ERR wrong number of arguments for 'get' command".to_string()));
        }

        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k,
            RespFrame::SimpleString(k) => k,
            _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        match self.db.get_string(key) {
            Some(val) => Ok(RespFrame::BulkString(Some(val))),
            None => Ok(RespFrame::BulkString(None)), // Null bulk string for miss
        }
    }

    async fn handle_ttl(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 2 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k, _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        
        let ttl = self.db.get_ttl(key);
        Ok(RespFrame::Integer(ttl))
    }

    async fn handle_del(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() < 2 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let mut count = 0;
        for i in 1..frames.len() {
            let key = match &frames[i] { RespFrame::BulkString(Some(k)) => k, _ => continue };
            if self.db.del(key) {
                 if let Err(e) = self.aof.append(&format!("DEL {}", key)) { log::error!("AOF error: {}", e); }
                 count += 1;
            }
        }
        Ok(RespFrame::Integer(count))
    }

    async fn handle_exists(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() < 2 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let mut count = 0;
        for i in 1..frames.len() {
            let key = match &frames[i] { RespFrame::BulkString(Some(k)) => k, _ => continue };
            if self.db.exists(key) { count += 1; }
        }
        Ok(RespFrame::Integer(count))
    }

    async fn handle_incr(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 2 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };

        match self.db.incr_by(key.clone(), 1) {
            Ok(val) => {
                 if let Err(e) = self.aof.append(&format!("INCR {}", key)) { log::error!("AOF error: {}", e); }
                 Ok(RespFrame::Integer(val))
            },
            Err(e) => Ok(RespFrame::Error(e)),
        }
    }

    async fn handle_incrby(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let by_val = match &frames[2] { 
            RespFrame::BulkString(Some(s)) => s.parse::<i64>().unwrap_or(0), 
            RespFrame::Integer(i) => *i,
            _ => return Ok(RespFrame::Error("ERR value not integer".to_string())) 
        };

        match self.db.incr_by(key.clone(), by_val) {
            Ok(val) => {
                 if let Err(e) = self.aof.append(&format!("INCRBY {} {}", key, by_val)) { log::error!("AOF error: {}", e); }
                 Ok(RespFrame::Integer(val))
            },
            Err(e) => Ok(RespFrame::Error(e)),
        }
    }

    async fn handle_set(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 3 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'set' command".to_string()));
        }

        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
            RespFrame::SimpleString(k) => k.to_string(),
            _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        let val = match &frames[2] {
            RespFrame::BulkString(Some(v)) => v.to_string(),
            RespFrame::SimpleString(v) => v.to_string(),
            RespFrame::Integer(i) => i.to_string(),
            _ => return Ok(RespFrame::Error("ERR invalid value".to_string())),
        };

    
    // AOF Log
        if let Err(e) = self.aof.append(&format!("SET {} {}", key, val)) {
             log::error!("Failed to append to AOF: {}", e);
        }

        self.db.set_string(key, val);
        Ok(RespFrame::SimpleString("OK".to_string()))
    }

    async fn handle_setex(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        // SETEX key seconds value
        if frames.len() != 4 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'setex' command".to_string()));
        }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let _seconds = match &frames[2] { RespFrame::BulkString(Some(s)) => s, _ => return Ok(RespFrame::Error("ERR seconds".to_string())) }; 
        let val = match &frames[3] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR value".to_string())) };

        if let Err(e) = self.aof.append(&format!("SET {} {}", key, val)) {
             log::error!("AOF error: {}", e);
        }
        self.db.set_string(key, val);
        Ok(RespFrame::SimpleString("OK".to_string()))
    }

    async fn handle_rpush(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() < 3 {
            return Ok(RespFrame::Error("ERR wrong number of arguments for 'rpush' command".to_string()));
        }

        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
            RespFrame::SimpleString(k) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        let mut count = 0;
        for i in 2..frames.len() {
            let val = match &frames[i] {
                RespFrame::BulkString(Some(v)) => v.to_string(),
                RespFrame::SimpleString(v) => v.to_string(),
                RespFrame::Integer(n) => n.to_string(),
                _ => continue,
            };
            count = self.db.list_push(key.clone(), val);
        }

        Ok(RespFrame::Integer(count as i64))
    }

    async fn handle_lpop(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 2 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        
        match self.db.list_pop(&key) {
            Some(v) => Ok(RespFrame::BulkString(Some(v))),
            None => Ok(RespFrame::BulkString(None)),
        }
    }

    async fn handle_lrange(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 4 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        
        let start = match &frames[2] { 
            RespFrame::BulkString(Some(s)) => s.parse::<i64>().unwrap_or(0), 
            RespFrame::Integer(i) => *i,
            _ => 0 
        };
        let stop = match &frames[3] { 
            RespFrame::BulkString(Some(s)) => s.parse::<i64>().unwrap_or(0), 
            RespFrame::Integer(i) => *i,
            _ => 0 
        };

        let items = self.db.list_range(&key, start, stop);
        let resp = items.into_iter().map(|s| RespFrame::BulkString(Some(s))).collect();
        Ok(RespFrame::Array(Some(resp)))
    }

    async fn handle_hset(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        // HSET key f1 v1 [f2 v2 ...]
        if frames.len() < 4 || frames.len() % 2 != 0 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'hset' command".to_string()));
        }
        
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        let mut count = 0;
        let mut i = 2;
        while i < frames.len() {
             let field = match &frames[i] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR field".to_string())) };
             let value = match &frames[i+1] { RespFrame::BulkString(Some(v)) => v.to_string(), _ => return Ok(RespFrame::Error("ERR value".to_string())) };
             
             count += self.db.hash_set(key.clone(), field, value);
             i += 2;
        }

        Ok(RespFrame::Integer(count as i64))
    }

    async fn handle_hget(&self, frames: &[RespFrame]) -> Result<RespFrame> {
         if frames.len() != 3 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'hget' command".to_string()));
        }

        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        let field = match &frames[2] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid field".to_string())),
        };

        match self.db.hash_get(&key, &field) {
            Some(val) => Ok(RespFrame::BulkString(Some(val))),
            None => Ok(RespFrame::BulkString(None)),
        }
    }

    async fn handle_zadd(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        // ZADD key score member [score member ...]
        if frames.len() < 4 || (frames.len() - 2) % 2 != 0 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'zadd' command".to_string()));
        }
        
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        let mut added_count = 0;
        let mut i = 2;
        while i < frames.len() {
             let score_str = match &frames[i] { RespFrame::BulkString(Some(s)) => s, _ => return Ok(RespFrame::Error("ERR score".to_string())) };
             let score = match score_str.parse::<f64>() { Ok(s) => s, Err(_) => return Ok(RespFrame::Error("ERR float".to_string())) };
             let member = match &frames[i+1] { RespFrame::BulkString(Some(m)) => m.to_string(), _ => return Ok(RespFrame::Error("ERR member".to_string())) };
             
             if self.db.zadd(key.clone(), score, member) { added_count += 1; }
             i += 2;
        }

        Ok(RespFrame::Integer(added_count))
    }

    async fn handle_zrange(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 4 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'zrange' command".to_string()));
        }
        // ZRANGE key start stop
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        let start = match &frames[2] {
            RespFrame::BulkString(Some(s)) => s.parse::<usize>().unwrap_or(0),
             _ => 0,
        };

        let end = match &frames[3] {
            RespFrame::BulkString(Some(s)) => s.parse::<usize>().unwrap_or(0),
             _ => 0,
        };

        let result = self.db.zrange(&key, start, end);
        let resp_array = result.into_iter()
            .map(|s| RespFrame::BulkString(Some(s)))
            .collect();
        
        Ok(RespFrame::Array(Some(resp_array)))
    }

    async fn handle_bitcount(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 2 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'bitcount' command".to_string()));
        }
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };
        let count = self.db.bitcount(&key);
        Ok(RespFrame::Integer(count as i64))
    }

    async fn handle_geoadd(&self, frames: &[RespFrame]) -> Result<RespFrame> {
         // GEOADD key lon lat member [lon lat member ...]
        if frames.len() < 5 || (frames.len() - 2) % 3 != 0 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'geoadd' command".to_string()));
        }
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        let mut count = 0;
        let mut i = 2;
        while i < frames.len() {
             let lon_s = match &frames[i] { RespFrame::BulkString(Some(s)) => s, _ => return Ok(RespFrame::Error("ERR lon".to_string())) };
             let lat_s = match &frames[i+1] { RespFrame::BulkString(Some(s)) => s, _ => return Ok(RespFrame::Error("ERR lat".to_string())) };
             let member = match &frames[i+2] { RespFrame::BulkString(Some(m)) => m.to_string(), _ => return Ok(RespFrame::Error("ERR member".to_string())) };
             
             let lon = lon_s.parse::<f64>().unwrap_or(0.0);
             let lat = lat_s.parse::<f64>().unwrap_or(0.0);
             
             self.db.geoadd(key.clone(), lon, lat, member);
             count += 1; 

             i += 3;
        }
        Ok(RespFrame::Integer(count))
    }

    async fn handle_xadd(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() < 4 { // XADD key ID field value
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'xadd' command".to_string()));
        }
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };
        let id_arg = match &frames[2] {
            RespFrame::BulkString(Some(s)) => s.to_string(),
             _ => "*".to_string(), // Default to auto
        };

        let mut fields = hashbrown::HashMap::new();
        let mut i = 3;
        while i < frames.len() {
            let f = match &frames[i] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => break };
            let v = if i + 1 < frames.len() {
                match &frames[i+1] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => "".to_string() }
            } else { "".to_string() };
            fields.insert(f, v);
            i += 2;
        }

        let new_id = self.db.xadd(key, Some(&id_arg), fields);
        Ok(RespFrame::BulkString(Some(new_id)))
    }

    async fn handle_xrange(&self, frames: &[RespFrame]) -> Result<RespFrame> {
         if frames.len() != 4 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'xrange' command".to_string()));
        }
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };
        let start = match &frames[2] {
            RespFrame::BulkString(Some(s)) => s.to_string(),
             _ => "-".to_string(),
        };
        let end = match &frames[3] {
            RespFrame::BulkString(Some(s)) => s.to_string(),
             _ => "+".to_string(),
        };

        // Simplified response: just return IDs for MVP or JSON-like
        let entries = self.db.xrange(&key, &start, &end);
        
        // Serialize manually to Array of Arrays
        let mut arr = Vec::new();
        for e in entries {
            let mut entry_arr = Vec::new();
            entry_arr.push(RespFrame::BulkString(Some(e.id)));
            let mut fields_arr = Vec::new();
            for (k, v) in e.fields {
                fields_arr.push(RespFrame::BulkString(Some(k)));
                fields_arr.push(RespFrame::BulkString(Some(v)));
            }
            entry_arr.push(RespFrame::Array(Some(fields_arr)));
            arr.push(RespFrame::Array(Some(entry_arr)));
        }

        Ok(RespFrame::Array(Some(arr)))
    }

    async fn handle_eval(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() < 2 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'eval' command".to_string()));
        }
        let script = match &frames[1] {
            RespFrame::BulkString(Some(s)) => s.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid script".to_string())),
        };
        
        match self.script_engine.eval(&script, vec![], vec![]) {
             Ok(res) => Ok(RespFrame::BulkString(Some(res))),
             Err(e) => Ok(RespFrame::Error(format!("ERR script error: {}", e))),
        }
    }

    async fn handle_sadd(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() < 3 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'sadd' command".to_string()));
        }
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };

        let mut count = 0;
        for i in 2..frames.len() {
             let member = match &frames[i] { RespFrame::BulkString(Some(m)) => m.to_string(), _ => continue };
             if self.db.sadd(key.clone(), member) {
                 count += 1;
             }
        }
        Ok(RespFrame::Integer(count))
    }

    async fn handle_smembers(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 2 {
             return Ok(RespFrame::Error("ERR wrong number of arguments for 'smembers' command".to_string()));
        }
        let key = match &frames[1] {
            RespFrame::BulkString(Some(k)) => k.to_string(),
             _ => return Ok(RespFrame::Error("ERR invalid key".to_string())),
        };
        
        let members = self.db.smembers(&key);
        let resp = members.into_iter().map(|m| RespFrame::BulkString(Some(m))).collect();
        Ok(RespFrame::Array(Some(resp)))
    }
    async fn handle_vadd(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() < 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        
        let mut vector = Vec::new();
        for i in 2..frames.len() {
             if let RespFrame::BulkString(Some(s)) = &frames[i] {
                 if let Ok(f) = s.parse::<f32>() { vector.push(f); }
             }
        }
        
        // MVP: vector must be f32s. 
        let ok = self.db.vadd(key, vector);
        Ok(RespFrame::Integer(if ok { 1 } else { 0 }))
    }

    async fn handle_bfadd(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let item = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR item".to_string())) };
        
        let ok = self.db.bf_add(key, item);
        Ok(RespFrame::Integer(if ok { 1 } else { 0 }))
    }
    
    async fn handle_jsonset(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let json = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR json".to_string())) };
        
        let ok = self.db.json_set(key, json);
        Ok(RespFrame::SimpleString(if ok { "OK".to_string() } else { "ERR".to_string() }))
    }
    async fn handle_vadd_text(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        // VADD.TEXT key text - Auto-embed text and store as hybrid vector
        if frames.len() != 3 { return Ok(RespFrame::Error("ERR wrong number of arguments for 'VADD.TEXT' command".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR invalid key".to_string())) };
        let text = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR invalid text".to_string())) };
        
        if let Some(model) = &self.bge_model {
             match model.embed_hybrid(&text) {
                 Ok((dense, sparse)) => {
                     let ok = self.db.vadd_hybrid(key, dense, Some(sparse));
                     Ok(RespFrame::Integer(if ok { 1 } else { 0 }))
                 },
                 Err(e) => Ok(RespFrame::Error(format!("ERR embedding failed: {}", e)))
             }
        } else {
             Ok(RespFrame::Error("ERR BGE-M3 model not loaded. Please ensure 'bge-m3' directory exists with model files.".to_string()))
        }
    }

    async fn handle_vsearch_text(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        // VSEARCH.TEXT index_prefix query k - Embed query and search for similar documents
        if frames.len() != 4 { return Ok(RespFrame::Error("ERR wrong number of arguments for 'VSEARCH.TEXT' command".to_string())); }
        let index_prefix = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR invalid index".to_string())) };
        let query = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR invalid query".to_string())) };
        let k = match &frames[3] { RespFrame::BulkString(Some(s)) => s.parse::<usize>().unwrap_or(1), _ => 1 };
        
        if let Some(model) = &self.bge_model {
             match model.embed_hybrid(&query) {
                 Ok((dense, sparse)) => {
                     // Search for similar vectors (use default alpha of 0.7 for hybrid search)
                     let results = self.db.vsearch_hybrid(&index_prefix, dense, Some(sparse), k, 0.7);
                     let mut resp = Vec::new();
                     for (id, score) in results {
                         resp.push(RespFrame::BulkString(Some(id)));
                         resp.push(RespFrame::SimpleString(format!("{:.4}", score)));
                     }
                     Ok(RespFrame::Array(Some(resp)))
                 },
                 Err(e) => Ok(RespFrame::Error(format!("ERR embedding failed: {}", e)))
             }
        } else {
             Ok(RespFrame::Error("ERR BGE-M3 model not loaded. Please ensure 'bge-m3' directory exists with model files.".to_string()))
        }
    }

    async fn handle_tsadd(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 4 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let ts = match &frames[2] { RespFrame::BulkString(Some(s)) => s.parse::<u64>().unwrap_or(0), _ => return Ok(RespFrame::Error("ERR ts".to_string())) };
        let val = match &frames[3] { RespFrame::BulkString(Some(s)) => s.parse::<f64>().unwrap_or(0.0), _ => return Ok(RespFrame::Error("ERR val".to_string())) };
        
        self.db.ts_add(key, ts, val);
        Ok(RespFrame::Integer(1)) 
    }

    async fn handle_tsrange(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 4 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let min = match &frames[2] { RespFrame::BulkString(Some(s)) => s.parse::<u64>().unwrap_or(0), _ => 0 };
        let max = match &frames[3] { RespFrame::BulkString(Some(s)) => s.parse::<u64>().unwrap_or(u64::MAX), _ => u64::MAX };

        let result = self.db.ts_range(&key, min, max);
        let mut arr = Vec::new();
        for (t, v) in result {
             let mut sample = Vec::new();
             sample.push(RespFrame::Integer(t as i64));
             sample.push(RespFrame::SimpleString(v.to_string()));
             arr.push(RespFrame::Array(Some(sample)));
        }
        Ok(RespFrame::Array(Some(arr)))
    }

    async fn handle_graphadd(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 4 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let u = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR u".to_string())) };
        let v = match &frames[3] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR v".to_string())) };
        
        self.db.graph_add_edge(key, u, v);
        Ok(RespFrame::Integer(1)) 
    }

    async fn handle_graphbfs(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 4 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let start = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR start".to_string())) };
        let depth = match &frames[3] { RespFrame::BulkString(Some(s)) => s.parse::<usize>().unwrap_or(1), _ => 1 };
        
        let nodes = self.db.graph_bfs(&key, &start, depth);
        let resp = nodes.into_iter().map(|n| RespFrame::BulkString(Some(n))).collect();
        Ok(RespFrame::Array(Some(resp)))
    }

    async fn handle_mlload(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let name = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR name".to_string())) };
        
        self.db.ml_load(key, name);
        Ok(RespFrame::SimpleString("OK".to_string()))
    }

    async fn handle_mlrun(&self, frames: &[RespFrame]) -> Result<RespFrame> {
         if frames.len() < 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
         let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
         
         let mut input = Vec::new();
         for i in 2..frames.len() {
             if let RespFrame::BulkString(Some(s)) = &frames[i] {
                 if let Ok(f) = s.parse::<f32>() { input.push(f); }
             }
         }

         match self.db.ml_run(&key, &input) {
             Some(res) => {
                 let arr: Vec<RespFrame> = res.into_iter().map(|f| RespFrame::SimpleString(f.to_string())).collect();
                 Ok(RespFrame::Array(Some(arr)))
             },
             None => Ok(RespFrame::Error("ERR no model or run fail".to_string())),
         }
    }


    async fn handle_vsearch(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        // VSEARCH key 1.0 2.0 ... K
        if frames.len() < 4 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        
        let k_arg = frames.last().unwrap(); // Simplification: last arg is K
        let k = match k_arg { RespFrame::BulkString(Some(s)) => s.parse::<usize>().unwrap_or(1), _ => 1 };

        let mut vector = Vec::new();
        for i in 2..frames.len()-1 {
             if let RespFrame::BulkString(Some(s)) = &frames[i] {
                 if let Ok(f) = s.parse::<f32>() { vector.push(f); }
             }
        }
        
        let results = self.db.vsearch(&key, vector, k);
        // Return Array of [Key, Score]
        let mut resp = Vec::new();
        for (id, score) in results {
            resp.push(RespFrame::BulkString(Some(id)));
            resp.push(RespFrame::SimpleString(score.to_string()));
        }
        Ok(RespFrame::Array(Some(resp)))
    }

    async fn handle_bfexists(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let item = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR item".to_string())) };
        
        let exists = self.db.bf_exists(&key, &item);
        Ok(RespFrame::Integer(if exists { 1 } else { 0 }))
    }

    async fn handle_jsonget(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        if frames.len() != 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let path = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR path".to_string())) };
        
        match self.db.json_get(&key, &path) {
            Some(v) => Ok(RespFrame::BulkString(Some(v))),
            None => Ok(RespFrame::BulkString(None)),
        }
    }
    async fn handle_vadd_m3(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        // VADD.M3 key text
        if frames.len() != 3 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let text = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR text".to_string())) };

        if let Some(model) = &self.bge_model {
             match model.embed_hybrid(&text) {
                 Ok((dense, sparse)) => {
                     let ok = self.db.vadd_hybrid(key, dense, Some(sparse));
                     Ok(RespFrame::Integer(if ok { 1 } else { 0 }))
                 },
                 Err(e) => Ok(RespFrame::Error(format!("ERR embedding error: {}", e)))
             }
        } else {
             Ok(RespFrame::Error("ERR BGE-M3 model not loaded".to_string()))
        }
    }

    async fn handle_vsearch_hybrid(&self, frames: &[RespFrame]) -> Result<RespFrame> {
        // VSEARCH.HYBRID key query k alpha
        if frames.len() < 4 { return Ok(RespFrame::Error("ERR args".to_string())); }
        let key = match &frames[1] { RespFrame::BulkString(Some(k)) => k.to_string(), _ => return Ok(RespFrame::Error("ERR key".to_string())) };
        let query = match &frames[2] { RespFrame::BulkString(Some(s)) => s.to_string(), _ => return Ok(RespFrame::Error("ERR query".to_string())) };
        let k = match &frames[3] { RespFrame::BulkString(Some(s)) => s.parse::<usize>().unwrap_or(1), _ => 1 };
        let alpha = if frames.len() > 4 {
             match &frames[4] { RespFrame::BulkString(Some(s)) => s.parse::<f32>().unwrap_or(0.5), _ => 0.5 }
        } else { 0.5 };

        if let Some(model) = &self.bge_model {
             match model.embed_hybrid(&query) {
                 Ok((dense, sparse)) => {
                     let results = self.db.vsearch_hybrid(&key, dense, Some(sparse), k, alpha);
                     let mut resp = Vec::new();
                     for (id, score) in results {
                         resp.push(RespFrame::BulkString(Some(id)));
                         resp.push(RespFrame::SimpleString(score.to_string()));
                     }
                     Ok(RespFrame::Array(Some(resp)))
                 },
                 Err(e) => Ok(RespFrame::Error(format!("ERR embedding error: {}", e)))
             }
        } else {
             Ok(RespFrame::Error("ERR BGE-M3 model not loaded".to_string()))
        }
    }
}
