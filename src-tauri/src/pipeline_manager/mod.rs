//! Pipeline Manager module for managing multi-phase development pipelines.
//!
//! This module provides a structured approach to executing development workflows
//! with checkpoints, verification, and human review gates.

pub mod checkpoints;
pub mod config;
pub mod phases;
pub mod types;

pub use config::PipelineConfig;
pub use types::{
    CheckpointResult, CheckpointType, FusionStrategy, Phase, PhaseStatus, Pipeline,
};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::meta_agent::MetaAgent;
use crate::orchestrator::{TaskOrchestrator, WorkflowStatus};
use crate::verification::VerificationEngine;

use checkpoints::execute_checkpoint;
use phases::{check_phase_tasks_complete, execute_implementation_phase, execute_planning_phase};

pub struct PipelineManager {
    pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
    meta_agent: Arc<Mutex<MetaAgent>>,
    #[allow(dead_code)]
    agent_manager: Arc<Mutex<AgentManager>>,
    orchestrator: Arc<Mutex<TaskOrchestrator>>,
    verification_engine: Arc<Mutex<VerificationEngine>>,
    config: Arc<Mutex<PipelineConfig>>,
}

impl PipelineManager {
    pub fn new(
        meta_agent: Arc<Mutex<MetaAgent>>,
        agent_manager: Arc<Mutex<AgentManager>>,
        orchestrator: Arc<Mutex<TaskOrchestrator>>,
        verification_engine: Arc<Mutex<VerificationEngine>>,
    ) -> Self {
        Self {
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            meta_agent,
            agent_manager,
            orchestrator,
            verification_engine,
            config: Arc::new(Mutex::new(PipelineConfig::default())),
        }
    }

    /// Create standard development pipeline phases
    fn create_standard_phases() -> Vec<Phase> {
        vec![
            Phase {
                id: "planning".to_string(),
                name: "Planning & Design".to_string(),
                description: "Meta-agent analyzes requirements and creates execution plan"
                    .to_string(),
                task_ids: vec![],
                checkpoint: CheckpointType::HumanReview,
                status: PhaseStatus::Pending,
                started_at: None,
                completed_at: None,
                checkpoint_result: None,
            },
            Phase {
                id: "implementation".to_string(),
                name: "Implementation".to_string(),
                description: "Execute tasks in parallel with auto-scaling pool".to_string(),
                task_ids: vec![], // Will be populated during decomposition
                checkpoint: CheckpointType::AutomaticValidation {
                    command: "cargo check".to_string(),
                    working_dir: ".".to_string(),
                },
                status: PhaseStatus::Pending,
                started_at: None,
                completed_at: None,
                checkpoint_result: None,
            },
            Phase {
                id: "testing".to_string(),
                name: "Testing & Validation".to_string(),
                description: "Run tests and verify outputs".to_string(),
                task_ids: vec![],
                checkpoint: CheckpointType::BestOfN {
                    n: 3,
                    strategy: FusionStrategy::FirstCorrect,
                },
                status: PhaseStatus::Pending,
                started_at: None,
                completed_at: None,
                checkpoint_result: None,
            },
            Phase {
                id: "review".to_string(),
                name: "Final Review".to_string(),
                description: "Human reviews all outputs and approves".to_string(),
                task_ids: vec![],
                checkpoint: CheckpointType::HumanReview,
                status: PhaseStatus::Pending,
                started_at: None,
                completed_at: None,
                checkpoint_result: None,
            },
        ]
    }

    /// Create a new pipeline (does not start execution automatically)
    pub async fn create_pipeline(
        &self,
        user_request: String,
        config: Option<PipelineConfig>,
    ) -> Result<String, String> {
        let pipeline_id = uuid::Uuid::new_v4().to_string();

        let pipeline = Pipeline {
            id: pipeline_id.clone(),
            user_request,
            phases: Self::create_standard_phases(),
            current_phase_index: 0,
            workflow_id: None,
            status: WorkflowStatus::Pending,
            created_at: SystemTime::now(),
            completed_at: None,
            error_message: None,
            allow_phase_skip: false,
        };

        self.pipelines
            .lock()
            .await
            .insert(pipeline_id.clone(), pipeline);

        // Update config if provided
        if let Some(cfg) = config {
            *self.config.lock().await = cfg;
        }

        Ok(pipeline_id)
    }

