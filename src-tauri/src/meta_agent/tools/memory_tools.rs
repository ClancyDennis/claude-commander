// Memory tools for MetaAgent
//
// Exposes the UpdateMemory tool that queues async memory updates.

use serde_json::{json, Value};
use std::sync::Arc;

use crate::meta_agent::memory_manager::MemoryManager;
use crate::meta_agent::memory_worker::MemoryWorker;

/// Update the persistent memory via async worker (non-blocking)
pub fn update_memory(input: Value, worker: &Arc<MemoryWorker>) -> Value {
    let instruction = match input["instruction"].as_str() {
        Some(i) => i,
        None => {
            return json!({
                "success": false,
                "error": "instruction is required"
            });
        }
    };

    // Queue the update and return immediately
    worker.queue_update(instruction.to_string());

    json!({
        "success": true,
        "message": "Memory update queued. It will be processed in the background.",
        "hint": "You can continue with other tasks. The memory will be updated shortly."
    })
}

/// Get the current memory content (for reading without updating)
pub fn get_current_memory() -> Option<String> {
    let manager = MemoryManager::new()?;
    manager.get_memory_for_context()
}
