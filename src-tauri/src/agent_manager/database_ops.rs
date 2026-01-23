// Database operations for agent runs
//
// This module handles recording agent run information in the database.

use std::sync::Arc;

use crate::agent_runs_db::{AgentRun, AgentRunsDB, RunStatus};
use crate::logger::Logger;

/// Record a new agent run in the database
pub(crate) async fn record_run_in_db(
    runs_db: &Option<Arc<AgentRunsDB>>,
    logger: &Option<Arc<Logger>>,
    agent_id: &str,
    working_dir: &str,
    github_url: &Option<String>,
    github_context: &Option<crate::types::GitHubContext>,
    source: &crate::types::AgentSource,
    pipeline_id: Option<String>,
    now: i64,
) {
    if let Some(ref runs_db) = runs_db {
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
            if let Some(ref logger) = logger {
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
