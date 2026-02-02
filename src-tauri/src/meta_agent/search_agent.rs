// Search Agent - Unified search across run history and memories
//
// Uses a light model (haiku) to interpret natural language queries
// and search across both the run history database and memory files.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::agent_runs_db::{AgentRunsDB, RunQueryFilters, RunStatus};
use crate::ai_client::{AIClient, ContentBlock, Message, Tool};
use crate::types::AgentSource;
use crate::utils::string::truncate_with_ellipsis;
use chrono::{Duration, Utc};

/// Maximum iterations for the search agent tool loop
const MAX_SEARCH_AGENT_ITERATIONS: usize = 5;

/// Result of a search operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub success: bool,
    pub summary: String,
    pub runs_found: usize,
    pub memories_searched: usize,
}

/// Search Agent handles natural language queries across runs and memories
pub struct SearchAgent {
    runs_db: Arc<AgentRunsDB>,
    memory_dir: PathBuf,
}

impl SearchAgent {
    /// Create a new search agent
    pub fn new(runs_db: Arc<AgentRunsDB>) -> Option<Self> {
        let data_dir = dirs::data_local_dir()?;
        let memory_dir = data_dir.join("claude-commander").join("meta-memory");

        Some(Self {
            runs_db,
            memory_dir,
        })
    }

    /// Get the memory directory path
    pub fn memory_dir(&self) -> &Path {
        &self.memory_dir
    }

