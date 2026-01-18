use std::sync::Arc;
use tokio::sync::Mutex;

use crate::pipeline_manager::types::{CheckpointResult, FusionStrategy};
use crate::verification::{VerificationConfig, VerificationEngine};

/// Run Best-of-N verification
pub async fn run_best_of_n(
    prompt: &str,
    n: usize,
    strategy: FusionStrategy,
    verification_engine: Arc<Mutex<VerificationEngine>>,
) -> CheckpointResult {
    let verif_config = VerificationConfig {
        n,
        fusion_strategy: strategy.clone(),
        confidence_threshold: 0.8,
        timeout: std::time::Duration::from_secs(120),
    };

    let engine = verification_engine.lock().await;
    match engine.best_of_n(prompt, verif_config).await {
        Ok(result) => {
            let passed = result.confidence >= 0.8;
            let message = if passed {
                format!(
                    "Best-of-{} verification passed (confidence: {:.2})",
                    n, result.confidence
                )
            } else {
                format!(
                    "Best-of-{} verification failed (confidence: {:.2} below threshold)",
                    n, result.confidence
                )
            };

            if passed {
                CheckpointResult::success(message)
            } else {
                CheckpointResult::failure(message)
            }
            .with_details(serde_json::json!({
                "n": n,
                "strategy": format!("{:?}", strategy),
                "confidence": result.confidence,
                "all_results": result.all_results,
                "fusion_reasoning": result.fusion_reasoning,
                "verification_time": result.verification_time,
            }))
        }
        Err(e) => {
            CheckpointResult::failure(format!("Best-of-{} verification error: {}", n, e))
                .with_details(serde_json::json!({
                    "error": e,
                }))
        }
    }
}
