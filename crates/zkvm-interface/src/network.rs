use serde::{Deserialize, Serialize};

/// Configuration for network-based proving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProverConfig {
    /// The endpoint URL of the prover network service
    pub endpoint: String,

    /// Optional API key for authentication
    pub api_key: Option<String>,
}

impl Default for NetworkProverConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            api_key: None,
        }
    }
}
