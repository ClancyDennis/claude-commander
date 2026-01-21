use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::pipeline_manager::checkpoints::{validation, verification};
use crate::pipeline_manager::config::PipelineConfig;
use crate::pipeline_manager::types::{CheckpointResult, CheckpointType, Pipeline};
use crate::verification::VerificationEngine;

/// Execute checkpoint based on type (wrapper for recursion support)
pub fn execute_checkpoint(
    pipeline_id: String,
    pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
    checkpoint_type: CheckpointType,
    verification_engine: Arc<Mutex<VerificationEngine>>,
    config: Arc<Mutex<PipelineConfig>>,
) -> Pin<Box<dyn Future<Output = Result<CheckpointResult, String>> + Send>> {
    Box::pin(async move {
        execute_checkpoint_impl(
            pipeline_id,
            pipelines,
            checkpoint_type,
            verification_engine,
            config,
        )
        .await
    })
}

/// Internal checkpoint execution implementation
async fn execute_checkpoint_impl(
    pipeline_id: String,
    pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
    checkpoint_type: CheckpointType,
    verification_engine: Arc<Mutex<VerificationEngine>>,
    config: Arc<Mutex<PipelineConfig>>,
) -> Result<CheckpointResult, String> {
    match checkpoint_type {
        CheckpointType::None => Ok(CheckpointResult::success("No checkpoint required")),

        CheckpointType::HumanReview => {
            // Wait for human approval via approve_checkpoint command
            // This will be handled externally, so we return an error to signal waiting
            Err("Awaiting human review".to_string())
        }

        CheckpointType::AutomaticValidation {
            command,
            working_dir,
        } => validation::run_validation(&command, &working_dir),

        CheckpointType::BestOfN { n, strategy } => {
            // Get the prompt from the pipeline
            let prompt = {
                let pl = pipelines.lock().await;
                let pipeline = pl.get(&pipeline_id).ok_or("Pipeline not found")?;
                format!("Verify the implementation for: {}", pipeline.user_request)
            };

            Ok(verification::run_best_of_n(&prompt, n, strategy, verification_engine).await)
        }

        CheckpointType::Conditional {
            condition,
            on_success,
            on_failure,
        } => {
            // Evaluate condition (simplified for now)
            let condition_met = evaluate_condition(&condition).await;

            let next_checkpoint = if condition_met {
                *on_success
            } else {
                *on_failure
            };

            // Recursively execute the selected checkpoint
            execute_checkpoint(
                pipeline_id,
                pipelines,
                next_checkpoint,
                verification_engine,
                config,
            )
            .await
        }
    }
}

/// Evaluate condition for Conditional checkpoint
async fn evaluate_condition(condition: &str) -> bool {
    // Simple condition evaluation (can be extended)
    match condition {
        "all_tests_passed" => true, // Placeholder
        "build_succeeded" => true,  // Placeholder
        _ => false,
    }
}
