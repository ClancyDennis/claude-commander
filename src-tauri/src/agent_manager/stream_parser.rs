// Stream parsing utilities - persistence and buffer management
//
// This module provides helper functions for persisting agent outputs
// to the database and managing output buffers.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_runs_db::{AgentOutputRecord, AgentRunsDB};
use crate::types::AgentOutputEvent;
use crate::utils::time::now_millis;

/// Helper to persist agent outputs to the database
pub(crate) async fn persist_output(
    runs_db: &Option<Arc<AgentRunsDB>>,
    agent_id: &str,
    pipeline_id: &Option<String>,
    output_type: &str,
    content: &str,
) {
    if let Some(ref db) = runs_db {
        let record = AgentOutputRecord {
            id: None,
            agent_id: agent_id.to_string(),
            pipeline_id: pipeline_id.clone(),
            output_type: output_type.to_string(),
            content: content.to_string(),
            metadata: None,
            timestamp: now_millis(),
        };

        eprintln!(
            "[persist_output] Inserting output: agent_id={}, pipeline_id={:?}, output_type={}",
            record.agent_id, record.pipeline_id, record.output_type
        );

        // Write synchronously to avoid race conditions
        match db.insert_agent_output(&record).await {
            Ok(id) => {
                eprintln!(
                    "[persist_output] Successfully inserted output with id={}",
                    id
                );
            }
            Err(e) => {
                eprintln!("[persist_output] ERROR inserting output: {}", e);
            }
        }
    } else {
        eprintln!(
            "[persist_output] WARNING: No runs_db available for agent_id={}",
            agent_id
        );
    }
}

/// Maximum output buffer size per agent (prevents unbounded memory growth)
const MAX_OUTPUT_BUFFER: usize = 500;

/// Store output in buffer, keeping last MAX_OUTPUT_BUFFER outputs
pub(crate) async fn store_in_buffer(
    output_event: AgentOutputEvent,
    buffer: Arc<Mutex<Vec<AgentOutputEvent>>>,
) {
    let mut buffer = buffer.lock().await;
    buffer.push(output_event);
    let len = buffer.len();
    if len > MAX_OUTPUT_BUFFER {
        // Remove oldest entries to stay under limit
        buffer.drain(0..len - MAX_OUTPUT_BUFFER);
    }
}