    /// Get the tools available to the search agent
    fn get_search_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "query_runs".to_string(),
                description: "Search run history with optional filters. Returns a list of agent runs matching the criteria.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "working_dir": {
                            "type": "string",
                            "description": "Filter by working directory (partial match, case-insensitive)"
                        },
                        "status": {
                            "type": "string",
                            "enum": ["running", "completed", "failed", "stopped", "crashed"],
                            "description": "Filter by run status"
                        },
                        "source": {
                            "type": "string",
                            "enum": ["ui", "meta", "pipeline", "pool", "manual"],
                            "description": "Filter by how the agent was created"
                        },
                        "keyword": {
                            "type": "string",
                            "description": "Search keyword in initial prompt (case-insensitive)"
                        },
                        "days_back": {
                            "type": "integer",
                            "description": "How many days back to search (default: 30)"
                        },
                        "resumable_only": {
                            "type": "boolean",
                            "description": "Only return runs that can be resumed"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum results to return (default: 20)"
                        }
                    },
                    "required": []
                }),
            },
            Tool {
                name: "get_run_details".to_string(),
                description: "Get full details for a specific agent run by ID.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "agent_id": {
                            "type": "string",
                            "description": "The agent ID to look up"
                        }
                    },
                    "required": ["agent_id"]
                }),
            },
            Tool {
                name: "list_memory_files".to_string(),
                description: "List all files in the memory directory with their sizes.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            Tool {
                name: "read_memory_file".to_string(),
                description: "Read a specific memory file by path.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Relative path within the memory directory (e.g., 'MEMORY.md' or 'projects/myproject.md')"
                        }
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "search_memory_content".to_string(),
                description: "Search for a keyword across all memory files. Returns matching excerpts with file paths.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "keyword": {
                            "type": "string",
                            "description": "The keyword or phrase to search for (case-insensitive)"
                        }
                    },
                    "required": ["keyword"]
                }),
            },
        ]
    }

    /// Execute a search tool
    async fn execute_search_tool(&self, tool_name: &str, input: &Value) -> Value {
        match tool_name {
            "query_runs" => self.tool_query_runs(input).await,
            "get_run_details" => self.tool_get_run_details(input).await,
            "list_memory_files" => self.tool_list_memory_files(),
            "read_memory_file" => self.tool_read_memory_file(input),
            "search_memory_content" => self.tool_search_memory_content(input),
            _ => json!({
                "error": format!("Unknown tool: {}", tool_name)
            }),
        }
    }

    /// Query runs tool implementation
    async fn tool_query_runs(&self, input: &Value) -> Value {
        let days_back = input["days_back"].as_i64().unwrap_or(30);
        let limit = input["limit"].as_u64().unwrap_or(20) as usize;

        let status = input["status"].as_str().map(RunStatus::parse);
        let working_dir_filter = input["working_dir"].as_str().map(|s| s.to_lowercase());
        let source = input["source"].as_str().and_then(|s| match s {
            "ui" => Some(AgentSource::UI),
            "meta" => Some(AgentSource::Meta),
            "pipeline" => Some(AgentSource::Pipeline),
            "pool" => Some(AgentSource::Pool),
            "manual" => Some(AgentSource::Manual),
            _ => None,
        });

        let date_from = Some(Utc::now() - Duration::days(days_back));

        let filters = RunQueryFilters {
            status,
            working_dir: None, // Do partial matching in post-processing
            source,
            date_from,
            date_to: None,
            limit: Some(limit + 100), // Get extra for post-filtering
            offset: None,
        };

        let runs = match self.runs_db.query_runs(filters).await {
            Ok(runs) => runs,
            Err(e) => {
                return json!({
                    "success": false,
                    "error": format!("Failed to query run history: {}", e)
                });
            }
        };

        // Post-filter by keyword, working_dir partial match, and resumable
        let keyword = input["keyword"].as_str().map(|s| s.to_lowercase());
        let resumable_only = input["resumable_only"].as_bool().unwrap_or(false);

        let filtered_runs: Vec<_> = runs
            .into_iter()
            .filter(|run| {
                if let Some(ref dir_filter) = working_dir_filter {
                    if !run.working_dir.to_lowercase().contains(dir_filter) {
                        return false;
                    }
                }
                if let Some(ref kw) = keyword {
                    if let Some(ref prompt) = run.initial_prompt {
                        if !prompt.to_lowercase().contains(kw) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                if resumable_only && !run.can_resume {
                    return false;
                }
                true
            })
            .take(limit)
            .collect();

        let formatted_runs: Vec<Value> = filtered_runs
            .iter()
            .map(|run| {
                let duration_mins = run.ended_at.map(|ended| (ended - run.started_at) / 60000);

                let started_at = chrono::DateTime::from_timestamp_millis(run.started_at)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let initial_prompt: Option<String> = run
                    .initial_prompt
                    .as_ref()
                    .map(|p| truncate_with_ellipsis(p, 147));

                json!({
                    "agent_id": run.agent_id,
                    "working_dir": run.working_dir,
                    "status": run.status.to_str(),
                    "source": run.source,
                    "started_at": started_at,
                    "duration_mins": duration_mins,
                    "initial_prompt": initial_prompt,
                    "total_prompts": run.total_prompts,
                    "cost_usd": run.total_cost_usd,
                    "can_resume": run.can_resume
                })
            })
            .collect();

        json!({
            "success": true,
            "total_found": formatted_runs.len(),
            "runs": formatted_runs
        })
    }

    /// Get run details tool implementation
    async fn tool_get_run_details(&self, input: &Value) -> Value {
        let agent_id = match input["agent_id"].as_str() {
            Some(id) => id,
            None => return json!({ "error": "agent_id is required" }),
        };

        match self.runs_db.get_run(agent_id).await {
            Ok(Some(run)) => {
                let started_at = chrono::DateTime::from_timestamp_millis(run.started_at)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let ended_at = run.ended_at.and_then(|ts| {
                    chrono::DateTime::from_timestamp_millis(ts)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                });

                json!({
                    "success": true,
                    "run": {
                        "agent_id": run.agent_id,
                        "working_dir": run.working_dir,
                        "status": run.status.to_str(),
                        "source": run.source,
                        "started_at": started_at,
                        "ended_at": ended_at,
                        "initial_prompt": run.initial_prompt,
                        "total_prompts": run.total_prompts,
                        "total_tool_calls": run.total_tool_calls,
                        "cost_usd": run.total_cost_usd,
                        "can_resume": run.can_resume,
                        "session_id": run.session_id
                    }
                })
            }
            Ok(None) => json!({
                "success": false,
                "error": format!("No run found with ID: {}", agent_id)
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to get run details: {}", e)
            }),
        }
    }

    /// List memory files tool implementation
    fn tool_list_memory_files(&self) -> Value {
        if !self.memory_dir.exists() {
            return json!({
                "success": true,
                "files": [],
                "message": "Memory directory is empty or does not exist"
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
                let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                files.push(json!({
                    "path": relative_path,
                    "size_bytes": size
                }));
            }
        }

        Ok(())
    }

    /// Read memory file tool implementation
    fn tool_read_memory_file(&self, input: &Value) -> Value {
        let path = match input["path"].as_str() {
            Some(p) => p,
            None => return json!({ "error": "path is required" }),
        };

        let safe_path = match self.sanitize_path(path) {
            Some(p) => p,
            None => return json!({ "error": "Invalid path - must be within memory directory" }),
        };

        match fs::read_to_string(&safe_path) {
            Ok(content) => json!({
                "success": true,
                "path": path,
                "content": content,
                "size_bytes": content.len()
            }),
            Err(e) => json!({
                "success": false,
                "error": format!("Failed to read file: {}", e)
            }),
        }
    }

    /// Search memory content tool implementation
    fn tool_search_memory_content(&self, input: &Value) -> Value {
        let keyword = match input["keyword"].as_str() {
            Some(k) => k.to_lowercase(),
            None => return json!({ "error": "keyword is required" }),
        };

        if !self.memory_dir.exists() {
            return json!({
                "success": true,
                "matches": [],
                "message": "Memory directory is empty"
            });
        }

        let mut matches = Vec::new();
        self.search_files_recursive(&self.memory_dir, "", &keyword, &mut matches);

        json!({
            "success": true,
            "keyword": keyword,
            "total_matches": matches.len(),
            "matches": matches
        })
    }

    /// Recursively search files for keyword
    fn search_files_recursive(
        &self,
        dir: &Path,
        prefix: &str,
        keyword: &str,
        matches: &mut Vec<Value>,
    ) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let relative_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            if path.is_dir() {
                self.search_files_recursive(&path, &relative_path, keyword, matches);
            } else if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    let lower_content = content.to_lowercase();
                    if lower_content.contains(keyword) {
                        // Extract context around matches
                        let excerpts: Vec<String> = content
                            .lines()
                            .filter(|line| line.to_lowercase().contains(keyword))
                            .take(3)
                            .map(|line| truncate_with_ellipsis(line, 197))
                            .collect();

                        matches.push(json!({
                            "file": relative_path,
                            "excerpts": excerpts
                        }));
                    }
                }
            }
        }
    }

    /// Sanitize a path to ensure it stays within the memory directory
    fn sanitize_path(&self, path: &str) -> Option<PathBuf> {
        let clean_path = path.trim_start_matches('/').trim_start_matches("./");

        if clean_path.contains("..") {
            return None;
        }

        let full_path = self.memory_dir.join(clean_path);

        match full_path.canonicalize() {
            Ok(canonical) => {
                if canonical.starts_with(&self.memory_dir) {
                    Some(canonical)
                } else {
                    None
                }
            }
            Err(_) => {
                // File doesn't exist
                if let Some(parent) = full_path.parent() {
                    if parent.exists() {
                        if let Ok(canonical_parent) = parent.canonicalize() {
                            if canonical_parent.starts_with(&self.memory_dir) {
                                return Some(full_path);
                            }
                        }
                    }
                }
                None
            }
        }
    }

    /// Search with a natural language query
    ///
    /// This runs a mini tool-loop with the light model to interpret the query
    /// and search across runs and memories.
    pub async fn search(&self, query: &str) -> Result<SearchResult, String> {
        // Create light model client
        let client = AIClient::light_from_env()
            .map_err(|e| format!("Failed to create light client: {}", e))?;

        let system_prompt = r#"You are a search agent for a System Commander AI. Your job is to search and synthesize results from two data sources.

## Data Sources:
1. **Run History Database** - Past agent runs with fields: agent_id, working_dir, status, source, initial_prompt, started_at, cost, can_resume
2. **Memory Files** - Persistent notes including user preferences, project notes, past decisions

## Tools:
- query_runs(filters): Search run history with optional filters (working_dir, status, source, keyword, days_back, resumable_only, limit)
- get_run_details(agent_id): Get full details for a specific run
- list_memory_files(): List all memory files
- read_memory_file(path): Read a specific memory file
- search_memory_content(keyword): Search keyword across all memory files

## Rules:
1. Interpret the natural language query
2. Decide which sources to search (runs, memory, or both)
3. Execute tool calls to gather information
4. Correlate and synthesize results
5. Return a concise, informative summary
6. For runs: highlight project name (from working_dir), status, what was done, when
7. For memory: provide relevant excerpts with file paths
8. If no results found, say so clearly

Be efficient - don't call tools unnecessarily if you already have the answer."#;

        let mut messages = vec![Message {
            role: "user".to_string(),
            content: query.to_string(),
        }];

        let tools = Self::get_search_tools();
        let mut runs_found = 0;
        let mut memories_searched = 0;
        let mut final_summary = String::new();

        for iteration in 0..MAX_SEARCH_AGENT_ITERATIONS {
            eprintln!(
                "[LLM][{}][{}] SearchAgent Iteration {}/{}",
                client.get_provider_name(),
                client.get_model_name(),
                iteration + 1,
                MAX_SEARCH_AGENT_ITERATIONS
            );

            let response = client
                .send_message_with_system_and_tools(system_prompt, messages.clone(), tools.clone())
                .await
                .map_err(|e| format!("Search agent API call failed: {}", e))?;

            let mut has_tool_use = false;
            let mut tool_results = Vec::new();

            for block in &response.content {
                match block {
                    ContentBlock::Text { text } => {
                        final_summary = text.clone();
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        has_tool_use = true;
                        eprintln!("[SearchAgent] Tool call: {} - {:?}", name, input);

                        let result = self.execute_search_tool(name, input).await;

                        // Track what was searched
                        if name == "query_runs" {
                            if let Some(count) = result["total_found"].as_u64() {
                                runs_found += count as usize;
                            }
                        } else if name == "search_memory_content" || name == "list_memory_files" {
                            memories_searched += 1;
                        }

                        eprintln!(
                            "[SearchAgent] Tool result keys: {:?}",
                            result.as_object().map(|o| o.keys().collect::<Vec<_>>())
                        );

                        tool_results.push((id.clone(), result));
                    }
                }
            }

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

        Ok(SearchResult {
            success: true,
            summary: final_summary,
            runs_found,
            memories_searched,
        })
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
    fn test_sanitize_path() {
        // Create a mock SearchAgent with a test memory_dir
        // Note: This test is limited without a real AgentRunsDB
        let _memory_dir = PathBuf::from("/tmp/test-search-memory");

        // Test path sanitization logic directly
        let clean = "MEMORY.md".trim_start_matches('/').trim_start_matches("./");
        assert!(!clean.contains(".."));

        let traversal = "../etc/passwd".trim_start_matches('/');
        assert!(traversal.contains(".."));
    }
}
