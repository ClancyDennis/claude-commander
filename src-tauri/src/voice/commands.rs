use super::attention::AttentionSession;
use super::discuss::DiscussSession;
use super::realtime::VoiceSession;
use super::tools;
use crate::AppState;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

// Global voice session state (for Dictate mode)
static VOICE_SESSION: std::sync::OnceLock<Arc<Mutex<Option<VoiceSession>>>> =
    std::sync::OnceLock::new();

// Global discuss session state (for Discuss mode)
static DISCUSS_SESSION: std::sync::OnceLock<Arc<Mutex<Option<DiscussSession>>>> =
    std::sync::OnceLock::new();

// Global attention session state (for Attention mode)
static ATTENTION_SESSION: std::sync::OnceLock<Arc<Mutex<Option<AttentionSession>>>> =
    std::sync::OnceLock::new();

fn get_voice_session() -> &'static Arc<Mutex<Option<VoiceSession>>> {
    VOICE_SESSION.get_or_init(|| Arc::new(Mutex::new(None)))
}

fn get_discuss_session() -> &'static Arc<Mutex<Option<DiscussSession>>> {
    DISCUSS_SESSION.get_or_init(|| Arc::new(Mutex::new(None)))
}

fn get_attention_session() -> &'static Arc<Mutex<Option<AttentionSession>>> {
    ATTENTION_SESSION.get_or_init(|| Arc::new(Mutex::new(None)))
}

#[derive(Clone, serde::Serialize)]
struct VoiceTranscriptEvent {
    transcript: String,
}

#[derive(Clone, serde::Serialize)]
struct VoiceResponseEvent {
    delta: String,
}

#[derive(Clone, serde::Serialize)]
struct VoiceAudioEvent {
    audio: String,
}

#[derive(Clone, serde::Serialize)]
pub struct VoiceStatus {
    pub is_active: bool,
    pub transcript: String,
}

/// Start a voice recording session
#[tauri::command]
pub async fn start_voice_session(app_handle: tauri::AppHandle) -> Result<(), String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not set. Please set it in your .env file.")?;

    let session_lock = get_voice_session();
    let mut session_guard = session_lock.lock().await;

    // Check if already active
    if let Some(session) = session_guard.as_ref() {
        if session.is_active().await {
            return Err("Voice session already active".to_string());
        }
    }

    // Create new session
    let mut session = VoiceSession::new();

    // Clone app_handle for the callbacks
    let app_handle_transcript = app_handle.clone();
    let app_handle_response = app_handle.clone();
    let app_handle_audio = app_handle.clone();

    // Connect with transcript, response, and audio callbacks
    session
        .connect(
            &api_key,
            move |transcript| {
                // Emit transcript event to frontend (user's speech)
                let _ = app_handle_transcript.emit(
                    "voice:transcript",
                    VoiceTranscriptEvent {
                        transcript: transcript.clone(),
                    },
                );
            },
            move |delta| {
                // Emit response event to frontend (model's response text)
                let _ = app_handle_response.emit(
                    "voice:response",
                    VoiceResponseEvent {
                        delta: delta.clone(),
                    },
                );
            },
            move |audio| {
                // Emit audio event to frontend (model's response audio for playback)
                let _ = app_handle_audio.emit(
                    "voice:audio",
                    VoiceAudioEvent {
                        audio: audio.clone(),
                    },
                );
            },
        )
        .await?;

    *session_guard = Some(session);

    // Emit status event
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

/// Send an audio chunk to the voice session
#[tauri::command]
pub async fn send_voice_audio(data: String) -> Result<(), String> {
    let session_lock = get_voice_session();
    let session_guard = session_lock.lock().await;

    if let Some(session) = session_guard.as_ref() {
        session.send_audio(data).await?;
        Ok(())
    } else {
        Err("No active voice session".to_string())
    }
}

