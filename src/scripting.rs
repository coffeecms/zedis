use mlua::{Lua, Result, IntoLua, Value, FromLua};
use std::sync::Arc;
use crate::core::storage::Db;
use crate::persistence::AofManager;

thread_local! {
    static LUA: Lua = Lua::new();
}

#[derive(Clone)]
pub struct ScriptEngine;

impl ScriptEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn eval(&self, script: &str, keys: Vec<String>, args: Vec<String>, db: Arc<Db>, aof: Arc<AofManager>) -> Result<String> {
        LUA.with(|lua| {
            let globals = lua.globals();
            
            // Inject KEYS and ARGV
            globals.set("KEYS", keys)?;
            globals.set("ARGV", args)?;

            // Register 'redis' module
            let redis = lua.create_table()?;
            
            let db_clone = db.clone();
            let aof_clone = aof.clone();
            
            // redis.call implementation
            // Note: In MVP, support basic GET/SET/INCR. Full dispatcher duplication is nice but verbosed.
            let call = lua.create_function(move |lua_ctx, args: mlua::MultiValue| -> Result<Value> {
                let vec: Vec<String> = args.iter().filter_map(|v| String::from_lua(v.clone(), lua_ctx).ok()).collect();
                if vec.is_empty() { return Ok(Value::Nil); }
                let cmd = vec[0].to_uppercase();
                
                // Sync dispatch
                match cmd.as_str() {
                    "GET" => {
                        if vec.len() < 2 { return Ok(Value::Nil); }
                        match db_clone.get_string(&vec[1]) {
                            Some(s) => return Ok(Value::String(lua_ctx.create_string(&s)?)),
                            None => return Ok(Value::Nil),
                        }
                    },
                    "SET" => {
                        if vec.len() < 3 { return Ok(Value::Nil); }
                        db_clone.set_string(vec[1].clone(), vec[2].clone());
                        let _ = aof_clone.append(&format!("SET {} {}", vec[1], vec[2]));
                        return Ok(Value::String(lua_ctx.create_string("OK")?));
                    },
                    "INCR" => {
                        if vec.len() < 2 { return Ok(Value::Nil); }
                        if let Ok(v) = db_clone.incr_by(vec[1].clone(), 1) {
                             let _ = aof_clone.append(&format!("INCR {}", vec[1]));
                             return Ok(Value::Integer(v));
                        }
                    },
                    // Add more mappings as needed (God Tier would map all)
                    _ => {
                        // Minimal fallback
                    }
                }
                
                Ok(Value::Nil)
            })?;
            
            redis.set("call", call)?;
            globals.set("redis", redis)?;

            match lua.load(script).eval::<Value>()? {
                 Value::String(s) => Ok(s.to_str()?.to_string()),
                 Value::Integer(i) => Ok(i.to_string()),
                 Value::Boolean(b) => Ok(b.to_string()),
                 _ => Ok("OK".to_string()), // Default/Nil
            }
        })
    }
    
    // Stub
    #[allow(dead_code)]
    pub fn register_api(&self, _db: Arc<Db>) {
    }
}

