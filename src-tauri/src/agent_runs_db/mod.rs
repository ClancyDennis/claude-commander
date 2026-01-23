// Agent runs database module
//
// Manages persistence of agent run records including statistics,
// cost tracking, and prompt history.

mod cost;
mod models;
mod orchestrator_events;
mod schema;

use rusqlite::{params, Connection, Result as SqliteResult};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use models::format_bytes;
pub use models::{
    AgentOutputRecord,
    AgentRun,
    CostSummary,
    DailyCost,
    DatabaseStats,
    DateRangeCostSummary,
    EventQueryFilters,
    ModelCostBreakdown,
    OrchestratorDecisionRecord,
    OrchestratorStateChangeRecord,
    // Orchestrator event records
    OrchestratorToolCallRecord,
    PipelineHistoryBundle,
    RunQueryFilters,
    RunStats,
    RunStatus,
    SessionCostRecord,
};

use cost::CostOperations;
use orchestrator_events::OrchestratorEventOps;

pub struct AgentRunsDB {
    db: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl AgentRunsDB {
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        let conn = Connection::open(&db_path)?;

        // Initialize all database tables and run migrations
        schema::initialize_schema(&conn)?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// Create a new run record when an agent starts
    pub async fn create_run(&self, run: &AgentRun) -> SqliteResult<i64> {
        let db = self.db.lock().await;

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
    }

    /// Update an existing run record
    pub async fn update_run(&self, run: &AgentRun) -> SqliteResult<()> {
        let db = self.db.lock().await;

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
    }

    /// Get a specific run by agent_id
    pub async fn get_run(&self, agent_id: &str) -> SqliteResult<Option<AgentRun>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            "SELECT id, agent_id, session_id, working_dir, github_url, github_context,
                    source, status, started_at, ended_at, last_activity,
                    initial_prompt, error_message, pipeline_id, total_prompts, total_tool_calls,
                    total_output_bytes, total_tokens_used, total_cost_usd, model_usage,
                    can_resume, resume_data
             FROM agent_runs WHERE agent_id = ?1",
        )?;

