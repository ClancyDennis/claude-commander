use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use std::process::Command;
use std::pin::Pin;
use std::future::Future;

use crate::orchestrator::{TaskOrchestrator, WorkflowStatus};
use crate::meta_agent::MetaAgent;
use crate::agent_manager::AgentManager;
use crate::verification::{VerificationEngine, VerificationConfig};

/// Fusion strategy for Best-of-N verification
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum FusionStrategy {
    MajorityVote,         // Most common result wins
    WeightedConsensus,    // Weighted by confidence/quality scores
    MetaAgentReview,      // Meta-agent picks best
    FirstCorrect,         // First agent that passes validation
}

/// Checkpoint types for phase gates
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum CheckpointType {
    None,                                 // No checkpoint, proceed immediately
    HumanReview,                          // Wait for user approval
    AutomaticValidation {
        command: String,                  // e.g., "cargo test", "npm run lint"
        working_dir: String,
    },
    BestOfN {
        n: usize,                         // Run verification with N agents
        strategy: FusionStrategy,
    },
    Conditional {
        condition: String,                // e.g., "all_tests_passed"
        on_success: Box<CheckpointType>,  // Checkpoint if condition true
        on_failure: Box<CheckpointType>,  // Checkpoint if condition false
    },
}

/// Phase status
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum PhaseStatus {
    Pending,           // Not started yet
    Running,           // Tasks executing
    WaitingCheckpoint, // Tasks done, waiting for checkpoint approval
    CheckpointFailed,  // Checkpoint validation failed
    Completed,         // Phase completed successfully
    Skipped,           // Phase skipped due to conditional
}

/// Checkpoint execution result
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CheckpointResult {
    pub passed: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub timestamp: SystemTime,
}

/// A phase in the pipeline with tasks and checkpoint
#[derive(Clone, Serialize, Deserialize)]
pub struct Phase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_ids: Vec<String>,           // Task IDs from workflow
    pub checkpoint: CheckpointType,
    pub status: PhaseStatus,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub checkpoint_result: Option<CheckpointResult>,
}

/// Pipeline execution state using phases
#[derive(Clone, Serialize)]
pub struct Pipeline {
    pub id: String,
    pub user_request: String,
    pub phases: Vec<Phase>,
    pub current_phase_index: usize,
    pub workflow_id: Option<String>,
    pub status: WorkflowStatus,
    pub created_at: SystemTime,
    pub completed_at: Option<SystemTime>,
    pub error_message: Option<String>,
    pub allow_phase_skip: bool,           // Can user skip failed phases?
}

/// Configuration for pipeline behavior
#[derive(Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub require_plan_review: bool,               // Skip plan review checkpoint?
    pub require_final_review: bool,              // Skip final review checkpoint?
    pub auto_approve_on_verification: bool,      // Auto-approve if verification passes?
    pub verification_strategy: Option<String>,   // "majority", "weighted", "meta", "first"
    pub verification_n: usize,                   // Number of agents for verification (default: 3)
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            require_plan_review: true,           // KEEP: Human gate for safety
            require_final_review: true,          // KEEP: Human gate for safety
            auto_approve_on_verification: false, // KEEP: No auto-approval
            verification_strategy: None,         // DISABLE: No verification initially
            verification_n: 1,                   // MINIMAL: Single agent only
        }
    }
}

pub struct PipelineManager {
    pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
    meta_agent: Arc<Mutex<MetaAgent>>,
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

