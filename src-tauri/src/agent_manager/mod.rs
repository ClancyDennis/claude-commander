// Agent manager module
//
// Handles spawning, managing, and communicating with Claude CLI agent processes.

pub mod claude_cli;
mod database_ops;
mod event_handlers;
mod message_handlers;
mod output_builder;
mod process_spawner;
mod result_handlers;
mod statistics;
mod stream_handler;
mod stream_parser;
mod types;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::agent_runs_db::{AgentRunsDB, RunStatus};
use crate::github;
use crate::logger::Logger;
use crate::security_monitor::SecurityMonitor;
use crate::types::{
    AgentActivityEvent, AgentInfo, AgentOutputEvent, AgentStatistics, AgentStatus, AgentStatusEvent,
};
use crate::utils::time::now_millis;

use database_ops::record_run_in_db;
use process_spawner::{create_hooks_config, spawn_claude_process};
use statistics::create_initial_stats;
use stream_handler::{spawn_stderr_handler, spawn_stdout_handler, StreamContext};

pub use types::AgentProcess;

pub struct AgentManager {
    pub agents: Arc<Mutex<HashMap<String, AgentProcess>>>,
    pub session_to_agent: Arc<Mutex<HashMap<String, String>>>,
    pub hook_port: u16,
    pub logger: Option<Arc<Logger>>,
    pub runs_db: Option<Arc<AgentRunsDB>>,
    pub on_agent_created: Option<Arc<dyn Fn(String, crate::types::AgentSource) + Send + Sync>>,
}