/// Stop the voice session and return the transcript
#[tauri::command]
pub async fn stop_voice_session(app_handle: tauri::AppHandle) -> Result<String, String> {
    let session_lock = get_voice_session();
    let mut session_guard = session_lock.lock().await;

    if let Some(mut session) = session_guard.take() {
        let transcript = session.stop().await;

        // Emit status event
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

/// Get the current voice session status
#[tauri::command]
pub async fn get_voice_status() -> Result<VoiceStatus, String> {
    let session_lock = get_voice_session();
    let session_guard = session_lock.lock().await;

    if let Some(session) = session_guard.as_ref() {
        Ok(VoiceStatus {
            is_active: session.is_active().await,
            transcript: String::new(), // We don't expose transcript here for privacy
        })
    } else {
        Ok(VoiceStatus {
            is_active: false,
            transcript: String::new(),
        })
    }
}

// ============================================================================
// Discuss Mode Commands
// ============================================================================

#[derive(Clone, serde::Serialize)]
struct DiscussToolCallEvent {
    name: String,
    call_id: String,
    args: String,
}

/// Start a discuss mode session
#[tauri::command]
pub async fn start_discuss_session(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not set. Please set it in your .env file.")?;

    let session_lock = get_discuss_session();
    let mut session_guard = session_lock.lock().await;

    // Check if already active
    if let Some(session) = session_guard.as_ref() {
        if session.is_active().await {
            return Err("Discuss session already active".to_string());
        }
    }

    // Create new session
    let mut session = DiscussSession::new();

    // Clone app_handle for the callbacks
    let app_handle_transcript = app_handle.clone();
    let app_handle_response = app_handle.clone();
    let app_handle_audio = app_handle.clone();
    let app_handle_tool = app_handle.clone();
    let app_handle_user_turn = app_handle.clone();
    let app_handle_assistant_turn = app_handle.clone();

    // Clone state references for tool callback
    let meta_agent = state.meta_agent.clone();
    let agent_manager = state.agent_manager.clone();

    // Connect with callbacks
    session
        .connect(
            &api_key,
            move |transcript| {
                // Emit transcript event (user's speech)
                let _ = app_handle_transcript.emit(
                    "discuss:transcript",
                    VoiceTranscriptEvent {
                        transcript: transcript.clone(),
                    },
                );
            },
            move |delta| {
                // Emit response event (AI's response text)
                let _ = app_handle_response.emit(
                    "discuss:response",
                    VoiceResponseEvent {
                        delta: delta.clone(),
                    },
                );
            },
            move |audio| {
                // Emit audio event (AI's response audio)
                let _ = app_handle_audio.emit(
                    "discuss:audio",
                    VoiceAudioEvent {
                        audio: audio.clone(),
                    },
                );
            },
            move |name, call_id, args| {
                // Execute tool and emit events
                let app_handle = app_handle_tool.clone();
                let name_clone = name.clone();
                let args_clone = args.clone();
                let meta_agent = meta_agent.clone();
                let agent_manager = agent_manager.clone();

                // Emit tool call event
                let _ = app_handle.emit(
                    "discuss:tool_call",
                    DiscussToolCallEvent {
                        name: name.clone(),
                        call_id: call_id.clone(),
                        args: args.clone(),
                    },
                );

                // Execute tool synchronously for now
                // (The callback doesn't support async, so we spawn a blocking task)
                let result = std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        tools::execute_tool_with_state(
                            &name_clone,
                            &args_clone,
                            meta_agent,
                            agent_manager,
                            app_handle,
                        )
                        .await
                    })
                })
                .join()
                .unwrap_or_else(|_| "Tool execution failed".to_string());

                result
            },
            move || {
                // User turn complete - emit event to commit message
                let _ = app_handle_user_turn.emit("discuss:user_turn_complete", ());
            },
            move || {
                // Assistant turn complete - emit event to commit message
                let _ = app_handle_assistant_turn.emit("discuss:assistant_turn_complete", ());
            },
        )
        .await?;

    *session_guard = Some(session);

    // Emit status event
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

/// Send audio to the discuss session
#[tauri::command]
pub async fn send_discuss_audio(data: String) -> Result<(), String> {
    let session_lock = get_discuss_session();
    let session_guard = session_lock.lock().await;

    if let Some(session) = session_guard.as_ref() {
        session.send_audio(data).await?;
        Ok(())
    } else {
        Err("No active discuss session".to_string())
    }
}

