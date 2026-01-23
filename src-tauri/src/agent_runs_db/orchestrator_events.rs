// Orchestrator event persistence module
//
// Handles all database operations for orchestrator events including:
// - Tool calls
// - State changes
// - Decisions
// - Agent outputs

use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::models::{
    AgentOutputRecord, EventQueryFilters, OrchestratorDecisionRecord,
    OrchestratorStateChangeRecord, OrchestratorToolCallRecord, PipelineHistoryBundle,
};

/// Operations for orchestrator event persistence
pub struct OrchestratorEventOps<'a> {
    db: &'a Arc<Mutex<Connection>>,
}

impl<'a> OrchestratorEventOps<'a> {
    pub fn new(db: &'a Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Insert a single orchestrator tool call record
    pub async fn insert_tool_call(&self, record: &OrchestratorToolCallRecord) -> SqliteResult<i64> {
        let db = self.db.lock().await;

        db.execute(
            "INSERT INTO orchestrator_tool_calls
             (pipeline_id, agent_id, tool_name, tool_input, is_error, summary,
              current_state, iteration, step_number, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.pipeline_id,
                record.agent_id,
                record.tool_name,
                record.tool_input,
                if record.is_error { 1 } else { 0 },
                record.summary,
                record.current_state,
                record.iteration,
                record.step_number,
                record.timestamp
            ],
        )?;

        Ok(db.last_insert_rowid())
    }

    /// Insert a single orchestrator state change record
    pub async fn insert_state_change(
        &self,
        record: &OrchestratorStateChangeRecord,
    ) -> SqliteResult<i64> {
        let db = self.db.lock().await;

        db.execute(
            "INSERT INTO orchestrator_state_changes
             (pipeline_id, old_state, new_state, iteration, generated_skills,
              generated_subagents, claudemd_generated, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.pipeline_id,
                record.old_state,
                record.new_state,
                record.iteration,
                record.generated_skills,
                record.generated_subagents,
                if record.claudemd_generated { 1 } else { 0 },
                record.timestamp
            ],
        )?;

        Ok(db.last_insert_rowid())
    }

    /// Insert a single orchestrator decision record
    pub async fn insert_decision(&self, record: &OrchestratorDecisionRecord) -> SqliteResult<i64> {
        let db = self.db.lock().await;

        let issues_json =
            serde_json::to_string(&record.issues).unwrap_or_else(|_| "[]".to_string());
        let suggestions_json =
            serde_json::to_string(&record.suggestions).unwrap_or_else(|_| "[]".to_string());

        db.execute(
            "INSERT INTO orchestrator_decisions
             (pipeline_id, decision, reasoning, issues, suggestions, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                record.pipeline_id,
                record.decision,
                record.reasoning,
                issues_json,
                suggestions_json,
                record.timestamp
            ],
        )?;

        Ok(db.last_insert_rowid())
    }

    /// Insert a single agent output record
    pub async fn insert_agent_output(&self, record: &AgentOutputRecord) -> SqliteResult<i64> {
        let db = self.db.lock().await;

        db.execute(
            "INSERT INTO agent_outputs
             (agent_id, pipeline_id, output_type, content, metadata, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                record.agent_id,
                record.pipeline_id,
                record.output_type,
                record.content,
                record.metadata,
                record.timestamp
            ],
        )?;

        Ok(db.last_insert_rowid())
    }

    /// Query orchestrator tool calls with filters
    pub async fn query_tool_calls(
        &self,
        filters: EventQueryFilters,
    ) -> SqliteResult<Vec<OrchestratorToolCallRecord>> {
        let db = self.db.lock().await;

        let mut query = "SELECT id, pipeline_id, agent_id, tool_name, tool_input, is_error,
                                summary, current_state, iteration, step_number, timestamp
                         FROM orchestrator_tool_calls WHERE 1=1"
            .to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref pipeline_id) = filters.pipeline_id {
            query.push_str(" AND pipeline_id = ?");
            params.push(Box::new(pipeline_id.clone()));
        }
        if let Some(ref agent_id) = filters.agent_id {
            query.push_str(" AND agent_id = ?");
            params.push(Box::new(agent_id.clone()));
        }
        if let Some(since) = filters.since_timestamp {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(since));
        }
        if let Some(until) = filters.until_timestamp {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(until));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = db.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let is_error_int: i32 = row.get(5)?;
            Ok(OrchestratorToolCallRecord {
                id: Some(row.get(0)?),
                pipeline_id: row.get(1)?,
                agent_id: row.get(2)?,
                tool_name: row.get(3)?,
                tool_input: row.get(4)?,
                is_error: is_error_int != 0,
                summary: row.get(6)?,
                current_state: row.get(7)?,
                iteration: row.get(8)?,
                step_number: row.get(9)?,
                timestamp: row.get(10)?,
            })
        })?;

        rows.collect()
    }

    /// Query orchestrator state changes with filters
    pub async fn query_state_changes(
        &self,
        filters: EventQueryFilters,
    ) -> SqliteResult<Vec<OrchestratorStateChangeRecord>> {
        let db = self.db.lock().await;

        let mut query = "SELECT id, pipeline_id, old_state, new_state, iteration,
                                generated_skills, generated_subagents, claudemd_generated, timestamp
                         FROM orchestrator_state_changes WHERE 1=1"
            .to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref pipeline_id) = filters.pipeline_id {
            query.push_str(" AND pipeline_id = ?");
            params.push(Box::new(pipeline_id.clone()));
        }
        if let Some(since) = filters.since_timestamp {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(since));
        }
        if let Some(until) = filters.until_timestamp {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(until));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = db.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let claudemd_int: i32 = row.get(7)?;
            Ok(OrchestratorStateChangeRecord {
                id: Some(row.get(0)?),
                pipeline_id: row.get(1)?,
                old_state: row.get(2)?,
                new_state: row.get(3)?,
                iteration: row.get(4)?,
                generated_skills: row.get(5)?,
                generated_subagents: row.get(6)?,
                claudemd_generated: claudemd_int != 0,
                timestamp: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    /// Query orchestrator decisions with filters
    pub async fn query_decisions(
        &self,
        filters: EventQueryFilters,
    ) -> SqliteResult<Vec<OrchestratorDecisionRecord>> {
        let db = self.db.lock().await;

        let mut query =
            "SELECT id, pipeline_id, decision, reasoning, issues, suggestions, timestamp
                         FROM orchestrator_decisions WHERE 1=1"
                .to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref pipeline_id) = filters.pipeline_id {
            query.push_str(" AND pipeline_id = ?");
            params.push(Box::new(pipeline_id.clone()));
        }
        if let Some(since) = filters.since_timestamp {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(since));
        }
        if let Some(until) = filters.until_timestamp {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(until));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = db.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            let issues_json: String = row.get(4)?;
            let suggestions_json: String = row.get(5)?;
            let issues: Vec<String> = serde_json::from_str(&issues_json).unwrap_or_default();
            let suggestions: Vec<String> =
                serde_json::from_str(&suggestions_json).unwrap_or_default();

            Ok(OrchestratorDecisionRecord {
                id: Some(row.get(0)?),
                pipeline_id: row.get(1)?,
                decision: row.get(2)?,
                reasoning: row.get(3)?,
                issues,
                suggestions,
                timestamp: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    /// Query agent outputs with filters
    pub async fn query_agent_outputs(
        &self,
        filters: EventQueryFilters,
    ) -> SqliteResult<Vec<AgentOutputRecord>> {
        let db = self.db.lock().await;

        let mut query =
            "SELECT id, agent_id, pipeline_id, output_type, content, metadata, timestamp
                         FROM agent_outputs WHERE 1=1"
                .to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref pipeline_id) = filters.pipeline_id {
            query.push_str(" AND pipeline_id = ?");
            params.push(Box::new(pipeline_id.clone()));
        }
        if let Some(ref agent_id) = filters.agent_id {
            query.push_str(" AND agent_id = ?");
            params.push(Box::new(agent_id.clone()));
        }
        if let Some(since) = filters.since_timestamp {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(since));
        }
        if let Some(until) = filters.until_timestamp {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(until));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = db.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(AgentOutputRecord {
                id: Some(row.get(0)?),
                agent_id: row.get(1)?,
                pipeline_id: row.get(2)?,
                output_type: row.get(3)?,
                content: row.get(4)?,
                metadata: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })?;

        rows.collect()
    }

    /// Get all history for a pipeline (for restoring UI state after reload)
    pub async fn get_pipeline_history(
        &self,
        pipeline_id: &str,
    ) -> SqliteResult<PipelineHistoryBundle> {
        let filters = EventQueryFilters {
            pipeline_id: Some(pipeline_id.to_string()),
            ..Default::default()
        };

        let tool_calls = self.query_tool_calls(filters.clone()).await?;
        let state_changes = self.query_state_changes(filters.clone()).await?;
        let decisions = self.query_decisions(filters).await?;

        Ok(PipelineHistoryBundle {
            tool_calls,
            state_changes,
            decisions,
        })
    }

    /// Clear all orchestrator events for a pipeline
    pub async fn clear_pipeline_events(&self, pipeline_id: &str) -> SqliteResult<()> {
        let db = self.db.lock().await;

        db.execute(
            "DELETE FROM orchestrator_tool_calls WHERE pipeline_id = ?1",
            params![pipeline_id],
        )?;
        db.execute(
            "DELETE FROM orchestrator_state_changes WHERE pipeline_id = ?1",
            params![pipeline_id],
        )?;
        db.execute(
            "DELETE FROM orchestrator_decisions WHERE pipeline_id = ?1",
            params![pipeline_id],
        )?;
        db.execute(
            "DELETE FROM agent_outputs WHERE pipeline_id = ?1",
            params![pipeline_id],
        )?;

        Ok(())
    }
}
