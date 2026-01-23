// Statistics management for agents

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::{AgentStatistics, ModelUsageStats};

/// Update statistics with output bytes
pub async fn update_output_bytes(stats: &Arc<Mutex<AgentStatistics>>, byte_size: usize) {
    let mut stats = stats.lock().await;
    stats.total_output_bytes += byte_size as u64;
    stats.last_activity = chrono::Utc::now().to_rfc3339();
}

/// Increment tool call counter
pub async fn increment_tool_calls(stats: &Arc<Mutex<AgentStatistics>>) {
    let mut stats = stats.lock().await;
    stats.total_tool_calls += 1;
    stats.last_activity = chrono::Utc::now().to_rfc3339();
}

/// Increment prompt counter
pub async fn increment_prompts(stats: &Arc<Mutex<AgentStatistics>>) {
    let mut stats = stats.lock().await;
    stats.total_prompts += 1;
    stats.last_activity = chrono::Utc::now().to_rfc3339();
}

/// Update statistics from a result message
pub async fn update_from_result(
    stats: &Arc<Mutex<AgentStatistics>>,
    json: &serde_json::Value,
    byte_size: usize,
) {
    let mut stats = stats.lock().await;

    // Extract total_cost_usd directly from result (most accurate)
    if let Some(total_cost) = json.get("total_cost_usd").and_then(|v| v.as_f64()) {
        stats.total_cost_usd = Some(stats.total_cost_usd.unwrap_or(0.0) + total_cost);
    }

    // Extract modelUsage for detailed per-model breakdown
    if let Some(model_usage_obj) = json.get("modelUsage").and_then(|v| v.as_object()) {
        let mut model_usage_map = stats.model_usage.take().unwrap_or_default();

        for (model_name, model_data) in model_usage_obj {
            let usage_stats = ModelUsageStats {
                input_tokens: model_data.get("inputTokens").and_then(|v| v.as_u64()),
                output_tokens: model_data.get("outputTokens").and_then(|v| v.as_u64()),
                cache_creation_input_tokens: model_data
                    .get("cacheCreationInputTokens")
                    .and_then(|v| v.as_u64()),
                cache_read_input_tokens: model_data
                    .get("cacheReadInputTokens")
                    .and_then(|v| v.as_u64()),
                cost_usd: model_data.get("costUSD").and_then(|v| v.as_f64()),
                context_window: model_data.get("contextWindow").and_then(|v| v.as_u64()),
                max_output_tokens: model_data.get("maxOutputTokens").and_then(|v| v.as_u64()),
            };

            // Accumulate or insert model usage
            let entry = model_usage_map
                .entry(model_name.clone())
                .or_insert_with(|| ModelUsageStats {
                    input_tokens: Some(0),
                    output_tokens: Some(0),
                    cache_creation_input_tokens: Some(0),
                    cache_read_input_tokens: Some(0),
                    cost_usd: Some(0.0),
                    context_window: usage_stats.context_window,
                    max_output_tokens: usage_stats.max_output_tokens,
                });

            if let Some(tokens) = usage_stats.input_tokens {
                entry.input_tokens = Some(entry.input_tokens.unwrap_or(0) + tokens);
            }
            if let Some(tokens) = usage_stats.output_tokens {
                entry.output_tokens = Some(entry.output_tokens.unwrap_or(0) + tokens);
            }
            if let Some(tokens) = usage_stats.cache_creation_input_tokens {
                entry.cache_creation_input_tokens =
                    Some(entry.cache_creation_input_tokens.unwrap_or(0) + tokens);
            }
            if let Some(tokens) = usage_stats.cache_read_input_tokens {
                entry.cache_read_input_tokens =
                    Some(entry.cache_read_input_tokens.unwrap_or(0) + tokens);
            }
            if let Some(cost) = usage_stats.cost_usd {
                entry.cost_usd = Some(entry.cost_usd.unwrap_or(0.0) + cost);
            }
        }

        stats.model_usage = Some(model_usage_map);
    }

    // Extract performance metrics
    if let Some(duration) = json.get("duration_api_ms").and_then(|v| v.as_u64()) {
        stats.duration_api_ms = Some(stats.duration_api_ms.unwrap_or(0) + duration);
    }
    if let Some(duration) = json.get("duration_ms").and_then(|v| v.as_u64()) {
        stats.duration_ms = Some(stats.duration_ms.unwrap_or(0) + duration);
    }
    if let Some(turns) = json.get("num_turns").and_then(|v| v.as_u64()) {
        stats.num_turns = Some(stats.num_turns.unwrap_or(0) + turns as u32);
    }

    // Extract token counts from usage object (for total_tokens_used)
    if let Some(usage) = json.get("usage") {
        let input_tokens = usage
            .get("input_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let output_tokens = usage
            .get("output_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let total_tokens = input_tokens + output_tokens;

        if total_tokens > 0 {
            stats.total_tokens_used =
                Some(stats.total_tokens_used.unwrap_or(0) + total_tokens as u32);
        }
    }

    stats.total_output_bytes += byte_size as u64;
    stats.last_activity = chrono::Utc::now().to_rfc3339();
}

/// Create initial statistics for a new agent
pub fn create_initial_stats(agent_id: String) -> AgentStatistics {
    let session_start = chrono::Utc::now().to_rfc3339();
    AgentStatistics {
        agent_id,
        total_prompts: 0,
        total_tool_calls: 0,
        total_output_bytes: 0,
        session_start: session_start.clone(),
        last_activity: session_start,
        total_tokens_used: None,
        total_cost_usd: None,
        model_usage: None,
        duration_api_ms: None,
        duration_ms: None,
        num_turns: None,
    }
}
