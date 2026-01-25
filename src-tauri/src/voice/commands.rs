// Voice Commands - Tauri command handlers for voice sessions
//
// This module provides the #[tauri::command] entry points for voice functionality.
// It uses the session_registry and session_manager modules for unified session handling.

use super::attention::AttentionSession;
use super::discuss::DiscussSession;
use super::realtime::VoiceSession;
use super::session_manager::{
    get_api_key, AttentionTimeoutEvent, ToolCallEvent, VoiceAudioEvent, VoiceResponseEvent,
    VoiceStatus, VoiceTranscriptEvent,
};
use super::session_registry::{attention_session, discuss_session, voice_session, SessionOps};
use super::tools;
use crate::AppState;
use tauri::Emitter;

// ============================================================================
// Voice Mode Commands (Dictate)
// ============================================================================

/// Start a voice recording session (Dictate mode).
#[tauri::command]
pub async fn start_voice_session(app_handle: tauri::AppHandle) -> Result<(), String> {
    let api_key = get_api_key()?;
    let session_lock = voice_session();
    let mut session_guard = session_lock.lock().await;

    // Check if already active
    if let Some(session) = session_guard.as_ref() {
        if session.is_active().await {
            return Err("Voice session already active".to_string());
        }
    }

    // Create new session with callbacks
    let mut session = VoiceSession::new();
    let (app_t, app_r, app_a) = (app_handle.clone(), app_handle.clone(), app_handle.clone());

    session
        .connect(
            &api_key,
            move |transcript| {
                let _ = app_t.emit("voice:transcript", VoiceTranscriptEvent { transcript });
            },
            move |delta| {
                let _ = app_r.emit("voice:response", VoiceResponseEvent { delta });
            },
            move |audio| {
                let _ = app_a.emit("voice:audio", VoiceAudioEvent { audio });
            },
        )
        .await?;

    *session_guard = Some(session);
    let _ = app_handle.emit(
        "voice:status",
        VoiceStatus {
            is_active: true,
            transcript: String::new(),
        },
    );

    println!("[Voice] Session started");
    Ok(())
}

/// Send an audio chunk to the voice session.
#[tauri::command]
pub async fn send_voice_audio(data: String) -> Result<(), String> {
    voice_session().send_audio_if_active(data).await
}

/// Stop the voice session and return the transcript.
#[tauri::command]
pub async fn stop_voice_session(app_handle: tauri::AppHandle) -> Result<String, String> {
    let session_lock = voice_session();
    let mut session_guard = session_lock.lock().await;

    if let Some(mut session) = session_guard.take() {
        let transcript = session.stop().await;
        let _ = app_handle.emit(
            "voice:status",
            VoiceStatus {
                is_active: false,
                transcript: transcript.clone(),
            },
        );
        println!("[Voice] Session stopped. Transcript: {}", transcript);
        Ok(transcript)
    } else {
        Err("No active voice session".to_string())
    }
}

/// Get the current voice session status.
#[tauri::command]
pub async fn get_voice_status() -> Result<VoiceStatus, String> {
    Ok(voice_session().get_status().await)
}

// ============================================================================
// Discuss Mode Commands
// ============================================================================

/// Start a discuss mode session.
#[tauri::command]
pub async fn start_discuss_session(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let session_lock = discuss_session();
    let mut session_guard = session_lock.lock().await;

    // Check if already active
    if let Some(session) = session_guard.as_ref() {
        if session.is_active().await {
            return Err("Discuss session already active".to_string());
        }
    }

    // Create new session with callbacks
    let mut session = DiscussSession::new();
    let (app_t, app_r, app_a, app_tool, app_user, app_asst) = (
        app_handle.clone(),
        app_handle.clone(),
        app_handle.clone(),
        app_handle.clone(),
        app_handle.clone(),
        app_handle.clone(),
    );

    let meta_agent = state.meta_agent.clone();
    let agent_manager = state.agent_manager.clone();

    session
        .connect(
            &api_key,
            move |transcript| {
                let _ = app_t.emit("discuss:transcript", VoiceTranscriptEvent { transcript });
            },
            move |delta| {
                let _ = app_r.emit("discuss:response", VoiceResponseEvent { delta });
            },
            move |audio| {
                let _ = app_a.emit("discuss:audio", VoiceAudioEvent { audio });
            },
            move |name, call_id, args| {
                let app = app_tool.clone();
                let (meta, mgr) = (meta_agent.clone(), agent_manager.clone());
                let (n, a) = (name.clone(), args.clone());

                let _ = app.emit(
                    "discuss:tool_call",
                    ToolCallEvent {
                        name: name.clone(),
                        call_id: call_id.clone(),
                        args: args.clone(),
                    },
                );

                execute_tool_blocking(n, a, meta, mgr, app)
            },
            move || {
                let _ = app_user.emit("discuss:user_turn_complete", ());
            },
            move || {
                let _ = app_asst.emit("discuss:assistant_turn_complete", ());
            },
        )
        .await?;

    *session_guard = Some(session);
    let _ = app_handle.emit(
        "discuss:status",
        VoiceStatus {
            is_active: true,
            transcript: String::new(),
        },
    );

    println!("[Discuss] Session started");
    Ok(())
}

