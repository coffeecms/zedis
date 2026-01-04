use std::time::Instant;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::SocketAddr;

// Token Bucket Implementation for Rate Limiting
pub struct TokenBucket {
    capacity: u32,
    tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: capacity as f64,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        
        // Refill tokens
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        self.last_refill = now;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub struct DdosGuard {
    // Map IP address to TokenBucket (using parking_lot for God Tier performance)
    limiters: Mutex<HashMap<std::net::IpAddr, (TokenBucket, Instant)>>,
    capacity: u32,
    rate: f64,
    cleanup_threshold: usize, // Max entries before cleanup
}

impl DdosGuard {
    pub fn new(capacity: u32, rate: f64) -> Self {
        Self {
            limiters: Mutex::new(HashMap::new()),
            capacity,
            rate,
            cleanup_threshold: 10000, // Cleanup when > 10k IPs tracked
        }
    }

    pub fn check_connection(&self, addr: &SocketAddr) -> bool {
        let ip = addr.ip();
        let now = Instant::now();
        let mut limiters = self.limiters.lock();
        
        // God Tier: Periodic cleanup to prevent memory leak from stale IPs
        if limiters.len() > self.cleanup_threshold {
            self.cleanup_stale_entries(&mut limiters, now);
        }

        let entry = limiters.entry(ip).or_insert_with(|| {
            (TokenBucket::new(self.capacity, self.rate), now)
        });
        
        // Update last access time
        entry.1 = now;
        entry.0.allow()
    }
    
    // Remove entries not accessed in the last 5 minutes
    fn cleanup_stale_entries(&self, limiters: &mut HashMap<std::net::IpAddr, (TokenBucket, Instant)>, now: Instant) {
        let stale_threshold = std::time::Duration::from_secs(300); // 5 minutes
        limiters.retain(|_, (_, last_access)| {
            now.duration_since(*last_access) < stale_threshold
        });
    }
}
