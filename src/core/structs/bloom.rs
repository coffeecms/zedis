use bit_vec::BitVec;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Basic Bloom Filter
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: BitVec,
    k_hashes: u32,
}

impl BloomFilter {
    pub fn new(size: usize, k: u32) -> Self {
        Self {
            bits: BitVec::from_elem(size, false),
            k_hashes: k,
        }
    }

    fn get_hashes(&self, item: &str) -> Vec<usize> {
        let mut hashes = Vec::with_capacity(self.k_hashes as usize);
        for i in 0..self.k_hashes {
            let mut hasher = DefaultHasher::new();
            item.hash(&mut hasher);
            i.hash(&mut hasher); // Salt with index
            hashes.push((hasher.finish() as usize) % self.bits.len());
        }
        hashes
    }

    pub fn insert(&mut self, item: &str) {
        for hash in self.get_hashes(item) {
            self.bits.set(hash, true);
        }
    }

    pub fn contains(&self, item: &str) -> bool {
        for hash in self.get_hashes(item) {
            if !self.bits.get(hash).unwrap_or(false) {
                return false;
            }
        }
        true
    }
}
