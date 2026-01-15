use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};

use crate::types::AgentSource;

/// Run status - tracks the lifecycle of an agent run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Running,      // Currently active
    Completed,    // Finished successfully
    Stopped,      // Manually stopped
    Crashed,      // Process crashed or failed
    WaitingInput, // Waiting for user input
}

impl RunStatus {
    fn to_string(&self) -> &'static str {
        match self {
            RunStatus::Running => "running",
            RunStatus::Completed => "completed",
            RunStatus::Stopped => "stopped",
            RunStatus::Crashed => "crashed",
            RunStatus::WaitingInput => "waiting_input",
        }
    }

    fn from_string(s: &str) -> Self {
        match s {
            "running" => RunStatus::Running,
            "completed" => RunStatus::Completed,
            "stopped" => RunStatus::Stopped,
            "crashed" => RunStatus::Crashed,
            "waiting_input" => RunStatus::WaitingInput,
            _ => RunStatus::Crashed,
        }
    }
}

/// A record of an agent run - stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: Option<i64>,
    pub agent_id: String,
    pub session_id: Option<String>,
    pub working_dir: String,
    pub github_url: Option<String>,
    pub github_context: Option<String>, // JSON serialized
    pub source: String,
    pub status: RunStatus,
    pub started_at: i64,         // Unix timestamp in milliseconds
    pub ended_at: Option<i64>,   // Unix timestamp in milliseconds
    pub last_activity: i64,      // Unix timestamp in milliseconds
    pub initial_prompt: Option<String>,
    pub error_message: Option<String>,

    // Statistics
    pub total_prompts: u32,
    pub total_tool_calls: u32,
    pub total_output_bytes: u64,
    pub total_tokens_used: Option<u32>,
    pub total_cost_usd: Option<f64>,

    // Recovery information
    pub can_resume: bool,
    pub resume_data: Option<String>, // JSON serialized state for recovery
}

/// Query filters for searching runs
#[derive(Debug, Clone, Default)]
pub struct RunQueryFilters {
    pub status: Option<RunStatus>,
    pub working_dir: Option<String>,
    pub source: Option<AgentSource>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

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
                initial_prompt, error_message, total_prompts, total_tool_calls,
                total_output_bytes, total_tokens_used, total_cost_usd,
                can_resume, resume_data
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                run.agent_id,
                run.session_id,
                run.working_dir,
                run.github_url,
                run.github_context,
                run.source,
                run.status.to_string(),
                run.started_at,
                run.ended_at,
                run.last_activity,
                run.initial_prompt,
                run.error_message,
                run.total_prompts,
                run.total_tool_calls,
                run.total_output_bytes,
                run.total_tokens_used,
                run.total_cost_usd,
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
                total_prompts = ?7,
                total_tool_calls = ?8,
                total_output_bytes = ?9,
                total_tokens_used = ?10,
                total_cost_usd = ?11,
                can_resume = ?12,
                resume_data = ?13
            WHERE agent_id = ?1",
            params![
                run.agent_id,
                run.session_id,
                run.status.to_string(),
                run.ended_at,
                run.last_activity,
                run.error_message,
                run.total_prompts,
                run.total_tool_calls,
                run.total_output_bytes,
                run.total_tokens_used,
                run.total_cost_usd,
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
                    initial_prompt, error_message, total_prompts, total_tool_calls,
                    total_output_bytes, total_tokens_used, total_cost_usd,
                    can_resume, resume_data
             FROM agent_runs WHERE agent_id = ?1"
        )?;

