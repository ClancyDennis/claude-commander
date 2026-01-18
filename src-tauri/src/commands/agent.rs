// Agent-related Tauri commands

use crate::types::{AgentInfo, AgentStatistics, AgentSource};
use crate::skill_generator;
use crate::AppState;
use std::sync::Arc;

#[tauri::command]
pub async fn create_agent(
    working_dir: String,
    github_url: Option<String>,
    selected_instruction_files: Option<Vec<String>>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    use tauri::Emitter;

    // Create the agent first - don't block on skill generation
    let agent_id = {
        let manager = state.agent_manager.lock().await;
        manager.create_agent(
            working_dir.clone(),
            github_url,
            selected_instruction_files.clone(),
            AgentSource::UI,
            Arc::new(app_handle.clone()),
        ).await?
    };

    // Spawn skill generation in the background if there are instruction files
    if let Some(instruction_files) = selected_instruction_files {
        if !instruction_files.is_empty() {
            let meta_agent = state.meta_agent.clone();
            let working_dir = working_dir.clone();
            let app_handle = app_handle.clone();
            let agent_id_for_events = agent_id.clone();

            tokio::spawn(async move {
                eprintln!("Generating {} skills from instruction files in background...", instruction_files.len());

                // Emit start event
                let _ = app_handle.emit("skill_generation:started", serde_json::json!({
                    "agent_id": agent_id_for_events,
                    "total": instruction_files.len(),
                }));

                let meta = meta_agent.lock().await;
                let ai_client = meta.get_ai_client();

                let mut completed = 0;
                let mut failed = 0;

                for instruction_file in &instruction_files {
                    let instruction_path = std::path::Path::new(&working_dir)
                        .join(".grove-instructions")
                        .join(instruction_file);

                    if instruction_path.exists() {
                        eprintln!("Generating skill from: {}", instruction_path.display());

                        // Emit progress event
                        let _ = app_handle.emit("skill_generation:progress", serde_json::json!({
                            "agent_id": agent_id_for_events,
                            "file": instruction_file,
                            "completed": completed,
                            "total": instruction_files.len(),
                        }));

                        match skill_generator::generate_skill_from_instruction(
                            instruction_path.to_str().unwrap(),
                            &working_dir,
                            ai_client,
                        ).await {
                            Ok(skill) => {
                                eprintln!("✓ Generated skill: {} at {}", skill.skill_name, skill.skill_path);
                                completed += 1;

                                // Emit skill completed event
                                let _ = app_handle.emit("skill_generation:skill_completed", serde_json::json!({
                                    "agent_id": agent_id_for_events,
                                    "file": instruction_file,
                                    "skill_name": skill.skill_name,
                                    "skill_path": skill.skill_path,
                                }));
                            }
                            Err(e) => {
                                eprintln!("✗ Failed to generate skill from {}: {}", instruction_file, e);
                                failed += 1;

                                // Emit skill failed event
                                let _ = app_handle.emit("skill_generation:skill_failed", serde_json::json!({
                                    "agent_id": agent_id_for_events,
                                    "file": instruction_file,
                                    "error": e.to_string(),
                                }));
                            }
                        }
                    } else {
                        eprintln!("Warning: Instruction file not found: {}", instruction_path.display());
                        failed += 1;
                    }
                }

                // Emit completion event
                let _ = app_handle.emit("skill_generation:completed", serde_json::json!({
                    "agent_id": agent_id_for_events,
                    "completed": completed,
                    "failed": failed,
                    "total": instruction_files.len(),
                }));

                eprintln!("Skill generation completed: {} successful, {} failed", completed, failed);
            });
        }
    }

    Ok(agent_id)
}

#[tauri::command]
pub async fn send_prompt(
    agent_id: String,
    prompt: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let manager = state.agent_manager.lock().await;
    manager.send_prompt(&agent_id, &prompt, Some(Arc::new(app_handle))).await
}

#[tauri::command]
pub async fn stop_agent(agent_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Get agent stats before stopping
    let _stats = {
        let manager = state.agent_manager.lock().await;
        manager.get_agent_statistics(&agent_id).await.ok()
    };

    // Stop the agent
    {
        let manager = state.agent_manager.lock().await;
        manager.stop_agent(&agent_id).await?;
    }

    // Cost data is now automatically tracked in agent_runs_db
    // via the AgentManager's integration with the database

    Ok(())
}

#[tauri::command]
pub async fn list_agents(state: tauri::State<'_, AppState>) -> Result<Vec<AgentInfo>, String> {
    let manager = state.agent_manager.lock().await;
    Ok(manager.list_agents().await)
}

#[tauri::command]
pub async fn get_agent_statistics(
    agent_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<AgentStatistics, String> {
    let manager = state.agent_manager.lock().await;
    manager.get_agent_statistics(&agent_id).await
}

#[tauri::command]
pub async fn list_github_repos() -> Result<Vec<serde_json::Value>, String> {
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
pub async fn resume_crashed_run(
    agent_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Get the run from database
    let run = state.agent_runs_db
        .get_run(&agent_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Run not found".to_string())?;

    // Check if it can be resumed
    if !run.can_resume {
        return Err("This run cannot be resumed".to_string());
    }

    // Recreate the agent with the same working directory and context
    let manager = state.agent_manager.lock().await;
    let new_agent_id = manager.create_agent(
        run.working_dir,
        run.github_url,
        None, // instruction files were already copied
        AgentSource::Manual,
        Arc::new(app_handle),
    ).await?;

    // If there was an initial prompt, resend it
    if let Some(initial_prompt) = run.initial_prompt {
        manager.send_prompt(&new_agent_id, &initial_prompt, None).await?;
    }

    Ok(new_agent_id)
}
