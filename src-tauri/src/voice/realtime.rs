use async_openai::types::realtime::{ClientEvent, InputAudioBufferAppendEvent, ServerEvent};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// OpenAI Realtime API session
pub struct VoiceSession {
    /// Sender for outgoing WebSocket messages
    ws_sender: Option<mpsc::Sender<String>>,
    /// Accumulated transcript
    transcript: Arc<Mutex<String>>,
    /// Session active flag
    is_active: Arc<Mutex<bool>>,
}

impl VoiceSession {
    pub fn new() -> Self {
        Self {
            ws_sender: None,
            transcript: Arc::new(Mutex::new(String::new())),
            is_active: Arc::new(Mutex::new(false)),
        }
    }

    /// Connect to OpenAI Realtime API
    pub async fn connect<F>(&mut self, api_key: &str, on_transcript: F) -> Result<(), String>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let on_transcript = Arc::new(on_transcript);
        let model = std::env::var("OPENAI_REALTIME_MODEL")
            .unwrap_or_else(|_| "gpt-realtime-mini".to_string());

        let url = format!("wss://api.openai.com/v1/realtime?model={}", model);

        println!(
            "[Voice] Connecting to OpenAI Realtime API with model: {}",
            model
        );

        // Build WebSocket request with auth header
        let request = http::Request::builder()
            .uri(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("OpenAI-Beta", "realtime=v1")
            .header(
                "Sec-WebSocket-Key",
                tokio_tungstenite::tungstenite::handshake::client::generate_key(),
            )
            .header("Sec-WebSocket-Version", "13")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Host", "api.openai.com")
            .body(())
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let (ws_stream, _) = connect_async(request)
            .await
            .map_err(|e| format!("WebSocket connection failed: {}", e))?;

        println!("[Voice] Connected to OpenAI Realtime API");

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // Build session configuration manually to match OpenAI's expected format
        // (async-openai types have incorrect fields for some parts)
        let session_config = json!({
            "type": "session.update",
            "session": {
                "modalities": ["text", "audio"],
                "input_audio_format": "pcm16",
                "input_audio_transcription": {
                    "model": "whisper-1"
                },
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": 0.5,
                    "prefix_padding_ms": 300,
                    "silence_duration_ms": 500
                }
            }
        });

