mod agent_manager;
mod hook_server;
mod types;
mod meta_agent;
mod tool_registry;
mod claude_client;
mod ai_client;
mod github;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

use agent_manager::AgentManager;
use meta_agent::MetaAgent;
use types::{AgentInfo, AgentStatistics, ChatMessage, ChatResponse};

struct AppState {
    agent_manager: Arc<Mutex<AgentManager>>,
    meta_agent: Arc<Mutex<MetaAgent>>,
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
    let manager = state.agent_manager.lock().await;
    manager.stop_agent(&agent_id).await
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load .env file if it exists (silently ignore if not found)
    let _ = dotenvy::dotenv();

    let hook_port: u16 = 19832;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let agent_manager = Arc::new(Mutex::new(AgentManager::new(hook_port)));

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

            app.manage(AppState { agent_manager, meta_agent });

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
            list_github_repos
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
