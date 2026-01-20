pub mod agent_manager;
pub mod hook_server;
pub mod types;
pub mod meta_agent;
pub mod tool_registry;
pub mod claude_client;
pub mod ai_client;
pub mod github;
pub mod logger;
pub mod pool_manager;
pub mod orchestrator;
pub mod thread_controller;
pub mod pipeline_manager;
pub mod verification;
pub mod instruction_manager;
pub mod skill_generator;
pub mod subagent_generator;
pub mod claudemd_generator;
pub mod agent_runs_db;
pub mod auto_pipeline;
pub mod utils;
pub mod commands;
pub mod events;
pub mod security_monitor;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use agent_manager::AgentManager;
use meta_agent::MetaAgent;
use logger::Logger;
use pool_manager::{AgentPool, PoolConfig};
use orchestrator::TaskOrchestrator;
use thread_controller::ThreadController;
use pipeline_manager::PipelineManager;
use verification::VerificationEngine;
use agent_runs_db::AgentRunsDB;
use auto_pipeline::AutoPipelineManager;
use security_monitor::{SecurityConfig, SecurityMonitor, ResponseConfig};

// Make AppState public so commands module can access it
pub struct AppState {
    pub agent_manager: Arc<Mutex<AgentManager>>,
    pub meta_agent: Arc<Mutex<MetaAgent>>,
    pub logger: Arc<Logger>,
    pub agent_pool: Option<Arc<Mutex<AgentPool>>>,
    pub orchestrator: Arc<Mutex<TaskOrchestrator>>,
    pub thread_controller: Arc<Mutex<ThreadController>>,
    pub pipeline_manager: Arc<Mutex<PipelineManager>>,
    pub verification_engine: Arc<Mutex<VerificationEngine>>,
    pub agent_runs_db: Arc<AgentRunsDB>,
    pub auto_pipeline_manager: Arc<Mutex<AutoPipelineManager>>,
    pub security_monitor: Option<Arc<SecurityMonitor>>,
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
                        Err(e2) => panic!("Failed to initialize logger even in temp directory: {}", e2),
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
                            println!("✓ Agent runs database initialized at temp location: {:?}", temp_db);
                            Arc::new(db)
                        }
                        Err(e2) => panic!("Failed to initialize agent runs database even in temp directory: {}", e2),
                    }
                }
            };

            let agent_manager = Arc::new(Mutex::new(AgentManager::with_logger_and_db(hook_port, logger.clone(), agent_runs_db.clone())));

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
                    // Start background analysis loop (returns JoinHandle, runs in background)
                    let _ = monitor.clone().start_background_analysis();
                    println!("✓ Security monitor initialized");
                    Some(monitor)
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to initialize security monitor: {}", e);
                    eprintln!("  Security monitoring will be disabled.");
                    None
                }
            };

            // Start hook server (with security monitor if available)
            let agent_manager_clone = agent_manager.clone();
            let app_handle = app.handle().clone();
            let security_monitor_for_hook = security_monitor.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) =
                    hook_server::start_hook_server(
                        agent_manager_clone,
                        Arc::new(app_handle),
                        hook_port,
                        security_monitor_for_hook,
                    ).await
                {
                    eprintln!("Hook server error: {}", e);
                }
            });

            // Initialize agent pool in tracking-only mode (no auto-spawn)
            let pool_config = PoolConfig::default();
            let app_handle_for_pool = app.handle().clone();
            let agent_pool = Some(AgentPool::new_tracking_only(
                pool_config,
                agent_manager.clone(),
                Some(app_handle_for_pool)
            ));
            println!("✓ Agent pool initialized in tracking mode");

            // Wire up auto-registration callback
            if let Some(pool) = &agent_pool {
                let pool_clone = pool.clone();
                let mut manager = tauri::async_runtime::block_on(agent_manager.lock());
                manager.set_on_agent_created(move |agent_id, source| {
                    let pool = pool_clone.clone();
                    tauri::async_runtime::spawn(async move {
                        let mut pool_lock = pool.lock().await;
                        if let Err(e) = pool_lock.register_agent(agent_id.clone(), source).await {
                            eprintln!("Failed to register agent {}: {}", agent_id, e);
                        }
                    });
                });
                println!("✓ Agent auto-registration callback configured");
            }

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

            // Initialize auto-pipeline manager
            let auto_pipeline_manager = match AutoPipelineManager::new() {
                Ok(manager) => Arc::new(Mutex::new(manager)),
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to initialize auto-pipeline manager: {}", e);
                    panic!("Auto-pipeline manager is required: {}", e);
                }
            };
            println!("✓ Auto-pipeline manager initialized");

            app.manage(AppState {
                agent_manager,
                meta_agent,
                logger,
                agent_pool,
                orchestrator,
                thread_controller,
                pipeline_manager,
                verification_engine,
                agent_runs_db,
                auto_pipeline_manager,
                security_monitor,
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
            // Pool commands
            commands::get_pool_stats,
            commands::configure_pool,
            commands::request_pool_agent,
            commands::release_pool_agent,
            // Workflow commands
            commands::create_workflow_from_request,
            commands::execute_workflow,
            commands::get_workflow,
            commands::list_workflows,
            commands::get_thread_config,
            commands::update_thread_config,
            commands::get_thread_stats,
            commands::emergency_shutdown_threads,
            // Pipeline commands
            commands::create_pipeline,
            commands::start_pipeline,
            commands::get_pipeline,
            commands::list_pipelines,
            commands::approve_pipeline_checkpoint,
            commands::get_pipeline_config,
            commands::update_pipeline_config,
            commands::run_best_of_n_verification,
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
            // Instruction analysis commands
            commands::analyze_instruction_content,
            commands::apply_instruction_suggestions,
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
            commands::clear_pipeline_events
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
