use super::session_manager::{
    AudioCb, ResponseCb, SimpleCb, ToolCb, TranscriptCb, VoiceCallbacks, VoiceSettings,
};
use async_openai::types::realtime::{ClientEvent, InputAudioBufferAppendEvent};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Discuss mode session - bidirectional voice with tool-use
pub struct DiscussSession {
    /// Sender for outgoing WebSocket messages
    ws_sender: Option<mpsc::Sender<String>>,
    /// Session active flag
    is_active: Arc<Mutex<bool>>,
    /// Conversation messages for context
    messages: Arc<Mutex<Vec<Value>>>,
}

impl DiscussSession {
    pub fn new() -> Self {
        Self {
            ws_sender: None,
            is_active: Arc::new(Mutex::new(false)),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Connect to OpenAI Realtime API in discuss mode
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
            .expect("Discuss mode requires on_tool_call callback");
        let on_user_turn_complete = callbacks
            .on_user_turn_complete
            .expect("Discuss mode requires on_user_turn_complete callback");
        let on_assistant_turn_complete = callbacks
            .on_assistant_turn_complete
            .expect("Discuss mode requires on_assistant_turn_complete callback");

        let model = std::env::var("OPENAI_REALTIME_MODEL")
            .unwrap_or_else(|_| "gpt-4o-realtime-preview".to_string());

        let url = format!("wss://api.openai.com/v1/realtime?model={}", model);

        println!(
            "[Discuss] Connecting to OpenAI Realtime API with model: {}",
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

        println!("[Discuss] Connected to OpenAI Realtime API");

        let (mut ws_write, mut ws_read) = ws_stream.split();

        // Build session configuration for discuss mode with tools
        let session_config = json!({
            "type": "session.update",
            "session": {
                "modalities": ["text", "audio"],
                "instructions": "You are a helpful assistant having a voice conversation with a user. \
                    You're helping them plan and discuss coding tasks. Be conversational and concise. \
                    When you need to perform any action (ask questions, create plans, start agents, get information), \
                    use the talk_to_mission_control tool. Mission Control is the central system that can do everything: \
                    answer complex questions using Claude, create implementation plans, start coding agents, and more. \
                    Just describe what you need in plain language and Mission Control will handle it.",
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
                        "description": "Send a message to Mission Control - the central command system. Mission Control can answer questions using Claude AI, create implementation plans, start coding agents, get project status, and perform any action. Use this whenever you need to take action or get information beyond simple conversation.",
                        "parameters": {
                            "type": "object",
                            "properties": {
                                "message": {
                                    "type": "string",
                                    "description": "The message or request for Mission Control. Be specific about what you need - e.g., 'Ask Claude how to implement authentication', 'Create a plan for adding dark mode', 'Start an agent to fix the login bug'."
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

        println!("[Discuss] Session configured with tools");

        // Set up channel for sending messages
        let (tx, mut rx) = mpsc::channel::<String>(100);
        self.ws_sender = Some(tx.clone());

        // Set active flag
        *self.is_active.lock().await = true;
        let is_active = self.is_active.clone();

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

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            while let Some(msg) = ws_read.next().await {
                if !*is_active.lock().await {
                    break;
                }

                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(raw) = serde_json::from_str::<Value>(&text) {
                            Self::handle_event(
                                &raw,
                                &messages,
                                &ws_sender_for_read,
                                &on_transcript,
                                &on_response,
                                &on_audio,
                                &on_tool_call,
                                &on_user_turn_complete,
                                &on_assistant_turn_complete,
                            )
                            .await;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("[Discuss] WebSocket closed");
                        break;
                    }
                    Err(e) => {
                        eprintln!("[Discuss] WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
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
        on_transcript: &TranscriptCb,
        on_response: &ResponseCb,
        on_audio: &AudioCb,
        on_tool_call: &ToolCb,
        on_user_turn_complete: &SimpleCb,
        on_assistant_turn_complete: &SimpleCb,
    ) {
        let event_type = raw.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match event_type {
            "session.created" => {
                println!("[Discuss] Session created");
            }
            "session.updated" => {
                println!("[Discuss] Session updated");
            }
            "input_audio_buffer.speech_started" => {
                println!("[Discuss] Speech started");
            }
            "input_audio_buffer.speech_stopped" => {
                println!("[Discuss] Speech stopped");
            }
            "conversation.item.input_audio_transcription.completed" => {
                if let Some(text) = raw.get("transcript").and_then(|t| t.as_str()) {
                    println!("[Discuss] User: {}", text);
                    (on_transcript)(text.to_string());

                    // Store user message for context
                    let mut msgs = messages.lock().await;
                    msgs.push(json!({"role": "user", "content": text}));

                    // User turn is complete when transcription finishes
                    (on_user_turn_complete)();
                }
            }
            "response.audio_transcript.delta" => {
                if let Some(delta) = raw.get("delta").and_then(|d| d.as_str()) {
                    if !delta.is_empty() {
                        (on_response)(delta.to_string());
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
                // Assistant response is complete (including audio)
                println!("[Discuss] Response complete");
                (on_assistant_turn_complete)();
            }
            "response.function_call_arguments.done" => {
                // Tool call completed - extract and execute
                if let (Some(call_id), Some(name), Some(args)) = (
                    raw.get("call_id").and_then(|c| c.as_str()),
                    raw.get("name").and_then(|n| n.as_str()),
                    raw.get("arguments").and_then(|a| a.as_str()),
                ) {
                    println!("[Discuss] Tool call: {} with args: {}", name, args);

                    // Execute the tool call via callback
                    let result =
                        (on_tool_call)(name.to_string(), call_id.to_string(), args.to_string());

                    println!("[Discuss] Tool result: {}", result);

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
                            eprintln!("[Discuss] Failed to send tool result: {}", e);
                        } else {
                            println!("[Discuss] Tool result sent to OpenAI");

                            // Trigger response generation after tool result
                            let response_event = json!({
                                "type": "response.create"
                            });
                            if let Ok(response_json) = serde_json::to_string(&response_event) {
                                if let Err(e) = ws_sender.send(response_json).await {
                                    eprintln!("[Discuss] Failed to trigger response: {}", e);
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
                eprintln!("[Discuss] Error: {}", error);
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
                println!("[Discuss] Event: {}", event_type);
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

    /// Send a tool result back to the session
    pub async fn send_tool_result(&self, call_id: &str, result: &str) -> Result<(), String> {
        if let Some(sender) = &self.ws_sender {
            let event = json!({
                "type": "conversation.item.create",
                "item": {
                    "type": "function_call_output",
                    "call_id": call_id,
                    "output": result
                }
            });

            let json_str = serde_json::to_string(&event)
                .map_err(|e| format!("Failed to serialize tool result: {}", e))?;

            sender
                .send(json_str)
                .await
                .map_err(|e| format!("Failed to send tool result: {}", e))?;

            // Trigger response generation after tool result
            let response_event = json!({
                "type": "response.create"
            });
            let response_json = serde_json::to_string(&response_event)
                .map_err(|e| format!("Failed to serialize response create: {}", e))?;

            sender
                .send(response_json)
                .await
                .map_err(|e| format!("Failed to send response create: {}", e))?;
        }
        Ok(())
    }

    /// Stop the session
    pub async fn stop(&mut self) {
        *self.is_active.lock().await = false;
        self.ws_sender = None;
        self.messages.lock().await.clear();
        println!("[Discuss] Session stopped");
    }

    /// Check if session is active
    pub async fn is_active(&self) -> bool {
        *self.is_active.lock().await
    }

    /// Get conversation history for context
    pub async fn get_messages(&self) -> Vec<Value> {
        self.messages.lock().await.clone()
    }
}

impl Default for DiscussSession {
    fn default() -> Self {
        Self::new()
    }
}
