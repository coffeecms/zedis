use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ZFlowConfig {
    #[serde(default)]
    pub flow: Vec<FlowItem>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct FlowItem {
    pub name: String,
    pub source: String, // Connection String
    pub table: String,
    pub target: FlowTarget,
    
    // Common
    #[serde(default = "default_interval")]
    pub interval: u64, // Seconds
    pub key_format: Option<String>,
    
    // Vector / JSON
    pub mapping: Option<String>,
    pub fields: Option<Vec<String>>,
    
    // Bloom
    pub item: Option<String>,
    pub error_rate: Option<f64>,
    pub key: Option<String>, // Redis Key for Global Structures (Bloom, Geo, Graph)

    // Time Series
    pub timestamp: Option<String>,
    pub value: Option<String>,

    // Graph
    pub source_node: Option<String>,
    pub destination_node: Option<String>,
    pub relation_label: Option<String>,
    pub graph_key: Option<String>,

    // Geo
    pub member: Option<String>,
    pub lat: Option<String>,
    pub lon: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FlowTarget {
    Json,
    Vector,
    Bloom,
    TimeSeries,
    Graph,
    Geo,
}

fn default_interval() -> u64 {
    5
}
