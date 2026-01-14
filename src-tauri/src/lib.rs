mod agent_manager;
mod hook_server;
mod types;
mod meta_agent;
mod tool_registry;
mod claude_client;
mod ai_client;
mod github;
mod logger;
mod pool_manager;
mod orchestrator;
mod thread_controller;
mod pipeline_manager;
mod verification;
mod cost_tracker;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use agent_manager::AgentManager;
use meta_agent::MetaAgent;
use logger::{Logger, LogEntry, LogLevel, LogStats};
use pool_manager::{AgentPool, PoolConfig, PoolStats};
use orchestrator::{TaskOrchestrator, Workflow};
use thread_controller::{ThreadController, ThreadConfig, ThreadStats};
use pipeline_manager::{PipelineManager, Pipeline, PipelineConfig};
use verification::{VerificationEngine, VerificationConfig, VerificationResult};
use cost_tracker::{CostTracker, SessionCostRecord, CostSummary, DateRangeCostSummary};
use types::{AgentInfo, AgentStatistics, ChatMessage, ChatResponse};

struct AppState {
    agent_manager: Arc<Mutex<AgentManager>>,
    meta_agent: Arc<Mutex<MetaAgent>>,
    logger: Arc<Logger>,
    agent_pool: Option<Arc<Mutex<AgentPool>>>,
    orchestrator: Arc<Mutex<TaskOrchestrator>>,
    thread_controller: Arc<Mutex<ThreadController>>,
    pipeline_manager: Arc<Mutex<PipelineManager>>,
    verification_engine: Arc<Mutex<VerificationEngine>>,
    cost_tracker: Arc<Mutex<CostTracker>>,
}

#[tauri::command]
async fn create_agent(
    working_dir: String,
    github_url: Option<String>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let manager = state.agent_manager.lock().await;
    manager.create_agent(working_dir, github_url, app_handle).await
}

#[tauri::command]
async fn send_prompt(
    agent_id: String,
    prompt: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let manager = state.agent_manager.lock().await;
    manager.send_prompt(&agent_id, &prompt, Some(app_handle)).await
}

#[tauri::command]
async fn stop_agent(agent_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Get agent stats before stopping
    let stats = {
        let manager = state.agent_manager.lock().await;
        manager.get_agent_statistics(&agent_id).await.ok()
    };

    // Stop the agent
    {
        let manager = state.agent_manager.lock().await;
        manager.stop_agent(&agent_id).await?;
    }

    // Record costs to persistent storage
    if let Some(stats) = stats {
        if let Some(cost) = stats.total_cost_usd {
            let cost_tracker = state.cost_tracker.lock().await;

            // Convert model usage to cost tracker format
            let model_usage = stats.model_usage.map(|usage| {
                usage.into_iter().map(|(model, stats_data)| {
                    (model, cost_tracker::ModelCostBreakdown {
                        input_tokens: stats_data.input_tokens.unwrap_or(0),
                        output_tokens: stats_data.output_tokens.unwrap_or(0),
                        cache_creation_input_tokens: stats_data.cache_creation_input_tokens.unwrap_or(0),
                        cache_read_input_tokens: stats_data.cache_read_input_tokens.unwrap_or(0),
                        cost_usd: stats_data.cost_usd.unwrap_or(0.0),
                    })
                }).collect()
            });

            let record = SessionCostRecord {
                session_id: format!("session_{}", agent_id),
                agent_id: agent_id.clone(),
                working_dir: "unknown".to_string(), // We'll need to enhance this
                started_at: stats.session_start,
                ended_at: Some(chrono::Utc::now().to_rfc3339()),
                total_cost_usd: cost,
                total_tokens: stats.total_tokens_used.unwrap_or(0) as u64,
                total_prompts: stats.total_prompts,
                total_tool_calls: stats.total_tool_calls,
                model_usage,
            };

            cost_tracker.record_session(record).await.ok();
        }
    }

    Ok(())
}

#[tauri::command]
async fn list_agents(state: tauri::State<'_, AppState>) -> Result<Vec<AgentInfo>, String> {
    let manager = state.agent_manager.lock().await;
    Ok(manager.list_agents().await)
}

