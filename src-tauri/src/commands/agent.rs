// Agent-related Tauri commands

use crate::skill_generator;
use crate::types::{AgentInfo, AgentSource, AgentStatistics};
use crate::AppState;
use std::sync::Arc;

/// Get the global instructions directory (~/.instructions/)
fn get_instructions_dir() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "Could not determine home directory".to_string())?;
    Ok(home.join(".instructions"))
}

#[tauri::command]
pub async fn create_agent(
    working_dir: String,
    github_url: Option<String>,
    selected_instruction_files: Option<Vec<String>>,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    use tauri::Emitter;

    // Generate skills BEFORE creating the agent (synchronously)
    // This ensures skills are available in .claude/skills/ when the agent starts
    let mut generated_skill_names: Vec<String> = Vec::new();

    if let Some(ref instruction_files) = selected_instruction_files {
        if !instruction_files.is_empty() {
            eprintln!(
                "Generating {} skills from instruction files...",
                instruction_files.len()
            );

            // Emit start event
            let _ = app_handle.emit(
                "skill_generation:started",
                serde_json::json!({
                    "total": instruction_files.len(),
                }),
            );

            let meta = state.meta_agent.lock().await;
            let ai_client = meta.get_ai_client();

            let mut completed = 0;
            let mut skipped = 0;
            let mut failed = 0;
            let mut shown_auth_warning = false;

            // Get the global instructions directory
            let instructions_dir = get_instructions_dir()?;

            for instruction_file in instruction_files {
                let instruction_path = instructions_dir.join(instruction_file);

                // Check if skill already exists for this instruction file
                if let Some(existing_skill) = skill_generator::find_existing_skill_for_instruction(
                    instruction_file,
                    &working_dir,
                ) {
                    eprintln!("⏭ Skill already exists: {} (skipping)", existing_skill);
                    skipped += 1;

                    // Emit skipped event
                    let _ = app_handle.emit(
                        "skill_generation:skill_skipped",
                        serde_json::json!({
                            "file": instruction_file,
                            "skill_name": existing_skill,
                            "reason": "already_exists",
                        }),
                    );

                    // Don't add to generated_skill_names - we didn't generate it, so don't clean it up
                    continue;
                }

                if instruction_path.exists() {
                    eprintln!("Generating skill from: {}", instruction_path.display());

                    // Emit progress event
                    let _ = app_handle.emit(
                        "skill_generation:progress",
                        serde_json::json!({
                            "file": instruction_file,
                            "completed": completed,
                            "total": instruction_files.len(),
                        }),
                    );

                    match skill_generator::generate_skill_from_instruction(
                        instruction_path.to_str().unwrap(),
                        &working_dir,
                        ai_client,
                    )
                    .await
                    {
                        Ok(skill) => {
                            eprintln!(
                                "✓ Generated skill: {} at {}",
                                skill.skill_name, skill.skill_path
                            );
                            generated_skill_names.push(skill.skill_name.clone());
                            completed += 1;

                            // Emit skill completed event
                            let _ = app_handle.emit(
                                "skill_generation:skill_completed",
                                serde_json::json!({
                                    "file": instruction_file,
                                    "skill_name": skill.skill_name,
                                    "skill_path": skill.skill_path,
                                }),
                            );
                        }
                        Err(e) => {
                            // Check if this is an authentication error - if so, use fallback
                            if skill_generator::is_auth_error(&e) {
                                eprintln!(
                                    "⚠ API key issue detected, creating basic skill from: {}",
                                    instruction_file
                                );

                                match skill_generator::create_fallback_skill_from_instruction(
                                    instruction_path.to_str().unwrap(),
                                    &working_dir,
                                ) {
                                    Ok(skill) => {
                                        eprintln!("✓ Created basic skill: {} (limited - no AI enhancement)", skill.skill_name);
                                        generated_skill_names.push(skill.skill_name.clone());
                                        completed += 1;

                                        // Emit skill completed event (marked as limited)
                                        let _ = app_handle.emit(
                                            "skill_generation:skill_completed",
                                            serde_json::json!({
                                                "file": instruction_file,
                                                "skill_name": skill.skill_name,
                                                "skill_path": skill.skill_path,
                                                "limited": true,
                                            }),
                                        );

                                        // Emit toast notification about limited skills (only once)
                                        if !shown_auth_warning {
                                            shown_auth_warning = true;
                                            let _ = app_handle.emit("toast", serde_json::json!({
                                                "type": "warning",
                                                "message": "Skills created in basic mode (API key not configured). Skills will work but without AI-enhanced structure.",
                                                "duration": 6000,
                                            }));
                                        }
                                    }
                                    Err(fallback_err) => {
                                        eprintln!(
                                            "✗ Failed to create basic skill: {}",
                                            fallback_err
                                        );
                                        failed += 1;

                                        let _ = app_handle.emit(
                                            "skill_generation:skill_failed",
                                            serde_json::json!({
                                                "file": instruction_file,
                                                "error": fallback_err,
                                            }),
                                        );
                                    }
                                }
                            } else {
                                // Print the full error with details for debugging
                                eprintln!("✗ Failed to generate skill from {}:", instruction_file);
                                for line in e.lines() {
                                    eprintln!("  {}", line);
                                }
                                failed += 1;

                                // Emit skill failed event with full error details
                                let _ = app_handle.emit(
                                    "skill_generation:skill_failed",
                                    serde_json::json!({
                                        "file": instruction_file,
                                        "error": e.to_string(),
                                    }),
                                );
                            }
                        }
                    }
                } else {
                    eprintln!(
                        "Warning: Instruction file not found: {}",
                        instruction_path.display()
                    );
                    failed += 1;
                }
            }

            // Emit completion event
            let _ = app_handle.emit(
                "skill_generation:completed",
                serde_json::json!({
                    "completed": completed,
                    "skipped": skipped,
                    "failed": failed,
                    "total": instruction_files.len(),
                }),
            );

            eprintln!(
                "Skill generation completed: {} generated, {} skipped, {} failed",
                completed, skipped, failed
            );
        }
    }

    // Create the agent with the generated skill names for cleanup tracking
    let agent_id = {
        let manager = state.agent_manager.lock().await;
        manager
            .create_agent_with_skills(
                working_dir.clone(),
                github_url,
                generated_skill_names,
                AgentSource::UI,
                Arc::new(app_handle.clone()),
            )
            .await?
    };

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
    manager
        .send_prompt(
            &agent_id,
            &prompt,
            Some(Arc::new(app_handle)),
            state.security_monitor.clone(),
        )
        .await
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
    let repos: Vec<serde_json::Value> =
        serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse gh output: {}", e))?;

    Ok(repos)
}

#[tauri::command]
pub async fn resume_crashed_run(
    agent_id: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // Get the run from database
    let run = state
        .agent_runs_db
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
    let new_agent_id = manager
        .create_agent(
            run.working_dir,
            run.github_url,
            None, // instruction files were already copied
            AgentSource::Manual,
            Arc::new(app_handle),
        )
        .await?;

    // If there was an initial prompt, resend it
    // Note: For resumed runs, we don't pass security_monitor to avoid re-analyzing
    // the prompt that was already analyzed in the original run
    if let Some(initial_prompt) = run.initial_prompt {
        manager
            .send_prompt(&new_agent_id, &initial_prompt, None, None)
            .await?;
    }

    Ok(new_agent_id)
}