impl AgentManager {
    pub fn new(hook_port: u16) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            session_to_agent: Arc::new(Mutex::new(HashMap::new())),
            hook_port,
            logger: None,
            runs_db: None,
            on_agent_created: None,
        }
    }

    pub fn with_logger(hook_port: u16, logger: Arc<Logger>) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            session_to_agent: Arc::new(Mutex::new(HashMap::new())),
            hook_port,
            logger: Some(logger),
            runs_db: None,
            on_agent_created: None,
        }
    }

    pub fn with_logger_and_db(
        hook_port: u16,
        logger: Arc<Logger>,
        runs_db: Arc<AgentRunsDB>,
    ) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            session_to_agent: Arc::new(Mutex::new(HashMap::new())),
            hook_port,
            logger: Some(logger),
            runs_db: Some(runs_db),
            on_agent_created: None,
        }
    }

    /// Set callback to be invoked when an agent is created
    pub fn set_on_agent_created<F>(&mut self, callback: F)
    where
        F: Fn(String, crate::types::AgentSource) + Send + Sync + 'static,
    {
        self.on_agent_created = Some(Arc::new(callback));
    }

    pub async fn create_agent(
        &self,
        working_dir: String,
        github_url: Option<String>,
        selected_instruction_files: Option<Vec<String>>,
        source: crate::types::AgentSource,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<String, String> {
        self.create_agent_with_pipeline(
            working_dir,
            github_url,
            selected_instruction_files,
            Vec::new(),
            source,
            app_handle,
            None,
            None,
        )
        .await
    }

    /// Create a new agent with pre-generated skills
    pub async fn create_agent_with_skills(
        &self,
        working_dir: String,
        github_url: Option<String>,
        generated_skill_names: Vec<String>,
        source: crate::types::AgentSource,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
    ) -> Result<String, String> {
        self.create_agent_with_pipeline(
            working_dir,
            github_url,
            None,
            generated_skill_names,
            source,
            app_handle,
            None,
            None,
        )
        .await
    }

    /// Create a new agent with optional pipeline linkage and title
    #[allow(clippy::too_many_arguments)]
    pub async fn create_agent_with_pipeline(
        &self,
        working_dir: String,
        github_url: Option<String>,
        _selected_instruction_files: Option<Vec<String>>,
        generated_skill_names: Vec<String>,
        source: crate::types::AgentSource,
        app_handle: Arc<dyn crate::events::AppEventEmitter>,
        pipeline_id: Option<String>,
        title: Option<String>,
    ) -> Result<String, String> {
        let agent_id = uuid::Uuid::new_v4().to_string();

        // Create hooks config
        let settings_path = create_hooks_config(self.hook_port, &agent_id)?;

        // Spawn claude process
        let mut child = spawn_claude_process(&settings_path, &working_dir, &agent_id)?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
        let stdin = child.stdin.take().ok_or("Failed to capture stdin")?;

        // Create channel for sending prompts
        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<String>(32);

        // Spawn task to handle stdin (capture JoinHandle for cleanup)
        let stdin_handle = tokio::spawn(async move {
            let mut stdin = stdin;
            while let Some(prompt) = stdin_rx.recv().await {
                if stdin.write_all(prompt.as_bytes()).await.is_err() {
                    break;
                }
                if stdin.write_all(b"\n").await.is_err() {
                    break;
                }
                if stdin.flush().await.is_err() {
                    break;
                }
            }
        });

        let now = now_millis();

        // Build GitHub context if available
        let github_context = github::build_github_context(&working_dir, github_url.clone());

        let agent_info = AgentInfo {
            id: agent_id.clone(),
            working_dir: working_dir.clone(),
            status: AgentStatus::Running,
            session_id: None,
            last_activity: Some(now),
            is_processing: false,
            pending_input: false,
            github_context: github_context.clone(),
            source: source.clone(),
            pooled: None,
            title,
        };

        // Store agent
        let last_activity = Arc::new(Mutex::new(Instant::now()));
        let is_processing = Arc::new(Mutex::new(false));
        let pending_input = Arc::new(Mutex::new(false));
        let stats = Arc::new(Mutex::new(create_initial_stats(agent_id.clone())));
        let output_buffer = Arc::new(Mutex::new(Vec::new()));

        // Record run in database
        record_run_in_db(
            &self.runs_db,
            &self.logger,
            &agent_id,
            &working_dir,
            &github_url,
            &github_context,
            &source,
            pipeline_id.clone(),
            now,
        )
        .await;

        // Emit event to notify frontend about new agent
        if let Ok(status_event) = serde_json::to_value(AgentStatusEvent {
            agent_id: agent_id.clone(),
            status: AgentStatus::Running,
            info: Some(agent_info.clone()),
        }) {
            let _ = app_handle.emit("agent:status", status_event);
        }

        // Create stream context for handlers
        let stream_ctx = StreamContext {
            agent_id: agent_id.clone(),
            agents: self.agents.clone(),
            session_map: self.session_to_agent.clone(),
            app_handle: app_handle.clone(),
            last_activity: last_activity.clone(),
            is_processing: is_processing.clone(),
            pending_input: pending_input.clone(),
            stats: stats.clone(),
            output_buffer: output_buffer.clone(),
            runs_db: self.runs_db.clone(),
            pipeline_id: pipeline_id.clone(),
        };

        // Spawn stream handlers (capture JoinHandles for proper cleanup)
        let stdout_handle = spawn_stdout_handler(stdout, stream_ctx);
        let stderr_handle = spawn_stderr_handler(
            stderr,
            agent_id.clone(),
            app_handle,
            self.runs_db.clone(),
            pipeline_id,
        );

        // Store agent with all handles for cleanup
        {
            let mut agents = self.agents.lock().await;
            agents.insert(
                agent_id.clone(),
                AgentProcess {
                    info: agent_info.clone(),
                    child: Some(child),
                    stdin_tx: Some(stdin_tx),
                    last_activity,
                    is_processing,
                    pending_input,
                    stats,
                    output_buffer,
                    generated_skill_names,
                    settings_path: Some(settings_path),
                    stdin_handle: Some(stdin_handle),
                    stdout_handle: Some(stdout_handle),
                    stderr_handle: Some(stderr_handle),
                    stopped_at: None,
                },
            );
        }

        // Call the on_agent_created callback if set
        if let Some(callback) = &self.on_agent_created {
            callback(agent_id.clone(), source.clone());
        }

        Ok(agent_id)
    }

    pub async fn send_prompt(
        &self,
        agent_id: &str,
        prompt: &str,
        app_handle: Option<Arc<dyn crate::events::AppEventEmitter>>,
        security_monitor: Option<Arc<SecurityMonitor>>,
    ) -> Result<(), String> {
        // Log prompt sending
        if let Some(ref logger) = self.logger {
            let _ = logger
                .info(
                    "agent_manager",
                    &format!("Sending prompt to agent: {}", prompt),
                    Some(agent_id.to_string()),
                    None,
                )
                .await;
        }

        // Seed security expectations from the prompt (if security monitor is available)
        // This should be done BEFORE processing so the first tool call can be checked
        if let Some(ref monitor) = security_monitor {
            let agents = self.agents.lock().await;
            if let Some(agent) = agents.get(agent_id) {
                let working_dir = agent.info.working_dir.clone();
                drop(agents); // Release lock before async call
                monitor.on_user_prompt(agent_id, &working_dir, prompt).await;
            }
        }

        let agents = self.agents.lock().await;
        let agent = agents
            .get(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        // Clear pending input flag and set processing
        *agent.pending_input.lock().await = false;
        *agent.is_processing.lock().await = true;

        // Emit activity event to update UI
        if let Some(app_handle) = app_handle.as_ref() {
            if let Ok(activity_event) = serde_json::to_value(AgentActivityEvent {
                agent_id: agent_id.to_string(),
                is_processing: true,
                pending_input: false,
                last_activity: now_millis(),
            }) {
                let _ = app_handle.emit("agent:activity", activity_event);
            }
        }

        // Increment prompt counter
        statistics::increment_prompts(&agent.stats).await;

        // Record prompt in database
        if let Some(ref runs_db) = self.runs_db {
            let timestamp = now_millis();

            if let Err(e) = runs_db.record_prompt(agent_id, prompt, timestamp).await {
                if let Some(ref logger) = self.logger {
                    let _ = logger
                        .error(
                            "agent_manager",
                            &format!("Failed to record prompt: {}", e),
                            Some(agent_id.to_string()),
                            None,
                        )
                        .await;
                }
            }

            // Update run in database
            if let Ok(Some(mut run)) = runs_db.get_run(agent_id).await {
                run.last_activity = timestamp;
                run.total_prompts += 1;
                if run.total_prompts == 1 {
                    run.initial_prompt = Some(prompt.to_string());
                }
                let _ = runs_db.update_run(&run).await;
            }
        }

        let stdin_tx = agent
            .stdin_tx
            .as_ref()
            .ok_or_else(|| "Agent stdin not available".to_string())?;

        // Format as stream-json message (JSONL format)
        let message = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [{
                    "type": "text",
                    "text": prompt
                }]
            }
        });

        let json_line = serde_json::to_string(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        stdin_tx.send(json_line).await.map_err(|e| {
            if let Some(ref logger) = self.logger {
                let _ = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        logger
                            .error(
                                "agent_manager",
                                &format!("Failed to send prompt: {}", e),
                                Some(agent_id.to_string()),
                                None,
                            )
                            .await
                    })
                });
            }
            format!("Failed to send prompt: {}", e)
        })?;

        Ok(())
    }

    pub async fn stop_agent(&self, agent_id: &str) -> Result<(), String> {
        // Get final stats and extract handles before stopping
        let (final_stats, stdin_handle, stdout_handle, stderr_handle, settings_path, session_id) = {
            let mut agents = self.agents.lock().await;
            let agent = agents
                .get_mut(agent_id)
                .ok_or_else(|| "Agent not found".to_string())?;

            // Step 1: Drop stdin_tx first to signal stdin handler to exit
            agent.stdin_tx = None;

            (
                Some(agent.stats.clone()),
                agent.stdin_handle.take(),
                agent.stdout_handle.take(),
                agent.stderr_handle.take(),
                agent.settings_path.take(),
                agent.info.session_id.clone(),
            )
        };

        // Step 2: Abort all handler tasks
        if let Some(handle) = stdin_handle {
            handle.abort();
        }
        if let Some(handle) = stdout_handle {
            handle.abort();
        }
        if let Some(handle) = stderr_handle {
            handle.abort();
        }

        // Step 2.5: Clean up session_to_agent map
        if let Some(sid) = session_id {
            let mut session_map = self.session_to_agent.lock().await;
            session_map.remove(&sid);
        }

        // Step 3: Give stream handlers a moment to notice abort before killing child
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Step 4: Now kill the child process and update state
        let mut agents = self.agents.lock().await;
        let agent = agents
            .get_mut(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        if let Some(mut child) = agent.child.take() {
            let _ = child.kill().await;
        }

        // Clean up generated skills
        if !agent.generated_skill_names.is_empty() {
            let working_dir = agent.info.working_dir.clone();
            let skill_names = agent.generated_skill_names.clone();
            if let Err(e) =
                crate::skill_generator::cleanup_generated_skills(&working_dir, &skill_names)
            {
                eprintln!("Warning: Failed to cleanup skills: {}", e);
            }
        }

        agent.info.status = AgentStatus::Stopped;
        agent.stopped_at = Some(Instant::now());

        // Step 5: Clean up hooks config file
        if let Some(path) = settings_path {
            if let Err(e) = std::fs::remove_file(&path) {
                eprintln!("Warning: Failed to cleanup hooks config {:?}: {}", path, e);
            }
        }

        // Update run in database
        if let Some(ref runs_db) = self.runs_db {
            if let Ok(Some(mut run)) = runs_db.get_run(agent_id).await {
                let now = now_millis();

                run.status = RunStatus::Stopped;
                run.ended_at = Some(now);
                run.last_activity = now;

                // Update stats if available
                if let Some(stats) = final_stats {
                    let stats_lock = stats.lock().await;
                    run.total_prompts = stats_lock.total_prompts;
                    run.total_tool_calls = stats_lock.total_tool_calls;
                    run.total_output_bytes = stats_lock.total_output_bytes;
                    run.total_tokens_used = stats_lock.total_tokens_used;
                    run.total_cost_usd = stats_lock.total_cost_usd;

                    if let Some(ref model_usage) = stats_lock.model_usage {
                        run.model_usage = serde_json::to_string(&model_usage).ok();
                    }
                }

                let _ = runs_db.update_run(&run).await;
            }
        }

        Ok(())
    }

    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.lock().await;
        agents.values().map(|a| a.info.clone()).collect()
    }

    pub async fn get_agent_by_session(&self, session_id: &str) -> Option<String> {
        let map = self.session_to_agent.lock().await;
        map.get(session_id).cloned()
    }

    /// Get info for a specific agent
    pub async fn get_agent_info(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.lock().await;
        agents.get(agent_id).map(|a| a.info.clone())
    }

    pub async fn get_agent_statistics(&self, agent_id: &str) -> Result<AgentStatistics, String> {
        let agents = self.agents.lock().await;
        let agent = agents
            .get(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        let stats = agent.stats.lock().await.clone();
        Ok(stats)
    }

    pub async fn get_agent_outputs(
        &self,
        agent_id: &str,
        last_n: usize,
    ) -> Result<Vec<AgentOutputEvent>, String> {
        let agents = self.agents.lock().await;
        let agent = agents
            .get(agent_id)
            .ok_or_else(|| "Agent not found".to_string())?;

        let buffer = agent.output_buffer.lock().await;
        let outputs = if last_n == 0 || last_n >= buffer.len() {
            buffer.clone()
        } else {
            buffer[buffer.len() - last_n..].to_vec()
        };
        Ok(outputs)
    }

    /// Remove an agent from memory (data is already persisted to DB)
    pub async fn remove_agent(&self, agent_id: &str) -> Result<(), String> {
        let mut agents = self.agents.lock().await;
        agents
            .remove(agent_id)
            .map(|_| ())
            .ok_or_else(|| format!("Agent {} not found", agent_id))
    }

    /// Cleanup stopped agents older than the specified duration
    /// Returns the IDs of agents that were removed
    pub async fn cleanup_stopped_agents(&self, max_age: std::time::Duration) -> Vec<String> {
        let mut agents = self.agents.lock().await;
        let now = Instant::now();

        // Find agents to remove
        let to_remove: Vec<String> = agents
            .iter()
            .filter_map(|(id, agent)| {
                if let Some(stopped_at) = agent.stopped_at {
                    if now.duration_since(stopped_at) > max_age {
                        return Some(id.clone());
                    }
                }
                None
            })
            .collect();

        for id in &to_remove {
            eprintln!("[AgentManager] Cleanup: removing stopped agent {}", id);
            agents.remove(id);
        }

        to_remove
    }
}
