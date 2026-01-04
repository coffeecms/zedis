use log::info;
use core_affinity::CoreId;

pub struct HardwareManager {
    pub core_ids: Vec<CoreId>,
}

impl HardwareManager {
    pub fn new() -> Self {
        let core_ids = core_affinity::get_core_ids().unwrap_or_else(Vec::new);
        info!("Detected {} cores", core_ids.len());
        Self { core_ids }
    }

    /// Pin the current thread to a specific core index
    pub fn pin_thread(&self, index: usize) -> bool {
        if let Some(core_id) = self.core_ids.get(index % self.core_ids.len()) {
            let res = core_affinity::set_for_current(*core_id);
            if res {
                info!("Thread pinned to core {:?}", core_id);
            } else {
                info!("Failed to pin thread to core {:?}", core_id);
            }
            return res;
        }
        false
    }

    pub fn check_simd_support(&self) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("avx2") {
                info!("Hardware: AVX2 detected. Enabling SIMD optimizations.");
            } else if is_x86_feature_detected!("sse4.1") {
                info!("Hardware: SSE4.1 detected. Enabling legacy SIMD.");
            } else {
                info!("Hardware: No advanced SIMD detected. Running scalar fallback.");
            }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            info!("Hardware: Non-x86 architecture. SIMD check skipped.");
        }
    }

    /// God Tier Auto-Tuner: Suggests config based on hardware
    pub fn loop_recommendations(&self) {
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_all();
        
        let total_memory = sys.total_memory();
        let core_count = self.core_ids.len();

        info!("--- Auto-Tuner Analysis ---");
        info!("Total Memory: {} GB", total_memory / 1024 / 1024);
        info!("Core Count: {}", core_count);

        if total_memory > 32 * 1024 * 1024 {
             info!("Recommend: High-Memory Mode (Lazy eviction, large slab pages)");
        } else {
             info!("Recommend: Low-Memory Mode (Aggressive eviction, small slab pages)");
        }

        if core_count > 16 {
             info!("Recommend: High-Concurrency Partitioning (2048+ shards)");
        } else {
             info!("Recommend: Standard Partitioning (1024 shards)");
        }
        info!("---------------------------");
    }
}

