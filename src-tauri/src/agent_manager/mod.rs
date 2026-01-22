// Agent manager module
//
// Handles spawning, managing, and communicating with Claude CLI agent processes.

mod output_builder;
mod statistics;
mod stream_handler;
mod types;

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::agent_runs_db::{AgentRun, AgentRunsDB, RunStatus};
use crate::github;
use crate::logger::Logger;

/// Attempt to find the Claude CLI in common installation locations
fn find_claude_cli() -> Result<String, std::env::VarError> {
    #[cfg(windows)]
    {
        // Check npm global install location on Windows
        if let Ok(appdata) = std::env::var("APPDATA") {
            let npm_path = std::path::PathBuf::from(&appdata)
                .join("npm")
                .join("claude.cmd");
            if npm_path.exists() {
                return Ok(npm_path.to_string_lossy().to_string());
            }
        }
        // Check Program Files
        let program_files =
            std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let pf_path = std::path::PathBuf::from(&program_files)
            .join("nodejs")
            .join("claude.cmd");
        if pf_path.exists() {
            return Ok(pf_path.to_string_lossy().to_string());
        }
    }

    #[cfg(not(windows))]
    {
        // Check common Unix locations
        if let Ok(home) = std::env::var("HOME") {
            // Check ~/.local/bin (common for user installs)
            let local_bin = std::path::PathBuf::from(&home).join(".local/bin/claude");
            if local_bin.exists() {
                return Ok(local_bin.to_string_lossy().to_string());
            }

            // Check nvm locations (try to find any node version)
            let nvm_dir = std::path::PathBuf::from(&home).join(".nvm/versions/node");
            if nvm_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&nvm_dir) {
                    for entry in entries.flatten() {
                        let claude_path = entry.path().join("bin/claude");
                        if claude_path.exists() {
                            return Ok(claude_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Check /usr/local/bin
        let usr_local = std::path::PathBuf::from("/usr/local/bin/claude");
        if usr_local.exists() {
            return Ok(usr_local.to_string_lossy().to_string());
        }
    }

    Err(std::env::VarError::NotPresent)
}
use crate::security_monitor::SecurityMonitor;

/// Environment variables to exclude from Claude Code child processes
/// when CLAUDE_CODE_API_KEY_MODE is set to "blocked".
/// This allows meta agents to use the Anthropic API while Claude Code uses OAuth.
const SENSITIVE_ENV_VARS: &[&str] = &["ANTHROPIC_API_KEY"];
use crate::types::{
    AgentActivityEvent, AgentInfo, AgentOutputEvent, AgentStatistics, AgentStatus, AgentStatusEvent,
};
use crate::utils::time::now_millis;

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
        let settings_path = self.create_hooks_config(&agent_id)?;

        // Spawn claude process
        let mut child = self.spawn_claude_process(&settings_path, &working_dir)?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
        let stdin = child.stdin.take().ok_or("Failed to capture stdin")?;

        // Create channel for sending prompts
        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel::<String>(32);

        // Spawn task to handle stdin
        tokio::spawn(async move {
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

        {
            let mut agents = self.agents.lock().await;
            agents.insert(
                agent_id.clone(),
                AgentProcess {
                    info: agent_info.clone(),
                    child: Some(child),
                    stdin_tx: Some(stdin_tx),
                    last_activity: last_activity.clone(),
                    is_processing: is_processing.clone(),
                    pending_input: pending_input.clone(),
                    stats: stats.clone(),
                    output_buffer: output_buffer.clone(),
                    generated_skill_names,
                },
            );
        }

        // Record run in database
        self.record_run_in_db(
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
        app_handle
            .emit(
                "agent:status",
                serde_json::to_value(AgentStatusEvent {
                    agent_id: agent_id.clone(),
                    status: AgentStatus::Running,
                    info: Some(agent_info.clone()),
                })
                .unwrap(),
            )
            .ok();

        // Create stream context for handlers
        let stream_ctx = StreamContext {
            agent_id: agent_id.clone(),
            agents: self.agents.clone(),
            session_map: self.session_to_agent.clone(),
            app_handle: app_handle.clone(),
            last_activity,
            is_processing,
            pending_input,
            stats,
            output_buffer,
            runs_db: self.runs_db.clone(),
            pipeline_id: pipeline_id.clone(),
        };

        // Spawn stream handlers
        spawn_stdout_handler(stdout, stream_ctx);
        spawn_stderr_handler(
            stderr,
            agent_id.clone(),
            app_handle,
            self.runs_db.clone(),
            pipeline_id,
        );

        // Call the on_agent_created callback if set
        if let Some(callback) = &self.on_agent_created {
            callback(agent_id.clone(), source.clone());
        }

        Ok(agent_id)
    }

    fn create_hooks_config(&self, agent_id: &str) -> Result<std::path::PathBuf, String> {
        let settings_path = std::env::temp_dir().join(format!("claude_hooks_{}.json", agent_id));
        let hooks_config = serde_json::json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "*",
                    "hooks": [{
                        "type": "command",
                        "command": format!("curl -s -X POST http://127.0.0.1:{}/hook -H 'Content-Type: application/json' -d @-", self.hook_port)
                    }]
                }],
                "PostToolUse": [{
                    "matcher": "*",
                    "hooks": [{
                        "type": "command",
                        "command": format!("curl -s -X POST http://127.0.0.1:{}/hook -H 'Content-Type: application/json' -d @-", self.hook_port)
                    }]
                }],
                "Stop": [{
                    "hooks": [{
                        "type": "command",
                        "command": format!("curl -s -X POST http://127.0.0.1:{}/hook -H 'Content-Type: application/json' -d @-", self.hook_port)
                    }]
                }]
            }
        });

        std::fs::write(
            &settings_path,
            serde_json::to_string_pretty(&hooks_config).unwrap(),
        )
        .map_err(|e| format!("Failed to create settings file: {}", e))?;

        Ok(settings_path)
    }

    fn spawn_claude_process(
        &self,
        settings_path: &std::path::Path,
        working_dir: &str,
    ) -> Result<tokio::process::Child, String> {
        let claude_path = std::env::var("CLAUDE_PATH")
            .or_else(|_| find_claude_cli())
            .unwrap_or_else(|_| {
                if cfg!(windows) {
                    "claude.cmd".to_string()
                } else {
                    "claude".to_string()
                }
            });

        // Check if we should block API keys from Claude Code
        // Default to "blocked" if not set (Claude Code uses OAuth)
        let api_key_mode =
            std::env::var("CLAUDE_CODE_API_KEY_MODE").unwrap_or_else(|_| "blocked".to_string());

        let mut cmd = Command::new(&claude_path);
        cmd.args([
            "-p",
            "--verbose",
            "--permission-mode",
            "bypassPermissions",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--settings",
            settings_path.to_str().unwrap(),
        ])
        .current_dir(working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        // Apply environment filtering based on mode
        if api_key_mode.to_lowercase() == "blocked" {
            // Filter out sensitive API keys so Claude Code uses OAuth authentication
            let filtered_env: HashMap<String, String> = std::env::vars()
                .filter(|(key, _)| !SENSITIVE_ENV_VARS.contains(&key.as_str()))
                .collect();
            cmd.env_clear().envs(&filtered_env);
        }
        // "passthrough" mode: default Command behavior, inherits all env vars

        cmd.spawn()
            .map_err(|e| format!("Failed to spawn claude: {}", e))
    }

    async fn record_run_in_db(
        &self,
        agent_id: &str,
        working_dir: &str,
        github_url: &Option<String>,
        github_context: &Option<crate::types::GitHubContext>,
        source: &crate::types::AgentSource,
        pipeline_id: Option<String>,
        now: i64,
    ) {
        if let Some(ref runs_db) = self.runs_db {
            let github_context_json = github_context
                .as_ref()
                .and_then(|gc| serde_json::to_string(gc).ok());

            let run = AgentRun {
                id: None,
                agent_id: agent_id.to_string(),
                session_id: None,
                working_dir: working_dir.to_string(),
                github_url: github_url.clone(),
                github_context: github_context_json,
                source: source.as_str().to_string(),
                status: RunStatus::Running,
                started_at: now,
                ended_at: None,
                last_activity: now,
                initial_prompt: None,
                error_message: None,
                pipeline_id, // Linked to orchestrator events for historical queries
                total_prompts: 0,
                total_tool_calls: 0,
                total_output_bytes: 0,
                total_tokens_used: None,
                total_cost_usd: None,
                model_usage: None,
                can_resume: true,
                resume_data: None,
            };

            if let Err(e) = runs_db.create_run(&run).await {
                if let Some(ref logger) = self.logger {
                    let _ = logger
                        .error(
                            "agent_manager",
                            &format!("Failed to record run in database: {}", e),
                            Some(agent_id.to_string()),
                            None,
                        )
                        .await;
                }
            }
        }
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
            app_handle
                .emit(
                    "agent:activity",
                    serde_json::to_value(AgentActivityEvent {
                        agent_id: agent_id.to_string(),
                        is_processing: true,
                        pending_input: false,
                        last_activity: now_millis(),
                    })
                    .unwrap(),
                )
                .ok();
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
        // Get final stats before stopping
        let final_stats = {
            let agents = self.agents.lock().await;
            agents.get(agent_id).map(|agent| agent.stats.clone())
        };

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
        agent.stdin_tx = None;

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
}
