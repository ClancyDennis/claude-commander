// Memory Manager - Persistent memory for the Meta Agent
//
// Uses a light model (haiku) to manage a memory directory with multiple files.
// Maintains a concise MEMORY.md summary that gets injected into the meta agent's context.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tiktoken_rs::cl100k_base;

use crate::ai_client::{AIClient, ContentBlock, Message, Tool};

/// Maximum tokens allowed in MEMORY.md
const MEMORY_MD_TOKEN_LIMIT: usize = 2000;

/// Maximum iterations for the memory agent tool loop
const MAX_MEMORY_AGENT_ITERATIONS: usize = 5;

/// Memory directory name under data dir
const MEMORY_DIR_NAME: &str = "meta-memory";

/// Main memory file name
const MEMORY_FILE_NAME: &str = "MEMORY.md";

/// Result of a memory update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUpdateResult {
    pub success: bool,
    pub memory_content: String,
    pub token_count: usize,
    pub message: String,
}

/// Memory Manager handles persistent storage for the meta agent
pub struct MemoryManager {
    memory_dir: PathBuf,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Option<Self> {
        let data_dir = dirs::data_local_dir()?;
        let memory_dir = data_dir.join("claude-commander").join(MEMORY_DIR_NAME);

        Some(Self { memory_dir })
    }

    /// Get the memory directory path
    #[allow(dead_code)]
    pub fn memory_dir(&self) -> &Path {
        &self.memory_dir
    }

    /// Ensure the memory directory exists
    pub fn ensure_directory(&self) -> Result<(), String> {
        fs::create_dir_all(&self.memory_dir)
            .map_err(|e| format!("Failed to create memory directory: {}", e))
    }

    /// Get the path to MEMORY.md
    fn memory_file_path(&self) -> PathBuf {
        self.memory_dir.join(MEMORY_FILE_NAME)
    }

    /// Read the current MEMORY.md content (or empty string if doesn't exist)
    pub fn read_memory(&self) -> String {
        let path = self.memory_file_path();
        fs::read_to_string(&path).unwrap_or_default()
    }

    /// Count tokens in a string using tiktoken
    fn count_tokens(text: &str) -> usize {
        match cl100k_base() {
            Ok(bpe) => bpe.encode_with_special_tokens(text).len(),
            Err(_) => {
                // Fallback: rough estimate of 4 chars per token
                text.len() / 4
            }
        }
    }

