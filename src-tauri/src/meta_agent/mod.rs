// Meta-agent module
//
// This module provides the MetaAgent - the central orchestrator that manages
// worker agents through a conversational interface.

mod action_logger;
mod conversation_manager;
pub mod helpers;
mod prompt_generator;
mod result_queue;
mod system_prompt;
mod tool_loop_engine;
pub mod tools;

pub use prompt_generator::CommanderPersonality;

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::ai_client::{AIClient, Message};
use crate::error::{ApiError, AppError, AppResult};
use crate::tool_registry::ToolRegistry;
use crate::types::{
    AgentResultStatus, ChatMessage, ChatResponse, ImageAttachment, MetaAgentThinkingEvent,
    QueueStatus, QueuedAgentResult,
};

use conversation_manager::ConversationManager;
use result_queue::ResultQueue;
use system_prompt::META_AGENT_SYSTEM_PROMPT;
use tool_loop_engine::ToolLoopEngine;

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
    // Commander personality settings
    personality: Option<CommanderPersonality>,
    cached_prompt: Option<String>,
    cached_prompt_hash: Option<u64>,
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

        Self {
            conversation: ConversationManager::new(),
            tool_registry: ToolRegistry::new(),
            ai_client,
            result_queue: ResultQueue::new(),
            tool_loop: ToolLoopEngine::new(),
            personality,
            cached_prompt,
            cached_prompt_hash,
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

    /// Process a user message (text only)
    pub async fn process_user_message(
        &mut self,
        user_message: String,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: AppHandle,
    ) -> AppResult<ChatResponse> {
        // Emit thinking event
        self.emit_thinking(&app_handle, true)?;

        // Add user message to history
        self.conversation.add_user_message(user_message);

        // Get a mutable reference to the conversation history for the tool loop
        let mut history = self.conversation.clone_history();

        // Run the tool loop with personalized prompt if available
        let system_prompt = self.get_system_prompt();
        let result = self
            .tool_loop
            .run_loop(
                &self.ai_client,
                system_prompt,
                &mut history,
                self.tool_registry.get_all_tools().to_vec(),
                agent_manager,
                &app_handle,
                || self.get_queue_status(),
            )
            .await;

        // Update the conversation history with the tool loop's additions
        self.sync_history_from_loop(history);

        // Emit thinking stopped
        let _ = self.emit_thinking(&app_handle, false);

        result
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

        // Emit thinking event
        self.emit_thinking(&app_handle, true)?;

        // Get personalized prompt (clone to avoid borrow conflicts)
        let system_prompt = self.get_system_prompt().to_string();

        // Build rich messages with the image for the first API call
        let rich_messages =
            self.conversation
                .build_rich_messages_with_image(&user_message, &image, &system_prompt);

        // Add to simple conversation history for future reference
        self.conversation
            .add_user_message_with_image(user_message, &image);

        // Get a mutable reference to the conversation history for the tool loop
        let mut history = self.conversation.clone_history();

        // Run the tool loop with initial rich messages
        let result = self
            .tool_loop
            .run_loop_with_initial_rich_messages(
                &self.ai_client,
                &system_prompt,
                rich_messages,
                &mut history,
                self.tool_registry.get_all_tools().to_vec(),
                agent_manager,
                &app_handle,
                || self.get_queue_status(),
            )
            .await;

        // Update the conversation history with the tool loop's additions
        self.sync_history_from_loop(history);

        // Emit thinking stopped
        let _ = self.emit_thinking(&app_handle, false);

        result
    }

    /// Sync the conversation history after the tool loop has modified it
    fn sync_history_from_loop(&mut self, history: Vec<Message>) {
        // The tool loop may have added assistant and tool result messages
        // We need to add any new messages to our conversation manager
        let current_len = self.conversation.get_history().len();
        for msg in history.into_iter().skip(current_len) {
            if msg.role == "assistant" {
                self.conversation.add_assistant_message(msg.content);
            } else if msg.role == "user" {
                self.conversation.add_user_message(msg.content);
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
    }

    pub fn get_ai_client(&self) -> &AIClient {
        &self.ai_client
    }

    pub fn get_chat_messages(&self) -> Vec<ChatMessage> {
        self.conversation.to_chat_messages()
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
                &self.ai_client,
                META_AGENT_SYSTEM_PROMPT,
                &personality,
            )
            .await?;

            eprintln!(
                "[MetaAgent] Generated personalized prompt ({} chars)",
                generated.len()
            );

            self.cached_prompt = Some(generated.clone());
            self.cached_prompt_hash = Some(new_hash);

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
            .unwrap_or(META_AGENT_SYSTEM_PROMPT)
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
            None => (META_AGENT_SYSTEM_PROMPT.to_string(), false),
        }
    }
}
