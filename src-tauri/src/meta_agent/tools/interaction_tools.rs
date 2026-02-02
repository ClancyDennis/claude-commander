// User interaction tools for MetaAgent
//
// These tools allow the meta-agent to communicate with users and manage its workflow:
// - Sleep: Pause with interruptible wait
// - UpdateUser: Send status updates
// - AskUserQuestion: Block and wait for user input
// - CompleteTask: Signal task completion (handled in mod.rs)

use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot, Mutex};

use crate::agent_manager::AgentManager;
use crate::ai_client::Message;
use crate::meta_agent::helpers::error;
use crate::meta_agent::memory_worker::MemoryWorker;
use crate::types::AgentWakeEvent;

// ============================================================================
// Shared State Types
// ============================================================================

/// State for a pending user question
pub struct PendingQuestion {
    pub question_id: String,
    pub response_tx: oneshot::Sender<String>,
}

/// Shared state for interruptible sleep
#[derive(Default)]
pub struct SleepState {
    pub is_sleeping: bool,
    pub cancel_tx: Option<oneshot::Sender<String>>,
    /// Receiver for agent wake events (created fresh each sleep)
    pub wake_rx: Option<mpsc::Receiver<AgentWakeEvent>>,
}

/// Sender for agent wake events - stored separately in AppState for access by result handlers
pub type AgentWakeSender = mpsc::Sender<AgentWakeEvent>;

// ============================================================================
// Sleep Tool
// ============================================================================

/// Sleep for a duration, interruptible by user messages or agent state changes.
/// Returns current agent status when waking up.
/// If sleep duration >= 1 minute, queues a memory evaluation task.
pub async fn sleep_tool(
    input: Value,
    app_handle: &AppHandle,
    sleep_state: Arc<Mutex<SleepState>>,
    agent_manager: Arc<Mutex<AgentManager>>,
    agent_wake_tx: Arc<Mutex<Option<AgentWakeSender>>>,
    memory_worker: Option<Arc<MemoryWorker>>,
    recent_messages: Option<Vec<Message>>,
) -> Value {
    let duration_mins = input["duration_minutes"].as_f64().unwrap_or(1.0);
    let duration_ms = (duration_mins * 60_000.0) as u64;
    let reason = input["reason"].as_str().unwrap_or("Waiting...");

    // Queue memory evaluation for sleeps >= 1 minute
    // The worker has its own persistent runtime and AIClient
    if duration_mins >= 1.0 {
        if let (Some(worker), Some(messages)) = (&memory_worker, recent_messages) {
            eprintln!(
                "[Sleep] Queuing memory evaluation ({} messages, {:.1}min sleep)",
                messages.len(),
                duration_mins
            );
            worker.queue_evaluation(messages);
        }
    }

    // Create cancellation channel for user messages
    let (cancel_tx, cancel_rx) = oneshot::channel::<String>();

    // Create wake channel for agent events (buffer of 16 should be plenty)
    let (wake_tx, mut wake_rx) = mpsc::channel::<AgentWakeEvent>(16);

    // Set sleep state and store the wake sender for result handlers
    {
        let mut state = sleep_state.lock().await;
        state.is_sleeping = true;
        state.cancel_tx = Some(cancel_tx);
    }
    {
        let mut wake_sender = agent_wake_tx.lock().await;
        *wake_sender = Some(wake_tx);
    }

    // Notify frontend that we're sleeping
    let _ = app_handle.emit(
        "meta-agent:status",
        json!({
            "status": "sleeping",
            "duration_ms": duration_ms,
            "reason": reason
        }),
    );

    // Race: sleep timer vs user interrupt vs agent wake event
    let result = tokio::select! {
        _ = tokio::time::sleep(std::time::Duration::from_millis(duration_ms)) => {
            // Natural wake - get current agent status
            let agents_status = get_agents_summary(&agent_manager).await;
            json!({
                "success": true,
                "completed": true,
                "slept_ms": duration_ms,
                "current_status": agents_status,
                "message": "Sleep completed. Here is the current status of all agents."
            })
        }
        result = cancel_rx => {
            match result {
                Ok(user_message) => {
                    // User interrupted - include their message
                    let agents_status = get_agents_summary(&agent_manager).await;
                    json!({
                        "success": true,
                        "interrupted": true,
                        "user_message": user_message,
                        "current_status": agents_status,
                        "message": "Sleep interrupted by user. Process their message."
                    })
                }
                Err(_) => {
                    // Channel closed unexpectedly
                    let agents_status = get_agents_summary(&agent_manager).await;
                    json!({
                        "success": true,
                        "interrupted": true,
                        "current_status": agents_status,
                        "message": "Sleep cancelled."
                    })
                }
            }
        }
        Some(wake_event) = wake_rx.recv() => {
            // Agent triggered wake event
            let agents_status = get_agents_summary(&agent_manager).await;
            json!({
                "success": true,
                "interrupted": true,
                "agent_wake": {
                    "agent_id": wake_event.agent_id,
                    "reason": wake_event.reason.as_str(),
                    "description": wake_event.reason.to_string()
                },
                "current_status": agents_status,
                "message": format!("Sleep interrupted: Agent {} is {}.",
                    &wake_event.agent_id[..8.min(wake_event.agent_id.len())],
                    wake_event.reason)
            })
        }
    };

    // Clear sleep state and remove wake sender
    {
        let mut state = sleep_state.lock().await;
        state.is_sleeping = false;
        state.cancel_tx = None;
    }
    {
        let mut wake_sender = agent_wake_tx.lock().await;
        *wake_sender = None;
    }

    // Notify frontend that we're awake
    let _ = app_handle.emit("meta-agent:status", json!({ "status": "awake" }));

    result
}

