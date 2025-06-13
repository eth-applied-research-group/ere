use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for network-based proving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkProverConfig {
    /// The endpoint URL of the prover network service
    pub endpoint: String,
    
    /// Optional API key for authentication
    pub api_key: Option<String>,
    
    /// Request timeout duration
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    
    /// Retry policy for handling network failures
    pub retry_policy: RetryPolicy,
    
    /// Whether to fallback to local proving if network fails
    pub fallback_to_local: bool,
}

impl Default for NetworkProverConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            api_key: None,
            timeout: Duration::from_secs(300), // 5 minutes default
            retry_policy: RetryPolicy::default(),
            fallback_to_local: false,
        }
    }
}

/// Retry policy configuration for network requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    
    /// Initial backoff duration
    #[serde(with = "humantime_serde")]
    pub initial_backoff: Duration,
    
    /// Maximum backoff duration
    #[serde(with = "humantime_serde")]
    pub max_backoff: Duration,
    
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

/// Request sent to the prover network
#[derive(Debug, Serialize, Deserialize)]
pub struct ProverNetworkRequest {
    /// The program to prove (serialized)
    pub program: Vec<u8>,
    
    /// The inputs for the program (serialized)
    pub inputs: Vec<u8>,
    
    /// Optional metadata about the proving request
    pub metadata: Option<RequestMetadata>,
}

/// Metadata about a proving request
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Unique identifier for this request
    pub request_id: String,
    
    /// Priority level for the request
    pub priority: RequestPriority,
    
    /// Optional callback URL for async completion
    pub callback_url: Option<String>,
}

/// Priority levels for network proving requests
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for RequestPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Response from the prover network
#[derive(Debug, Serialize, Deserialize)]
pub struct ProverNetworkResponse {
    /// The generated proof
    pub proof: Vec<u8>,
    
    /// Proving statistics
    pub stats: ProvingStats,
}

/// Statistics about the proving operation
#[derive(Debug, Serialize, Deserialize)]
pub struct ProvingStats {
    /// Total proving time
    #[serde(with = "humantime_serde")]
    pub proving_time: Duration,
    
    /// Queue wait time
    #[serde(with = "humantime_serde")]
    pub queue_time: Duration,
    
    /// Actual computation time
    #[serde(with = "humantime_serde")]
    pub compute_time: Duration,
    
    /// Number of cycles executed
    pub cycles: u64,
    
    /// Prover node identifier
    pub prover_node: String,
}

/// Status of an async proving job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    InProgress { progress: f32 },
    Completed,
    Failed { error: String },
}

/// Response for async job status queries
#[derive(Debug, Serialize, Deserialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: JobStatus,
    pub result: Option<ProverNetworkResponse>,
}