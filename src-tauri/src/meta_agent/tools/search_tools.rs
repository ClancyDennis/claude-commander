// Search tools for MetaAgent
//
// Exposes the Search tool that invokes the search agent subagent.

use serde_json::{json, Value};
use std::sync::Arc;

use crate::agent_runs_db::AgentRunsDB;
use crate::meta_agent::search_agent::SearchAgent;

/// Search across run history and memories using natural language
pub async fn search(input: Value, runs_db: Arc<AgentRunsDB>) -> Value {
    let query = match input["query"].as_str() {
        Some(q) => q,
        None => {
            return json!({
                "success": false,
                "error": "query is required"
            });
        }
    };

    // Create search agent
    let agent = match SearchAgent::new(runs_db) {
        Some(a) => a,
        None => {
            return json!({
                "success": false,
                "error": "Failed to initialize search agent - could not determine data directory"
            });
        }
    };

    // Execute the search
    match agent.search(query).await {
        Ok(result) => json!({
            "success": result.success,
            "summary": result.summary,
            "runs_found": result.runs_found,
            "memories_searched": result.memories_searched,
            "hint": "This search queried both run history and persistent memories."
        }),
        Err(e) => json!({
            "success": false,
            "error": format!("Search failed: {}", e)
        }),
    }
}
