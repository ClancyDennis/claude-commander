pub mod agent_manager;
pub mod agent_runs_db;
pub mod ai_client;
pub mod auto_pipeline;
pub mod claude_client;
pub mod claudemd_generator;
pub mod commands;
pub mod db_utils;
pub mod elevation;
pub mod error;
pub mod events;
pub mod first_run;
pub mod github;
pub mod hook_server;
pub mod instruction_manager;
pub mod logger;
pub mod meta_agent;
pub mod security_monitor;
pub mod skill_generator;
pub mod subagent_generator;
pub mod tool_registry;
pub mod types;
pub mod utils;
pub mod voice;

use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use agent_manager::AgentManager;
use agent_runs_db::AgentRunsDB;
use auto_pipeline::AutoPipelineManager;
use logger::Logger;
use meta_agent::tools::{PendingQuestion, SleepState};
use meta_agent::MetaAgent;
use security_monitor::{ResponseConfig, SecurityConfig, SecurityMonitor};
use types::PendingElevatedCommand;

// Make AppState public so commands module can access it
pub struct AppState {
    pub agent_manager: Arc<Mutex<AgentManager>>,
    pub meta_agent: Arc<Mutex<MetaAgent>>,
    pub logger: Arc<Logger>,
    pub agent_runs_db: Arc<AgentRunsDB>,
    pub auto_pipeline_manager: Option<Arc<Mutex<AutoPipelineManager>>>,
    pub security_monitor: Option<Arc<SecurityMonitor>>,
    // Elevation state (shared with hook server)
    pub pending_elevated: Arc<Mutex<HashMap<String, PendingElevatedCommand>>>,
    pub approved_scopes: Arc<Mutex<HashMap<String, i64>>>,
    pub app_handle: Arc<dyn events::AppEventEmitter>,
    // Meta-agent interaction state (accessible without locking meta_agent)
    pub pending_meta_question: Arc<Mutex<Option<PendingQuestion>>>,
    pub meta_sleep_state: Arc<Mutex<SleepState>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // First-run initialization - copies template files before env loading
    first_run::run_if_needed();

    // Load .env file from multiple locations (in priority order):
    // 1. App config directory (user's persistent settings)
    // 2. Current working directory (for development overrides)
    if let Some(config_dir) = dirs::config_dir() {
        let app_env = config_dir.join("claude-commander").join(".env");
        if app_env.exists() && dotenvy::from_path(&app_env).is_ok() {
            println!("✓ Loaded .env from {:?}", app_env);
        }
    }
    // Allow cwd .env to override config dir (useful for development)
    let _ = dotenvy::dotenv();

