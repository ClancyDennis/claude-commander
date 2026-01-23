// Result queue management for MetaAgent

use std::collections::VecDeque;
use tauri::{AppHandle, Emitter};

use crate::types::{QueueStatus, QueuedAgentResult, ResultQueueUpdatedEvent};

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
