// Agent runs database module
//
// Manages persistence of agent run records including statistics,
// cost tracking, and prompt history.

mod cost;
mod models;

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

pub struct AgentRunsDB {
    db: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl AgentRunsDB {
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        let conn = Connection::open(&db_path)?;

        // Create agent_runs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL UNIQUE,
                session_id TEXT,
                working_dir TEXT NOT NULL,
                github_url TEXT,
                github_context TEXT,
                source TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                ended_at INTEGER,
                last_activity INTEGER NOT NULL,
                initial_prompt TEXT,
                error_message TEXT,
                total_prompts INTEGER DEFAULT 0,
                total_tool_calls INTEGER DEFAULT 0,
                total_output_bytes INTEGER DEFAULT 0,
                total_tokens_used INTEGER,
                total_cost_usd REAL,
                model_usage TEXT,
                can_resume INTEGER DEFAULT 0,
                resume_data TEXT
            )",
            [],
        )?;

        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runs_agent_id ON agent_runs(agent_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runs_status ON agent_runs(status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runs_started_at ON agent_runs(started_at DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runs_working_dir ON agent_runs(working_dir)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_runs_source ON agent_runs(source)",
            [],
        )?;

        // Create prompts table for tracking conversation history
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_prompts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                prompt TEXT NOT NULL,
                response_summary TEXT,
                FOREIGN KEY (agent_id) REFERENCES agent_runs(agent_id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_prompts_agent ON agent_prompts(agent_id, timestamp)",
            [],
        )?;

        // Migration: Add model_usage column if it doesn't exist
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(agent_runs)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;

        if !columns.contains(&"model_usage".to_string()) {
            conn.execute("ALTER TABLE agent_runs ADD COLUMN model_usage TEXT", [])?;
        }

        // Migration: Add pipeline_id column for linking runs to orchestrator events
        if !columns.contains(&"pipeline_id".to_string()) {
            conn.execute("ALTER TABLE agent_runs ADD COLUMN pipeline_id TEXT", [])?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_runs_pipeline_id ON agent_runs(pipeline_id)",
                [],
            )?;
        }

        // ====================================================================
        // Orchestrator event tables (for hybrid persistence)
        // ====================================================================

        // Create orchestrator_tool_calls table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS orchestrator_tool_calls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pipeline_id TEXT NOT NULL,
                agent_id TEXT,
                tool_name TEXT NOT NULL,
                tool_input TEXT,
                is_error INTEGER DEFAULT 0,
                summary TEXT,
                current_state TEXT NOT NULL,
                iteration INTEGER NOT NULL,
                step_number INTEGER,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tool_calls_pipeline ON orchestrator_tool_calls(pipeline_id, timestamp DESC)",
            [],
        )?;

        // Create orchestrator_state_changes table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS orchestrator_state_changes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pipeline_id TEXT NOT NULL,
                old_state TEXT NOT NULL,
                new_state TEXT NOT NULL,
                iteration INTEGER NOT NULL,
                generated_skills INTEGER DEFAULT 0,
                generated_subagents INTEGER DEFAULT 0,
                claudemd_generated INTEGER DEFAULT 0,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_state_changes_pipeline ON orchestrator_state_changes(pipeline_id, timestamp DESC)",
            [],
        )?;

        // Create orchestrator_decisions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS orchestrator_decisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pipeline_id TEXT NOT NULL,
                decision TEXT NOT NULL,
                reasoning TEXT,
                issues TEXT,
                suggestions TEXT,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_decisions_pipeline ON orchestrator_decisions(pipeline_id, timestamp DESC)",
            [],
        )?;

        // Create agent_outputs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_outputs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                pipeline_id TEXT,
                output_type TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_outputs_agent ON agent_outputs(agent_id, timestamp DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_outputs_pipeline ON agent_outputs(pipeline_id, timestamp DESC)",
            [],
        )?;

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

    // Cost operations - delegated to CostOperations

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
    // Orchestrator Event Persistence Methods
    // ========================================================================

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

    // ========================================================================
    // Orchestrator Event Query Methods
    // ========================================================================

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
