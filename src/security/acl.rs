use hashbrown::HashMap;
use parking_lot::RwLock;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct User {
    pub name: String,
    pub password_hash: String,
    pub allowed_commands: Vec<String>, // Simplification for now, optimal would be a Bitmap
    pub allowed_keys: Vec<String>,     // Glob patterns
}

pub struct AclEngine {
    users: RwLock<HashMap<String, User>>,
}

impl AclEngine {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        // Default User
        users.insert(
            "default".to_string(),
            User {
                name: "default".to_string(),
                password_hash: "".to_string(), // No password by default
                allowed_commands: vec!["*".to_string()],
                allowed_keys: vec!["*".to_string()],
            },
        );
        Self {
            users: RwLock::new(users),
        }
    }

    pub fn check_permission(&self, username: &str, command: &str) -> bool {
        // God Tier: Use parking_lot RwLock (no poison, faster)
        let users = self.users.read();
        if let Some(user) = users.get(username) {
            // Check Command Permission (Simple Glob-like check)
            for pattern in &user.allowed_commands {
                if pattern == "*" || pattern.eq_ignore_ascii_case(command) {
                    return true;
                }
            }
        }
        false
    }
    
    // God Tier: Add user management methods for production
    #[allow(dead_code)]
    pub fn add_user(&self, name: String, password_hash: String, commands: Vec<String>, keys: Vec<String>) {
        let mut users = self.users.write();
        users.insert(name.clone(), User {
            name,
            password_hash,
            allowed_commands: commands,
            allowed_keys: keys,
        });
    }
    
    #[allow(dead_code)]
    pub fn remove_user(&self, name: &str) -> bool {
        let mut users = self.users.write();
        users.remove(name).is_some()
    }
}
