// Meta-agent module
//
// This module provides the MetaAgent - the central orchestrator that manages
// worker agents through a conversational interface.

mod action_logger;
mod conversation_manager;
pub mod helpers;
mod result_queue;
mod system_prompt;
mod tool_loop_engine;
pub mod tools;

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
}

impl MetaAgent {
    pub fn new_with_client(ai_client: AIClient) -> Self {
        Self {
            conversation: ConversationManager::new(),
            tool_registry: ToolRegistry::new(),
            ai_client,
            result_queue: ResultQueue::new(),
            tool_loop: ToolLoopEngine::new(),
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

        // Run the tool loop
        let result = self
            .tool_loop
            .run_loop(
                &self.ai_client,
                META_AGENT_SYSTEM_PROMPT,
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

        // Build rich messages with the image for the first API call
        let rich_messages = self.conversation.build_rich_messages_with_image(
            &user_message,
            &image,
            META_AGENT_SYSTEM_PROMPT,
        );

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
                META_AGENT_SYSTEM_PROMPT,
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
}
