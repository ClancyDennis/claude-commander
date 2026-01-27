use super::session_manager::{
    AudioCb, ResponseCb, ToolCb, TranscriptCb, VoiceCallbacks, VoiceSettings,
};
use async_openai::types::realtime::{ClientEvent, InputAudioBufferAppendEvent};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Attention mode session - announces task completion and allows follow-up
/// Auto-closes after inactivity timeout
pub struct AttentionSession {
    /// Sender for outgoing WebSocket messages
    ws_sender: Option<mpsc::Sender<String>>,
    /// Session active flag
    is_active: Arc<Mutex<bool>>,
    /// Last activity timestamp
    last_activity: Arc<Mutex<Instant>>,
    /// Conversation messages for context
    messages: Arc<Mutex<Vec<Value>>>,
}

impl AttentionSession {
    pub fn new() -> Self {
        Self {
            ws_sender: None,
            is_active: Arc::new(Mutex::new(false)),
            last_activity: Arc::new(Mutex::new(Instant::now())),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Update last activity timestamp
    async fn touch_activity(last_activity: &Arc<Mutex<Instant>>) {
        *last_activity.lock().await = Instant::now();
    }

    /// Connect to OpenAI Realtime API in attention mode
    pub async fn connect(
        &mut self,
        api_key: &str,
        settings: VoiceSettings,
        callbacks: VoiceCallbacks,
    ) -> Result<(), String> {
        let on_transcript = callbacks.on_transcript;
        let on_response = callbacks.on_response;
        let on_audio = callbacks.on_audio;
        let on_tool_call = callbacks
            .on_tool_call
            .expect("Attention mode requires on_tool_call callback");
        let on_timeout = callbacks
            .on_timeout
            .expect("Attention mode requires on_timeout callback");

        let model = std::env::var("OPENAI_REALTIME_MODEL")
            .unwrap_or_else(|_| "gpt-4o-realtime-preview".to_string());

        let url = format!("wss://api.openai.com/v1/realtime?model={}", model);

        println!(
            "[Attention] Connecting to OpenAI Realtime API with model: {}",
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

        println!("[Attention] Connected to OpenAI Realtime API");

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // Build session configuration for attention mode
        // Shorter instructions focused on announcing results briefly
        let session_config = json!({
            "type": "session.update",
            "session": {
                "modalities": ["text", "audio"],
                "instructions": "You are announcing task completion results to the user. \
                    Read the provided result briefly (1-2 sentences max), then ask if they need anything else. \
                    Be concise - this is a voice notification. If the user stays silent, the session will close automatically. \
                    When you need to perform any action, use the talk_to_mission_control tool.",
                "voice": settings.voice,
                "input_audio_format": "pcm16",
                "output_audio_format": "pcm16",
                "input_audio_transcription": {
                    "model": "whisper-1"
                },
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": 0.5,
                    "prefix_padding_ms": 300,
                    "silence_duration_ms": 700
                },
                "tools": [
                    {
                        "type": "function",
                        "name": "talk_to_mission_control",
                        "description": "Send a message to Mission Control. Use this when you need to take action or get more information.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "message": {
                                    "type": "string",
                                    "description": "The message or request for Mission Control."
                                }
                            },
                            "required": ["message"]
                        }
                    }
                ],
                "tool_choice": "auto"
            }
        });

