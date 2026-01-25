// CRUD operations for agent runs
//
// Provides create, read, update, delete operations for agent runs and prompts.

use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db_utils::{columns, DatabaseOps, QueryBuilder};

use super::models::{AgentRun, RunQueryFilters, RunStatus};

/// Helper to convert a row to AgentRun
pub fn row_to_run(row: &rusqlite::Row) -> SqliteResult<AgentRun> {
    let status_str: String = row.get(7)?;
    let can_resume_int: i32 = row.get(20)?;

    Ok(AgentRun {
        id: Some(row.get(0)?),
        agent_id: row.get(1)?,
        session_id: row.get(2)?,
        working_dir: row.get(3)?,
        github_url: row.get(4)?,
        github_context: row.get(5)?,
        source: row.get(6)?,
        status: RunStatus::parse(&status_str),
        started_at: row.get(8)?,
        ended_at: row.get(9)?,
        last_activity: row.get(10)?,
        initial_prompt: row.get(11)?,
        error_message: row.get(12)?,
        pipeline_id: row.get(13)?,
        total_prompts: row.get(14)?,
        total_tool_calls: row.get(15)?,
        total_output_bytes: row.get(16)?,
        total_tokens_used: row.get(17)?,
        total_cost_usd: row.get(18)?,
        model_usage: row.get(19)?,
        can_resume: can_resume_int != 0,
        resume_data: row.get(21)?,
    })
}

/// CRUD operations for agent runs
pub struct CrudOperations<'a> {
    db: &'a Arc<Mutex<Connection>>,
}