    /// Start pipeline execution (call this after user provides task)
    pub async fn start_pipeline(
        &self,
        pipeline_id: &str,
        user_request: String,
    ) -> Result<(), String> {
        // Update the user request
        {
            let mut pipelines = self.pipelines.lock().await;
            let pipeline = pipelines.get_mut(pipeline_id).ok_or("Pipeline not found")?;
            pipeline.user_request = user_request;
        }

        // Start pipeline execution in background
        let pipelines = self.pipelines.clone();
        let meta_agent = self.meta_agent.clone();
        let orchestrator = self.orchestrator.clone();
        let verification_engine = self.verification_engine.clone();
        let config_arc = self.config.clone();
        let pipeline_id_clone = pipeline_id.to_string();

        tokio::spawn(async move {
            Self::execute_pipeline(
                pipeline_id_clone,
                pipelines,
                meta_agent,
                orchestrator,
                verification_engine,
                config_arc,
            )
            .await;
        });

        Ok(())
    }

    /// Main pipeline execution loop - phase-based
    async fn execute_pipeline(
        pipeline_id: String,
        pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
        meta_agent: Arc<Mutex<MetaAgent>>,
        orchestrator: Arc<Mutex<TaskOrchestrator>>,
        verification_engine: Arc<Mutex<VerificationEngine>>,
        config: Arc<Mutex<PipelineConfig>>,
    ) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let mut pipelines_lock = pipelines.lock().await;
            let pipeline = match pipelines_lock.get_mut(&pipeline_id) {
                Some(p) => p,
                None => return,
            };

            // Check if pipeline is finished
            if pipeline.is_finished() {
                return;
            }

            // Check if all phases are done
            if pipeline.all_phases_done() {
                pipeline.mark_completed();
                return;
            }

            let phase = match pipeline.current_phase_mut() {
                Some(p) => p,
                None => return,
            };

            let phase_status = phase.status.clone();
            let phase_id = phase.id.clone();
            let checkpoint_type = phase.checkpoint.clone();

