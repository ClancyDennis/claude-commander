// Session Registry - Unified storage for all voice session types
//
// This module replaces the three separate OnceLock statics with a unified
// registry pattern that provides type-safe access to each session type.

use super::attention::AttentionSession;
use super::discuss::DiscussSession;
use super::realtime::VoiceSession;
use super::session_manager::VoiceStatus;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

/// Global session registry holding all voice session types.
static SESSION_REGISTRY: OnceLock<SessionRegistry> = OnceLock::new();

/// Unified registry for all voice session types.
///
/// This replaces the three separate `OnceLock<Arc<Mutex<Option<T>>>>` statics
/// with a single registry that provides consistent access patterns.
pub struct SessionRegistry {
    voice: Arc<Mutex<Option<VoiceSession>>>,
    discuss: Arc<Mutex<Option<DiscussSession>>>,
    attention: Arc<Mutex<Option<AttentionSession>>>,
}

impl SessionRegistry {
    /// Create a new empty session registry.
    fn new() -> Self {
        Self {
            voice: Arc::new(Mutex::new(None)),
            discuss: Arc::new(Mutex::new(None)),
            attention: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the global session registry instance.
    pub fn global() -> &'static SessionRegistry {
        SESSION_REGISTRY.get_or_init(SessionRegistry::new)
    }

    /// Get the voice session lock.
    pub fn voice(&self) -> &Arc<Mutex<Option<VoiceSession>>> {
        &self.voice
    }

    /// Get the discuss session lock.
    pub fn discuss(&self) -> &Arc<Mutex<Option<DiscussSession>>> {
        &self.discuss
    }

    /// Get the attention session lock.
    pub fn attention(&self) -> &Arc<Mutex<Option<AttentionSession>>> {
        &self.attention
    }
}

/// Session type discriminant for unified operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Voice,
    Discuss,
    Attention,
}

impl SessionType {
    /// Get the event prefix for this session type.
    pub fn event_prefix(&self) -> &'static str {
        match self {
            SessionType::Voice => "voice",
            SessionType::Discuss => "discuss",
            SessionType::Attention => "attention",
        }
    }

    /// Get a human-readable name for logging.
    pub fn name(&self) -> &'static str {
        match self {
            SessionType::Voice => "Voice",
            SessionType::Discuss => "Discuss",
            SessionType::Attention => "Attention",
        }
    }
}

/// Helper trait for unified session operations.
///
/// This trait provides common operations across all session types,
/// enabling generic handling in command functions.
#[async_trait::async_trait]
pub trait SessionOps<T> {
    /// Check if a session exists and is active.
    async fn is_session_active(&self) -> bool;

    /// Get the current session status.
    async fn get_status(&self) -> VoiceStatus;

    /// Send audio to the session if active.
    async fn send_audio_if_active(&self, data: String) -> Result<(), String>;

    /// Get a reference to the session if active.
    async fn with_session<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&T) -> R + Send,
        R: Send;

    /// Get a mutable reference to the session if active.
    async fn with_session_mut<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut T) -> R + Send,
        R: Send;
}

#[async_trait::async_trait]
impl SessionOps<VoiceSession> for Arc<Mutex<Option<VoiceSession>>> {
    async fn is_session_active(&self) -> bool {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            session.is_active().await
        } else {
            false
        }
    }

    async fn get_status(&self) -> VoiceStatus {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            VoiceStatus {
                is_active: session.is_active().await,
                transcript: String::new(),
            }
        } else {
            VoiceStatus {
                is_active: false,
                transcript: String::new(),
            }
        }
    }

    async fn send_audio_if_active(&self, data: String) -> Result<(), String> {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            session.send_audio(data).await
        } else {
            Err("No active voice session".to_string())
        }
    }

    async fn with_session<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&VoiceSession) -> R + Send,
        R: Send,
    {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            Ok(f(session))
        } else {
            Err("No active voice session".to_string())
        }
    }

    async fn with_session_mut<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut VoiceSession) -> R + Send,
        R: Send,
    {
        let mut guard = self.lock().await;
        if let Some(session) = guard.as_mut() {
            Ok(f(session))
        } else {
            Err("No active voice session".to_string())
        }
    }
}

#[async_trait::async_trait]
impl SessionOps<DiscussSession> for Arc<Mutex<Option<DiscussSession>>> {
    async fn is_session_active(&self) -> bool {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            session.is_active().await
        } else {
            false
        }
    }

    async fn get_status(&self) -> VoiceStatus {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            VoiceStatus {
                is_active: session.is_active().await,
                transcript: String::new(),
            }
        } else {
            VoiceStatus {
                is_active: false,
                transcript: String::new(),
            }
        }
    }

    async fn send_audio_if_active(&self, data: String) -> Result<(), String> {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            session.send_audio(data).await
        } else {
            Err("No active discuss session".to_string())
        }
    }

    async fn with_session<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&DiscussSession) -> R + Send,
        R: Send,
    {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            Ok(f(session))
        } else {
            Err("No active discuss session".to_string())
        }
    }

    async fn with_session_mut<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut DiscussSession) -> R + Send,
        R: Send,
    {
        let mut guard = self.lock().await;
        if let Some(session) = guard.as_mut() {
            Ok(f(session))
        } else {
            Err("No active discuss session".to_string())
        }
    }
}

#[async_trait::async_trait]
impl SessionOps<AttentionSession> for Arc<Mutex<Option<AttentionSession>>> {
    async fn is_session_active(&self) -> bool {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            session.is_active().await
        } else {
            false
        }
    }

    async fn get_status(&self) -> VoiceStatus {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            VoiceStatus {
                is_active: session.is_active().await,
                transcript: String::new(),
            }
        } else {
            VoiceStatus {
                is_active: false,
                transcript: String::new(),
            }
        }
    }

    async fn send_audio_if_active(&self, data: String) -> Result<(), String> {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            session.send_audio(data).await
        } else {
            Err("No active attention session".to_string())
        }
    }

    async fn with_session<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&AttentionSession) -> R + Send,
        R: Send,
    {
        let guard = self.lock().await;
        if let Some(session) = guard.as_ref() {
            Ok(f(session))
        } else {
            Err("No active attention session".to_string())
        }
    }

    async fn with_session_mut<F, R>(&self, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut AttentionSession) -> R + Send,
        R: Send,
    {
        let mut guard = self.lock().await;
        if let Some(session) = guard.as_mut() {
            Ok(f(session))
        } else {
            Err("No active attention session".to_string())
        }
    }
}

/// Convenience functions for accessing sessions.
pub fn voice_session() -> &'static Arc<Mutex<Option<VoiceSession>>> {
    SessionRegistry::global().voice()
}

pub fn discuss_session() -> &'static Arc<Mutex<Option<DiscussSession>>> {
    SessionRegistry::global().discuss()
}

pub fn attention_session() -> &'static Arc<Mutex<Option<AttentionSession>>> {
    SessionRegistry::global().attention()
}
