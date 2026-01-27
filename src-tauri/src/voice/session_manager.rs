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

use std::sync::Arc;

/// Arc-wrapped callback type aliases for use in async contexts.
pub type TranscriptCb = Arc<dyn Fn(String) + Send + Sync>;
pub type ResponseCb = Arc<dyn Fn(String) + Send + Sync>;
pub type AudioCb = Arc<dyn Fn(String) + Send + Sync>;
pub type ToolCb = Arc<dyn Fn(String, String, String) -> String + Send + Sync>;
pub type SimpleCb = Arc<dyn Fn() + Send + Sync>;

/// Unified callbacks for voice sessions.
///
/// This struct consolidates all possible callbacks used across
/// VoiceSession, DiscussSession, and AttentionSession types.
/// Optional callbacks are used for mode-specific functionality.
pub struct VoiceCallbacks {
    /// Called when user speech is transcribed
    pub on_transcript: TranscriptCb,
    /// Called when assistant response text is received
    pub on_response: ResponseCb,
    /// Called when audio data is received
    pub on_audio: AudioCb,
    /// Called when a tool call is needed (discuss, attention)
    pub on_tool_call: Option<ToolCb>,
    /// Called on inactivity timeout (attention only)
    pub on_timeout: Option<SimpleCb>,
    /// Called when user turn completes (discuss only)
    pub on_user_turn_complete: Option<SimpleCb>,
    /// Called when assistant turn completes (discuss only)
    pub on_assistant_turn_complete: Option<SimpleCb>,
}

impl VoiceCallbacks {
    /// Create callbacks for basic voice mode (transcript, response, audio only)
    pub fn basic<F, R, A>(on_transcript: F, on_response: R, on_audio: A) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
        R: Fn(String) + Send + Sync + 'static,
        A: Fn(String) + Send + Sync + 'static,
    {
        Self {
            on_transcript: Arc::new(on_transcript),
            on_response: Arc::new(on_response),
            on_audio: Arc::new(on_audio),
            on_tool_call: None,
            on_timeout: None,
            on_user_turn_complete: None,
            on_assistant_turn_complete: None,
        }
    }

    /// Add tool call callback
    pub fn with_tool_call<T>(mut self, on_tool_call: T) -> Self
    where
        T: Fn(String, String, String) -> String + Send + Sync + 'static,
    {
        self.on_tool_call = Some(Arc::new(on_tool_call));
        self
    }

    /// Add timeout callback (for attention mode)
    pub fn with_timeout<O>(mut self, on_timeout: O) -> Self
    where
        O: Fn() + Send + Sync + 'static,
    {
        self.on_timeout = Some(Arc::new(on_timeout));
        self
    }

    /// Add turn completion callbacks (for discuss mode)
    pub fn with_turn_callbacks<U, V>(
        mut self,
        on_user_turn_complete: U,
        on_assistant_turn_complete: V,
    ) -> Self
    where
        U: Fn() + Send + Sync + 'static,
        V: Fn() + Send + Sync + 'static,
    {
        self.on_user_turn_complete = Some(Arc::new(on_user_turn_complete));
        self.on_assistant_turn_complete = Some(Arc::new(on_assistant_turn_complete));
        self
    }
}

/// Voice settings from commander personality
#[derive(Clone, Debug)]
pub struct VoiceSettings {
    /// OpenAI voice: alloy, ash, ballad, coral, echo, sage, shimmer, verse
    pub voice: String,
    /// Listen timeout in seconds (for attention mode)
    pub timeout_secs: u64,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            voice: "alloy".to_string(),
            timeout_secs: 10,
        }
    }
}

/// Helper to get API key from environment.
pub fn get_api_key() -> Result<String, String> {
    std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY not set. Please set it in your .env file.".to_string())
}
