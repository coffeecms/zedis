#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub host: String,
    #[allow(dead_code)]
    pub worker_threads: usize,
    pub shadow_addr: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 6379,
            host: "127.0.0.1".to_string(),
            worker_threads: num_cpus::get(),
            shadow_addr: None, // e.g., Some("127.0.0.1:6380".to_string())
        }
    }
}