        let config_json = serde_json::to_string(&session_config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        ws_write
            .send(Message::Text(config_json))
            .await
            .map_err(|e| format!("Failed to send config: {}", e))?;

        println!("[Voice] Session configured");

        // Set up channel for sending audio
        let (tx, mut rx) = mpsc::channel::<String>(100);
        self.ws_sender = Some(tx);

        // Set active flag
        *self.is_active.lock().await = true;
        let is_active = self.is_active.clone();
        let transcript = self.transcript.clone();

        // Spawn task to handle outgoing messages
        let is_active_write = is_active.clone();
        tokio::spawn(async move {
            while let Some(audio_data) = rx.recv().await {
                if !*is_active_write.lock().await {
                    break;
                }

                // Use async-openai type for audio buffer append
                let append_event = InputAudioBufferAppendEvent {
                    event_id: None,
                    audio: audio_data,
                };
                let client_event: ClientEvent = append_event.into();

                if let Ok(json) = serde_json::to_string(&client_event) {
                    if ws_write.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        });

        // Spawn task to handle incoming messages
        let on_transcript_clone = on_transcript.clone();
        tokio::spawn(async move {
            while let Some(msg) = ws_read.next().await {
                if !*is_active.lock().await {
                    break;
                }

                match msg {
                    Ok(Message::Text(text)) => {
                        // Try parsing with async-openai first, fall back to raw JSON
                        match serde_json::from_str::<ServerEvent>(&text) {
                            Ok(event) => {
                                Self::handle_server_event(event, &transcript, &on_transcript_clone)
                                    .await;
                            }
                            Err(_) => {
                                // async-openai types don't match all API responses
                                // Handle events manually via raw JSON
                                if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&text) {
                                    Self::handle_raw_event(&raw, &transcript, &on_transcript_clone)
                                        .await;
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("[Voice] WebSocket closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("[Voice] WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    async fn handle_server_event<F>(
        event: ServerEvent,
        transcript: &Arc<Mutex<String>>,
        on_transcript: &Arc<F>,
    ) where
        F: Fn(String) + Send + Sync,
    {
        match event {
            ServerEvent::ConversationItemInputAudioTranscriptionCompleted(e) => {
                println!("[Voice] Transcript completed: {}", e.transcript);
                let mut t = transcript.lock().await;
                if !t.is_empty() {
                    t.push(' ');
                }
                t.push_str(&e.transcript);
                (on_transcript)(e.transcript);
            }
            ServerEvent::ConversationItemInputAudioTranscriptionFailed(e) => {
                eprintln!("[Voice] Transcription failed: {:?}", e.error);
            }
            ServerEvent::InputAudioBufferSpeechStarted(_) => {
                println!("[Voice] Speech started");
            }
            ServerEvent::InputAudioBufferSpeechStopped(_) => {
                println!("[Voice] Speech stopped");
            }
            ServerEvent::SessionCreated(_) => {
                println!("[Voice] Session created");
            }
            ServerEvent::SessionUpdated(_) => {
                println!("[Voice] Session updated");
            }
            ServerEvent::Error(e) => {
                eprintln!("[Voice] Error: {:?}", e.error);
            }
            _ => {
                // Ignore other events
            }
        }
    }

    /// Handle events that async-openai types can't parse correctly
    async fn handle_raw_event<F>(
        raw: &serde_json::Value,
        transcript: &Arc<Mutex<String>>,
        on_transcript: &Arc<F>,
    ) where
        F: Fn(String) + Send + Sync,
    {
        let event_type = raw.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match event_type {
            "session.created" => {
                println!("[Voice] Session created");
            }
            "session.updated" => {
                println!("[Voice] Session updated");
            }
            "input_audio_buffer.speech_started" => {
                println!("[Voice] Speech started");
            }
            "input_audio_buffer.speech_stopped" => {
                println!("[Voice] Speech stopped");
            }
            "conversation.item.input_audio_transcription.completed" => {
                if let Some(text) = raw.get("transcript").and_then(|t| t.as_str()) {
                    println!("[Voice] Transcript completed: {}", text);
                    let mut t = transcript.lock().await;
                    if !t.is_empty() {
                        t.push(' ');
                    }
                    t.push_str(text);
                    (on_transcript)(text.to_string());
                }
            }
            "conversation.item.input_audio_transcription.failed" => {
                let error = raw
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                eprintln!("[Voice] Transcription failed: {}", error);
            }
            "error" => {
                let error = raw
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                eprintln!("[Voice] Error: {}", error);
            }
            // Silently ignore common events we don't need to handle
            "response.created"
            | "response.done"
            | "response.output_item.added"
            | "response.output_item.done"
            | "response.content_part.added"
            | "response.content_part.done"
            | "response.audio.delta"
            | "response.audio.done"
            | "response.audio_transcript.delta"
            | "response.audio_transcript.done"
            | "rate_limits.updated"
            | "input_audio_buffer.committed"
            | "input_audio_buffer.cleared"
            | "conversation.item.created" => {}
            _ => {
                // Log unknown events for debugging
                println!("[Voice] Unknown event: {}", event_type);
            }
        }
    }

    /// Send audio chunk (base64-encoded PCM16)
    pub async fn send_audio(&self, audio_data: String) -> Result<(), String> {
        if let Some(sender) = &self.ws_sender {
            sender
                .send(audio_data)
                .await
                .map_err(|e| format!("Failed to send audio: {}", e))?;
        }
        Ok(())
    }

    /// Stop the session and return the accumulated transcript
    pub async fn stop(&mut self) -> String {
        *self.is_active.lock().await = false;
        self.ws_sender = None;
        let transcript = self.transcript.lock().await.clone();
        *self.transcript.lock().await = String::new();
        transcript
    }

    /// Check if session is active
    pub async fn is_active(&self) -> bool {
        *self.is_active.lock().await
    }
}

impl Default for VoiceSession {
    fn default() -> Self {
        Self::new()
    }
}