#[tauri::command]
async fn get_agent_statistics(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AgentStatistics, String> {
    let manager = state.agent_manager.lock().await;
    manager.get_agent_statistics(&agent_id).await
}

#[tauri::command]
async fn send_chat_message(
    message: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent
        .process_user_message(message, state.agent_manager.clone(), app_handle)
        .await
}

#[tauri::command]
async fn get_chat_history(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    let meta_agent = state.meta_agent.lock().await;
    Ok(meta_agent.get_chat_messages())
}

#[tauri::command]
async fn clear_chat_history(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut meta_agent = state.meta_agent.lock().await;
    meta_agent.clear_conversation_history();
    Ok(())
}

#[tauri::command]
async fn process_agent_results(
    agent_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    // Get agent outputs
    let manager = state.agent_manager.lock().await;
    let outputs = manager.get_agent_outputs(&agent_id, 0).await?;

    // Get agent info
    let agents = manager.list_agents().await;
    let agent_info = agents.iter().find(|a| a.id == agent_id);
    let agent_name = agent_info.map(|a| a.working_dir.clone()).unwrap_or_else(|| agent_id.clone());

    // Format outputs
    let mut formatted_output = format!("Results from agent in {}:\n\n", agent_name);

    for output in outputs.iter() {
        match output.output_type.as_str() {
            "text" => {
                formatted_output.push_str(&format!("Assistant: {}\n\n", output.content));
            }
            "tool_use" => {
                // Extract tool name from content
                let tool_name = if output.content.contains("Using tool:") {
                    output.content.lines().next().unwrap_or("Unknown tool")
                } else {
                    "Using tool"
                };
                formatted_output.push_str(&format!("{}\n", tool_name));
            }
            "tool_result" => {
                // Truncate long tool results
                let truncated = if output.content.len() > 500 {
                    format!("{}...[truncated]", &output.content[..500])
                } else {
                    output.content.clone()
                };
                formatted_output.push_str(&format!("Result: {}\n\n", truncated));
            }
            "result" => {
                formatted_output.push_str("\n--- Final Results ---\n");
                if let Some(parsed) = &output.parsed_json {
                    if let Some(cost) = parsed.get("total_cost_usd").and_then(|v| v.as_f64()) {
                        formatted_output.push_str(&format!("Cost: ${:.4}\n", cost));
                    }
                    if let Some(usage) = parsed.get("usage") {
                        if let Some(input_tokens) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                            if let Some(output_tokens) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                                formatted_output.push_str(&format!("Tokens: {} input, {} output\n", input_tokens, output_tokens));
                            }
                        }
                    }
                }
                formatted_output.push_str("\n");
            }
            _ => {}
        }
    }

    drop(manager);

    // Process the formatted output as a user message through the meta agent
    let mut meta_agent = state.meta_agent.lock().await;
    let response = meta_agent.process_user_message(
        formatted_output,
        state.agent_manager.clone(),
        app_handle
    ).await?;

    Ok(response)
}

#[tauri::command]
async fn list_github_repos() -> Result<Vec<serde_json::Value>, String> {
    use std::process::Command;

    // Run gh repo list with JSON output
    let output = Command::new("gh")
        .args([
            "repo",
            "list",
            "--limit",
            "100",
            "--json",
            "nameWithOwner,name,description,updatedAt,url",
        ])
        .output()
        .map_err(|e| format!("Failed to execute gh command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh command failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let repos: Vec<serde_json::Value> = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse gh output: {}", e))?;

    Ok(repos)
}

#[tauri::command]
async fn get_pool_stats(state: tauri::State<'_, AppState>) -> Result<PoolStats, String> {
    if let Some(pool_arc) = &state.agent_pool {
        let pool = pool_arc.lock().await;
        Ok(pool.get_stats().await)
    } else {
        Err("Pool not initialized".to_string())
    }
}

#[tauri::command]
async fn configure_pool(
    config: PoolConfig,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // This would require mutable access to AppState which isn't supported
    // For now, pool configuration is set at startup only
    Err("Pool reconfiguration not yet supported".to_string())
}