        let mut rows = stmt.query(params![agent_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row_to_run(row)?))
        } else {
            Ok(None)
        }
    }

    /// Query runs with filters
    pub async fn query_runs(&self, filters: RunQueryFilters) -> SqliteResult<Vec<AgentRun>> {
        let db = self.db.lock().await;

        let mut query = "SELECT id, agent_id, session_id, working_dir, github_url, github_context,
                                source, status, started_at, ended_at, last_activity,
                                initial_prompt, error_message, pipeline_id, total_prompts, total_tool_calls,
                                total_output_bytes, total_tokens_used, total_cost_usd, model_usage,
                                can_resume, resume_data
                         FROM agent_runs WHERE 1=1"
            .to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(status) = filters.status {
            query.push_str(" AND status = ?");
            params.push(Box::new(status.to_str().to_string()));
        }

        if let Some(working_dir) = filters.working_dir {
            query.push_str(" AND working_dir = ?");
            params.push(Box::new(working_dir));
        }

        if let Some(source) = filters.source {
            query.push_str(" AND source = ?");
            params.push(Box::new(source.as_str().to_string()));
        }

        if let Some(date_from) = filters.date_from {
            query.push_str(" AND started_at >= ?");
            params.push(Box::new(date_from.timestamp_millis()));
        }

        if let Some(date_to) = filters.date_to {
            query.push_str(" AND started_at <= ?");
            params.push(Box::new(date_to.timestamp_millis()));
        }

        query.push_str(" ORDER BY started_at DESC");

        if let Some(limit) = filters.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filters.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = db.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let runs = stmt.query_map(param_refs.as_slice(), |row| row_to_run(row))?;

        let mut result = Vec::new();
        for run in runs {
            result.push(run?);
        }

        Ok(result)
    }

    /// Get all crashed runs that can be resumed
    pub async fn get_resumable_runs(&self) -> SqliteResult<Vec<AgentRun>> {
        self.query_runs(RunQueryFilters {
            status: Some(RunStatus::Crashed),
            ..Default::default()
        })
        .await
        .map(|runs| runs.into_iter().filter(|r| r.can_resume).collect())
    }

    /// Record a prompt sent to an agent
    pub async fn record_prompt(
        &self,
        agent_id: &str,
        prompt: &str,
        timestamp: i64,
    ) -> SqliteResult<()> {
        let db = self.db.lock().await;

        db.execute(
            "INSERT INTO agent_prompts (agent_id, timestamp, prompt) VALUES (?1, ?2, ?3)",
            params![agent_id, timestamp, prompt],
        )?;

        Ok(())
    }

    /// Get prompt history for an agent
    pub async fn get_prompts(&self, agent_id: &str) -> SqliteResult<Vec<(String, i64)>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            "SELECT prompt, timestamp FROM agent_prompts WHERE agent_id = ?1 ORDER BY timestamp ASC",
        )?;

        let prompts = stmt.query_map(params![agent_id], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut result = Vec::new();
        for prompt in prompts {
            result.push(prompt?);
        }

        Ok(result)
    }

    /// Clean up old runs (older than days_to_keep)
    pub async fn cleanup_old_runs(&self, days_to_keep: i64) -> SqliteResult<usize> {
        let db = self.db.lock().await;
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
    }

    /// Get statistics about all runs
    pub async fn get_stats(&self) -> SqliteResult<RunStats> {
        let db = self.db.lock().await;

        let total: i64 = db.query_row("SELECT COUNT(*) FROM agent_runs", [], |row| row.get(0))?;

        let by_status: Vec<(String, i64)> = {
            let mut stmt = db.prepare("SELECT status, COUNT(*) FROM agent_runs GROUP BY status")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect::<SqliteResult<Vec<_>>>()?
        };

        let by_source: Vec<(String, i64)> = {
            let mut stmt = db.prepare("SELECT source, COUNT(*) FROM agent_runs GROUP BY source")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect::<SqliteResult<Vec<_>>>()?
        };

        let total_cost: f64 = db.query_row(
            "SELECT COALESCE(SUM(total_cost_usd), 0.0) FROM agent_runs WHERE total_cost_usd IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        let resumable_count: i64 = db.query_row(
            "SELECT COUNT(*) FROM agent_runs WHERE can_resume = 1 AND status = 'crashed'",
            [],
            |row| row.get(0),
        )?;

        Ok(RunStats {
            total_runs: total,
            by_status,
            by_source,
            total_cost_usd: total_cost,
            resumable_runs: resumable_count,
        })
    }

    /// Get database statistics including size and record counts
    pub async fn get_database_stats(&self) -> SqliteResult<DatabaseStats> {
        let db = self.db.lock().await;

        // Get database file size
        let db_size_bytes = fs::metadata(&self.db_path).map(|m| m.len()).unwrap_or(0);
        let db_size_formatted = format_bytes(db_size_bytes);

        // Get total runs
        let total_runs: i64 =
            db.query_row("SELECT COUNT(*) FROM agent_runs", [], |row| row.get(0))?;

        // Get total prompts
        let total_prompts: i64 =
            db.query_row("SELECT COUNT(*) FROM agent_prompts", [], |row| row.get(0))?;

        // Get runs by status
        let runs_by_status: Vec<(String, i64)> = {
            let mut stmt = db.prepare("SELECT status, COUNT(*) FROM agent_runs GROUP BY status")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect::<SqliteResult<Vec<_>>>()?
        };

        // Get runs by source
        let runs_by_source: Vec<(String, i64)> = {
            let mut stmt = db.prepare("SELECT source, COUNT(*) FROM agent_runs GROUP BY source")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect::<SqliteResult<Vec<_>>>()?
        };

        // Get total cost
        let total_cost_usd: f64 = db.query_row(
            "SELECT COALESCE(SUM(total_cost_usd), 0.0) FROM agent_runs WHERE total_cost_usd IS NOT NULL",
            [],
            |row| row.get(0),
        )?;

        Ok(DatabaseStats {
            db_size_bytes,
            db_size_formatted,
            total_runs,
            total_prompts,
            runs_by_status,
            runs_by_source,
            total_cost_usd,
        })
    }

    // ========================================================================
    // Cost operations - delegated to CostOperations
    // ========================================================================

    /// Get cost summary aggregated by working directory
    pub async fn get_cost_by_working_dir(&self) -> SqliteResult<Vec<(String, f64)>> {
        CostOperations::new(&self.db)
            .get_cost_by_working_dir()
            .await
    }

    /// Get daily cost breakdown
    pub async fn get_daily_costs(&self, days: i64) -> SqliteResult<Vec<DailyCost>> {
        CostOperations::new(&self.db).get_daily_costs(days).await
    }

    /// Get total cost for current month
    pub async fn get_current_month_cost(&self) -> Result<f64, String> {
        CostOperations::new(&self.db).get_current_month_cost().await
    }

    /// Get total cost for today
    pub async fn get_today_cost(&self) -> Result<f64, String> {
        CostOperations::new(&self.db).get_today_cost().await
    }

    /// Get cost summary - aggregated from all agent runs
    pub async fn get_cost_summary(&self) -> Result<CostSummary, String> {
        CostOperations::new(&self.db).get_cost_summary().await
    }

    /// Get date range cost summary
    pub async fn get_date_range_summary(
        &self,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<DateRangeCostSummary, String> {
        CostOperations::new(&self.db)
            .get_date_range_summary(start_date, end_date)
            .await
    }

    /// Clear all cost history
    pub async fn clear_cost_history(&self) -> Result<(), String> {
        CostOperations::new(&self.db).clear_cost_history().await
    }

    // ========================================================================
    // Orchestrator Event Persistence - delegated to OrchestratorEventOps
    // ========================================================================

    /// Insert a single orchestrator tool call record
    pub async fn insert_tool_call(&self, record: &OrchestratorToolCallRecord) -> SqliteResult<i64> {
        OrchestratorEventOps::new(&self.db)
            .insert_tool_call(record)
            .await
    }

    /// Insert a single orchestrator state change record
    pub async fn insert_state_change(
        &self,
        record: &OrchestratorStateChangeRecord,
    ) -> SqliteResult<i64> {
        OrchestratorEventOps::new(&self.db)
            .insert_state_change(record)
            .await
    }

    /// Insert a single orchestrator decision record
    pub async fn insert_decision(&self, record: &OrchestratorDecisionRecord) -> SqliteResult<i64> {
        OrchestratorEventOps::new(&self.db)
            .insert_decision(record)
            .await
    }

    /// Insert a single agent output record
    pub async fn insert_agent_output(&self, record: &AgentOutputRecord) -> SqliteResult<i64> {
        OrchestratorEventOps::new(&self.db)
            .insert_agent_output(record)
            .await
    }

    /// Query orchestrator tool calls with filters
    pub async fn query_tool_calls(
        &self,
        filters: EventQueryFilters,
    ) -> SqliteResult<Vec<OrchestratorToolCallRecord>> {
        OrchestratorEventOps::new(&self.db)
            .query_tool_calls(filters)
            .await
    }

    /// Query orchestrator state changes with filters
    pub async fn query_state_changes(
        &self,
        filters: EventQueryFilters,
    ) -> SqliteResult<Vec<OrchestratorStateChangeRecord>> {
        OrchestratorEventOps::new(&self.db)
            .query_state_changes(filters)
            .await
    }

    /// Query orchestrator decisions with filters
    pub async fn query_decisions(
        &self,
        filters: EventQueryFilters,
    ) -> SqliteResult<Vec<OrchestratorDecisionRecord>> {
        OrchestratorEventOps::new(&self.db)
            .query_decisions(filters)
            .await
    }

    /// Query agent outputs with filters
    pub async fn query_agent_outputs(
        &self,
        filters: EventQueryFilters,
    ) -> SqliteResult<Vec<AgentOutputRecord>> {
        OrchestratorEventOps::new(&self.db)
            .query_agent_outputs(filters)
            .await
    }

    /// Get all history for a pipeline (for restoring UI state after reload)
    pub async fn get_pipeline_history(
        &self,
        pipeline_id: &str,
    ) -> SqliteResult<PipelineHistoryBundle> {
        OrchestratorEventOps::new(&self.db)
            .get_pipeline_history(pipeline_id)
            .await
    }

    /// Clear all orchestrator events for a pipeline
    pub async fn clear_pipeline_events(&self, pipeline_id: &str) -> SqliteResult<()> {
        OrchestratorEventOps::new(&self.db)
            .clear_pipeline_events(pipeline_id)
            .await
    }
}

/// Helper to convert a row to AgentRun
fn row_to_run(row: &rusqlite::Row) -> SqliteResult<AgentRun> {
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
        status: RunStatus::from_str(&status_str),
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
