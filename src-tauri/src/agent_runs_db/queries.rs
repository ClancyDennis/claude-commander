// Complex query operations and statistics for agent runs
//
// Provides aggregation queries, statistics, and reporting functionality.

use rusqlite::{Connection, Result as SqliteResult};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::db_utils::DatabaseOps;

use super::models::{format_bytes, DatabaseStats, RunStats};

/// Query and statistics operations for agent runs
pub struct QueryOperations<'a> {
    db: &'a Arc<Mutex<Connection>>,
    db_path: &'a PathBuf,
}

impl<'a> QueryOperations<'a> {
    pub fn new(db: &'a Arc<Mutex<Connection>>, db_path: &'a PathBuf) -> Self {
        Self { db, db_path }
    }

    /// Get statistics about all runs
    pub async fn get_stats(&self) -> SqliteResult<RunStats> {
        self.db
            .with_db(|db| {
                let total: i64 =
                    db.query_row("SELECT COUNT(*) FROM agent_runs", [], |row| row.get(0))?;

                let by_status: Vec<(String, i64)> = {
                    let mut stmt =
                        db.prepare("SELECT status, COUNT(*) FROM agent_runs GROUP BY status")?;
                    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
                    rows.collect::<SqliteResult<Vec<_>>>()?
                };

                let by_source: Vec<(String, i64)> = {
                    let mut stmt =
                        db.prepare("SELECT source, COUNT(*) FROM agent_runs GROUP BY source")?;
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
            })
            .await
    }

    /// Get database statistics including size and record counts
    pub async fn get_database_stats(&self) -> SqliteResult<DatabaseStats> {
        // Get database file size outside of the db closure (fs operation)
        let db_size_bytes = fs::metadata(self.db_path).map(|m| m.len()).unwrap_or(0);
        let db_size_formatted = format_bytes(db_size_bytes);

        self.db
            .with_db(move |db| {
                // Get total runs
                let total_runs: i64 =
                    db.query_row("SELECT COUNT(*) FROM agent_runs", [], |row| row.get(0))?;

                // Get total prompts
                let total_prompts: i64 =
                    db.query_row("SELECT COUNT(*) FROM agent_prompts", [], |row| row.get(0))?;

                // Get runs by status
                let runs_by_status: Vec<(String, i64)> = {
                    let mut stmt =
                        db.prepare("SELECT status, COUNT(*) FROM agent_runs GROUP BY status")?;
                    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
                    rows.collect::<SqliteResult<Vec<_>>>()?
                };

                // Get runs by source
                let runs_by_source: Vec<(String, i64)> = {
                    let mut stmt =
                        db.prepare("SELECT source, COUNT(*) FROM agent_runs GROUP BY source")?;
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
            })
            .await
    }
}
