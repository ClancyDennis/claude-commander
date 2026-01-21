use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use crate::pool_manager::AgentPool;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum AgentRole {
    Planner,      // Analyzes requirements, creates plans
    Executor,     // Implements code, writes files
    Tester,       // Runs tests, validates output
    Reviewer,     // Reviews code, suggests improvements
    Monitor,      // Watches long-running tasks
    General,      // General-purpose worker
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Blocked,      // Waiting for dependencies
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorkflowTask {
    pub id: String,
    pub description: String,
    pub role: AgentRole,
    pub prompt: String,              // Prompt to send to agent
    pub dependencies: Vec<String>,   // Task IDs that must complete first
    pub status: TaskStatus,
    pub agent_id: Option<String>,    // Assigned agent (when running)
    pub result: Option<String>,      // Output when completed
    pub working_dir: String,
    pub retry_count: usize,
    pub max_retries: usize,          // default: 3
    pub error_message: Option<String>,
    pub allow_failure: bool,         // continue even if this fails
}

#[derive(Clone, Serialize)]
pub struct Workflow {
    pub id: String,
    pub description: String,
    pub tasks: HashMap<String, WorkflowTask>,
    pub status: WorkflowStatus,
    pub created_at: SystemTime,
    pub completed_at: Option<SystemTime>,
}

#[derive(Clone, Serialize, PartialEq, Debug)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    PartiallyCompleted,
}

pub struct TaskOrchestrator {
    workflows: Arc<Mutex<HashMap<String, Workflow>>>,
    agent_pool: Option<Arc<Mutex<AgentPool>>>,
    max_completed_workflows: usize,
}

impl TaskOrchestrator {
    pub fn new(
        agent_pool: Option<Arc<Mutex<AgentPool>>>,
    ) -> Self {
        Self {
            workflows: Arc::new(Mutex::new(HashMap::new())),
            agent_pool,
            max_completed_workflows: 100, // Keep last 100 workflows
        }
    }

    /// Create a workflow from a list of tasks
    ///
    /// Workflows are created manually with explicit task definitions.
    /// Use this method to define task dependencies and execution order.
    pub async fn create_workflow(
        &self,
        description: String,
        tasks: Vec<WorkflowTask>,
    ) -> Result<String, String> {
        let workflow_id = uuid::Uuid::new_v4().to_string();

        let mut task_map = HashMap::new();
        for task in tasks {
            task_map.insert(task.id.clone(), task);
        }

        let workflow = Workflow {
            id: workflow_id.clone(),
            description,
            tasks: task_map,
            status: WorkflowStatus::Pending,
            created_at: SystemTime::now(),
            completed_at: None,
        };

        self.workflows.lock().await.insert(workflow_id.clone(), workflow);
        Ok(workflow_id)
    }

    /// Create a workflow from user request
    ///
    /// Note: Automatic decomposition is not supported. Use create_workflow() with
    /// explicit task definitions instead.
    pub async fn create_workflow_from_request(
        &self,
        _user_request: &str,
    ) -> Result<String, String> {
        Err("Automatic workflow decomposition not supported. Use create_workflow() with explicit task definitions.".to_string())
    }

    /// Execute a workflow by running tasks in dependency order
    pub async fn execute_workflow(&self, workflow_id: &str) -> Result<(), String> {
        // Mark workflow as running
        {
            let mut workflows = self.workflows.lock().await;
            if let Some(workflow) = workflows.get_mut(workflow_id) {
                workflow.status = WorkflowStatus::Running;
            } else {
                return Err("Workflow not found".to_string());
            }
        }

        // Spawn background task to execute workflow
        let workflows = self.workflows.clone();
        let pool = self.agent_pool.clone();
        let workflow_id = workflow_id.to_string();
        let max_completed = self.max_completed_workflows;

        tokio::spawn(async move {
            Self::workflow_executor(workflow_id, workflows, pool, max_completed).await
        });

        Ok(())
    }

