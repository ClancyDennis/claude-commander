use serde::{Deserialize, Serialize};

/// Configuration for pipeline behavior
#[derive(Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    // P-Thread: Agent Pool
    pub use_agent_pool: bool,
    pub pool_priority: String,

    // B-Thread: Orchestration
    pub enable_orchestration: bool,
    pub auto_decompose: bool,
    pub max_parallel_tasks: usize,

    // F-Thread: Verification
    pub enable_verification: bool,
    pub verification_strategy: Option<String>, // "majority", "weighted", "meta", "first"
    pub verification_n: usize,                 // Number of agents for verification (default: 3)
    pub confidence_threshold: f32,

    // C-Thread: Checkpoints
    pub require_plan_review: bool,          // Skip plan review checkpoint?
    pub require_final_review: bool,         // Skip final review checkpoint?
    pub auto_validation_command: String,
    pub auto_approve_on_verification: bool, // Auto-approve if verification passes?
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            use_agent_pool: false,
            pool_priority: "normal".to_string(),
            enable_orchestration: false,
            auto_decompose: false,
            max_parallel_tasks: 3,
            enable_verification: false,
            verification_strategy: None,
            verification_n: 1,
            confidence_threshold: 0.8,
            require_plan_review: true,
            require_final_review: true,
            auto_validation_command: "cargo check".to_string(),
            auto_approve_on_verification: false,
        }
    }
}
