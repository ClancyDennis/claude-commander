use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::fs;
use tokio::sync::Mutex;
use chrono::{DateTime, Datelike, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCostRecord {
    pub id: Option<i64>,
    pub session_id: String,
    pub agent_id: String,
    pub working_dir: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub total_cost_usd: f64,
    pub total_tokens: u64,
    pub total_prompts: u32,
    pub total_tool_calls: u32,
    pub model_usage: Option<HashMap<String, ModelCostBreakdown>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCostBreakdown {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_cost_usd: f64,
    pub total_sessions: usize,
    pub total_tokens: u64,
    pub total_prompts: u32,
    pub total_tool_calls: u32,
    pub session_records: Vec<SessionCostRecord>,
    pub cost_by_model: HashMap<String, f64>,
    pub cost_by_working_dir: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRangeCostSummary {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub total_cost_usd: f64,
    pub session_count: usize,
    pub daily_costs: Vec<DailyCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCost {
    pub date: String,
    pub cost_usd: f64,
    pub session_count: usize,
}

pub struct CostTrackerDB {
    db: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl CostTrackerDB {
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        let conn = Connection::open(&db_path)?;

        // Create cost_sessions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cost_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL UNIQUE,
                agent_id TEXT NOT NULL,
                working_dir TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                total_cost_usd REAL NOT NULL DEFAULT 0.0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                total_prompts INTEGER NOT NULL DEFAULT 0,
                total_tool_calls INTEGER NOT NULL DEFAULT 0,
                model_usage TEXT
            )",
            [],
        )?;

        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cost_session_id ON cost_sessions(session_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cost_agent_id ON cost_sessions(agent_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cost_working_dir ON cost_sessions(working_dir)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cost_started_at ON cost_sessions(started_at DESC)",
            [],
        )?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// Record or update a session's cost data
    pub async fn record_session(&self, record: SessionCostRecord) -> Result<(), String> {
        let db = self.db.lock().await;

        let model_usage_json = if let Some(usage) = &record.model_usage {
            serde_json::to_string(usage).ok()
        } else {
            None
        };

        // Use REPLACE to insert or update
        db.execute(
            "REPLACE INTO cost_sessions
             (session_id, agent_id, working_dir, started_at, ended_at,
              total_cost_usd, total_tokens, total_prompts, total_tool_calls, model_usage)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.session_id,
                record.agent_id,
                record.working_dir,
                record.started_at,
                record.ended_at,
                record.total_cost_usd,
                record.total_tokens as i64,
                record.total_prompts as i64,
                record.total_tool_calls as i64,
                model_usage_json,
            ],
        ).map_err(|e| format!("Failed to record session: {}", e))?;

        Ok(())
    }

    /// Get all sessions
    pub async fn get_all_sessions(&self) -> Result<Vec<SessionCostRecord>, String> {
        let db = self.db.lock().await;

        let mut stmt = db
            .prepare(
                "SELECT id, session_id, agent_id, working_dir, started_at, ended_at,
                        total_cost_usd, total_tokens, total_prompts, total_tool_calls, model_usage
                 FROM cost_sessions
                 ORDER BY started_at DESC"
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let sessions = stmt
            .query_map([], |row| {
                Ok(self.row_to_session(row)?)
            })
            .map_err(|e| format!("Failed to query sessions: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect sessions: {}", e))?;

        Ok(sessions)
    }

    /// Get sessions within a date range
    pub async fn get_sessions_by_date_range(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<SessionCostRecord>, String> {
        let db = self.db.lock().await;

        let mut query = String::from(
            "SELECT id, session_id, agent_id, working_dir, started_at, ended_at,
                    total_cost_usd, total_tokens, total_prompts, total_tool_calls, model_usage
             FROM cost_sessions WHERE 1=1"
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(start) = start_date {
            query.push_str(" AND started_at >= ?");
            params.push(start.to_rfc3339());
        }

        if let Some(end) = end_date {
            query.push_str(" AND started_at <= ?");
            params.push(end.to_rfc3339());
        }

        query.push_str(" ORDER BY started_at DESC");

        let mut stmt = db
            .prepare(&query)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let sessions = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(self.row_to_session(row)?)
            })
            .map_err(|e| format!("Failed to query sessions: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect sessions: {}", e))?;

        Ok(sessions)
    }

    /// Get cost summary for all sessions
    pub async fn get_cost_summary(&self) -> Result<CostSummary, String> {
        let sessions = self.get_all_sessions().await?;

        let mut total_cost = 0.0;
        let mut total_tokens = 0u64;
        let mut total_prompts = 0u32;
        let mut total_tool_calls = 0u32;
        let mut cost_by_model: HashMap<String, f64> = HashMap::new();
        let mut cost_by_working_dir: HashMap<String, f64> = HashMap::new();

        for session in &sessions {
            total_cost += session.total_cost_usd;
            total_tokens += session.total_tokens;
            total_prompts += session.total_prompts;
            total_tool_calls += session.total_tool_calls;

            // Aggregate by model
            if let Some(model_usage) = &session.model_usage {
                for (model_name, breakdown) in model_usage {
                    *cost_by_model.entry(model_name.clone()).or_insert(0.0) += breakdown.cost_usd;
                }
            }

            // Aggregate by working directory
            *cost_by_working_dir.entry(session.working_dir.clone()).or_insert(0.0) += session.total_cost_usd;
        }

        Ok(CostSummary {
            total_cost_usd: total_cost,
            total_sessions: sessions.len(),
            total_tokens,
            total_prompts,
            total_tool_calls,
            session_records: sessions,
            cost_by_model,
            cost_by_working_dir,
        })
    }

    /// Get date range summary
    pub async fn get_date_range_summary(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<DateRangeCostSummary, String> {
        let sessions = self.get_sessions_by_date_range(start_date, end_date).await?;

        let mut daily_costs_map: HashMap<String, (f64, usize)> = HashMap::new();
        let mut total_cost = 0.0;

        for session in &sessions {
            total_cost += session.total_cost_usd;

            if let Ok(dt) = DateTime::parse_from_rfc3339(&session.started_at) {
                let date_key = dt.format("%Y-%m-%d").to_string();
                let entry = daily_costs_map.entry(date_key).or_insert((0.0, 0));
                entry.0 += session.total_cost_usd;
                entry.1 += 1;
            }
        }

        let mut daily_costs: Vec<DailyCost> = daily_costs_map
            .into_iter()
            .map(|(date, (cost, count))| DailyCost {
                date,
                cost_usd: cost,
                session_count: count,
            })
            .collect();

        daily_costs.sort_by(|a, b| a.date.cmp(&b.date));

        Ok(DateRangeCostSummary {
            start_date: start_date.map(|dt| dt.to_rfc3339()),
            end_date: end_date.map(|dt| dt.to_rfc3339()),
            total_cost_usd: total_cost,
            session_count: sessions.len(),
            daily_costs,
        })
    }

    /// Get current month cost
    pub async fn get_current_month_cost(&self) -> Result<f64, String> {
        let now = Utc::now();
        let start_of_month = now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let start_date = DateTime::<Utc>::from_naive_utc_and_offset(start_of_month, Utc);

        let summary = self.get_date_range_summary(Some(start_date), None).await?;
        Ok(summary.total_cost_usd)
    }

    /// Get today's cost
    pub async fn get_today_cost(&self) -> Result<f64, String> {
        let now = Utc::now();
        let start_of_day = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let start_date = DateTime::<Utc>::from_naive_utc_and_offset(start_of_day, Utc);

        let summary = self.get_date_range_summary(Some(start_date), None).await?;
        Ok(summary.total_cost_usd)
    }

    /// Clear all history
    pub async fn clear_history(&self) -> Result<(), String> {
        let db = self.db.lock().await;

        db.execute("DELETE FROM cost_sessions", [])
            .map_err(|e| format!("Failed to clear history: {}", e))?;

        Ok(())
    }

    /// Get database statistics
    pub async fn get_database_stats(&self) -> Result<DatabaseStats, String> {
        let db = self.db.lock().await;

        // Get database file size
        let db_size_bytes = fs::metadata(&self.db_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Get total sessions
        let total_sessions: i64 = db
            .query_row("SELECT COUNT(*) FROM cost_sessions", [], |row| row.get(0))
            .map_err(|e| format!("Failed to count sessions: {}", e))?;

        // Get total cost
        let total_cost: f64 = db
            .query_row("SELECT COALESCE(SUM(total_cost_usd), 0.0) FROM cost_sessions", [], |row| row.get(0))
            .map_err(|e| format!("Failed to sum costs: {}", e))?;

        Ok(DatabaseStats {
            db_size_bytes,
            total_sessions: total_sessions as usize,
            total_cost_usd: total_cost,
        })
    }

    /// Import sessions from JSON (for migration)
    pub async fn import_sessions(&self, sessions: Vec<SessionCostRecord>) -> Result<usize, String> {
        let mut imported = 0;
        for session in sessions {
            self.record_session(session).await?;
            imported += 1;
        }
        Ok(imported)
    }

    fn row_to_session(&self, row: &rusqlite::Row) -> SqliteResult<SessionCostRecord> {
        let model_usage_str: Option<String> = row.get(10)?;
        let model_usage = model_usage_str
            .and_then(|s| serde_json::from_str(&s).ok());

        Ok(SessionCostRecord {
            id: Some(row.get(0)?),
            session_id: row.get(1)?,
            agent_id: row.get(2)?,
            working_dir: row.get(3)?,
            started_at: row.get(4)?,
            ended_at: row.get(5)?,
            total_cost_usd: row.get(6)?,
            total_tokens: row.get::<_, i64>(7)? as u64,
            total_prompts: row.get::<_, i64>(8)? as u32,
            total_tool_calls: row.get::<_, i64>(9)? as u32,
            model_usage,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub db_size_bytes: u64,
    pub total_sessions: usize,
    pub total_cost_usd: f64,
}