    /// Get the tools available to the memory agent
    fn get_memory_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "read_file".to_string(),
                description: "Read a file from the memory directory. Returns the file content and its token count.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path within the memory directory (e.g., 'preferences.md' or 'projects/myproject.md')"
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "write_file".to_string(),
                description: "Write content to a file in the memory directory. Creates parent directories if needed. Returns the token count of the written content.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path within the memory directory (e.g., 'MEMORY.md' or 'projects/myproject.md')"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write to the file"
                        }
                    },
                    "required": ["path", "content"]
                }),
            },
            Tool {
                name: "list_files".to_string(),
                description: "List all files in the memory directory with their sizes and token counts.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        ]
    }

    /// Execute a memory tool
    fn execute_memory_tool(&self, tool_name: &str, input: &Value) -> Value {
        match tool_name {
            "read_file" => self.tool_read_file(input),
            "write_file" => self.tool_write_file(input),
            "list_files" => self.tool_list_files(),
            _ => json!({
                "error": format!("Unknown tool: {}", tool_name)
            }),
        }
    }

    /// Read file tool implementation
    fn tool_read_file(&self, input: &Value) -> Value {
        let path = match input["path"].as_str() {
            Some(p) => p,
            None => return json!({ "error": "path is required" }),
        };

        // Sanitize path to prevent directory traversal
        let safe_path = self.sanitize_path(path);
        if safe_path.is_none() {
            return json!({ "error": "Invalid path - must be within memory directory" });
        }
        let full_path = safe_path.unwrap();

        match fs::read_to_string(&full_path) {
            Ok(content) => {
                let token_count = Self::count_tokens(&content);
                json!({
                    "success": true,
                    "path": path,
                    "content": content,
                    "token_count": token_count,
                    "char_count": content.len()
                })
            }
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to read file: {}", e)
            }),
        }
    }

    /// Write file tool implementation
    fn tool_write_file(&self, input: &Value) -> Value {
        let path = match input["path"].as_str() {
            Some(p) => p,
            None => return json!({ "error": "path is required" }),
        };

        let content = match input["content"].as_str() {
            Some(c) => c,
            None => return json!({ "error": "content is required" }),
        };

        // Sanitize path
        let safe_path = self.sanitize_path(path);
        if safe_path.is_none() {
            return json!({ "error": "Invalid path - must be within memory directory" });
        }
        let full_path = safe_path.unwrap();

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return json!({
                    "success": false,
                    "error": format!("Failed to create directory: {}", e)
                });
            }
        }

        // Check token limit for MEMORY.md
        let token_count = Self::count_tokens(content);
        if path == MEMORY_FILE_NAME && token_count > MEMORY_MD_TOKEN_LIMIT {
            return json!({
                "success": false,
                "error": format!(
                    "MEMORY.md exceeds token limit: {} tokens (limit: {}). Move detailed content to topic files.",
                    token_count, MEMORY_MD_TOKEN_LIMIT
                ),
                "token_count": token_count,
                "token_limit": MEMORY_MD_TOKEN_LIMIT
            });
        }

        match fs::write(&full_path, content) {
            Ok(()) => json!({
                "success": true,
                "path": path,
                "token_count": token_count,
                "char_count": content.len()
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to write file: {}", e)
            }),
        }
    }

    /// List files tool implementation
    fn tool_list_files(&self) -> Value {
        if !self.memory_dir.exists() {
            return json!({
                "success": true,
                "files": [],
                "message": "Memory directory is empty"
            });
        }

        let mut files = Vec::new();
        if let Err(e) = self.collect_files(&self.memory_dir, "", &mut files) {
            return json!({
                "success": false,
                "error": format!("Failed to list files: {}", e)
            });
        }

        json!({
            "success": true,
            "files": files,
            "total_files": files.len()
        })
    }

    /// Recursively collect files from directory
    fn collect_files(
        &self,
        dir: &Path,
        prefix: &str,
        files: &mut Vec<Value>,
    ) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let relative_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            if path.is_dir() {
                self.collect_files(&path, &relative_path, files)?;
            } else if path.is_file() {
                let content = fs::read_to_string(&path).unwrap_or_default();
                let token_count = Self::count_tokens(&content);
                files.push(json!({
                    "path": relative_path,
                    "size_bytes": content.len(),
                    "token_count": token_count
                }));
            }
        }

        Ok(())
    }

    /// Sanitize a path to ensure it stays within the memory directory
    fn sanitize_path(&self, path: &str) -> Option<PathBuf> {
        // Remove leading slashes and normalize
        let clean_path = path.trim_start_matches('/').trim_start_matches("./");

        // Check for path traversal attempts
        if clean_path.contains("..") {
            return None;
        }

        let full_path = self.memory_dir.join(clean_path);

        // Verify the resolved path is still within memory_dir
        match full_path.canonicalize() {
            Ok(canonical) => {
                if canonical.starts_with(&self.memory_dir) {
                    Some(canonical)
                } else {
                    None
                }
            }
            Err(_) => {
                // File doesn't exist yet - check if parent would be valid
                if let Some(parent) = full_path.parent() {
                    // If parent exists, check it's within bounds
                    if parent.exists() {
                        if let Ok(canonical_parent) = parent.canonicalize() {
                            if canonical_parent.starts_with(&self.memory_dir) {
                                return Some(full_path);
                            }
                        }
                    } else {
                        // Parent doesn't exist - just verify no ".." in path
                        if !path.contains("..") {
                            return Some(full_path);
                        }
                    }
                }
                None
            }
        }
    }

    /// Update memory with an instruction
    ///
    /// This runs a mini tool-loop with the light model to process the instruction.
    pub async fn update_memory(&self, instruction: &str) -> Result<MemoryUpdateResult, String> {
        // Ensure directory exists
        self.ensure_directory()?;

        // Create light model client
        let client = AIClient::light_from_env()
            .map_err(|e| format!("Failed to create light client: {}", e))?;

        // Read current MEMORY.md and count tokens
        let current_memory = self.read_memory();
        let current_tokens = Self::count_tokens(&current_memory);

        // Build system prompt with token budget info
        let system_prompt = format!(
            r#"You are a memory management agent. You manage a persistent memory directory for a System Commander AI.

## Tools available:
- read_file(path): Read a file from memory directory
- write_file(path, content): Write/create a file in memory directory
- list_files(): List all files in memory directory

## Token budget:
- MEMORY.md current: {} tokens
- MEMORY.md limit: {} tokens
- Remaining budget: {} tokens

## Rules:
1. MEMORY.md is the main summary/index - it MUST stay under {} tokens
2. Store detailed info in topic-specific files (preferences.md, projects/projectname.md, people.md, etc.)
3. Always update MEMORY.md to reflect what's stored and where
4. If MEMORY.md is over budget, consolidate or move details to topic files
5. Keep entries concise - bullet points preferred
6. When done, always write the updated MEMORY.md

## Current MEMORY.md:
```
{}
```

Now process this instruction and update memory accordingly."#,
            current_tokens,
            MEMORY_MD_TOKEN_LIMIT,
            MEMORY_MD_TOKEN_LIMIT.saturating_sub(current_tokens),
            MEMORY_MD_TOKEN_LIMIT,
            if current_memory.is_empty() {
                "(empty)".to_string()
            } else {
                current_memory.clone()
            }
        );

        // Run the tool loop
        let mut messages = vec![Message {
            role: "user".to_string(),
            content: instruction.to_string(),
        }];

        let tools = Self::get_memory_tools();

        for iteration in 0..MAX_MEMORY_AGENT_ITERATIONS {
            eprintln!(
                "[MemoryManager] Iteration {}/{}",
                iteration + 1,
                MAX_MEMORY_AGENT_ITERATIONS
            );

            // Send message to light model
            let response = client
                .send_message_with_system_and_tools(&system_prompt, messages.clone(), tools.clone())
                .await
                .map_err(|e| format!("Memory agent API call failed: {}", e))?;

            // Process response
            let mut has_tool_use = false;
            let mut assistant_content = String::new();
            let mut tool_results = Vec::new();

            for block in &response.content {
                match block {
                    ContentBlock::Text { text } => {
                        assistant_content.push_str(text);
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        has_tool_use = true;
                        eprintln!("[MemoryManager] Tool call: {} - {:?}", name, input);

                        let result = self.execute_memory_tool(name, input);
                        eprintln!("[MemoryManager] Tool result: {:?}", result);

                        tool_results.push((id.clone(), result));
                    }
                }
            }

            // If no tool use, we're done
            if !has_tool_use {
                break;
            }

            // Add assistant message
            messages.push(Message {
                role: "assistant".to_string(),
                content: format_assistant_response(&response.content),
            });

            // Add tool results
            for (tool_id, result) in tool_results {
                messages.push(Message {
                    role: "user".to_string(),
                    content: format!(
                        r#"{{"type": "tool_result", "tool_use_id": "{}", "content": {}}}"#,
                        tool_id,
                        serde_json::to_string(&result).unwrap_or_default()
                    ),
                });
            }
        }

        // Read final MEMORY.md
        let final_memory = self.read_memory();
        let final_tokens = Self::count_tokens(&final_memory);

        Ok(MemoryUpdateResult {
            success: true,
            memory_content: final_memory,
            token_count: final_tokens,
            message: "Memory updated successfully".to_string(),
        })
    }

    /// Get current memory content for injection into system prompt
    pub fn get_memory_for_context(&self) -> Option<String> {
        let content = self.read_memory();
        if content.is_empty() {
            None
        } else {
            Some(content)
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize memory manager")
    }
}