        let mut rows = stmt.query(params![agent_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_run(row)?))
        } else {
            Ok(None)
        }
    }

    /// Query runs with filters
    pub async fn query_runs(&self, filters: RunQueryFilters) -> SqliteResult<Vec<AgentRun>> {
        let db = self.db.lock().await;

        let mut query = "SELECT id, agent_id, session_id, working_dir, github_url, github_context,
                                source, status, started_at, ended_at, last_activity,
                                initial_prompt, error_message, total_prompts, total_tool_calls,
                                total_output_bytes, total_tokens_used, total_cost_usd,
                                can_resume, resume_data
                         FROM agent_runs WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(status) = filters.status {
            query.push_str(" AND status = ?");
            params.push(Box::new(status.to_string().to_string()));
        }

        if let Some(working_dir) = filters.working_dir {
            query.push_str(" AND working_dir = ?");
            params.push(Box::new(working_dir));
        }

        if let Some(source) = filters.source {
            let source_str = match source {
                AgentSource::UI => "ui",
                AgentSource::Meta => "meta",
                AgentSource::Pipeline => "pipeline",
                AgentSource::Pool => "pool",
                AgentSource::Manual => "manual",
            };
            query.push_str(" AND source = ?");
            params.push(Box::new(source_str.to_string()));
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

        let runs = stmt.query_map(param_refs.as_slice(), |row| {
            self.row_to_run(row)
        })?;

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
        }).await.map(|runs| {
            runs.into_iter().filter(|r| r.can_resume).collect()
        })
    }

    /// Record a prompt sent to an agent
    pub async fn record_prompt(&self, agent_id: &str, prompt: &str, timestamp: i64) -> SqliteResult<()> {
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
            "SELECT prompt, timestamp FROM agent_prompts WHERE agent_id = ?1 ORDER BY timestamp ASC"
        )?;

        let prompts = stmt.query_map(params![agent_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        let mut result = Vec::new();
        for prompt in prompts {
            result.push(prompt?);
        }

        Ok(result)
    }

    /// Clean up old runs (older than days_to_keep)
    pub async fn cleanup_old_runs(&self, days_to_keep: i64) -> SqliteResult<usize> {
        let db = self.db.lock().await;
        let cutoff_timestamp = chrono::Utc::now().timestamp_millis() - (days_to_keep * 24 * 60 * 60 * 1000);

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

    /// Helper to convert a row to AgentRun
    fn row_to_run(&self, row: &rusqlite::Row) -> SqliteResult<AgentRun> {
        let status_str: String = row.get(7)?;
        let can_resume_int: i32 = row.get(18)?;

        Ok(AgentRun {
            id: Some(row.get(0)?),
            agent_id: row.get(1)?,
            session_id: row.get(2)?,
            working_dir: row.get(3)?,
            github_url: row.get(4)?,
            github_context: row.get(5)?,
            source: row.get(6)?,
            status: RunStatus::from_string(&status_str),
            started_at: row.get(8)?,
            ended_at: row.get(9)?,
            last_activity: row.get(10)?,
            initial_prompt: row.get(11)?,
            error_message: row.get(12)?,
            total_prompts: row.get(13)?,
            total_tool_calls: row.get(14)?,
            total_output_bytes: row.get(15)?,
            total_tokens_used: row.get(16)?,
            total_cost_usd: row.get(17)?,
            can_resume: can_resume_int != 0,
            resume_data: row.get(19)?,
        })
    }

    /// Get statistics about all runs
    pub async fn get_stats(&self) -> SqliteResult<RunStats> {
        let db = self.db.lock().await;

        let total: i64 = db.query_row(
            "SELECT COUNT(*) FROM agent_runs",
            [],
            |row| row.get(0),
        )?;

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
        let db_size_bytes = fs::metadata(&self.db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Format size as human readable
        let db_size_formatted = format_bytes(db_size_bytes);

        // Get total runs
        let total_runs: i64 = db.query_row(
            "SELECT COUNT(*) FROM agent_runs",
            [],
            |row| row.get(0),
        )?;

        // Get total prompts
        let total_prompts: i64 = db.query_row(
            "SELECT COUNT(*) FROM agent_prompts",
            [],
            |row| row.get(0),
        )?;

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
}

/// Format bytes into human readable size
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let base = 1024_f64;
    let exp = (bytes as f64).log(base).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);

    let size = bytes as f64 / base.powi(exp as i32);

    format!("{:.2} {}", size, UNITS[exp])
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunStats {
    pub total_runs: i64,
    pub by_status: Vec<(String, i64)>,
    pub by_source: Vec<(String, i64)>,
    pub total_cost_usd: f64,
    pub resumable_runs: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub db_size_bytes: u64,
    pub db_size_formatted: String,
    pub total_runs: i64,
    pub total_prompts: i64,
    pub runs_by_status: Vec<(String, i64)>,
    pub runs_by_source: Vec<(String, i64)>,
    pub total_cost_usd: f64,
}
