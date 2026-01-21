// Result queue management for MetaAgent

use std::collections::VecDeque;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::types::{
    AgentResultStatus, ChatResponse, QueueStatus, QueuedAgentResult, ResultQueueUpdatedEvent,
};

/// Manages the queue of agent results waiting to be processed
pub struct ResultQueue {
    queue: VecDeque<QueuedAgentResult>,
}

impl ResultQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// Add a result to the queue when an agent completes
    pub fn push(&mut self, result: QueuedAgentResult) {
        self.queue.push_back(result);
    }

    /// Get the next pending result from the queue
    pub fn pop(&mut self) -> Option<QueuedAgentResult> {
        self.queue.pop_front()
    }

    /// Get the current queue status
    pub fn status(&self) -> QueueStatus {
        QueueStatus {
            pending: self.queue.len(),
            items: self.queue.iter().map(|r| r.summary()).collect(),
        }
    }

    /// Check if there are pending results in the queue
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get number of pending results
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Emit event when queue is updated
    pub fn emit_updated(&self, app_handle: &AppHandle) {
        let _ = app_handle.emit(
            "result-queue:updated",
            ResultQueueUpdatedEvent {
                queue_status: self.status(),
            },
        );
    }
}

impl Default for ResultQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Process the next result in the queue
pub async fn process_next_queued_result(
    queue: &mut ResultQueue,
    process_message: impl std::future::Future<Output = Result<ChatResponse, String>>,
    app_handle: &AppHandle,
) -> Result<Option<ChatResponse>, String> {
    if let Some(mut result) = queue.pop() {
        result.status = AgentResultStatus::Processing;

        // Format the result as a message to process
        let _message = format!(
            "Agent in {} has completed. Here are the results:\n\n{}",
            result.working_dir, result.output
        );

        // Process it through the normal message flow
        let response = process_message.await?;

        // Emit queue updated event
        queue.emit_updated(app_handle);

        Ok(Some(response))
    } else {
        Ok(None)
    }
}
