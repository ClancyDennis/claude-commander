use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

// Re-export types from cost_tracker_db
pub use crate::cost_tracker_db::{
    SessionCostRecord, ModelCostBreakdown, CostSummary,
    DateRangeCostSummary, CostTrackerDB, DatabaseStats
};

/// Legacy JSON database structure for migration
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyCostDatabase {
    pub sessions: Vec<SessionCostRecord>,
    pub version: String,
}

/// Wrapper around CostTrackerDB that provides the same interface
/// and handles migration from JSON to SQLite
pub struct CostTracker {
    db: Arc<CostTrackerDB>,
    json_file: Option<PathBuf>, // Keep track of JSON file for migration
}

impl CostTracker {
    pub fn new(data_dir: Option<PathBuf>) -> Result<Self, String> {
        let (db_path, json_file) = if let Some(dir) = data_dir.clone() {
            fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
            (dir.join("cost_history.db"), Some(dir.join("cost_history.json")))
        } else {
            // Use default location - same as the rest of the app
            let db_path = dirs::data_local_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
                .map(|d| d.join("grove").join("cost_history.db"))
                .unwrap_or_else(|| std::env::temp_dir().join("grove_cost_history.db"));

            let json_file = dirs::data_local_dir()
                .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
                .map(|d| d.join("grove").join("cost_history.json"))
                .or_else(|| {
                    // Also check legacy location
                    std::env::var("HOME")
                        .or_else(|_| std::env::var("USERPROFILE"))
                        .ok()
                        .map(|home| PathBuf::from(home).join(".grove_agent_manager").join("cost_history.json"))
                });

            (db_path, json_file)
        };

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

        // Create or open SQLite database
        let db = CostTrackerDB::new(db_path.clone())
            .map_err(|e| format!("Failed to open cost database: {}", e))?;

        let tracker = Self {
            db: Arc::new(db),
            json_file,
        };

        // Attempt to migrate from JSON if it exists
        if let Some(ref json_path) = tracker.json_file {
            if json_path.exists() {
                println!("Found legacy JSON cost data at {:?}, attempting migration...", json_path);
                match tracker.migrate_from_json_sync(json_path) {
                    Ok(count) => {
                        println!("✓ Successfully migrated {} cost records to SQLite", count);
                        // Backup the JSON file
                        let backup_path = json_path.with_extension("json.backup");
                        if let Err(e) = fs::rename(json_path, &backup_path) {
                            eprintln!("⚠ Warning: Could not backup JSON file: {}", e);
                        } else {
                            println!("✓ Backed up JSON file to {:?}", backup_path);
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠ Warning: Failed to migrate JSON cost data: {}", e);
                    }
                }
            }
        }

        Ok(tracker)
    }

    /// Synchronous migration helper (used during initialization)
    fn migrate_from_json_sync(&self, json_path: &PathBuf) -> Result<usize, String> {
        let content = fs::read_to_string(json_path)
            .map_err(|e| format!("Failed to read JSON file: {}", e))?;

        let legacy_db: LegacyCostDatabase = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Use tokio runtime to run async migration
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("Failed to create runtime: {}", e))?;

        rt.block_on(async {
            self.db.import_sessions(legacy_db.sessions).await
        })
    }

    pub async fn record_session(&self, record: SessionCostRecord) -> Result<(), String> {
        self.db.record_session(record).await
    }

    pub async fn get_all_sessions(&self) -> Result<Vec<SessionCostRecord>, String> {
        self.db.get_all_sessions().await
    }

    pub async fn get_sessions_by_date_range(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<SessionCostRecord>, String> {
        self.db.get_sessions_by_date_range(start_date, end_date).await
    }

    pub async fn get_cost_summary(&self) -> Result<CostSummary, String> {
        self.db.get_cost_summary().await
    }

    pub async fn get_date_range_summary(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<DateRangeCostSummary, String> {
        self.db.get_date_range_summary(start_date, end_date).await
    }

    pub async fn get_current_month_cost(&self) -> Result<f64, String> {
        self.db.get_current_month_cost().await
    }

    pub async fn get_today_cost(&self) -> Result<f64, String> {
        self.db.get_today_cost().await
    }

    pub async fn clear_history(&self) -> Result<(), String> {
        self.db.clear_history().await
    }

    pub async fn get_database_stats(&self) -> Result<DatabaseStats, String> {
        self.db.get_database_stats().await
    }
}
