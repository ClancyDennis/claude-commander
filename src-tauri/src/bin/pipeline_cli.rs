
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value;
use claude_agent_manager_lib::events::AppEventEmitter;
use claude_agent_manager_lib::agent_manager::AgentManager;
use claude_agent_manager_lib::auto_pipeline::AutoPipelineManager;
use claude_agent_manager_lib::logger::Logger;
use claude_agent_manager_lib::agent_runs_db::AgentRunsDB;
use claude_agent_manager_lib::hook_server;

struct CliEventEmitter;

impl AppEventEmitter for CliEventEmitter {
    fn emit(&self, event: &str, payload: Value) -> Result<(), String> {
        // Pretty print important events
        match event {
            "agent:status" => {
                if let Some(status) = payload.get("status").and_then(|s| s.as_str()) {
                    println!("[AGENT STATUS] {}", status);
                }
            }
            "auto_pipeline:step_completed" => {
                let step = payload.get("step_number").and_then(|s| s.as_u64()).unwrap_or(0);
                println!("[PIPELINE] Step {} Completed", step);
            }
            "auto_pipeline:completed" => {
                println!("[PIPELINE] Completed!");
                if let Some(report) = payload.get("verification_report") {
                    println!("{}", serde_json::to_string_pretty(report).unwrap_or_default());
                }
            }
            "agent:output" => {
                 if let Some(content) = payload.get("content").and_then(|s| s.as_str()) {
                    let output_type = payload.get("output_type").and_then(|s| s.as_str()).unwrap_or("unknown");
                    if output_type != "stream_event" {
                         println!("[AGENT {}] {}", output_type, content.trim());
                    }
                 }
            }
            _ => {
                // Ignore other events or log debug
                // println!("[EVENT] {}: {:?}", event, payload);
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Pipeline CLI...");
    let _ = dotenvy::dotenv();

    // Check for API Keys
    if std::env::var("ANTHROPIC_API_KEY").is_err() && std::env::var("OPENAI_API_KEY").is_err() {
        eprintln!("Error: ANTHROPIC_API_KEY or OPENAI_API_KEY must be set.");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <user_request> [working_directory]", args[0]);
        std::process::exit(1);
    }
    let user_request = &args[1];
    let working_dir = if args.len() > 2 {
        args[2].clone()
    } else {
        std::env::current_dir()?.to_string_lossy().to_string()
    };
    
    // Ensure working directory exists
    std::fs::create_dir_all(&working_dir)?;
    let working_dir = std::fs::canonicalize(&working_dir)?.to_string_lossy().to_string();
    println!("Using working directory: {}", working_dir);

    // Setup infrastructure
    let hook_port = 19833; // Different from app
    let temp_dir = std::env::temp_dir();
    let logger = Arc::new(Logger::new(temp_dir.join("cli_logs.db")).unwrap());
    let runs_db = Arc::new(AgentRunsDB::new(temp_dir.join("cli_runs.db")).unwrap());

    let agent_manager = Arc::new(Mutex::new(AgentManager::with_logger_and_db(
        hook_port,
        logger,
        runs_db
    )));

    // Start Hook Server
    let event_emitter = Arc::new(CliEventEmitter);
    let am_clone = agent_manager.clone();
    let ee_clone = event_emitter.clone();
    
    tokio::spawn(async move {
        if let Err(e) = hook_server::start_hook_server(am_clone, ee_clone, hook_port).await {
            eprintln!("Hook server failed: {}", e);
        }
    });

    // Create Pipeline Manager
    let pipeline_manager = AutoPipelineManager::new().map_err(|e| format!("Failed to init manager: {}", e))?;
    
    println!("Creating pipeline for request: {}", user_request);
    let pipeline_id = pipeline_manager.create_pipeline(user_request.to_string(), working_dir).await?;
    println!("Pipeline created: {}", pipeline_id);

    println!("Executing pipeline...");
    pipeline_manager.execute_pipeline(pipeline_id, agent_manager, event_emitter).await?;

    println!("Done.");
    Ok(())
}