/// Stop the discuss session
#[tauri::command]
pub async fn stop_discuss_session(app_handle: tauri::AppHandle) -> Result<(), String> {
    let session_lock = get_discuss_session();
    let mut session_guard = session_lock.lock().await;

    if let Some(mut session) = session_guard.take() {
        session.stop().await;

        // Emit status event
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

/// Get discuss session status
#[tauri::command]
pub async fn get_discuss_status() -> Result<VoiceStatus, String> {
    let session_lock = get_discuss_session();
    let session_guard = session_lock.lock().await;

    if let Some(session) = session_guard.as_ref() {
        Ok(VoiceStatus {
            is_active: session.is_active().await,
            transcript: String::new(),
        })
    } else {
        Ok(VoiceStatus {
            is_active: false,
            transcript: String::new(),
        })
    }
}

// ============================================================================
// Attention Mode Commands
// ============================================================================

#[derive(Clone, serde::Serialize)]
struct AttentionTimeoutEvent {
    agent_id: String,
}

/// Start an attention mode session
/// This announces task completion and allows voice follow-up
#[tauri::command]
pub async fn start_attention_session(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
    agent_id: String,
    agent_title: String,
    summary: String,
) -> Result<(), String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not set. Please set it in your .env file.")?;

    let session_lock = get_attention_session();
    let mut session_guard = session_lock.lock().await;

    // Check if already active
    if let Some(session) = session_guard.as_ref() {
        if session.is_active().await {
            return Err("Attention session already active".to_string());
        }
    }

    // Create new session
    let mut session = AttentionSession::new();

    // Clone app_handle for the callbacks
    let app_handle_transcript = app_handle.clone();
    let app_handle_response = app_handle.clone();
    let app_handle_audio = app_handle.clone();
    let app_handle_tool = app_handle.clone();
    let app_handle_timeout = app_handle.clone();
    let agent_id_timeout = agent_id.clone();

    // Clone state references for tool callback
    let meta_agent = state.meta_agent.clone();
    let agent_manager = state.agent_manager.clone();

    // Connect with callbacks
    session
        .connect(
            &api_key,
            move |transcript| {
                // Emit transcript event (user's speech)
                let _ = app_handle_transcript.emit(
                    "attention:transcript",
                    VoiceTranscriptEvent {
                        transcript: transcript.clone(),
                    },
                );
            },
            move |delta| {
                // Emit response event (AI's response text)
                let _ = app_handle_response.emit(
                    "attention:response",
                    VoiceResponseEvent {
                        delta: delta.clone(),
                    },
                );
            },
            move |audio| {
                // Emit audio event (AI's response audio)
                let _ = app_handle_audio.emit(
                    "attention:audio",
                    VoiceAudioEvent {
                        audio: audio.clone(),
                    },
                );
            },
            move |name, call_id, args| {
                // Execute tool and emit events
                let app_handle = app_handle_tool.clone();
                let name_clone = name.clone();
                let args_clone = args.clone();
                let meta_agent = meta_agent.clone();
                let agent_manager = agent_manager.clone();

                // Emit tool call event
                let _ = app_handle.emit(
                    "attention:tool_call",
                    DiscussToolCallEvent {
                        name: name.clone(),
                        call_id: call_id.clone(),
                        args: args.clone(),
                    },
                );

                // Execute tool synchronously
                let result = std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        tools::execute_tool_with_state(
                            &name_clone,
                            &args_clone,
                            meta_agent,
                            agent_manager,
                            app_handle,
                        )
                        .await
                    })
                })
                .join()
                .unwrap_or_else(|_| "Tool execution failed".to_string());

                result
            },
            move || {
                // Timeout callback - emit event to frontend
                let _ = app_handle_timeout.emit(
                    "attention:timeout",
                    AttentionTimeoutEvent {
                        agent_id: agent_id_timeout.clone(),
                    },
                );
            },
        )
        .await?;

    // Send the initial prompt with the summary
    session.send_initial_prompt(&summary).await?;

    *session_guard = Some(session);

    // Emit status event
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

/// Send audio to the attention session
#[tauri::command]
pub async fn send_attention_audio(data: String) -> Result<(), String> {
    let session_lock = get_attention_session();
    let session_guard = session_lock.lock().await;

    if let Some(session) = session_guard.as_ref() {
        session.send_audio(data).await?;
        Ok(())
    } else {
        Err("No active attention session".to_string())
    }
}

/// Stop the attention session
#[tauri::command]
pub async fn stop_attention_session(app_handle: tauri::AppHandle) -> Result<(), String> {
    let session_lock = get_attention_session();
    let mut session_guard = session_lock.lock().await;

    if let Some(mut session) = session_guard.take() {
        session.stop().await;

        // Emit status event
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

/// Get attention session status
#[tauri::command]
pub async fn get_attention_status() -> Result<VoiceStatus, String> {
    let session_lock = get_attention_session();
    let session_guard = session_lock.lock().await;

    if let Some(session) = session_guard.as_ref() {
        Ok(VoiceStatus {
            is_active: session.is_active().await,
            transcript: String::new(),
        })
    } else {
        Ok(VoiceStatus {
            is_active: false,
            transcript: String::new(),
        })
    }
}