    async fn workflow_executor(
        workflow_id: String,
        workflows: Arc<Mutex<HashMap<String, Workflow>>>,
        pool: Option<Arc<Mutex<AgentPool>>>,
        max_completed_workflows: usize,
    ) {
        loop {
            // Cleanup old workflows
            Self::cleanup_old_workflows(&workflows, max_completed_workflows).await;

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            let mut workflows_lock = workflows.lock().await;
            let workflow = match workflows_lock.get_mut(&workflow_id) {
                Some(wf) => wf,
                None => return,
            };

            if workflow.status == WorkflowStatus::Completed
                || workflow.status == WorkflowStatus::Failed
            {
                return;
            }

            // Handle failed tasks (retry logic)
            let failed_tasks: Vec<String> = workflow
                .tasks
                .iter()
                .filter(|(_, task)| task.status == TaskStatus::Failed)
                .map(|(id, _)| id.clone())
                .collect();

            for task_id in failed_tasks {
                let task = workflow.tasks.get_mut(&task_id).unwrap();

                if task.retry_count < task.max_retries {
                    // Retry
                    task.retry_count += 1;
                    task.status = TaskStatus::Pending;
                    eprintln!(
                        "Retrying task {} (attempt {}/{})",
                        task_id,
                        task.retry_count + 1,
                        task.max_retries
                    );
                } else if task.allow_failure {
                    // Mark as completed with warning
                    task.status = TaskStatus::Completed;
                    task.result = Some(format!("Task failed but marked as completed (allow_failure=true). Error: {}",
                        task.error_message.as_ref().unwrap_or(&"Unknown error".to_string())));
                } else {
                    // Fail entire workflow
                    workflow.status = WorkflowStatus::Failed;
                    workflow.completed_at = Some(SystemTime::now());
                    eprintln!("Workflow {} failed due to task {}", workflow_id, task_id);
                    return;
                }
            }

            // Find tasks ready to run (dependencies met, status=pending)
            let ready_tasks: Vec<String> = workflow
                .tasks
                .iter()
                .filter(|(_, task)| {
                    task.status == TaskStatus::Pending
                        && Self::dependencies_met(task, &workflow.tasks)
                })
                .map(|(id, _)| id.clone())
                .collect();

            if ready_tasks.is_empty() {
                // Check if workflow is complete
                let all_complete = workflow
                    .tasks
                    .values()
                    .all(|t| t.status == TaskStatus::Completed || t.status == TaskStatus::Failed);

                if all_complete {
                    let any_failed = workflow.tasks.values().any(|t| t.status == TaskStatus::Failed);
                    workflow.status = if any_failed {
                        WorkflowStatus::PartiallyCompleted
                    } else {
                        WorkflowStatus::Completed
                    };
                    workflow.completed_at = Some(SystemTime::now());
                    eprintln!("Workflow {} completed with status {:?}", workflow_id, workflow.status);
                    return;
                }

                // Wait for running tasks
                continue;
            }

            // Start ready tasks
            for task_id in ready_tasks {
                let task = workflow.tasks.get_mut(&task_id).unwrap();

                // Acquire agent from pool (if available)
                if let Some(pool_arc) = &pool {
                    let mut pool_lock = pool_arc.lock().await;
                    match pool_lock.acquire_agent().await {
                        Ok(agent_id) => {
                            task.agent_id = Some(agent_id.clone());
                            task.status = TaskStatus::Running;
                            eprintln!("Assigned task {} to agent {}", task_id, agent_id);

                            // TODO: Send prompt to agent via agent_manager
                            // For now, just simulate completion after delay
                            let workflows_clone = workflows.clone();
                            let workflow_id_clone = workflow_id.clone();
                            let task_id_clone = task_id.clone();
                            let pool_clone = pool.clone();
                            let agent_id_clone = agent_id.clone();

                            tokio::spawn(async move {
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                                // Mark task as completed
                                let mut workflows = workflows_clone.lock().await;
                                if let Some(workflow) = workflows.get_mut(&workflow_id_clone) {
                                    if let Some(task) = workflow.tasks.get_mut(&task_id_clone) {
                                        task.status = TaskStatus::Completed;
                                        task.result =
                                            Some(format!("Task {} completed successfully", task_id_clone));
                                    }
                                }

                                // Release agent back to pool
                                if let Some(pool_arc) = pool_clone {
                                    let mut pool = pool_arc.lock().await;
                                    pool.release_agent(agent_id_clone).await;
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to acquire agent for task {}: {}", task_id, e);
                            task.status = TaskStatus::Failed;
                            task.error_message = Some(format!("Failed to acquire agent: {}", e));
                        }
                    }
                } else {
                    // No pool available - mark as failed
                    task.status = TaskStatus::Failed;
                    task.error_message = Some("No agent pool available".to_string());
                }
            }
        }
    }

    fn dependencies_met(task: &WorkflowTask, all_tasks: &HashMap<String, WorkflowTask>) -> bool {
        task.dependencies.iter().all(|dep_id| {
            all_tasks
                .get(dep_id)
                .map(|dep| dep.status == TaskStatus::Completed)
                .unwrap_or(false)
        })
    }

    pub async fn get_workflow(&self, workflow_id: &str) -> Option<Workflow> {
        self.workflows.lock().await.get(workflow_id).cloned()
    }

    pub async fn list_workflows(&self) -> Vec<Workflow> {
        self.workflows.lock().await.values().cloned().collect()
    }

    async fn cleanup_old_workflows(
        workflows: &Arc<Mutex<HashMap<String, Workflow>>>,
        max_completed_workflows: usize,
    ) {
        let mut workflows = workflows.lock().await;

        // Get completed workflows sorted by completion time
        let mut completed: Vec<(String, SystemTime)> = workflows
            .iter()
            .filter(|(_, wf)| wf.status == WorkflowStatus::Completed && wf.completed_at.is_some())
            .map(|(id, wf)| (id.clone(), wf.completed_at.unwrap()))
            .collect();

        completed.sort_by(|a, b| a.1.cmp(&b.1));

        // Remove oldest if over limit
        while completed.len() > max_completed_workflows {
            if let Some((old_id, _)) = completed.first() {
                workflows.remove(old_id);
                completed.remove(0);
            }
        }
    }
}