impl<'a> CrudOperations<'a> {
    pub fn new(db: &'a Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Create a new run record when an agent starts
    pub async fn create_run(&self, run: &AgentRun) -> SqliteResult<i64> {
        self.db
            .with_db(|db| {
                db.execute(
                    "INSERT INTO agent_runs (
                        agent_id, session_id, working_dir, github_url, github_context,
                        source, status, started_at, ended_at, last_activity,
                        initial_prompt, error_message, pipeline_id, total_prompts, total_tool_calls,
                        total_output_bytes, total_tokens_used, total_cost_usd, model_usage,
                        can_resume, resume_data
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
                    params![
                        run.agent_id,
                        run.session_id,
                        run.working_dir,
                        run.github_url,
                        run.github_context,
                        run.source,
                        run.status.to_str(),
                        run.started_at,
                        run.ended_at,
                        run.last_activity,
                        run.initial_prompt,
                        run.error_message,
                        run.pipeline_id,
                        run.total_prompts,
                        run.total_tool_calls,
                        run.total_output_bytes,
                        run.total_tokens_used,
                        run.total_cost_usd,
                        run.model_usage,
                        if run.can_resume { 1 } else { 0 },
                        run.resume_data,
                    ],
                )?;

                Ok(db.last_insert_rowid())
            })
            .await
    }

    /// Update an existing run record
    pub async fn update_run(&self, run: &AgentRun) -> SqliteResult<()> {
        self.db
            .with_db(|db| {
                db.execute(
                    "UPDATE agent_runs SET
                        session_id = ?2,
                        status = ?3,
                        ended_at = ?4,
                        last_activity = ?5,
                        error_message = ?6,
                        pipeline_id = ?7,
                        total_prompts = ?8,
                        total_tool_calls = ?9,
                        total_output_bytes = ?10,
                        total_tokens_used = ?11,
                        total_cost_usd = ?12,
                        model_usage = ?13,
                        can_resume = ?14,
                        resume_data = ?15
                    WHERE agent_id = ?1",
                    params![
                        run.agent_id,
                        run.session_id,
                        run.status.to_str(),
                        run.ended_at,
                        run.last_activity,
                        run.error_message,
                        run.pipeline_id,
                        run.total_prompts,
                        run.total_tool_calls,
                        run.total_output_bytes,
                        run.total_tokens_used,
                        run.total_cost_usd,
                        run.model_usage,
                        if run.can_resume { 1 } else { 0 },
                        run.resume_data,
                    ],
                )?;

                Ok(())
            })
            .await
    }

    /// Get a specific run by agent_id
    pub async fn get_run(&self, agent_id: &str) -> SqliteResult<Option<AgentRun>> {
        let agent_id = agent_id.to_string();
        self.db
            .with_db(move |db| {
                let query = format!(
                    "SELECT {} FROM agent_runs WHERE agent_id = ?1",
                    columns::AGENT_RUNS
                );
                let mut stmt = db.prepare(&query)?;
                let mut rows = stmt.query(params![agent_id])?;

                if let Some(row) = rows.next()? {
                    Ok(Some(row_to_run(row)?))
                } else {
                    Ok(None)
                }
            })
            .await
    }

    /// Query runs with filters
    pub async fn query_runs(&self, filters: RunQueryFilters) -> SqliteResult<Vec<AgentRun>> {
        self.db
            .with_db(move |db| {
                let base_query =
                    format!("SELECT {} FROM agent_runs WHERE 1=1", columns::AGENT_RUNS);
                let mut builder = QueryBuilder::new(&base_query);

                if let Some(status) = filters.status {
                    builder.add_condition("status = ?", status.to_str().to_string());
                }

                if let Some(working_dir) = filters.working_dir {
                    builder.add_condition("working_dir = ?", working_dir);
                }

                if let Some(source) = filters.source {
                    builder.add_condition("source = ?", source.as_str().to_string());
                }

                if let Some(date_from) = filters.date_from {
                    builder.add_condition("started_at >= ?", date_from.timestamp_millis());
                }

                if let Some(date_to) = filters.date_to {
                    builder.add_condition("started_at <= ?", date_to.timestamp_millis());
                }

                builder.add_order_by("started_at DESC");

                if let Some(limit) = filters.limit {
                    builder.add_limit(limit);
                }

                if let Some(offset) = filters.offset {
                    builder.add_offset(offset);
                }

                let (query, params) = builder.build();
                let mut stmt = db.prepare(&query)?;
                let param_refs = QueryBuilder::params_as_refs(&params);

                let runs = stmt.query_map(param_refs.as_slice(), row_to_run)?;

                runs.collect()
            })
            .await
    }

    /// Get all crashed runs that can be resumed
    pub async fn get_resumable_runs(&self) -> SqliteResult<Vec<AgentRun>> {
        let runs = self
            .query_runs(RunQueryFilters {
                status: Some(RunStatus::Crashed),
                ..Default::default()
            })
            .await?;

        Ok(runs.into_iter().filter(|r| r.can_resume).collect())
    }

    /// Record a prompt sent to an agent
    pub async fn record_prompt(
        &self,
        agent_id: &str,
        prompt: &str,
        timestamp: i64,
    ) -> SqliteResult<()> {
        let agent_id = agent_id.to_string();
        let prompt = prompt.to_string();

        self.db
            .with_db(move |db| {
                db.execute(
                    "INSERT INTO agent_prompts (agent_id, timestamp, prompt) VALUES (?1, ?2, ?3)",
                    params![agent_id, timestamp, prompt],
                )?;
                Ok(())
            })
            .await
    }

    /// Get prompt history for an agent
    pub async fn get_prompts(&self, agent_id: &str) -> SqliteResult<Vec<(String, i64)>> {
        let agent_id = agent_id.to_string();

        self.db
            .with_db(move |db| {
                let mut stmt = db.prepare(
                    "SELECT prompt, timestamp FROM agent_prompts WHERE agent_id = ?1 ORDER BY timestamp ASC",
                )?;

                let prompts = stmt.query_map(params![agent_id], |row| Ok((row.get(0)?, row.get(1)?)))?;

                prompts.collect()
            })
            .await
    }

    /// Clean up old runs (older than days_to_keep)
    pub async fn cleanup_old_runs(&self, days_to_keep: i64) -> SqliteResult<usize> {
        self.db
            .with_db(move |db| {
                let cutoff_timestamp =
                    chrono::Utc::now().timestamp_millis() - (days_to_keep * 24 * 60 * 60 * 1000);

                // Delete associated prompts first
                db.execute(
                    "DELETE FROM agent_prompts WHERE agent_id IN (
                        SELECT agent_id FROM agent_runs WHERE started_at < ?1
                    )",
                    params![cutoff_timestamp],
                )?;

                // Delete old runs
                db.execute(
                    "DELETE FROM agent_runs WHERE started_at < ?1",
                    params![cutoff_timestamp],
                )
            })
            .await
    }

    /// Mark all "running" or "waiting_input" agents as "crashed" - called on app startup
    /// This handles orphaned runs from previous sessions that didn't terminate cleanly
    pub async fn reconcile_stale_runs(&self) -> SqliteResult<usize> {
        self.db
            .with_db(|db| {
                let now = chrono::Utc::now().timestamp_millis();

                db.execute(
                    "UPDATE agent_runs SET
                        status = 'crashed',
                        ended_at = ?1,
                        last_activity = ?1,
                        error_message = COALESCE(error_message, 'Process terminated unexpectedly (app restart)'),
                        can_resume = 1
                     WHERE status = 'running' OR status = 'waiting_input'",
                    params![now],
                )
            })
            .await
    }
}