    let hook_port: u16 = 19832;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            // Initialize logger first so other components can use it
            let log_db_path = dirs::data_local_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
                .map(|d| d.join("claude-commander").join("logs.db"))
                .unwrap_or_else(|| std::env::temp_dir().join("cc_logs.db"));

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
                    let temp_log = std::env::temp_dir().join("cc_logs.db");
                    match Logger::new(temp_log.clone()) {
                        Ok(logger) => {
                            println!("✓ Logger initialized at temp location: {:?}", temp_log);
                            Arc::new(logger)
                        }
                        Err(e2) => {
                            panic!("Failed to initialize logger even in temp directory: {}", e2)
                        }
                    }
                }
            };

            // Initialize agent runs database early (needed for agent manager)
            let runs_db_path = dirs::data_local_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
                .map(|d| d.join("claude-commander").join("agent_runs.db"))
                .unwrap_or_else(|| std::env::temp_dir().join("cc_agent_runs.db"));

            // Ensure parent directory exists
            if let Some(parent) = runs_db_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            let agent_runs_db = match AgentRunsDB::new(runs_db_path.clone()) {
                Ok(db) => {
                    println!("✓ Agent runs database initialized at {:?}", runs_db_path);
                    Arc::new(db)
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to initialize agent runs database: {}", e);
                    // Try fallback to temp directory
                    let temp_db = std::env::temp_dir().join("cc_agent_runs.db");
                    match AgentRunsDB::new(temp_db.clone()) {
                        Ok(db) => {
                            println!(
                                "✓ Agent runs database initialized at temp location: {:?}",
                                temp_db
                            );
                            Arc::new(db)
                        }
                        Err(e2) => panic!(
                            "Failed to initialize agent runs database even in temp directory: {}",
                            e2
                        ),
                    }
                }
            };

            // Reconcile stale runs from previous session
            // This marks any "running" agents as "crashed" since the app just started
            let runs_db_for_reconcile = agent_runs_db.clone();
            tauri::async_runtime::spawn(async move {
                match runs_db_for_reconcile.reconcile_stale_runs().await {
                    Ok(count) if count > 0 => {
                        println!(
                            "✓ Reconciled {} stale agent runs from previous session",
                            count
                        );
                    }
                    Ok(_) => {}
                    Err(e) => eprintln!("⚠ Warning: Failed to reconcile stale runs: {}", e),
                }
            });

            let agent_manager = Arc::new(Mutex::new(AgentManager::with_logger_and_db(
                hook_port,
                logger.clone(),
                agent_runs_db.clone(),
            )));

            // Initialize meta-agent - tries ANTHROPIC_API_KEY first, then OPENAI_API_KEY
            let meta_agent = match MetaAgent::new() {
                Ok(agent) => {
                    // Log which provider and model is being used
                    let provider_info = if std::env::var("ANTHROPIC_API_KEY").is_ok() {
                        let model = std::env::var("ANTHROPIC_MODEL")
                            .unwrap_or_else(|_| "claude-sonnet-4-5-20250929".to_string());
                        format!("Claude ({})", model)
                    } else if std::env::var("OPENAI_API_KEY").is_ok() {
                        let model =
                            std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());
                        format!("OpenAI ({})", model)
                    } else {
                        "Unknown".to_string()
                    };
                    println!(
                        "✓ Meta-agent initialized successfully using {}",
                        provider_info
                    );
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
                        }),
                    )))
                }
            };

            // Initialize security monitor (optional - works even without LLM)
            let security_monitor = match SecurityMonitor::new(
                agent_manager.clone(),
                logger.clone(),
                Arc::new(app.handle().clone()),
                SecurityConfig::default(),
                ResponseConfig::default(),
            ) {
                Ok(monitor) => {
                    let monitor = Arc::new(monitor);
                    // Start background analysis loop inside async context
                    let monitor_for_bg = monitor.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = monitor_for_bg.start_background_analysis().await;
                    });
                    println!("✓ Security monitor initialized");
                    Some(monitor)
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to initialize security monitor: {}", e);
                    eprintln!("  Security monitoring will be disabled.");
                    None
                }
            };

            // Create shared elevation state (used by both hook server and Tauri commands)
            let pending_elevated: Arc<Mutex<HashMap<String, PendingElevatedCommand>>> =
                Arc::new(Mutex::new(HashMap::new()));
            let approved_scopes: Arc<Mutex<HashMap<String, i64>>> =
                Arc::new(Mutex::new(HashMap::new()));

            // Start hook server (with security monitor and elevation state)
            let agent_manager_clone = agent_manager.clone();
            let app_handle = Arc::new(app.handle().clone());
            let app_handle_for_hook = app_handle.clone();
            let security_monitor_for_hook = security_monitor.clone();
            let pending_elevated_for_hook = pending_elevated.clone();
            let approved_scopes_for_hook = approved_scopes.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = hook_server::start_hook_server(
                    agent_manager_clone,
                    app_handle_for_hook,
                    hook_port,
                    security_monitor_for_hook,
                    pending_elevated_for_hook,
                    approved_scopes_for_hook,
                )
                .await
                {
                    eprintln!("Hook server error: {}", e);
                }
            });

            // Start periodic cleanup task for stopped agents (every 60 seconds)
            let agent_manager_for_cleanup = agent_manager.clone();
            let security_monitor_for_cleanup = security_monitor.clone();
            tauri::async_runtime::spawn(async move {
                let cleanup_interval = std::time::Duration::from_secs(60);
                let max_stopped_age = std::time::Duration::from_secs(5 * 60); // 5 minutes

                loop {
                    tokio::time::sleep(cleanup_interval).await;

                    let manager = agent_manager_for_cleanup.lock().await;
                    let removed_ids = manager.cleanup_stopped_agents(max_stopped_age).await;

                    if !removed_ids.is_empty() {
                        eprintln!(
                            "[Cleanup] Removed {} stopped agents from memory",
                            removed_ids.len()
                        );

                        // Cleanup security monitor expectations for removed agents
                        if let Some(ref monitor) = security_monitor_for_cleanup {
                            for agent_id in &removed_ids {
                                monitor.remove_agent_expectations(agent_id).await;
                            }
                        }
                    }
                }
            });

            // Initialize auto-pipeline manager (optional - requires API key)
            let auto_pipeline_manager = match AutoPipelineManager::new() {
                Ok(manager) => {
                    println!("✓ Auto-pipeline manager initialized");
                    Some(Arc::new(Mutex::new(manager)))
                }
                Err(e) => {
                    eprintln!("⚠ Auto-pipeline manager unavailable (no API key): {}", e);
                    None
                }
            };

            // Create shared pending question state (accessible without locking meta_agent)
            let pending_meta_question: Arc<Mutex<Option<PendingQuestion>>> =
                Arc::new(Mutex::new(None));

            // Create shared sleep state (accessible without locking meta_agent for interrupt)
            let meta_sleep_state: Arc<Mutex<SleepState>> =
                Arc::new(Mutex::new(SleepState::default()));

            // Set the shared states on the meta agent
            {
                let mut ma = meta_agent.blocking_lock();
                ma.set_pending_question(pending_meta_question.clone());
                ma.set_sleep_state(meta_sleep_state.clone());
                ma.set_conversation_db(agent_runs_db.clone());
            }

            app.manage(AppState {
                agent_manager,
                meta_agent,
                logger,
                agent_runs_db,
                auto_pipeline_manager,
                security_monitor,
                pending_elevated,
                approved_scopes,
                app_handle,
                pending_meta_question,
                meta_sleep_state,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Agent commands
            commands::create_agent,
            commands::send_prompt,
            commands::stop_agent,
            commands::list_agents,
            commands::get_agent_statistics,
            commands::list_github_repos,
            commands::resume_crashed_run,
            // Chat commands
            commands::send_chat_message,
            commands::get_chat_history,
            commands::clear_chat_history,
            commands::process_agent_results,
            commands::set_commander_personality,
            commands::get_commander_system_prompt,
            commands::reset_commander_personality,
            commands::answer_meta_agent_question,
            // Conversation persistence commands
            commands::list_conversations,
            commands::load_conversation,
            commands::new_conversation,
            commands::delete_conversation,
            commands::rename_conversation,
            commands::get_current_conversation_id,
            // Cost commands
            commands::get_cost_summary,
            commands::get_cost_by_date_range,
            commands::get_current_month_cost,
            commands::get_today_cost,
            commands::clear_cost_history,
            commands::get_cost_by_working_dir,
            commands::get_daily_costs,
            commands::get_runs_current_month_cost,
            commands::get_runs_today_cost,
            // Logging commands
            commands::query_logs,
            commands::get_recent_logs,
            commands::get_log_stats,
            commands::cleanup_old_logs,
            // Instruction commands
            commands::list_instruction_files,
            commands::get_instruction_file_content,
            commands::save_instruction_file,
            commands::open_instructions_directory,
            // Instruction analysis commands
            commands::analyze_instruction_content,
            commands::apply_instruction_suggestions,
            // Instruction wizard commands
            commands::generate_instruction_draft,
            commands::create_test_agent,
            commands::analyze_test_results,
            commands::stop_test_agent,
            commands::enhance_instruction_from_test,
            // Skill commands
            commands::generate_skill_from_instruction,
            commands::list_generated_skills,
            commands::delete_generated_skill,
            commands::get_skill_content,
            // Database commands
            commands::get_database_stats,
            commands::get_cost_database_stats,
            commands::get_all_runs,
            commands::get_run_by_id,
            commands::query_runs,
            commands::get_resumable_runs,
            commands::get_run_prompts,
            commands::get_run_stats,
            commands::cleanup_old_runs,
            commands::reconcile_stale_runs,
            // Auto-pipeline commands
            commands::create_auto_pipeline,
            commands::start_auto_pipeline,
            commands::get_auto_pipeline,
            // Security commands
            commands::get_security_status,
            commands::set_security_enabled,
            commands::get_pending_security_reviews,
            commands::approve_security_action,
            commands::reject_security_action,
            commands::clear_security_reviews,
            commands::scan_agent_activity,
            // Elevated command approval commands
            commands::get_pending_elevated_commands,
            commands::approve_elevated_command,
            commands::deny_elevated_command,
            // Event persistence commands
            commands::persist_tool_call,
            commands::persist_state_change,
            commands::persist_decision,
            commands::persist_agent_output,
            commands::get_orchestrator_tool_calls,
            commands::get_orchestrator_state_changes,
            commands::get_orchestrator_decisions,
            commands::get_agent_output_history,
            commands::get_pipeline_history,
            commands::clear_pipeline_events,
            // Config commands
            commands::check_claude_code_installed,
            commands::get_config_status,
            commands::open_config_directory,
            commands::create_env_placeholder,
            commands::update_config_value,
            commands::update_config_batch,
            commands::validate_api_key,
            // Voice commands (Dictate mode)
            voice::start_voice_session,
            voice::send_voice_audio,
            voice::stop_voice_session,
            voice::get_voice_status,
            // Discuss mode commands
            voice::start_discuss_session,
            voice::send_discuss_audio,
            voice::stop_discuss_session,
            voice::get_discuss_status,
            // Attention mode commands
            voice::start_attention_session,
            voice::send_attention_audio,
            voice::stop_attention_session,
            voice::get_attention_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
