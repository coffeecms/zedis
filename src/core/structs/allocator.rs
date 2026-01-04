use slab::Slab;

// God Tier Optimization: Custom Slab Allocator
// Used for high-frequency small object allocation (like list nodes or stream entries)
// to reduce allocator fragmentation and improve cache locality.

#[allow(dead_code)]
pub struct ZedisSlab<T> {
    slab: Slab<T>,
}

#[allow(dead_code)]
impl<T> ZedisSlab<T> {
    pub fn new() -> Self {
        Self {
            slab: Slab::with_capacity(1024), // Pre-allocate hot page
        }
    }

    pub fn alloc(&mut self, val: T) -> usize {
        self.slab.insert(val)
    }

    pub fn get(&self, id: usize) -> Option<&T> {
        self.slab.get(id)
    }
    
    pub fn remove(&mut self, id: usize) -> Option<T> {
        if self.slab.contains(id) {
             Some(self.slab.remove(id))
        } else {
             None
        }
    }
}