/// Helper to get a summary of all agents' status
async fn get_agents_summary(agent_manager: &Arc<Mutex<AgentManager>>) -> Value {
    let manager = agent_manager.lock().await;
    let agents = manager.list_agents().await;

    let agent_summaries: Vec<Value> = agents
        .iter()
        .map(|a| {
            json!({
                "id": &a.id[..8.min(a.id.len())],  // Shortened ID
                "status": format!("{:?}", a.status).to_lowercase(),
                "working_dir": a.working_dir,
                "is_processing": a.is_processing,
                "pending_input": a.pending_input
            })
        })
        .collect();

    json!({
        "agents": agent_summaries,
        "total_count": agents.len(),
        "running_count": agents.iter().filter(|a| a.is_processing).count()
    })
}

// ============================================================================
// UpdateUser Tool
// ============================================================================

/// Send a non-blocking status update to the user
pub async fn update_user(input: Value, app_handle: &AppHandle) -> Value {
    let message = match input["message"].as_str() {
        Some(m) if !m.is_empty() => m,
        _ => return error("message is required"),
    };

    let level = input["level"].as_str().unwrap_or("info");
    let timestamp = chrono::Utc::now().timestamp_millis();

    // Emit event to frontend
    let _ = app_handle.emit(
        "meta-agent:user-update",
        json!({
            "message": message,
            "level": level,
            "timestamp": timestamp
        }),
    );

    json!({
        "success": true,
        "message": "Update sent to user"
    })
}

// ============================================================================
// AskUserQuestion Tool
// ============================================================================

/// Ask the user a question and wait for their response (blocking)
pub async fn ask_user_question(
    input: Value,
    app_handle: &AppHandle,
    pending_question: Arc<Mutex<Option<PendingQuestion>>>,
) -> Value {
    let question = match input["question"].as_str() {
        Some(q) if !q.is_empty() => q,
        _ => return error("question is required"),
    };

    let options: Option<Vec<String>> = input["options"].as_array().map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    });

    let question_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel::<String>();

    // Store pending question so the answer command can find it
    {
        let mut pending = pending_question.lock().await;
        *pending = Some(PendingQuestion {
            question_id: question_id.clone(),
            response_tx: tx,
        });
    }

    // Emit question event to frontend
    let _ = app_handle.emit(
        "meta-agent:question",
        json!({
            "question_id": question_id,
            "question": question,
            "options": options,
            "timestamp": chrono::Utc::now().timestamp_millis()
        }),
    );

    // Wait for response with 5 minute timeout
    let timeout_duration = std::time::Duration::from_secs(300);
    match tokio::time::timeout(timeout_duration, rx).await {
        Ok(Ok(answer)) => {
            // Clear pending question
            {
                let mut pending = pending_question.lock().await;
                *pending = None;
            }
            json!({
                "success": true,
                "answer": answer,
                "question_id": question_id
            })
        }
        Ok(Err(_)) => {
            // Channel closed (question cancelled)
            {
                let mut pending = pending_question.lock().await;
                *pending = None;
            }
            json!({
                "success": false,
                "error": "Question was cancelled",
                "question_id": question_id
            })
        }
        Err(_) => {
            // Timeout
            {
                let mut pending = pending_question.lock().await;
                *pending = None;
            }
            json!({
                "success": false,
                "error": "User did not respond within 5 minutes",
                "question_id": question_id,
                "timed_out": true
            })
        }
    }
}