    /// Create standard development pipeline
    fn create_standard_phases() -> Vec<Phase> {
        vec![
            Phase {
                id: "planning".to_string(),
                name: "Planning & Design".to_string(),
                description: "Meta-agent analyzes requirements and creates execution plan".to_string(),
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
                task_ids: vec![],  // Will be populated during decomposition
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

    /// Start a new pipeline execution
    pub async fn create_pipeline(
        &self,
        user_request: String,
        config: Option<PipelineConfig>,
    ) -> Result<String, String> {
        let pipeline_id = uuid::Uuid::new_v4().to_string();

        let pipeline = Pipeline {
            id: pipeline_id.clone(),
            user_request: user_request.clone(),
            phases: Self::create_standard_phases(),
            current_phase_index: 0,
            workflow_id: None,
            status: WorkflowStatus::Pending,
            created_at: SystemTime::now(),
            completed_at: None,
            error_message: None,
            allow_phase_skip: false,
        };

        self.pipelines.lock().await.insert(pipeline_id.clone(), pipeline);

        // Update config if provided
        if let Some(cfg) = config {
            *self.config.lock().await = cfg;
        }

        // Start pipeline execution in background
        let pipelines = self.pipelines.clone();
        let meta_agent = self.meta_agent.clone();
        let agent_manager = self.agent_manager.clone();
        let orchestrator = self.orchestrator.clone();
        let verification_engine = self.verification_engine.clone();
        let config_arc = self.config.clone();
        let pipeline_id_clone = pipeline_id.clone();

        tokio::spawn(async move {
            Self::execute_pipeline(
                pipeline_id_clone,
                pipelines,
                meta_agent,
                agent_manager,
                orchestrator,
                verification_engine,
                config_arc,
            ).await;
        });

        Ok(pipeline_id)
    }

    /// Main pipeline execution loop - phase-based
    async fn execute_pipeline(
        pipeline_id: String,
        pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
        meta_agent: Arc<Mutex<MetaAgent>>,
        _agent_manager: Arc<Mutex<AgentManager>>,
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

            // Check if pipeline is complete or failed
            if pipeline.status == WorkflowStatus::Completed || pipeline.status == WorkflowStatus::Failed {
                return;
            }

            // Get current phase
            if pipeline.current_phase_index >= pipeline.phases.len() {
                pipeline.status = WorkflowStatus::Completed;
                pipeline.completed_at = Some(SystemTime::now());
                return;
            }

            let current_phase = &mut pipeline.phases[pipeline.current_phase_index];
            let phase_status = current_phase.status.clone();
            let phase_id = current_phase.id.clone();
            let checkpoint_type = current_phase.checkpoint.clone();

            match phase_status {
                PhaseStatus::Pending => {
                    // Start phase execution
                    println!("Starting phase: {}", current_phase.name);
                    current_phase.status = PhaseStatus::Running;
                    current_phase.started_at = Some(SystemTime::now());

                    // Execute phase-specific logic
                    drop(pipelines_lock);

                    match phase_id.as_str() {
                        "planning" => {
                            if let Err(e) = Self::execute_planning_phase(
                                &pipeline_id,
                                pipelines.clone(),
                                meta_agent.clone(),
                            ).await {
                                let mut pl = pipelines.lock().await;
                                if let Some(p) = pl.get_mut(&pipeline_id) {
                                    p.status = WorkflowStatus::Failed;
                                    p.error_message = Some(format!("Planning failed: {}", e));
                                }
                            }
                        }
                        "implementation" => {
                            if let Err(e) = Self::execute_implementation_phase(
                                &pipeline_id,
                                pipelines.clone(),
                                orchestrator.clone(),
                            ).await {
                                let mut pl = pipelines.lock().await;
                                if let Some(p) = pl.get_mut(&pipeline_id) {
                                    p.status = WorkflowStatus::Failed;
                                    p.error_message = Some(format!("Implementation failed: {}", e));
                                }
                            }
                        }
                        _ => {
                            // For other phases, just mark as complete (placeholder)
                            let mut pl = pipelines.lock().await;
                            if let Some(p) = pl.get_mut(&pipeline_id) {
                                if let Some(phase) = p.phases.get_mut(p.current_phase_index) {
                                    phase.status = PhaseStatus::WaitingCheckpoint;
                                }
                            }
                        }
                    }
                }

                PhaseStatus::Running => {
                    // Check if phase tasks are complete
                    drop(pipelines_lock);

                    let tasks_complete = Self::check_phase_tasks_complete(
                        &pipeline_id,
                        pipelines.clone(),
                        orchestrator.clone(),
                    ).await;

                    if tasks_complete {
                        let mut pl = pipelines.lock().await;
                        if let Some(p) = pl.get_mut(&pipeline_id) {
                            if let Some(phase) = p.phases.get_mut(p.current_phase_index) {
                                phase.status = PhaseStatus::WaitingCheckpoint;
                            }
                        }
                    }
                }

                PhaseStatus::WaitingCheckpoint => {
                    // Execute checkpoint
                    drop(pipelines_lock);

                    let result = Self::execute_checkpoint(
                        pipeline_id.clone(),
                        pipelines.clone(),
                        checkpoint_type,
                        verification_engine.clone(),
                        config.clone(),
                    ).await;

                    let mut pl = pipelines.lock().await;
                    if let Some(p) = pl.get_mut(&pipeline_id) {
                        if let Some(phase) = p.phases.get_mut(p.current_phase_index) {
                            match result {
                                Ok(checkpoint_result) => {
                                    if checkpoint_result.passed {
                                        phase.status = PhaseStatus::Completed;
                                        phase.completed_at = Some(SystemTime::now());
                                        phase.checkpoint_result = Some(checkpoint_result);
                                        p.current_phase_index += 1;
                                    } else {
                                        phase.status = PhaseStatus::CheckpointFailed;
                                        phase.checkpoint_result = Some(checkpoint_result.clone());
                                        p.status = WorkflowStatus::Failed;
                                        p.error_message = Some(checkpoint_result.message);
                                    }
                                }
                                Err(e) => {
                                    phase.status = PhaseStatus::CheckpointFailed;
                                    p.status = WorkflowStatus::Failed;
                                    p.error_message = Some(format!("Checkpoint error: {}", e));
                                }
                            }
                        }
                    }
                }

                PhaseStatus::Completed => {
                    // Move to next phase
                    pipeline.current_phase_index += 1;
                    drop(pipelines_lock);
                }

                PhaseStatus::CheckpointFailed => {
                    // Stop execution
                    pipeline.status = WorkflowStatus::Failed;
                    return;
                }

                PhaseStatus::Skipped => {
                    // Move to next phase
                    pipeline.current_phase_index += 1;
                    drop(pipelines_lock);
                }
            }
        }
    }

    /// Execute checkpoint based on type
    fn execute_checkpoint(
        pipeline_id: String,
        pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
        checkpoint_type: CheckpointType,
        verification_engine: Arc<Mutex<VerificationEngine>>,
        config: Arc<Mutex<PipelineConfig>>,
    ) -> Pin<Box<dyn Future<Output = Result<CheckpointResult, String>> + Send>> {
        Box::pin(async move {
            Self::execute_checkpoint_impl(pipeline_id, pipelines, checkpoint_type, verification_engine, config).await
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
            CheckpointType::None => {
                Ok(CheckpointResult {
                    passed: true,
                    message: "No checkpoint required".to_string(),
                    details: None,
                    timestamp: SystemTime::now(),
                })
            }

            CheckpointType::HumanReview => {
                // Wait for human approval via approve_checkpoint command
                // This will be handled externally, so we wait here
                Err("Awaiting human review".to_string())
            }

            CheckpointType::AutomaticValidation { command, working_dir } => {
                println!("Running validation command: {}", command);

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .current_dir(&working_dir)
                    .output()
                    .map_err(|e| format!("Failed to run command: {}", e))?;

                let passed = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                Ok(CheckpointResult {
                    passed,
                    message: if passed {
                        format!("Validation passed: {}", command)
                    } else {
                        format!("Validation failed: {}\nStderr: {}", command, stderr)
                    },
                    details: Some(serde_json::json!({
                        "stdout": stdout.to_string(),
                        "stderr": stderr.to_string(),
                        "exit_code": output.status.code(),
                    })),
                    timestamp: SystemTime::now(),
                })
            }

            CheckpointType::BestOfN { n, strategy } => {
                // Get the prompt from the current phase task
                let prompt = {
                    let pl = pipelines.lock().await;
                    let pipeline = pl.get(&pipeline_id).ok_or("Pipeline not found")?;
                    format!("Verify the implementation for: {}", pipeline.user_request)
                };

                // Run Best-of-N verification
                let verif_config = VerificationConfig {
                    n,
                    fusion_strategy: strategy.clone(),
                    confidence_threshold: 0.8,
                    timeout: std::time::Duration::from_secs(120),
                };

                let engine = verification_engine.lock().await;
                match engine.best_of_n(&prompt, verif_config).await {
                    Ok(result) => {
                        let passed = result.confidence >= 0.8;
                        Ok(CheckpointResult {
                            passed,
                            message: if passed {
                                format!("Best-of-{} verification passed (confidence: {:.2})", n, result.confidence)
                            } else {
                                format!("Best-of-{} verification failed (confidence: {:.2} below threshold)", n, result.confidence)
                            },
                            details: Some(serde_json::json!({
                                "n": n,
                                "strategy": format!("{:?}", strategy),
                                "confidence": result.confidence,
                                "all_results": result.all_results,
                                "fusion_reasoning": result.fusion_reasoning,
                                "verification_time": result.verification_time,
                            })),
                            timestamp: SystemTime::now(),
                        })
                    }
                    Err(e) => {
                        Ok(CheckpointResult {
                            passed: false,
                            message: format!("Best-of-{} verification error: {}", n, e),
                            details: Some(serde_json::json!({
                                "error": e,
                            })),
                            timestamp: SystemTime::now(),
                        })
                    }
                }
            }

            CheckpointType::Conditional { condition, on_success, on_failure } => {
                // Evaluate condition (simplified for now)
                let condition_met = Self::evaluate_condition(&condition, pipelines.clone(), &pipeline_id).await;

                let next_checkpoint = if condition_met {
                    *on_success
                } else {
                    *on_failure
                };

                // Recursively execute the selected checkpoint using wrapper
                Self::execute_checkpoint(pipeline_id, pipelines, next_checkpoint, verification_engine, config).await
            }
        }
    }

    /// Evaluate condition for Conditional checkpoint
    async fn evaluate_condition(
        condition: &str,
        _pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
        _pipeline_id: &str,
    ) -> bool {
        // Simple condition evaluation (can be extended)
        match condition {
            "all_tests_passed" => true, // Placeholder
            "build_succeeded" => true,  // Placeholder
            _ => false,
        }
    }

    /// Phase 1: Planning - Use meta-agent to create execution plan
    async fn execute_planning_phase(
        pipeline_id: &str,
        pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
        _meta_agent: Arc<Mutex<MetaAgent>>,
    ) -> Result<(), String> {
        let user_request = {
            let pl = pipelines.lock().await;
            let pipeline = pl.get(pipeline_id).ok_or("Pipeline not found")?;
            pipeline.user_request.clone()
        };

        println!("Creating execution plan for: {}", user_request);

        // TODO: Call meta-agent to create plan
        // For now, create a placeholder plan
        let _plan = format!(
            "Execution Plan:\n\
             1. Analyze requirements\n\
             2. Design solution\n\
             3. Implement core functionality\n\
             4. Add tests\n\
             5. Verify and review"
        );

        // Store plan in phase task_ids or details
        // For now, just mark as complete
        let mut pl = pipelines.lock().await;
        if let Some(p) = pl.get_mut(pipeline_id) {
            if let Some(phase) = p.phases.get_mut(p.current_phase_index) {
                phase.status = PhaseStatus::WaitingCheckpoint;
            }
        }

        Ok(())
    }

    /// Phase 2: Implementation - Create and execute workflow
    async fn execute_implementation_phase(
        pipeline_id: &str,
        pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
        orchestrator: Arc<Mutex<TaskOrchestrator>>,
    ) -> Result<(), String> {
        println!("Starting implementation phase");

        // TODO: Use orchestrator to create workflow from plan
        // For now, create a simple workflow

        let user_request = {
            let pl = pipelines.lock().await;
            let pipeline = pl.get(pipeline_id).ok_or("Pipeline not found")?;
            pipeline.user_request.clone()
        };

        // Create workflow (simplified - would normally parse plan)
        let orch = orchestrator.lock().await;
        let workflow_id = orch.create_workflow_from_request(&user_request).await
            .unwrap_or_else(|_| {
                // If auto-creation fails, return error
                String::new()
            });

        if workflow_id.is_empty() {
            return Err("Failed to create workflow".to_string());
        }

        // Execute workflow
        orch.execute_workflow(&workflow_id).await?;
        drop(orch);

        // Store workflow_id
        let mut pl = pipelines.lock().await;
        if let Some(p) = pl.get_mut(pipeline_id) {
            p.workflow_id = Some(workflow_id);
        }

        Ok(())
    }

    /// Check if phase tasks are complete
    async fn check_phase_tasks_complete(
        pipeline_id: &str,
        pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
        orchestrator: Arc<Mutex<TaskOrchestrator>>,
    ) -> bool {
        let workflow_id = {
            let pl = pipelines.lock().await;
            let pipeline = match pl.get(pipeline_id) {
                Some(p) => p,
                None => return false,
            };
            match &pipeline.workflow_id {
                Some(id) => id.clone(),
                None => return true, // No workflow = tasks complete
            }
        };

        let orch = orchestrator.lock().await;
        let workflow = match orch.get_workflow(&workflow_id).await {
            Some(w) => w,
            None => return false,
        };

        workflow.status == WorkflowStatus::Completed
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
        let pipeline = pipelines.get_mut(pipeline_id)
            .ok_or("Pipeline not found")?;

        if phase_index >= pipeline.phases.len() {
            return Err("Invalid phase index".to_string());
        }

        let phase = &mut pipeline.phases[phase_index];

        if phase.status != PhaseStatus::WaitingCheckpoint {
            return Err("Phase not waiting for checkpoint".to_string());
        }

        if approved {
            phase.status = PhaseStatus::Completed;
            phase.completed_at = Some(SystemTime::now());
            phase.checkpoint_result = Some(CheckpointResult {
                passed: true,
                message: format!("Approved by user: {}", comment.unwrap_or_default()),
                details: None,
                timestamp: SystemTime::now(),
            });
            pipeline.current_phase_index += 1;
        } else {
            phase.status = PhaseStatus::CheckpointFailed;
            phase.checkpoint_result = Some(CheckpointResult {
                passed: false,
                message: format!("Rejected by user: {}", comment.unwrap_or_default()),
                details: None,
                timestamp: SystemTime::now(),
            });
            pipeline.status = WorkflowStatus::Failed;
            pipeline.error_message = Some(format!("Phase {} rejected", phase.name));
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
