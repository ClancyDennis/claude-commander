// Integrated test for the OrchestratorAgent with actual Claude Code agents
//
// This test uses the orchestrator with AgentManager integration,
// so the tools automatically spawn Claude Code agents.
//
// Usage: cargo run --bin orchestrator_integrated_test "your task here" [working_directory]

use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value;

use claude_commander_lib::auto_pipeline::{OrchestratorAgent, OrchestratorAction};
use claude_commander_lib::events::AppEventEmitter;
use claude_commander_lib::agent_manager::AgentManager;
use claude_commander_lib::logger::Logger;
use claude_commander_lib::agent_runs_db::AgentRunsDB;
use claude_commander_lib::hook_server;

struct CliEventEmitter;

impl AppEventEmitter for CliEventEmitter {
    fn emit(&self, event: &str, payload: Value) -> Result<(), String> {
        match event {
            "agent:status" => {
                if let Some(status) = payload.get("status").and_then(|s| s.as_str()) {
                    println!("[STATUS] {}", status);
                }
            }
            "agent:output" => {
                if let Some(content) = payload.get("content").and_then(|s| s.as_str()) {
                    let output_type = payload.get("output_type").and_then(|s| s.as_str()).unwrap_or("unknown");
                    if output_type != "stream_event" && !content.trim().is_empty() {
                        // Truncate long outputs (char-safe)
                        let display: String = content.chars().take(300).collect();
                        let display = if content.chars().count() > 300 {
                            format!("{}...", display)
                        } else {
                            display
                        };
                        println!("[{}] {}", output_type.to_uppercase(), display.trim());
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Orchestrator Integrated Test ===\n");
    let _ = dotenvy::dotenv();

    // Check for API Keys
    if std::env::var("ANTHROPIC_API_KEY").is_err() && std::env::var("OPENAI_API_KEY").is_err() {
        eprintln!("Error: ANTHROPIC_API_KEY or OPENAI_API_KEY must be set.");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <user_request> [working_directory]", args[0]);
        eprintln!("\nExample:");
        eprintln!("  {} \"Create a hello world Python script\"", args[0]);
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

    println!("Working directory: {}", working_dir);
    println!("User request: {}\n", user_request);

    // Setup infrastructure
    let hook_port = 19834;
    let temp_dir = std::env::temp_dir();
    let logger = Arc::new(Logger::new(temp_dir.join("orchestrator_integrated_logs.db")).unwrap());
    let runs_db = Arc::new(AgentRunsDB::new(temp_dir.join("orchestrator_integrated_runs.db")).unwrap());

    let agent_manager = Arc::new(Mutex::new(AgentManager::with_logger_and_db(
        hook_port,
        logger,
        runs_db
    )));

    // Start Hook Server
    let event_emitter: Arc<dyn AppEventEmitter> = Arc::new(CliEventEmitter);
    let am_clone = agent_manager.clone();
    let ee_clone = event_emitter.clone();

    tokio::spawn(async move {
        if let Err(e) = hook_server::start_hook_server(am_clone, ee_clone, hook_port, None).await {
            eprintln!("Hook server failed: {}", e);
        }
    });

    // Give hook server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Create the orchestrator agent WITH agent manager integration
    println!("Creating orchestrator agent with agent manager...\n");
    let mut orchestrator = OrchestratorAgent::with_agent_manager(
        working_dir.clone(),
        user_request.to_string(),
        None, // No custom instructions
        5,    // Max iterations
        agent_manager,
        event_emitter,
    )?;

    println!("Starting pipeline...\n");
    println!("The orchestrator will:");
    println!("  1. Analyze instruction files and create relevant skills");
    println!("  2. Spawn a planning agent to create an implementation plan");
    println!("  3. Spawn a build agent to implement the plan");
    println!("  4. Spawn a verification agent to review the implementation");
    println!("  5. Decide: complete, iterate, replan, or give up\n");
    println!("========================================\n");

    // Run until we get a terminal decision
    loop {
        let action = orchestrator.run_until_action().await?;

        match action {
            OrchestratorAction::Complete { summary } => {
                println!("\n========================================");
                println!("=== PIPELINE COMPLETED SUCCESSFULLY ===");
                println!("========================================\n");
                println!("Summary: {}", summary);
                println!("\nSkills created: {:?}", orchestrator.generated_skills());
                break;
            }

            OrchestratorAction::Iterate { issues, suggestions } => {
                println!("\n--- Iterating (fixing issues) ---");
                println!("Issues: {:?}", issues);
                println!("Suggestions: {:?}", suggestions);

                orchestrator.increment_iteration();
                if orchestrator.at_max_iterations() {
                    println!("\nMax iterations reached!");
                    break;
                }

                println!("Continuing to next iteration...\n");
                // Loop continues - orchestrator will call start_execution again
            }

            OrchestratorAction::Replan { reason, issues, suggestions } => {
                println!("\n--- Replanning ---");
                println!("Reason: {}", reason);
                println!("Issues: {:?}", issues);
                println!("Suggestions: {:?}", suggestions);

                orchestrator.increment_iteration();
                if orchestrator.at_max_iterations() {
                    println!("\nMax iterations reached!");
                    break;
                }

                println!("Going back to planning phase...\n");
                // Loop continues - orchestrator will call start_planning again
            }

            OrchestratorAction::GiveUp { reason } => {
                println!("\n========================================");
                println!("=== PIPELINE GAVE UP ===");
                println!("========================================\n");
                println!("Reason: {}", reason);
                break;
            }

            // These are internal actions - shouldn't be returned anymore
            // but keeping for backward compatibility
            _ => {
                println!("Unexpected action: {:?}", action);
                continue;
            }
        }
    }

    println!("\nFinal state: {:?}", orchestrator.current_state());
    println!("\nDone.");
    Ok(())
}
