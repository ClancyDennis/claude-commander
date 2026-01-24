use super::realtime::VoiceSession;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

// Global voice session state
static VOICE_SESSION: std::sync::OnceLock<Arc<Mutex<Option<VoiceSession>>>> =
    std::sync::OnceLock::new();

fn get_voice_session() -> &'static Arc<Mutex<Option<VoiceSession>>> {
    VOICE_SESSION.get_or_init(|| Arc::new(Mutex::new(None)))
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

    // Connect with transcript and response callbacks
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
                // Emit response event to frontend (model's response)
                let _ = app_handle_response.emit(
                    "voice:response",
                    VoiceResponseEvent {
                        delta: delta.clone(),
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