        let config_json = serde_json::to_string(&session_config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        ws_write
            .send(Message::Text(config_json))
            .await
            .map_err(|e| format!("Failed to send config: {}", e))?;

        println!("[Attention] Session configured");

        // Set up channel for sending messages
        let (tx, mut rx) = mpsc::channel::<String>(100);
        self.ws_sender = Some(tx.clone());

        // Set active flag and initialize activity timestamp
        *self.is_active.lock().await = true;
        *self.last_activity.lock().await = Instant::now();

        let is_active = self.is_active.clone();
        let last_activity = self.last_activity.clone();

        // Clone for write task
        let is_active_write = is_active.clone();

        // Spawn task to handle outgoing messages
        let mut ws_write = ws_write;
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if !*is_active_write.lock().await {
                    break;
                }

                // Check if this is raw JSON or audio data
                if msg.starts_with('{') {
                    // Raw JSON message (e.g., tool result)
                    if ws_write.send(Message::Text(msg)).await.is_err() {
                        break;
                    }
                } else {
                    // Audio data
                    let append_event = InputAudioBufferAppendEvent {
                        event_id: None,
                        audio: msg,
                    };
                    let client_event: ClientEvent = append_event.into();

                    if let Ok(json) = serde_json::to_string(&client_event) {
                        if ws_write.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Clone for read task
        let messages = self.messages.clone();
        let ws_sender_for_read = tx.clone();
        let last_activity_read = last_activity.clone();
        let is_active_read = is_active.clone();

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            while let Some(msg) = ws_read.next().await {
                if !*is_active_read.lock().await {
                    break;
                }

                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(raw) = serde_json::from_str::<Value>(&text) {
                            Self::handle_event(
                                &raw,
                                &messages,
                                &ws_sender_for_read,
                                &last_activity_read,
                                &on_transcript,
                                &on_response,
                                &on_audio,
                                &on_tool_call,
                            )
                            .await;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("[Attention] WebSocket closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("[Attention] WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Spawn inactivity timeout checker
        let is_active_timeout = is_active.clone();
        let last_activity_timeout = last_activity.clone();
        let timeout_duration = Duration::from_secs(settings.timeout_secs);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;

                if !*is_active_timeout.lock().await {
                    break;
                }

                let last = *last_activity_timeout.lock().await;
                if last.elapsed() >= timeout_duration {
                    println!(
                        "[Attention] Inactivity timeout reached ({}s)",
                        timeout_duration.as_secs()
                    );
                    *is_active_timeout.lock().await = false;
                    (on_timeout)();
                    break;
                }
            }
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn handle_event(
        raw: &Value,
        messages: &Arc<Mutex<Vec<Value>>>,
        ws_sender: &mpsc::Sender<String>,
        last_activity: &Arc<Mutex<Instant>>,
        on_transcript: &TranscriptCb,
        on_response: &ResponseCb,
        on_audio: &AudioCb,
        on_tool_call: &ToolCb,
    ) {
        let event_type = raw.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match event_type {
            "session.created" => {
                println!("[Attention] Session created");
            }
            "session.updated" => {
                println!("[Attention] Session updated");
                // Touch activity when session is ready
                Self::touch_activity(last_activity).await;
            }
            "input_audio_buffer.speech_started" => {
                println!("[Attention] Speech started");
                // User is speaking - touch activity
                Self::touch_activity(last_activity).await;
            }
            "input_audio_buffer.speech_stopped" => {
                println!("[Attention] Speech stopped");
                Self::touch_activity(last_activity).await;
            }
            "conversation.item.input_audio_transcription.completed" => {
                if let Some(text) = raw.get("transcript").and_then(|t| t.as_str()) {
                    println!("[Attention] User: {}", text);
                    (on_transcript)(text.to_string());
                    Self::touch_activity(last_activity).await;

                    // Store user message for context
                    let mut msgs = messages.lock().await;
                    msgs.push(json!({"role": "user", "content": text}));
                }
            }
            "response.audio_transcript.delta" => {
                if let Some(delta) = raw.get("delta").and_then(|d| d.as_str()) {
                    if !delta.is_empty() {
                        (on_response)(delta.to_string());
                        // Touch activity on response
                        Self::touch_activity(last_activity).await;
                    }
                }
            }
            "response.audio.delta" => {
                if let Some(delta) = raw.get("delta").and_then(|d| d.as_str()) {
                    if !delta.is_empty() {
                        (on_audio)(delta.to_string());
                    }
                }
            }
            "response.done" => {
                println!("[Attention] Response complete");
                Self::touch_activity(last_activity).await;
            }
            "response.function_call_arguments.done" => {
                // Tool call completed - extract and execute
                if let (Some(call_id), Some(name), Some(args)) = (
                    raw.get("call_id").and_then(|c| c.as_str()),
                    raw.get("name").and_then(|n| n.as_str()),
                    raw.get("arguments").and_then(|a| a.as_str()),
                ) {
                    println!("[Attention] Tool call: {} with args: {}", name, args);
                    Self::touch_activity(last_activity).await;

                    // Execute the tool call via callback
                    let result =
                        (on_tool_call)(name.to_string(), call_id.to_string(), args.to_string());

                    println!("[Attention] Tool result: {}", result);

                    // Store in messages for context
                    let mut msgs = messages.lock().await;
                    msgs.push(json!({
                        "role": "assistant",
                        "tool_calls": [{
                            "id": call_id,
                            "type": "function",
                            "function": {
                                "name": name,
                                "arguments": args
                            }
                        }]
                    }));
                    msgs.push(json!({
                        "role": "tool",
                        "tool_call_id": call_id,
                        "content": &result
                    }));

                    // Send tool result back to OpenAI Realtime
                    let tool_result_event = json!({
                        "type": "conversation.item.create",
                        "item": {
                            "type": "function_call_output",
                            "call_id": call_id,
                            "output": result
                        }
                    });

                    if let Ok(json_str) = serde_json::to_string(&tool_result_event) {
                        if let Err(e) = ws_sender.send(json_str).await {
                            eprintln!("[Attention] Failed to send tool result: {}", e);
                        } else {
                            println!("[Attention] Tool result sent to OpenAI");

                            // Trigger response generation after tool result
                            // Must specify modalities to get audio output
                            let response_event = json!({
                                "type": "response.create",
                                "response": {
                                    "modalities": ["text", "audio"]
                                }
                            });
                            if let Ok(response_json) = serde_json::to_string(&response_event) {
                                if let Err(e) = ws_sender.send(response_json).await {
                                    eprintln!("[Attention] Failed to trigger response: {}", e);
                                }
                            }
                        }
                    }
                }
            }
            "error" => {
                let error = raw
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown error");
                eprintln!("[Attention] Error: {}", error);
            }
            // Silently ignore common events
            "response.created"
            | "response.output_item.added"
            | "response.output_item.done"
            | "response.content_part.added"
            | "response.content_part.done"
            | "response.audio.done"
            | "response.audio_transcript.done"
            | "response.function_call_arguments.delta"
            | "rate_limits.updated"
            | "input_audio_buffer.committed"
            | "input_audio_buffer.cleared"
            | "conversation.item.created" => {}
            _ => {
                println!("[Attention] Event: {}", event_type);
            }
        }
    }

    /// Send initial prompt to trigger the announcement
    /// This sends a user message with the summary and triggers a response
    pub async fn send_initial_prompt(&self, summary: &str) -> Result<(), String> {
        if let Some(sender) = &self.ws_sender {
            // Create a user message with the summary
            let prompt = format!(
                "Task completed. Here is the result:\n\n{}\n\nPlease read this briefly to the user.",
                summary
            );
            println!("[Attention] Initial prompt:\n{}", prompt);

            // Add the message to the conversation
            let item_event = json!({
                "type": "conversation.item.create",
                "item": {
                    "type": "message",
                    "role": "user",
                    "content": [{
                        "type": "input_text",
                        "text": prompt
                    }]
                }
            });

            let item_json = serde_json::to_string(&item_event)
                .map_err(|e| format!("Failed to serialize item: {}", e))?;

            sender
                .send(item_json)
                .await
                .map_err(|e| format!("Failed to send item: {}", e))?;

            // Trigger response generation with audio output
            // Must specify modalities explicitly when triggering programmatically
            let response_event = json!({
                "type": "response.create",
                "response": {
                    "modalities": ["text", "audio"]
                }
            });

            let response_json = serde_json::to_string(&response_event)
                .map_err(|e| format!("Failed to serialize response: {}", e))?;

            sender
                .send(response_json)
                .await
                .map_err(|e| format!("Failed to send response create: {}", e))?;

            println!("[Attention] Initial prompt sent with audio modality");

            // Touch activity after sending prompt
            Self::touch_activity(&self.last_activity).await;
        }
        Ok(())
    }

    /// Send audio chunk (base64-encoded PCM16)
    pub async fn send_audio(&self, audio_data: String) -> Result<(), String> {
        if let Some(sender) = &self.ws_sender {
            // Touch activity when receiving audio
            Self::touch_activity(&self.last_activity).await;

            sender
                .send(audio_data)
                .await
                .map_err(|e| format!("Failed to send audio: {}", e))?;
        }
        Ok(())
    }

    /// Stop the session
    pub async fn stop(&mut self) {
        *self.is_active.lock().await = false;
        self.ws_sender = None;
        self.messages.lock().await.clear();
        println!("[Attention] Session stopped");
    }

    /// Check if session is active
    pub async fn is_active(&self) -> bool {
        *self.is_active.lock().await
    }
}

impl Default for AttentionSession {
    fn default() -> Self {
        Self::new()
    }
}
