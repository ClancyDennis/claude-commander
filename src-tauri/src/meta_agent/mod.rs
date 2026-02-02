// Meta-agent module
//
// This module provides the MetaAgent - the central orchestrator that manages
// worker agents through a conversational interface.

mod action_logger;
mod context_config;
mod context_summarizer;
mod context_tracker;
mod conversation_manager;
pub mod helpers;
mod memory_manager;
mod memory_worker;
mod output_compressor;
mod prompt_generator;
mod result_queue;
pub mod search_agent;
mod system_prompt;
mod tool_loop_engine;
pub mod tools;

pub use prompt_generator::CommanderPersonality;

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::agent_runs_db::{AgentRunsDB, MetaConversationRecord, MetaMessageRecord};
use crate::ai_client::{AIClient, Message, RichContentBlock, RichMessage, RichMessageContent};
use crate::error::{ApiError, AppError, AppResult};
use crate::tool_registry::ToolRegistry;
use crate::types::{
    AgentResultStatus, ChatMessage, ChatResponse, ContextInfoEvent, ImageAttachment,
    MetaAgentThinkingEvent, QueueStatus, QueuedAgentResult,
};
use crate::utils::string::{truncate_utf8, truncate_with_ellipsis};

use context_config::ContextConfig;
use conversation_manager::ConversationManager;
use memory_worker::MemoryWorker;
use result_queue::ResultQueue;
use system_prompt::build_system_prompt;
use tool_loop_engine::{ToolLoopConfig, ToolLoopEngine};
use tools::{AgentWakeSender, PendingQuestion, SleepState};

/// The MetaAgent orchestrates worker agents through a conversational interface.
///
/// It maintains conversation history, executes tools, and manages a queue of
/// pending agent results.
pub struct MetaAgent {
    conversation: ConversationManager,
    tool_registry: ToolRegistry,
    ai_client: AIClient,
    result_queue: ResultQueue,
    tool_loop: ToolLoopEngine,
    tool_loop_config: ToolLoopConfig,
    // Base system prompt with max_iterations filled in
    base_system_prompt: String,
    // Commander personality settings
    personality: Option<CommanderPersonality>,
    cached_prompt: Option<String>,
    cached_prompt_hash: Option<u64>,
    // Interaction state for Sleep and AskUserQuestion tools
    sleep_state: Arc<Mutex<SleepState>>,
    pending_question: Arc<Mutex<Option<PendingQuestion>>>,
    // Wake sender storage for agents to wake meta-agent from sleep
    agent_wake_tx: Arc<Mutex<Option<AgentWakeSender>>>,
    // Conversation persistence
    conversation_db: Option<Arc<AgentRunsDB>>,
    current_conversation_id: Option<String>,
    // Async memory worker for non-blocking updates
    memory_worker: Arc<MemoryWorker>,
}

impl MetaAgent {
    pub fn new_with_client(ai_client: AIClient) -> Self {
        // Try to load cached prompt from disk
        let (cached_prompt, cached_prompt_hash, personality) =
            if let Some(cache) = prompt_generator::load_cached_prompt() {
                eprintln!(
                    "[MetaAgent] Restored cached prompt from disk (hash: {})",
                    cache.settings_hash
                );
                (
                    Some(cache.prompt),
                    Some(cache.settings_hash),
                    Some(cache.personality),
                )
            } else {
                (None, None, None)
            };

        let tool_loop_config = ToolLoopConfig::default();
        // Build base system prompt with max_iterations from config
        let base_system_prompt = build_system_prompt(tool_loop_config.max_iterations);

        // Initialize context config based on AI provider
        let context_config = ContextConfig::for_provider(ai_client.get_provider_name());
        let mut conversation = ConversationManager::with_config(context_config);

        // Set system prompt for token tracking
        let system_prompt = cached_prompt.as_deref().unwrap_or(&base_system_prompt);
        conversation.set_system_prompt(system_prompt);

        // Start the async memory worker
        let memory_worker = Arc::new(MemoryWorker::start());

        Self {
            conversation,
            tool_registry: ToolRegistry::new(),
            ai_client,
            result_queue: ResultQueue::new(),
            tool_loop: ToolLoopEngine::new(),
            tool_loop_config,
            base_system_prompt,
            personality,
            cached_prompt,
            cached_prompt_hash,
            sleep_state: Arc::new(Mutex::new(SleepState::default())),
            pending_question: Arc::new(Mutex::new(None)),
            agent_wake_tx: Arc::new(Mutex::new(None)),
            conversation_db: None,
            current_conversation_id: None,
            memory_worker,
        }
    }

