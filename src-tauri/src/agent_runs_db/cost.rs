// Cost-related operations for agent runs database

use chrono::{DateTime, Datelike, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::models::{
    CostSummary, DailyCost, DateRangeCostSummary, SessionCostRecord,
};

/// Cost operations extension for AgentRunsDB
pub struct CostOperations<'a> {
    db: &'a Arc<Mutex<Connection>>,
}

impl<'a> CostOperations<'a> {
    pub fn new(db: &'a Arc<Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Get cost summary aggregated by working directory
    pub async fn get_cost_by_working_dir(&self) -> SqliteResult<Vec<(String, f64)>> {
        let db = self.db.lock().await;

        let mut stmt = db.prepare(
            "SELECT working_dir, COALESCE(SUM(total_cost_usd), 0.0)
             FROM agent_runs
             WHERE total_cost_usd IS NOT NULL
             GROUP BY working_dir
             ORDER BY SUM(total_cost_usd) DESC",
        )?;

        let results = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut cost_by_dir = Vec::new();
        for result in results {
            cost_by_dir.push(result?);
        }

        Ok(cost_by_dir)
    }

    /// Get daily cost breakdown
    pub async fn get_daily_costs(&self, days: i64) -> SqliteResult<Vec<DailyCost>> {
        let db = self.db.lock().await;

        let cutoff_timestamp =
            chrono::Utc::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);

        let mut stmt = db.prepare(
            "SELECT DATE(started_at / 1000, 'unixepoch') as date,
                    COALESCE(SUM(total_cost_usd), 0.0) as cost,
                    COUNT(*) as count
             FROM agent_runs
             WHERE started_at >= ?1 AND total_cost_usd IS NOT NULL
             GROUP BY date
             ORDER BY date DESC",
        )?;

        let results = stmt.query_map(params![cutoff_timestamp], |row| {
            Ok(DailyCost {
                date: row.get(0)?,
                cost_usd: row.get(1)?,
                session_count: row.get(2)?,
            })
        })?;

        let mut daily_costs = Vec::new();
        for result in results {
            daily_costs.push(result?);
        }

        Ok(daily_costs)
    }

    /// Get total cost for current month
    pub async fn get_current_month_cost(&self) -> Result<f64, String> {
        let db = self.db.lock().await;

        let now = chrono::Utc::now();
        let naive_now = now.naive_utc();
        let start_of_month = chrono::NaiveDate::from_ymd_opt(naive_now.year(), naive_now.month(), 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis();

        let cost: f64 = db
            .query_row(
                "SELECT COALESCE(SUM(total_cost_usd), 0.0)
             FROM agent_runs
             WHERE started_at >= ?1 AND total_cost_usd IS NOT NULL",
                params![start_of_month],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to query current month cost: {}", e))?;

        Ok(cost)
    }

    /// Get total cost for today
    pub async fn get_today_cost(&self) -> Result<f64, String> {
        let db = self.db.lock().await;

        let today_start = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis();

        let cost: f64 = db
            .query_row(
                "SELECT COALESCE(SUM(total_cost_usd), 0.0)
             FROM agent_runs
             WHERE started_at >= ?1 AND total_cost_usd IS NOT NULL",
                params![today_start],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to query today cost: {}", e))?;

        Ok(cost)
    }

    /// Get cost summary - aggregated from all agent runs
    pub async fn get_cost_summary(&self) -> Result<CostSummary, String> {
        let db = self.db.lock().await;

        let mut stmt = db
            .prepare(
                "SELECT agent_id, session_id, working_dir, started_at, ended_at,
                        total_prompts, total_tool_calls, total_tokens_used, total_cost_usd, model_usage
                 FROM agent_runs
                 WHERE total_cost_usd IS NOT NULL
                 ORDER BY started_at DESC",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let sessions = stmt
            .query_map([], |row| {
                let model_usage_str: Option<String> = row.get(9)?;
                let model_usage = model_usage_str.and_then(|s| serde_json::from_str(&s).ok());

                Ok(SessionCostRecord {
                    agent_id: row.get(0)?,
                    session_id: row
                        .get::<_, Option<String>>(1)?
                        .unwrap_or_else(|| format!("session_{}", row.get::<_, String>(0).unwrap_or_default())),
                    working_dir: row.get(2)?,
                    started_at: {
                        let ts: i64 = row.get(3)?;
                        DateTime::<Utc>::from_timestamp_millis(ts)
                            .map(|dt| dt.to_rfc3339())
                            .unwrap_or_else(|| "unknown".to_string())
                    },
                    ended_at: row
                        .get::<_, Option<i64>>(4)?
                        .and_then(|ts| DateTime::<Utc>::from_timestamp_millis(ts))
                        .map(|dt| dt.to_rfc3339()),
                    total_prompts: row.get::<_, i64>(5)? as u32,
                    total_tool_calls: row.get::<_, i64>(6)? as u32,
                    total_tokens: row.get::<_, Option<i64>>(7)?.unwrap_or(0) as u64,
                    total_cost_usd: row.get(8)?,
                    model_usage,
                })
            })
            .map_err(|e| format!("Failed to query sessions: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect sessions: {}", e))?;

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
            *cost_by_working_dir
                .entry(session.working_dir.clone())
                .or_insert(0.0) += session.total_cost_usd;
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

    /// Get date range cost summary
    pub async fn get_date_range_summary(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<DateRangeCostSummary, String> {
        let db = self.db.lock().await;

        let mut query = String::from(
            "SELECT started_at, total_cost_usd
             FROM agent_runs
             WHERE total_cost_usd IS NOT NULL",
        );

        let mut params_vec: Vec<i64> = Vec::new();

        if let Some(start) = start_date {
            query.push_str(" AND started_at >= ?");
            params_vec.push(start.timestamp_millis());
        }

        if let Some(end) = end_date {
            query.push_str(" AND started_at <= ?");
            params_vec.push(end.timestamp_millis());
        }

        query.push_str(" ORDER BY started_at");

        let mut stmt = db
            .prepare(&query)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let param_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

        let results = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
            })
            .map_err(|e| format!("Failed to query: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect: {}", e))?;

        let mut daily_costs_map: HashMap<String, (f64, usize)> = HashMap::new();
        let mut total_cost = 0.0;

        for (ts, cost) in results {
            total_cost += cost;

            if let Some(dt) = DateTime::<Utc>::from_timestamp_millis(ts) {
                let date_key = dt.format("%Y-%m-%d").to_string();
                let entry = daily_costs_map.entry(date_key).or_insert((0.0, 0));
                entry.0 += cost;
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
            start_date: start_date.map(|d| d.to_rfc3339()),
            end_date: end_date.map(|d| d.to_rfc3339()),
            total_cost_usd: total_cost,
            session_count: daily_costs.iter().map(|d| d.session_count).sum(),
            daily_costs,
        })
    }

    /// Clear all cost history
    pub async fn clear_cost_history(&self) -> Result<(), String> {
        let db = self.db.lock().await;

        db.execute(
            "UPDATE agent_runs SET total_cost_usd = NULL, total_tokens_used = NULL, model_usage = NULL",
            [],
        )
        .map_err(|e| format!("Failed to clear cost history: {}", e))?;

        Ok(())
    }
}
