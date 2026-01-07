use std::collections::HashMap;


use serde::{Serialize, Deserialize};

// --- Z-TIER: Tiered Storage ---
pub struct TierManager {
    pub hot_threshold_hits: u64,
}

impl TierManager {
    pub fn new() -> Self { Self { hot_threshold_hits: 1000 } }
}

// --- Z-TIME: Time Series (Per Key) ---
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeSeries {
    // In a real Universe Tier, this would be a Gorilla-compressed blob
    samples: Vec<(u64, f64)>,
}

impl TimeSeries {
    pub fn new() -> Self { 
        Self { samples: Vec::new() } 
    }
    
    pub fn add(&mut self, ts: u64, val: f64) {
        self.samples.push((ts, val));
    }

    pub fn range(&self, min_ts: u64, max_ts: u64) -> Vec<(u64, f64)> {
        self.samples.iter().cloned()
            .filter(|(t, _)| *t >= min_ts && *t <= max_ts)
            .collect()
    }
}

// --- Z-GRAPH: Graph (Per Key) ---
use std::collections::VecDeque;
use std::collections::HashSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    // Adjacency List for THIS graph key
    adj: HashMap<String, Vec<String>>,
}

impl Graph {
    pub fn new() -> Self {
        Self { adj: HashMap::new() }
    }
    
    pub fn add_edge(&mut self, u: String, v: String) {
        self.adj.entry(u).or_insert_with(Vec::new).push(v);
    }
    
    pub fn bfs(&self, start: &str, max_depth: usize) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        if !self.adj.contains_key(start) { return result; }

        visited.insert(start.to_string());
        queue.push_back((start.to_string(), 0));

        while let Some((node, depth)) = queue.pop_front() {
            result.push(node.clone());
            if depth >= max_depth { continue; }

            if let Some(neighbors) = self.adj.get(&node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push_back((neighbor.clone(), depth + 1));
                    }
                }
            }
        }
        result
    }
}

// --- Z-ML: Model (Per Key) ---
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    // Stub: Model weights/bytes would be here
}

impl Model {
    pub fn new(name: String) -> Self {
         Self { name }
    }

    pub fn run(&self, input: &[f32]) -> Vec<f32> {
        // Stub Inference: Just scale input
        input.iter().map(|x| x * 1.5).collect()
    }
}
