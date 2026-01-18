// Test CLI for the new OrchestratorAgent tool-calling loop
//
// Usage: cargo run --bin orchestrator_test "your task here" [working_directory]

use claude_commander_lib::auto_pipeline::{OrchestratorAgent, OrchestratorAction, PipelineState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Orchestrator Agent Test ===\n");
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

    // Create the orchestrator agent
    println!("Creating orchestrator agent...");
    let mut agent = OrchestratorAgent::new(
        working_dir.clone(),
        user_request.to_string(),
        None, // No custom instructions
        5,    // Max iterations
    )?;

    println!("Initial state: {:?}\n", agent.current_state());

    // Run the skill synthesis phase
    println!("--- Phase A: Skill Synthesis ---");
    println!("Running orchestrator until phase transition...\n");

    loop {
        let action = agent.run_until_action().await?;

        println!("\n>>> Action received: {:?}", action);

        match action {
            OrchestratorAction::Continue => {
                // Keep running
                continue;
            }
            OrchestratorAction::StartPlanning { summary } => {
                println!("\n--- Transitioning to Planning Phase ---");
                println!("Summary: {}", summary);
                println!("Skills created: {:?}", agent.generated_skills());
                agent.set_state(PipelineState::Planning);

                // In a real pipeline, we would spawn the planning agent here
                // For this test, we'll simulate some output
                let simulated_plan = r#"
## Plan
1. Create the file
2. Add the code
3. Test it

## Questions
- What Python version should be used?
"#;
                agent.add_context("user", &format!(
                    "Planning agent output:\n{}\n\nReview this plan and call approve_plan if it looks good.",
                    simulated_plan
                ));
            }
            OrchestratorAction::ApprovePlan { assessment } => {
                println!("\n--- Plan Approved ---");
                println!("Assessment: {}", assessment);
                agent.set_state(PipelineState::ReadyForExecution);

                // Simulate execution
                agent.add_context("user", "Execution complete. The hello_world.py file was created. Call start_verification.");
            }
            OrchestratorAction::StartExecution { notes } => {
                println!("\n--- Starting Execution ---");
                if let Some(n) = notes {
                    println!("Notes: {}", n);
                }
                agent.set_state(PipelineState::Executing);

                // Simulate execution result
                agent.add_context("user", "Build agent output:\n- Created hello_world.py\n- File contains: print('Hello World')\n\nCall start_verification to review.");
            }
            OrchestratorAction::StartVerification { focus_areas } => {
                println!("\n--- Starting Verification ---");
                println!("Focus areas: {:?}", focus_areas);
                agent.set_state(PipelineState::Verifying);

                // Simulate verification result
                agent.add_context("user", r#"Verification report:
- File exists: YES
- Syntax valid: YES
- Runs successfully: YES
- Output correct: YES

All checks passed. Call complete if satisfied, or iterate/replan if issues found."#);
            }
            OrchestratorAction::Complete { summary } => {
                println!("\n=== PIPELINE COMPLETED ===");
                println!("Summary: {}", summary);
                println!("Total skills created: {}", agent.generated_skills().len());
                println!("Skills: {:?}", agent.generated_skills());
                break;
            }
            OrchestratorAction::Iterate { issues, suggestions } => {
                println!("\n--- Iterating ---");
                println!("Issues: {:?}", issues);
                println!("Suggestions: {:?}", suggestions);
                agent.increment_iteration();

                if agent.at_max_iterations() {
                    println!("Max iterations reached!");
                    break;
                }

                agent.set_state(PipelineState::Executing);
                agent.add_context("user", "Please fix the issues and try again.");
            }
            OrchestratorAction::Replan { reason, issues, suggestions } => {
                println!("\n--- Replanning ---");
                println!("Reason: {}", reason);
                println!("Issues: {:?}", issues);
                println!("Suggestions: {:?}", suggestions);
                agent.increment_iteration();

                if agent.at_max_iterations() {
                    println!("Max iterations reached!");
                    break;
                }

                agent.set_state(PipelineState::Planning);
                agent.add_context("user", "Please create a new plan addressing the issues.");
            }
            OrchestratorAction::GiveUp { reason } => {
                println!("\n=== PIPELINE GAVE UP ===");
                println!("Reason: {}", reason);
                break;
            }
        }
    }

    println!("\nFinal state: {:?}", agent.current_state());
    println!("\nDone.");
    Ok(())
}
