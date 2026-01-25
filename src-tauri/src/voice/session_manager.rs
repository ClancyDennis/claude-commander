// Session Manager - Defines traits for voice session lifecycle management
//
// This module provides a generic trait that captures the common behavior
// across VoiceSession, DiscussSession, and AttentionSession types.

use async_trait::async_trait;

/// Common trait for all voice session types.
///
/// This trait captures the shared behavior across Voice, Discuss, and Attention sessions,
/// enabling unified session management and reducing code duplication in command handlers.
#[async_trait]
pub trait SessionHandler: Send + Sync {
    /// Check if the session is currently active.
    async fn is_active(&self) -> bool;

    /// Send audio data to the session.
    ///
    /// # Arguments
    /// * `audio_data` - Base64-encoded PCM16 audio data
    async fn send_audio(&self, audio_data: String) -> Result<(), String>;

    /// Stop the session and perform cleanup.
    async fn stop(&mut self);

    /// Get the session type name for logging purposes.
    fn session_type_name(&self) -> &'static str;
}

/// Session lifecycle events that can be emitted to the frontend.
#[derive(Clone, serde::Serialize)]
pub struct VoiceTranscriptEvent {
    pub transcript: String,
}

#[derive(Clone, serde::Serialize)]
pub struct VoiceResponseEvent {
    pub delta: String,
}

#[derive(Clone, serde::Serialize)]
pub struct VoiceAudioEvent {
    pub audio: String,
}

#[derive(Clone, serde::Serialize)]
pub struct VoiceStatus {
    pub is_active: bool,
    pub transcript: String,
}

#[derive(Clone, serde::Serialize)]
pub struct ToolCallEvent {
    pub name: String,
    pub call_id: String,
    pub args: String,
}

#[derive(Clone, serde::Serialize)]
pub struct AttentionTimeoutEvent {
    pub agent_id: String,
}

/// Callback types used by session handlers.
/// These are type aliases to document the expected callback signatures.
pub mod callbacks {
    /// Callback for transcript events (user's speech).
    pub type OnTranscript = Box<dyn Fn(String) + Send + Sync + 'static>;

    /// Callback for response events (AI's response text).
    pub type OnResponse = Box<dyn Fn(String) + Send + Sync + 'static>;

    /// Callback for audio events (AI's response audio).
    pub type OnAudio = Box<dyn Fn(String) + Send + Sync + 'static>;

    /// Callback for tool call events. Returns the tool result.
    pub type OnToolCall = Box<dyn Fn(String, String, String) -> String + Send + Sync + 'static>;

    /// Callback for turn completion events.
    pub type OnTurnComplete = Box<dyn Fn() + Send + Sync + 'static>;

    /// Callback for timeout events (attention mode).
    pub type OnTimeout = Box<dyn Fn() + Send + Sync + 'static>;
}

/// Helper to get API key from environment.
pub fn get_api_key() -> Result<String, String> {
    std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not set. Please set it in your .env file.".to_string())
}