            match phase_status {
                PhaseStatus::Pending => {
                    // Start phase execution
                    println!("Starting phase: {}", phase.name);
                    phase.start();

                    // Execute phase-specific logic
                    drop(pipelines_lock);

                    match phase_id.as_str() {
                        "planning" => {
                            if let Err(e) = execute_planning_phase(
                                &pipeline_id,
                                pipelines.clone(),
                                meta_agent.clone(),
                            )
                            .await
                            {
                                let mut pl = pipelines.lock().await;
                                if let Some(p) = pl.get_mut(&pipeline_id) {
                                    p.mark_failed(format!("Planning failed: {}", e));
                                }
                            }
                        }
                        "implementation" => {
                            if let Err(e) = execute_implementation_phase(
                                &pipeline_id,
                                pipelines.clone(),
                                orchestrator.clone(),
                            )
                            .await
                            {
                                let mut pl = pipelines.lock().await;
                                if let Some(p) = pl.get_mut(&pipeline_id) {
                                    p.mark_failed(format!("Implementation failed: {}", e));
                                }
                            }
                        }
                        _ => {
                            // For other phases, just mark as waiting for checkpoint
                            let mut pl = pipelines.lock().await;
                            if let Some(p) = pl.get_mut(&pipeline_id) {
                                if let Some(phase) = p.current_phase_mut() {
                                    phase.mark_waiting_checkpoint();
                                }
                            }
                        }
                    }
                }

                PhaseStatus::Running => {
                    // Check if phase tasks are complete
                    drop(pipelines_lock);

                    let tasks_complete = check_phase_tasks_complete(
                        &pipeline_id,
                        pipelines.clone(),
                        orchestrator.clone(),
                    )
                    .await;

                    if tasks_complete {
                        let mut pl = pipelines.lock().await;
                        if let Some(p) = pl.get_mut(&pipeline_id) {
                            if let Some(phase) = p.current_phase_mut() {
                                phase.mark_waiting_checkpoint();
                            }
                        }
                    }
                }

                PhaseStatus::WaitingCheckpoint => {
                    // Execute checkpoint
                    drop(pipelines_lock);

                    let result = execute_checkpoint(
                        pipeline_id.clone(),
                        pipelines.clone(),
                        checkpoint_type,
                        verification_engine.clone(),
                        config.clone(),
                    )
                    .await;

                    let mut pl = pipelines.lock().await;
                    if let Some(p) = pl.get_mut(&pipeline_id) {
                        if let Some(phase) = p.current_phase_mut() {
                            match result {
                                Ok(checkpoint_result) => {
                                    if checkpoint_result.passed {
                                        phase.mark_completed();
                                        phase.checkpoint_result = Some(checkpoint_result);
                                        p.advance_phase();
                                    } else {
                                        let msg = checkpoint_result.message.clone();
                                        phase.mark_failed(checkpoint_result);
                                        p.mark_failed(msg);
                                    }
                                }
                                Err(e) => {
                                    phase.mark_failed(CheckpointResult::failure(format!(
                                        "Checkpoint error: {}",
                                        e
                                    )));
                                    p.mark_failed(format!("Checkpoint error: {}", e));
                                }
                            }
                        }
                    }
                }

                PhaseStatus::Completed => {
                    // Move to next phase
                    pipeline.advance_phase();
                    drop(pipelines_lock);
                }

                PhaseStatus::CheckpointFailed => {
                    // Stop execution
                    pipeline.status = WorkflowStatus::Failed;
                    return;
                }

                PhaseStatus::Skipped => {
                    // Move to next phase
                    pipeline.advance_phase();
                    drop(pipelines_lock);
                }
            }
        }
    }

    /// Approve checkpoint (for HumanReview checkpoints)
    pub async fn approve_checkpoint(
        &self,
        pipeline_id: &str,
        phase_index: usize,
        approved: bool,
        comment: Option<String>,
    ) -> Result<(), String> {
        let mut pipelines = self.pipelines.lock().await;
        let pipeline = pipelines.get_mut(pipeline_id).ok_or("Pipeline not found")?;

        if phase_index >= pipeline.phases.len() {
            return Err("Invalid phase index".to_string());
        }

        let phase = &mut pipeline.phases[phase_index];

        if phase.status != PhaseStatus::WaitingCheckpoint {
            return Err("Phase not waiting for checkpoint".to_string());
        }

        if approved {
            phase.mark_completed();
            phase.checkpoint_result = Some(CheckpointResult::success(format!(
                "Approved by user: {}",
                comment.unwrap_or_default()
            )));
            pipeline.advance_phase();
        } else {
            let phase_name = phase.name.clone();
            let result = CheckpointResult::failure(format!(
                "Rejected by user: {}",
                comment.unwrap_or_default()
            ));
            phase.mark_failed(result);
            pipeline.mark_failed(format!("Phase {} rejected", phase_name));
        }

        Ok(())
    }

    pub async fn get_pipeline(&self, pipeline_id: &str) -> Option<Pipeline> {
        self.pipelines.lock().await.get(pipeline_id).cloned()
    }

    pub async fn list_pipelines(&self) -> Vec<Pipeline> {
        self.pipelines.lock().await.values().cloned().collect()
    }

    pub async fn get_config(&self) -> PipelineConfig {
        self.config.lock().await.clone()
    }

    pub async fn update_config(&self, new_config: PipelineConfig) {
        *self.config.lock().await = new_config;
    }
}