#[tauri::command]
async fn request_pool_agent(state: tauri::State<'_, AppState>) -> Result<String, String> {
    if let Some(pool_arc) = &state.agent_pool {
        let mut pool = pool_arc.lock().await;
        pool.acquire_agent().await.map_err(|e| e.to_string())
    } else {
        Err("Pool not initialized".to_string())
    }
}

#[tauri::command]
async fn release_pool_agent(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some(pool_arc) = &state.agent_pool {
        let mut pool = pool_arc.lock().await;
        pool.release_agent(agent_id).await;
        Ok(())
    } else {
        Err("Pool not initialized".to_string())
    }
}

#[tauri::command]
async fn create_workflow_from_request(
    request: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let orchestrator = state.orchestrator.lock().await;
    orchestrator.create_workflow_from_request(&request).await
}

#[tauri::command]
async fn execute_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let orchestrator = state.orchestrator.lock().await;
    orchestrator.execute_workflow(&workflow_id).await
}

#[tauri::command]
async fn get_workflow(
    workflow_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Workflow, String> {
    let orchestrator = state.orchestrator.lock().await;
    orchestrator
        .get_workflow(&workflow_id)
        .await
        .ok_or_else(|| "Workflow not found".to_string())
}

#[tauri::command]
async fn list_workflows(state: tauri::State<'_, AppState>) -> Result<Vec<Workflow>, String> {
    let orchestrator = state.orchestrator.lock().await;
    Ok(orchestrator.list_workflows().await)
}

#[tauri::command]
async fn get_thread_config(state: tauri::State<'_, AppState>) -> Result<ThreadConfig, String> {
    let controller = state.thread_controller.lock().await;
    Ok(controller.get_config().await)
}

#[tauri::command]
async fn update_thread_config(
    config: ThreadConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let controller = state.thread_controller.lock().await;
    controller.update_config(config).await;
    Ok(())
}

#[tauri::command]
async fn get_thread_stats(state: tauri::State<'_, AppState>) -> Result<ThreadStats, String> {
    let controller = state.thread_controller.lock().await;
    Ok(controller.get_stats().await)
}

#[tauri::command]
async fn emergency_shutdown_threads(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let controller = state.thread_controller.lock().await;
    controller.emergency_shutdown().await
}

#[tauri::command]
async fn create_pipeline(
    user_request: String,
    config: Option<PipelineConfig>,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.pipeline_manager.lock().await;
    manager.create_pipeline(user_request, config).await
}

#[tauri::command]
async fn get_pipeline(
    pipeline_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Pipeline, String> {
    let manager = state.pipeline_manager.lock().await;
    manager.get_pipeline(&pipeline_id).await
        .ok_or_else(|| "Pipeline not found".to_string())
}

#[tauri::command]
async fn list_pipelines(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Pipeline>, String> {
    let manager = state.pipeline_manager.lock().await;
    Ok(manager.list_pipelines().await)
}

#[tauri::command]
async fn approve_pipeline_checkpoint(
    pipeline_id: String,
    phase_index: usize,
    approved: bool,
    comment: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.pipeline_manager.lock().await;
    manager.approve_checkpoint(&pipeline_id, phase_index, approved, comment).await
}

#[tauri::command]
async fn get_pipeline_config(
    state: tauri::State<'_, AppState>,
) -> Result<PipelineConfig, String> {
    let manager = state.pipeline_manager.lock().await;
    Ok(manager.get_config().await)
}

#[tauri::command]
async fn update_pipeline_config(
    config: PipelineConfig,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let manager = state.pipeline_manager.lock().await;
    manager.update_config(config).await;
    Ok(())
}

#[tauri::command]
async fn run_best_of_n_verification(
    prompt: String,
    config: VerificationConfig,
    state: tauri::State<'_, AppState>,
) -> Result<VerificationResult, String> {
    let engine = state.verification_engine.lock().await;
    engine.best_of_n(&prompt, config).await
}

// Cost Tracking Commands

#[tauri::command]
async fn get_cost_summary(
    state: tauri::State<'_, AppState>,
) -> Result<CostSummary, String> {
    let tracker = state.cost_tracker.lock().await;
    Ok(tracker.get_cost_summary().await)
}

#[tauri::command]
async fn get_cost_by_date_range(
    start_date: Option<String>,
    end_date: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<DateRangeCostSummary, String> {
    use chrono::DateTime;

    let start = start_date
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let end = end_date
        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let tracker = state.cost_tracker.lock().await;
    Ok(tracker.get_date_range_summary(start, end).await)
}

#[tauri::command]
async fn get_current_month_cost(
    state: tauri::State<'_, AppState>,
) -> Result<f64, String> {
    let tracker = state.cost_tracker.lock().await;
    Ok(tracker.get_current_month_cost().await)
}

#[tauri::command]
async fn get_today_cost(
    state: tauri::State<'_, AppState>,
) -> Result<f64, String> {
    let tracker = state.cost_tracker.lock().await;
    Ok(tracker.get_today_cost().await)
}

#[tauri::command]
async fn clear_cost_history(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let tracker = state.cost_tracker.lock().await;
    tracker.clear_history().await
}

// Logging Commands

#[tauri::command]
async fn query_logs(
    level: Option<String>,
    component: Option<String>,
    agent_id: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LogEntry>, String> {
    let log_level = level.and_then(|l| match l.as_str() {
        "debug" => Some(LogLevel::Debug),
        "info" => Some(LogLevel::Info),
        "warning" => Some(LogLevel::Warning),
        "error" => Some(LogLevel::Error),
        _ => None,
    });

    state.logger
        .query(log_level, component, agent_id, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_recent_logs(
    limit: usize,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LogEntry>, String> {
    state.logger
        .recent(limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_log_stats(
    state: tauri::State<'_, AppState>,
) -> Result<LogStats, String> {
    state.logger
        .stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn cleanup_old_logs(
    days_to_keep: i64,
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    state.logger
        .cleanup(days_to_keep)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file if it exists (silently ignore if not found)
    let _ = dotenvy::dotenv();

    let hook_port: u16 = 19832;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            // Initialize logger first so other components can use it
            let log_db_path = dirs::data_local_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
                .map(|d| d.join("grove").join("logs.db"))
                .unwrap_or_else(|| std::env::temp_dir().join("grove_logs.db"));

            // Ensure parent directory exists
            if let Some(parent) = log_db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let logger = match Logger::new(log_db_path.clone()) {
                Ok(logger) => {
                    println!("✓ Logger initialized at {:?}", log_db_path);
                    Arc::new(logger)
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to initialize logger: {}", e);
                    // Try fallback to temp directory
                    let temp_log = std::env::temp_dir().join("grove_logs.db");
                    match Logger::new(temp_log.clone()) {
                        Ok(logger) => {
                            println!("✓ Logger initialized at temp location: {:?}", temp_log);
                            Arc::new(logger)
                        }
                        Err(e2) => panic!("Failed to initialize logger even in temp directory: {}", e2),
                    }
                }
            };

            let agent_manager = Arc::new(Mutex::new(AgentManager::with_logger(hook_port, logger.clone())));

            // Initialize meta-agent - tries ANTHROPIC_API_KEY first, then OPENAI_API_KEY
            let meta_agent = match MetaAgent::new() {
                Ok(agent) => {
                    // Log which provider and model is being used
                    let provider_info = if std::env::var("ANTHROPIC_API_KEY").is_ok() {
                        let model = std::env::var("ANTHROPIC_MODEL")
                            .unwrap_or_else(|_| "claude-sonnet-4-5-20250929".to_string());
                        format!("Claude ({})", model)
                    } else if std::env::var("OPENAI_API_KEY").is_ok() {
                        let model = std::env::var("OPENAI_MODEL")
                            .unwrap_or_else(|_| "gpt-4o".to_string());
                        format!("OpenAI ({})", model)
                    } else {
                        "Unknown".to_string()
                    };
                    println!("✓ Meta-agent initialized successfully using {}", provider_info);
                    Arc::new(Mutex::new(agent))
                }
                Err(e) => {
                    eprintln!("⚠ Warning: {}. Chat functionality will not work.", e);
                    eprintln!("  Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable.");
                    // Create a dummy agent with no API key (will fail on first use)
                    Arc::new(Mutex::new(MetaAgent::new_with_client(
                        ai_client::AIClient::new(ai_client::Provider::Claude {
                            api_key: String::new(),
                            model: "claude-sonnet-4-20250514".to_string(),
                        })
                    )))
                }
            };

            // Start hook server
            let agent_manager_clone = agent_manager.clone();
            let app_handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) =
                    hook_server::start_hook_server(agent_manager_clone, app_handle, hook_port).await
                {
                    eprintln!("Hook server error: {}", e);
                }
            });

            // Initialize agent pool with default config
            let pool_config = PoolConfig::default();
            let app_handle_for_pool = app.handle().clone();
            let agent_pool = match tauri::async_runtime::block_on(async {
                AgentPool::new(pool_config, agent_manager.clone(), Some(app_handle_for_pool)).await
            }) {
                Ok(pool) => {
                    println!("✓ Agent pool initialized with default config");
                    Some(pool)
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to initialize agent pool: {}", e);
                    None
                }
            };

            // Initialize orchestrator
            let orchestrator = Arc::new(Mutex::new(TaskOrchestrator::new(
                agent_pool.clone(),
                meta_agent.clone(),
            )));
            println!("✓ Task orchestrator initialized");

            // Initialize thread controller
            let thread_controller = Arc::new(Mutex::new(ThreadController::new(
                agent_pool.clone(),
                orchestrator.clone(),
            )));
            println!("✓ Thread controller initialized");

            // Start stats update loop
            let thread_controller_clone = thread_controller.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    let controller = thread_controller_clone.lock().await;
                    controller.update_stats().await;
                }
            });

            // Initialize verification engine first
            let verification_engine = Arc::new(Mutex::new(VerificationEngine::new(
                agent_pool.clone(),
                meta_agent.clone(),
            )));
            println!("✓ Verification engine initialized");

            // Initialize pipeline manager with verification engine
            let pipeline_manager = Arc::new(Mutex::new(PipelineManager::new(
                meta_agent.clone(),
                agent_manager.clone(),
                orchestrator.clone(),
                verification_engine.clone(),
            )));
            println!("✓ Pipeline manager initialized");

            // Initialize cost tracker
            let cost_tracker = match CostTracker::new(None) {
                Ok(tracker) => Arc::new(Mutex::new(tracker)),
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to initialize cost tracker: {}", e);
                    eprintln!("  Cost tracking will not persist across sessions.");
                    // Create a tracker with a temporary location as fallback
                    Arc::new(Mutex::new(CostTracker::new(Some(std::env::temp_dir())).unwrap()))
                }
            };
            println!("✓ Cost tracker initialized");

            app.manage(AppState {
                agent_manager,
                meta_agent,
                logger,
                agent_pool,
                orchestrator,
                thread_controller,
                pipeline_manager,
                verification_engine,
                cost_tracker,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_agent,
            send_prompt,
            stop_agent,
            list_agents,
            get_agent_statistics,
            send_chat_message,
            get_chat_history,
            clear_chat_history,
            process_agent_results,
            list_github_repos,
            get_pool_stats,
            configure_pool,
            request_pool_agent,
            release_pool_agent,
            create_workflow_from_request,
            execute_workflow,
            get_workflow,
            list_workflows,
            get_thread_config,
            update_thread_config,
            get_thread_stats,
            emergency_shutdown_threads,
            create_pipeline,
            get_pipeline,
            list_pipelines,
            approve_pipeline_checkpoint,
            get_pipeline_config,
            update_pipeline_config,
            run_best_of_n_verification,
            get_cost_summary,
            get_cost_by_date_range,
            get_current_month_cost,
            get_today_cost,
            clear_cost_history,
            query_logs,
            get_recent_logs,
            get_log_stats,
            cleanup_old_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
