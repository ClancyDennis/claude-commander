use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCostRecord {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CostDatabase {
    pub sessions: Vec<SessionCostRecord>,
    pub version: String,
}

pub struct CostTracker {
    data_file: PathBuf,
    database: Arc<Mutex<CostDatabase>>,
}

impl CostTracker {
    pub fn new(data_dir: Option<PathBuf>) -> Result<Self, String> {
        let data_file = if let Some(dir) = data_dir {
            fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
            dir.join("cost_history.json")
        } else {
            // Use default location in user's home directory
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .map_err(|_| "Could not determine home directory")?;
            let data_dir = PathBuf::from(home).join(".grove_agent_manager");
            fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
            data_dir.join("cost_history.json")
        };

        let database = if data_file.exists() {
            let content = fs::read_to_string(&data_file)
                .map_err(|e| format!("Failed to read cost database: {}", e))?;
            serde_json::from_str(&content)
                .unwrap_or_else(|_| CostDatabase {
                    sessions: Vec::new(),
                    version: "1.0".to_string(),
                })
        } else {
            CostDatabase {
                sessions: Vec::new(),
                version: "1.0".to_string(),
            }
        };

        Ok(Self {
            data_file,
            database: Arc::new(Mutex::new(database)),
        })
    }

    pub async fn record_session(&self, record: SessionCostRecord) -> Result<(), String> {
        let mut db = self.database.lock().await;

        // Check if session already exists and update it, otherwise add new
        if let Some(existing) = db.sessions.iter_mut().find(|s| s.session_id == record.session_id) {
            *existing = record;
        } else {
            db.sessions.push(record);
        }

        self.save_database(&db).await?;
        Ok(())
    }

    pub async fn get_all_sessions(&self) -> Vec<SessionCostRecord> {
        let db = self.database.lock().await;
        db.sessions.clone()
    }

    pub async fn get_sessions_by_date_range(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Vec<SessionCostRecord> {
        let db = self.database.lock().await;

        db.sessions
            .iter()
            .filter(|session| {
                let session_date = DateTime::parse_from_rfc3339(&session.started_at)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok();

                if let Some(date) = session_date {
                    let after_start = start_date.map_or(true, |start| date >= start);
                    let before_end = end_date.map_or(true, |end| date <= end);
                    after_start && before_end
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    pub async fn get_cost_summary(&self) -> CostSummary {
        let db = self.database.lock().await;

        let mut total_cost = 0.0;
        let mut total_tokens = 0u64;
        let mut total_prompts = 0u32;
        let mut total_tool_calls = 0u32;
        let mut cost_by_model: HashMap<String, f64> = HashMap::new();
        let mut cost_by_working_dir: HashMap<String, f64> = HashMap::new();

        for session in &db.sessions {
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

        CostSummary {
            total_cost_usd: total_cost,
            total_sessions: db.sessions.len(),
            total_tokens,
            total_prompts,
            total_tool_calls,
            session_records: db.sessions.clone(),
            cost_by_model,
            cost_by_working_dir,
        }
    }

    pub async fn get_date_range_summary(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> DateRangeCostSummary {
        let sessions = self.get_sessions_by_date_range(start_date, end_date).await;

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

        DateRangeCostSummary {
            start_date: start_date.map(|dt| dt.to_rfc3339()),
            end_date: end_date.map(|dt| dt.to_rfc3339()),
            total_cost_usd: total_cost,
            session_count: sessions.len(),
            daily_costs,
        }
    }

    pub async fn get_current_month_cost(&self) -> f64 {
        let now = Utc::now();
        let start_of_month = now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let start_date = DateTime::<Utc>::from_naive_utc_and_offset(start_of_month, Utc);

        let summary = self.get_date_range_summary(Some(start_date), None).await;
        summary.total_cost_usd
    }

    pub async fn get_today_cost(&self) -> f64 {
        let now = Utc::now();
        let start_of_day = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let start_date = DateTime::<Utc>::from_naive_utc_and_offset(start_of_day, Utc);

        let summary = self.get_date_range_summary(Some(start_date), None).await;
        summary.total_cost_usd
    }

    pub async fn clear_history(&self) -> Result<(), String> {
        let mut db = self.database.lock().await;
        db.sessions.clear();
        self.save_database(&db).await?;
        Ok(())
    }

    async fn save_database(&self, db: &CostDatabase) -> Result<(), String> {
        let content = serde_json::to_string_pretty(db)
            .map_err(|e| format!("Failed to serialize database: {}", e))?;
        fs::write(&self.data_file, content)
            .map_err(|e| format!("Failed to write database: {}", e))?;
        Ok(())
    }
}
