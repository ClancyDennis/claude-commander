// Memory tools for MetaAgent
//
// Exposes the UpdateMemory tool that invokes the memory manager subagent.

use serde_json::{json, Value};

use crate::meta_agent::memory_manager::MemoryManager;

/// Update the persistent memory via the memory manager subagent
pub async fn update_memory(input: Value) -> Value {
    let instruction = match input["instruction"].as_str() {
        Some(i) => i,
        None => {
            return json!({
                "success": false,
                "error": "instruction is required"
            });
        }
    };

    // Create memory manager
    let manager = match MemoryManager::new() {
        Some(m) => m,
        None => {
            return json!({
                "success": false,
                "error": "Failed to initialize memory manager - could not determine data directory"
            });
        }
    };

    // Process the instruction
    match manager.update_memory(instruction).await {
        Ok(result) => json!({
            "success": result.success,
            "memory_content": result.memory_content,
            "token_count": result.token_count,
            "message": result.message,
            "hint": "Your persistent memory has been updated. This memory persists across sessions."
        }),
        Err(e) => json!({
            "success": false,
            "error": format!("Memory update failed: {}", e)
        }),
    }
}

/// Get the current memory content (for reading without updating)
pub fn get_current_memory() -> Option<String> {
    let manager = MemoryManager::new()?;
    manager.get_memory_for_context()
}