    pub fn new() -> AppResult<Self> {
        let ai_client = AIClient::from_env().map_err(|e| {
            AppError::Api(ApiError::AuthenticationFailed(format!(
                "Failed to initialize AI client: {}",
                e
            )))
        })?;
        Ok(Self::new_with_client(ai_client))
    }

    // =========================================================================
    // Result Queue Management
    // =========================================================================

    /// Add a result to the queue when an agent completes
    pub fn queue_agent_result(&mut self, result: QueuedAgentResult) {
        self.result_queue.push(result);
    }

    /// Get the next pending result from the queue
    pub fn pop_next_result(&mut self) -> Option<QueuedAgentResult> {
        self.result_queue.pop()
    }

    /// Get the current queue status
    pub fn get_queue_status(&self) -> QueueStatus {
        self.result_queue.status()
    }

    /// Check if there are pending results in the queue
    pub fn has_pending_results(&self) -> bool {
        !self.result_queue.is_empty()
    }

    /// Process the next result in the queue
    pub async fn process_next_queued_result(
        &mut self,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> AppResult<Option<ChatResponse>> {
        if let Some(mut result) = self.result_queue.pop() {
            result.status = AgentResultStatus::Processing;

            // Format the result as a message to process
            let message = format!(
                "Agent in {} has completed. Here are the results:\n\n{}",
                result.working_dir, result.output
            );

            // Process it through the normal message flow
            let response = self
                .process_user_message(message, agent_manager, app_handle.clone())
                .await?;

            // Emit queue updated event
            self.result_queue.emit_updated(&app_handle);

            Ok(Some(response))
        } else {
            Ok(None)
        }
    }

    // =========================================================================
    // Message Processing
    // =========================================================================

    /// Emit the thinking event to notify the frontend
    fn emit_thinking(&self, app_handle: &AppHandle, is_thinking: bool) -> AppResult<()> {
        app_handle
            .emit(
                "meta-agent:thinking",
                MetaAgentThinkingEvent { is_thinking },
            )
            .map_err(|e| AppError::Internal(format!("Failed to emit thinking event: {}", e)))
    }

    /// Emit context usage information to the frontend
    fn emit_context_info(&self, app_handle: &AppHandle) {
        let info = self.conversation.get_context_info();
        let event = ContextInfoEvent {
            usage_percent: info.usage_percent,
            current_tokens: info.current_tokens,
            available_tokens: info.available_tokens,
            remaining_tokens: info.remaining_tokens,
            state: info.state.description().to_string(),
            warning_message: info.warning_message(),
        };
        let _ = app_handle.emit("meta-agent:context-info", event);
    }

    /// Process a user message (text only)
    pub async fn process_user_message(
        &mut self,
        user_message: String,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> AppResult<ChatResponse> {
        // Check if currently sleeping - interrupt it with user message
        {
            let mut sleep_state = self.sleep_state.lock().await;
            if sleep_state.is_sleeping {
                if let Some(cancel_tx) = sleep_state.cancel_tx.take() {
                    let _ = cancel_tx.send(user_message.clone());
                    // Sleep tool will return with the user message
                    // and the tool loop will continue processing
                }
            }
        }

        // Ensure we have a conversation for persistence
        if let Err(e) = self.ensure_conversation().await {
            eprintln!("[MetaAgent] Warning: Failed to ensure conversation: {}", e);
        }

        // Emit thinking event
        self.emit_thinking(&app_handle, true)?;

        // Add user message to history
        self.conversation.add_user_message(user_message.clone());

        // Persist user message
        self.persist_message("user", &user_message, None).await;

        // Check for context compaction at idle moment (after user input processed)
        if self.conversation.compact_if_needed().await {
            eprintln!("[MetaAgent] Context compacted before processing");
        }

        // Get conversation history as RichMessages for proper tool_use/tool_result handling
        let mut history = self.conversation.get_history_as_rich_messages();

        // Run the tool loop with personalized prompt if available (includes memory)
        let system_prompt = self.get_system_prompt_with_memory();

        // Create a closure to get context info for tools
        // We need to capture a reference, but closures can't capture &mut self
        // So we'll pass None for now and use the direct result after the loop
        let loop_result = self
            .tool_loop
            .run_loop(
                &self.ai_client,
                &system_prompt,
                &mut history,
                self.tool_registry.get_all_tools().to_vec(),
                agent_manager,
                &app_handle,
                self.sleep_state.clone(),
                self.pending_question.clone(),
                self.agent_wake_tx.clone(),
                self.memory_worker.clone(),
                || self.get_queue_status(),
                || None, // Context info will be added after we can get it
            )
            .await;

        // Update the conversation history with the tool loop's additions
        let history_before = self.conversation.get_history().len();

        // Sync history, accounting for summary messages that were prepended
        let summary_offset = if self.conversation.has_context_summary() {
            2
        } else {
            0
        };
        let new_history: Vec<RichMessage> = history.into_iter().skip(summary_offset).collect();
        self.sync_history_from_loop(new_history);

        // Record token usage from the loop
        if let Ok(ref result) = loop_result {
            self.conversation.record_usage(
                result.total_usage.input_tokens,
                result.total_usage.output_tokens,
            );
            // Emit context info to frontend
            self.emit_context_info(&app_handle);
        }

        // Persist any new assistant messages
        let history_after = self.conversation.get_history();
        for msg in history_after.iter().skip(history_before) {
            self.persist_message(&msg.role, &msg.content, None).await;
        }

        // Check for context compaction at idle moment (after tool loop completes)
        if self.conversation.compact_if_needed().await {
            eprintln!("[MetaAgent] Context compacted after tool loop");
            // Emit updated context info after compaction
            self.emit_context_info(&app_handle);
        }

        // Emit thinking stopped
        let _ = self.emit_thinking(&app_handle, false);

        loop_result.map(|r| r.response)
    }

    /// Process a user message with an optional image attachment
    pub async fn process_user_message_with_image(
        &mut self,
        user_message: String,
        image: Option<ImageAttachment>,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> AppResult<ChatResponse> {
        // If no image, use the regular method
        if image.is_none() {
            return self
                .process_user_message(user_message, agent_manager, app_handle)
                .await;
        }

        let image = image.unwrap();

        // Check if currently sleeping - interrupt it with user message
        {
            let mut sleep_state = self.sleep_state.lock().await;
            if sleep_state.is_sleeping {
                if let Some(cancel_tx) = sleep_state.cancel_tx.take() {
                    let _ = cancel_tx.send(user_message.clone());
                }
            }
        }

        // Ensure we have a conversation for persistence
        if let Err(e) = self.ensure_conversation().await {
            eprintln!("[MetaAgent] Warning: Failed to ensure conversation: {}", e);
        }

        // Emit thinking event
        self.emit_thinking(&app_handle, true)?;

        // Get personalized prompt with memory (clone to avoid borrow conflicts)
        let system_prompt = self.get_system_prompt_with_memory();

        // Build rich messages with the image for the first API call
        let rich_messages =
            self.conversation
                .build_rich_messages_with_image(&user_message, &image, &system_prompt);

        // Add to simple conversation history for future reference
        self.conversation
            .add_user_message_with_image(user_message.clone(), &image);

        // Persist user message with image indicator (we don't persist actual image data)
        let content_with_image = if user_message.is_empty() {
            "[Image attached]".to_string()
        } else {
            format!("[Image attached] {}", user_message)
        };
        self.persist_message("user", &content_with_image, None)
            .await;

        // Check for context compaction at idle moment (after user input processed)
        if self.conversation.compact_if_needed().await {
            eprintln!("[MetaAgent] Context compacted before processing");
        }

        // Get conversation history as RichMessages for proper tool_use/tool_result handling
        let mut history = self.conversation.get_history_as_rich_messages();

        // Run the tool loop with initial rich messages
        let loop_result = self
            .tool_loop
            .run_loop_with_initial_rich_messages(
                &self.ai_client,
                &system_prompt,
                rich_messages,
                &mut history,
                self.tool_registry.get_all_tools().to_vec(),
                agent_manager,
                &app_handle,
                self.sleep_state.clone(),
                self.pending_question.clone(),
                self.agent_wake_tx.clone(),
                self.memory_worker.clone(),
                || self.get_queue_status(),
                || None, // Context info will be added after we can get it
            )
            .await;

        // Update the conversation history with the tool loop's additions
        let history_before = self.conversation.get_history().len();

        // Sync history, accounting for summary messages that were prepended
        let summary_offset = if self.conversation.has_context_summary() {
            2
        } else {
            0
        };
        let new_history: Vec<RichMessage> = history.into_iter().skip(summary_offset).collect();
        self.sync_history_from_loop(new_history);

        // Record token usage from the loop
        if let Ok(ref result) = loop_result {
            self.conversation.record_usage(
                result.total_usage.input_tokens,
                result.total_usage.output_tokens,
            );
            // Emit context info to frontend
            self.emit_context_info(&app_handle);
        }

        // Persist any new assistant messages
        let history_after = self.conversation.get_history();
        for msg in history_after.iter().skip(history_before) {
            self.persist_message(&msg.role, &msg.content, None).await;
        }

        // Check for context compaction at idle moment (after tool loop completes)
        if self.conversation.compact_if_needed().await {
            eprintln!("[MetaAgent] Context compacted after tool loop");
            // Emit updated context info after compaction
            self.emit_context_info(&app_handle);
        }

        // Emit thinking stopped
        let _ = self.emit_thinking(&app_handle, false);

        loop_result.map(|r| r.response)
    }

    /// Sync the conversation history after the tool loop has modified it
    fn sync_history_from_loop(&mut self, history: Vec<RichMessage>) {
        // The tool loop may have added assistant and tool result messages
        // We need to add any new messages to our conversation manager
        let current_len = self.conversation.get_history().len();
        for msg in history.into_iter().skip(current_len) {
            // Extract text content from RichMessage
            let content = Self::extract_text_from_rich_message(&msg);
            if msg.role == "assistant" {
                self.conversation.add_assistant_message(content);
            } else if msg.role == "user" {
                self.conversation.add_user_message(content);
            }
        }
    }

    /// Extract text content from a RichMessage (for storage in simple Message format)
    fn extract_text_from_rich_message(msg: &RichMessage) -> String {
        match &msg.content {
            RichMessageContent::Text(s) => s.clone(),
            RichMessageContent::Blocks(blocks) => {
                // Extract text from blocks, serializing tool_use/tool_result as JSON
                let parts: Vec<String> = blocks
                    .iter()
                    .map(|block| match block {
                        RichContentBlock::Text { text } => text.clone(),
                        RichContentBlock::ToolUse { id, name, input } => {
                            // Serialize tool_use for reference in history
                            serde_json::json!({
                                "type": "tool_use",
                                "id": id,
                                "name": name,
                                "input": input
                            })
                            .to_string()
                        }
                        RichContentBlock::ToolResult {
                            tool_use_id,
                            content,
                            is_error,
                        } => {
                            // Serialize tool_result for reference in history
                            let mut result = serde_json::json!({
                                "type": "tool_result",
                                "tool_use_id": tool_use_id,
                                "content": content
                            });
                            if let Some(err) = is_error {
                                result["is_error"] = serde_json::json!(err);
                            }
                            result.to_string()
                        }
                        RichContentBlock::Image { .. } => "[Image]".to_string(),
                    })
                    .collect();
                parts.join("\n")
            }
        }
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    pub fn get_conversation_history(&self) -> &[Message] {
        self.conversation.get_history()
    }

    pub fn clear_conversation_history(&mut self) {
        self.conversation.clear();
        self.current_conversation_id = None;
    }

    pub fn get_ai_client(&self) -> &AIClient {
        &self.ai_client
    }

    pub fn get_chat_messages(&self) -> Vec<ChatMessage> {
        self.conversation.to_chat_messages()
    }

    /// Get the pending question state for answering user questions
    pub fn get_pending_question(&self) -> Arc<Mutex<Option<PendingQuestion>>> {
        self.pending_question.clone()
    }

    /// Set the pending question state (for sharing with AppState)
    pub fn set_pending_question(&mut self, pending: Arc<Mutex<Option<PendingQuestion>>>) {
        self.pending_question = pending;
    }

    /// Set the sleep state (for sharing with AppState to allow interrupt before locking)
    pub fn set_sleep_state(&mut self, state: Arc<Mutex<SleepState>>) {
        self.sleep_state = state;
    }

    /// Set the agent wake sender storage (for sharing with AppState to allow result handlers to wake meta-agent)
    pub fn set_agent_wake_tx(&mut self, tx: Arc<Mutex<Option<AgentWakeSender>>>) {
        self.agent_wake_tx = tx;
    }

    /// Get the agent wake sender storage (for tool loop to pass to sleep tool)
    pub fn get_agent_wake_tx(&self) -> Arc<Mutex<Option<AgentWakeSender>>> {
        self.agent_wake_tx.clone()
    }

    /// Get the memory worker for async memory updates
    pub fn get_memory_worker(&self) -> Arc<MemoryWorker> {
        self.memory_worker.clone()
    }

    /// Set the conversation database for persistence
    pub fn set_conversation_db(&mut self, db: Arc<AgentRunsDB>) {
        self.conversation_db = Some(db);
    }

    /// Get the current conversation ID
    pub fn get_current_conversation_id(&self) -> Option<&str> {
        self.current_conversation_id.as_deref()
    }

    // =========================================================================
    // Conversation Persistence
    // =========================================================================

    /// Start a new conversation, creating a DB record
    pub async fn start_new_conversation(&mut self) -> Result<String, String> {
        // Generate new UUID
        let conversation_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();

        // Clear in-memory history
        self.conversation.clear();

        // Create DB record if we have a database
        if let Some(db) = &self.conversation_db {
            let record = MetaConversationRecord {
                id: None,
                conversation_id: conversation_id.clone(),
                title: None,
                created_at: now,
                updated_at: now,
                message_count: 0,
                is_archived: false,
                preview_text: None,
            };

            db.create_meta_conversation(&record)
                .await
                .map_err(|e| format!("Failed to create conversation: {}", e))?;
        }

        self.current_conversation_id = Some(conversation_id.clone());
        eprintln!("[MetaAgent] Started new conversation: {}", conversation_id);

        Ok(conversation_id)
    }

    /// Load an existing conversation from the database
    pub async fn load_conversation(
        &mut self,
        conversation_id: &str,
    ) -> Result<Vec<ChatMessage>, String> {
        let db = self
            .conversation_db
            .as_ref()
            .ok_or_else(|| "No database configured".to_string())?;

        // Check conversation exists
        let conv = db
            .get_meta_conversation(conversation_id)
            .await
            .map_err(|e| format!("Failed to get conversation: {}", e))?
            .ok_or_else(|| format!("Conversation {} not found", conversation_id))?;

        // Get messages
        let messages = db
            .get_meta_messages(conversation_id)
            .await
            .map_err(|e| format!("Failed to get messages: {}", e))?;

        // Load into ConversationManager
        self.conversation.load_from_records(&messages);
        self.current_conversation_id = Some(conversation_id.to_string());

        eprintln!(
            "[MetaAgent] Loaded conversation {} with {} messages",
            conversation_id, conv.message_count
        );

        // Return as ChatMessages for frontend
        Ok(self.conversation.to_chat_messages())
    }

    /// Persist a message to the database
    async fn persist_message(&self, role: &str, content: &str, image_data: Option<String>) {
        let Some(db) = &self.conversation_db else {
            return;
        };
        let Some(conv_id) = &self.current_conversation_id else {
            return;
        };

        let message_index = self.conversation.get_history().len() as u32;
        let now = chrono::Utc::now().timestamp_millis();

        let record = MetaMessageRecord {
            id: None,
            conversation_id: conv_id.clone(),
            message_index,
            role: role.to_string(),
            content: content.to_string(),
            image_data,
            tool_calls: None,
            timestamp: now,
        };

        if let Err(e) = db.insert_meta_message(&record).await {
            eprintln!("[MetaAgent] Failed to persist message: {}", e);
            return;
        }

        // Update conversation metadata (safely truncate at UTF-8 boundaries)
        let preview = Some(truncate_utf8(content, 100));

        // Auto-generate title from first user message
        let title = if message_index == 0 && role == "user" {
            Some(truncate_with_ellipsis(content, 50))
        } else {
            None
        };

        if let Err(e) = db
            .update_meta_conversation_after_message(conv_id, preview, title.as_deref())
            .await
        {
            eprintln!("[MetaAgent] Failed to update conversation metadata: {}", e);
        }
    }

    /// Ensure we have a conversation (create one if needed)
    async fn ensure_conversation(&mut self) -> Result<(), String> {
        if self.current_conversation_id.is_none() {
            self.start_new_conversation().await?;
        }
        Ok(())
    }

    // =========================================================================
    // Commander Personality
    // =========================================================================

    /// Set the commander personality and regenerate the cached prompt if needed
    pub async fn set_personality(
        &mut self,
        personality: CommanderPersonality,
    ) -> Result<(), String> {
        let new_hash = personality.settings_hash();

        // Only regenerate if settings actually changed
        if self.cached_prompt_hash != Some(new_hash) {
            eprintln!(
                "[MetaAgent] Personality changed, regenerating prompt (hash: {} -> {})",
                self.cached_prompt_hash.unwrap_or(0),
                new_hash
            );

            let generated = prompt_generator::generate_personalized_prompt(
                &self.base_system_prompt,
                &personality,
            )
            .await?;

            eprintln!(
                "[MetaAgent] Generated personalized prompt ({} chars)",
                generated.len()
            );

            self.cached_prompt = Some(generated.clone());
            self.cached_prompt_hash = Some(new_hash);

            // Update system prompt in conversation manager for token tracking
            self.conversation.set_system_prompt(&generated);

            // Persist to disk
            let cache = prompt_generator::PromptCache {
                prompt: generated,
                settings_hash: new_hash,
                personality: personality.clone(),
            };
            if let Err(e) = prompt_generator::save_cached_prompt(&cache) {
                eprintln!("[MetaAgent] Warning: Failed to persist prompt cache: {}", e);
            }
        }

        self.personality = Some(personality);
        Ok(())
    }

    /// Get the current system prompt (personalized if available, base otherwise)
    fn get_system_prompt(&self) -> &str {
        self.cached_prompt
            .as_deref()
            .unwrap_or(&self.base_system_prompt)
    }

    /// Get the system prompt with memory content appended
    ///
    /// This builds the full system prompt including any persistent memory content.
    fn get_system_prompt_with_memory(&self) -> String {
        let base_prompt = self.get_system_prompt();

        // Try to get memory content
        let memory_section = if let Some(manager) = memory_manager::MemoryManager::new() {
            if let Some(memory_content) = manager.get_memory_for_context() {
                format!(
                    r#"

## Your Persistent Memory
You have access to persistent memory that survives across sessions.

**Tools:**
- **UpdateMemory**: Save notes, user preferences, project context
- **Search**: Query your memories with natural language (also searches run history)

**Proactive Memory Updates:**
Update memory when you learn new facts about:
- User preferences (coding style, tools, workflows)
- Project context (tech stack, key files, architecture)
- Important decisions or outcomes

Current memory:
---
{}
---"#,
                    memory_content
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        format!("{}{}", base_prompt, memory_section)
    }

    /// Get the current personality settings
    pub fn get_personality(&self) -> Option<&CommanderPersonality> {
        self.personality.as_ref()
    }

    /// Clear the personality and cached prompt, reverting to base prompt
    pub fn clear_personality(&mut self) -> Result<(), String> {
        // Clear in-memory state
        self.personality = None;
        self.cached_prompt = None;
        self.cached_prompt_hash = None;

        // Clear from disk
        prompt_generator::clear_cached_prompt()?;

        eprintln!("[MetaAgent] Cleared personality and reverted to base prompt");
        Ok(())
    }

    /// Get the current system prompt and whether it is personalized
    pub fn get_system_prompt_snapshot(&self) -> (String, bool) {
        match &self.cached_prompt {
            Some(prompt) => (prompt.clone(), true),
            None => (self.base_system_prompt.clone(), false),
        }
    }

    /// Get the max iterations from the tool loop config
    pub fn get_max_iterations(&self) -> usize {
        self.tool_loop_config.max_iterations
    }
}
