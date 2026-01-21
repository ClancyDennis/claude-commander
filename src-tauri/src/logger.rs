use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl LogLevel {
    fn to_string(&self) -> &'static str {
        match self {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Option<i64>,
    pub timestamp: i64, // Unix timestamp in milliseconds
    pub level: LogLevel,
    pub component: String, // e.g., "agent_manager", "meta_agent", "tool_registry"
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub message: String,
    pub metadata: Option<String>, // JSON string for additional context
}

pub struct Logger {
    db: Arc<Mutex<Connection>>,
}

impl Logger {
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        let conn = Connection::open(db_path)?;

        // Create logs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                level TEXT NOT NULL,
                component TEXT NOT NULL,
                agent_id TEXT,
                session_id TEXT,
                message TEXT NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_agent ON logs(agent_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_logs_component ON logs(component)",
            [],
        )?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn log(&self, entry: LogEntry) -> SqliteResult<i64> {
        let db = self.db.lock().await;

        db.execute(
            "INSERT INTO logs (timestamp, level, component, agent_id, session_id, message, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.timestamp,
                entry.level.to_string(),
                entry.component,
                entry.agent_id,
                entry.session_id,
                entry.message,
                entry.metadata,
            ],
        )?;

        Ok(db.last_insert_rowid())
    }

    pub async fn debug(&self, component: &str, message: &str, agent_id: Option<String>, metadata: Option<String>) -> SqliteResult<i64> {
        self.log(LogEntry {
            id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            level: LogLevel::Debug,
            component: component.to_string(),
            agent_id,
            session_id: None,
            message: message.to_string(),
            metadata,
        }).await
    }

    pub async fn info(&self, component: &str, message: &str, agent_id: Option<String>, metadata: Option<String>) -> SqliteResult<i64> {
        self.log(LogEntry {
            id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            level: LogLevel::Info,
            component: component.to_string(),
            agent_id,
            session_id: None,
            message: message.to_string(),
            metadata,
        }).await
    }

    pub async fn warning(&self, component: &str, message: &str, agent_id: Option<String>, metadata: Option<String>) -> SqliteResult<i64> {
        self.log(LogEntry {
            id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            level: LogLevel::Warning,
            component: component.to_string(),
            agent_id,
            session_id: None,
            message: message.to_string(),
            metadata,
        }).await
    }

    pub async fn error(&self, component: &str, message: &str, agent_id: Option<String>, metadata: Option<String>) -> SqliteResult<i64> {
        self.log(LogEntry {
            id: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
            level: LogLevel::Error,
            component: component.to_string(),
            agent_id,
            session_id: None,
            message: message.to_string(),
            metadata,
        }).await
    }

    /// Query logs with filters
    pub async fn query(
        &self,
        level: Option<LogLevel>,
        component: Option<String>,
        agent_id: Option<String>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> SqliteResult<Vec<LogEntry>> {
        let db = self.db.lock().await;

        let mut query = "SELECT id, timestamp, level, component, agent_id, session_id, message, metadata FROM logs WHERE 1=1".to_string();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(lvl) = level {
            query.push_str(" AND level = ?");
            params.push(Box::new(lvl.to_string().to_string()));
        }

        if let Some(comp) = component {
            query.push_str(" AND component = ?");
            params.push(Box::new(comp));
        }

        if let Some(aid) = agent_id {
            query.push_str(" AND agent_id = ?");
            params.push(Box::new(aid));
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }

        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET {}", off));
        }

        let mut stmt = db.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let entries = stmt.query_map(param_refs.as_slice(), |row| {
            let level_str: String = row.get(2)?;
            let level = match level_str.as_str() {
                "debug" => LogLevel::Debug,
                "info" => LogLevel::Info,
                "warning" => LogLevel::Warning,
                "error" => LogLevel::Error,
                _ => LogLevel::Info,
            };

            Ok(LogEntry {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                level,
                component: row.get(3)?,
                agent_id: row.get(4)?,
                session_id: row.get(5)?,
                message: row.get(6)?,
                metadata: row.get(7)?,
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    /// Get recent logs (last 100 by default)
    pub async fn recent(&self, limit: usize) -> SqliteResult<Vec<LogEntry>> {
        self.query(None, None, None, Some(limit), None).await
    }

    /// Clear old logs (older than days_to_keep)
    pub async fn cleanup(&self, days_to_keep: i64) -> SqliteResult<usize> {
        let db = self.db.lock().await;
        let cutoff_timestamp = chrono::Utc::now().timestamp_millis() - (days_to_keep * 24 * 60 * 60 * 1000);

        db.execute(
            "DELETE FROM logs WHERE timestamp < ?1",
            params![cutoff_timestamp],
        )
    }

    /// Get log statistics
    pub async fn stats(&self) -> SqliteResult<LogStats> {
        let db = self.db.lock().await;

        let total: i64 = db.query_row(
            "SELECT COUNT(*) FROM logs",
            [],
            |row| row.get(0),
        )?;

        let by_level: Vec<(String, i64)> = {
            let mut stmt = db.prepare("SELECT level, COUNT(*) FROM logs GROUP BY level")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect::<SqliteResult<Vec<_>>>()?
        };

        let by_component: Vec<(String, i64)> = {
            let mut stmt = db.prepare("SELECT component, COUNT(*) FROM logs GROUP BY component ORDER BY COUNT(*) DESC LIMIT 10")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect::<SqliteResult<Vec<_>>>()?
        };

        Ok(LogStats {
            total_logs: total,
            by_level,
            by_component,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogStats {
    pub total_logs: i64,
    pub by_level: Vec<(String, i64)>,
    pub by_component: Vec<(String, i64)>,
}

// Helper macro for easy logging
#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $component:expr, $msg:expr) => {
        if let Some(logger) = $logger.as_ref() {
            let _ = logger.debug($component, $msg, None, None).await;
        }
    };
    ($logger:expr, $component:expr, $msg:expr, $agent_id:expr) => {
        if let Some(logger) = $logger.as_ref() {
            let _ = logger.debug($component, $msg, Some($agent_id.to_string()), None).await;
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $component:expr, $msg:expr) => {
        if let Some(logger) = $logger.as_ref() {
            let _ = logger.info($component, $msg, None, None).await;
        }
    };
    ($logger:expr, $component:expr, $msg:expr, $agent_id:expr) => {
        if let Some(logger) = $logger.as_ref() {
            let _ = logger.info($component, $msg, Some($agent_id.to_string()), None).await;
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $component:expr, $msg:expr) => {
        if let Some(logger) = $logger.as_ref() {
            let _ = logger.error($component, $msg, None, None).await;
        }
    };
    ($logger:expr, $component:expr, $msg:expr, $agent_id:expr) => {
        if let Some(logger) = $logger.as_ref() {
            let _ = logger.error($component, $msg, Some($agent_id.to_string()), None).await;
        }
    };
}