/// Send audio to the discuss session.
#[tauri::command]
pub async fn send_discuss_audio(data: String) -> Result<(), String> {
    discuss_session().send_audio_if_active(data).await
}

/// Stop the discuss session.
#[tauri::command]
pub async fn stop_discuss_session(app_handle: tauri::AppHandle) -> Result<(), String> {
    let session_lock = discuss_session();
    let mut session_guard = session_lock.lock().await;

    if let Some(mut session) = session_guard.take() {
        session.stop().await;
        let _ = app_handle.emit(
            "discuss:status",
            VoiceStatus {
                is_active: false,
                transcript: String::new(),
            },
        );
        println!("[Discuss] Session stopped");
        Ok(())
    } else {
        Err("No active discuss session".to_string())
    }
}

/// Get discuss session status.
#[tauri::command]
pub async fn get_discuss_status() -> Result<VoiceStatus, String> {
    Ok(discuss_session().get_status().await)
}

// ============================================================================
// Attention Mode Commands
// ============================================================================

/// Start an attention mode session for announcing task completion.
#[tauri::command]
pub async fn start_attention_session(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    agent_id: String,
    agent_title: String,
    summary: String,
) -> Result<(), String> {
    let api_key = get_api_key()?;
    let session_lock = attention_session();
    let mut session_guard = session_lock.lock().await;

    // Check if already active
    if let Some(session) = session_guard.as_ref() {
        if session.is_active().await {
            return Err("Attention session already active".to_string());
        }
    }

    // Create new session with callbacks
    let mut session = AttentionSession::new();
    let (app_t, app_r, app_a, app_tool, app_timeout) = (
        app_handle.clone(),
        app_handle.clone(),
        app_handle.clone(),
        app_handle.clone(),
        app_handle.clone(),
    );

    let meta_agent = state.meta_agent.clone();
    let agent_manager = state.agent_manager.clone();
    let agent_id_timeout = agent_id.clone();

    session
        .connect(
            &api_key,
            move |transcript| {
                let _ = app_t.emit("attention:transcript", VoiceTranscriptEvent { transcript });
            },
            move |delta| {
                let _ = app_r.emit("attention:response", VoiceResponseEvent { delta });
            },
            move |audio| {
                let _ = app_a.emit("attention:audio", VoiceAudioEvent { audio });
            },
            move |name, call_id, args| {
                let app = app_tool.clone();
                let (meta, mgr) = (meta_agent.clone(), agent_manager.clone());
                let (n, a) = (name.clone(), args.clone());

                let _ = app.emit(
                    "attention:tool_call",
                    ToolCallEvent {
                        name: name.clone(),
                        call_id: call_id.clone(),
                        args: args.clone(),
                    },
                );

                execute_tool_blocking(n, a, meta, mgr, app)
            },
            move || {
                let _ = app_timeout.emit(
                    "attention:timeout",
                    AttentionTimeoutEvent {
                        agent_id: agent_id_timeout.clone(),
                    },
                );
            },
        )
        .await?;

    // Send initial prompt with the summary
    session.send_initial_prompt(&summary).await?;

    *session_guard = Some(session);
    let _ = app_handle.emit(
        "attention:status",
        VoiceStatus {
            is_active: true,
            transcript: String::new(),
        },
    );

    println!(
        "[Attention] Session started for agent: {} ({})",
        agent_title, agent_id
    );
    Ok(())
}

/// Send audio to the attention session.
#[tauri::command]
pub async fn send_attention_audio(data: String) -> Result<(), String> {
    attention_session().send_audio_if_active(data).await
}

/// Stop the attention session.
#[tauri::command]
pub async fn stop_attention_session(app_handle: tauri::AppHandle) -> Result<(), String> {
    let session_lock = attention_session();
    let mut session_guard = session_lock.lock().await;

    if let Some(mut session) = session_guard.take() {
        session.stop().await;
        let _ = app_handle.emit(
            "attention:status",
            VoiceStatus {
                is_active: false,
                transcript: String::new(),
            },
        );
        println!("[Attention] Session stopped");
        Ok(())
    } else {
        Err("No active attention session".to_string())
    }
}

/// Get attention session status.
#[tauri::command]
pub async fn get_attention_status() -> Result<VoiceStatus, String> {
    Ok(attention_session().get_status().await)
}

// ============================================================================
// Shared Helpers
// ============================================================================

/// Execute a tool call in a blocking context (for sync callbacks).
///
/// This helper spawns a thread with its own tokio runtime to execute
/// the async tool call, since the callback doesn't support async directly.
fn execute_tool_blocking(
    name: String,
    args: String,
    meta_agent: std::sync::Arc<tokio::sync::Mutex<crate::meta_agent::MetaAgent>>,
    agent_manager: std::sync::Arc<tokio::sync::Mutex<crate::agent_manager::AgentManager>>,
    app_handle: tauri::AppHandle,
) -> String {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            tools::execute_tool_with_state(&name, &args, meta_agent, agent_manager, app_handle)
                .await
        })
    })
    .join()
    .unwrap_or_else(|_| "Tool execution failed".to_string())
}