/// Format assistant response content for message history
fn format_assistant_response(content: &[ContentBlock]) -> String {
    let mut parts = Vec::new();

    for block in content {
        match block {
            ContentBlock::Text { text } => {
                parts.push(text.clone());
            }
            ContentBlock::ToolUse { id, name, input } => {
                parts.push(format!(
                    r#"[Tool call: {} (id: {})] Input: {}"#,
                    name,
                    id,
                    serde_json::to_string(input).unwrap_or_default()
                ));
            }
        }
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_tokens() {
        let text = "Hello, world! This is a test.";
        let count = MemoryManager::count_tokens(text);
        // Should be a reasonable number of tokens
        assert!(count > 0);
        assert!(count < 20);
    }

    #[test]
    fn test_sanitize_path() {
        let manager = MemoryManager {
            memory_dir: PathBuf::from("/tmp/test-memory"),
        };

        // Valid paths
        assert!(manager.sanitize_path("MEMORY.md").is_some());
        assert!(manager.sanitize_path("preferences.md").is_some());
        assert!(manager.sanitize_path("projects/myproject.md").is_some());

        // Invalid paths (traversal attempts)
        assert!(manager.sanitize_path("../etc/passwd").is_none());
        assert!(manager.sanitize_path("foo/../../bar").is_none());
    }
}
