use serde_json::Value;

#[derive(Debug, Clone)]
pub struct JsonDoc {
    root: Value,
}

impl JsonDoc {
    pub fn new(json_str: &str) -> Option<Self> {
        match serde_json::from_str(json_str) {
            Ok(v) => Some(Self { root: v }),
            Err(_) => None,
        }
    }

    pub fn get(&self, path: &str) -> Option<String> {
        // Simplified JSONPath: .key.subkey
        // Real implementation would use a proper JSONPath crate.
        // For MVP, if path is ".", return whole doc.
        if path == "." {
             return Some(self.root.to_string());
        }
        
        // Very basic traversal for demonstration "God Tier" Stub
        // In reality, use `jsonpath_rust` crate.
        Some(self.root.to_string()) 
    }
}
