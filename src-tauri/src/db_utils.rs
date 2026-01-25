// Database utilities module
//
// Provides helper functions and abstractions for common database operations,
// reducing boilerplate for lock acquisition and query building.

use rusqlite::{Connection, Result as SqliteResult, ToSql};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Helper trait for executing database operations with automatic lock management.
///
/// This trait abstracts the common pattern of:
/// ```ignore
/// let db = self.db.lock().await;
/// // ... do something with db ...
/// ```
///
/// Into a cleaner:
/// ```ignore
/// self.with_db(|db| {
///     // ... do something with db ...
/// }).await
/// ```
#[async_trait::async_trait]
pub trait DatabaseOps {
    /// Execute a closure with a locked database connection.
    ///
    /// # Example
    /// ```ignore
    /// let result = self.with_db(|db| {
    ///     db.execute("INSERT INTO ...", params![...])?;
    ///     Ok(db.last_insert_rowid())
    /// }).await?;
    /// ```
    async fn with_db<F, T>(&self, f: F) -> SqliteResult<T>
    where
        F: FnOnce(&Connection) -> SqliteResult<T> + Send,
        T: Send;

    /// Execute a closure with a mutable database connection.
    /// Use this when you need mutable access (e.g., for transactions).
    async fn with_db_mut<F, T>(&self, f: F) -> SqliteResult<T>
    where
        F: FnOnce(&mut Connection) -> SqliteResult<T> + Send,
        T: Send;
}

/// Implementation of DatabaseOps for Arc<Mutex<Connection>>
#[async_trait::async_trait]
impl DatabaseOps for Arc<Mutex<Connection>> {
    async fn with_db<F, T>(&self, f: F) -> SqliteResult<T>
    where
        F: FnOnce(&Connection) -> SqliteResult<T> + Send,
        T: Send,
    {
        let db = self.lock().await;
        f(&db)
    }

    async fn with_db_mut<F, T>(&self, f: F) -> SqliteResult<T>
    where
        F: FnOnce(&mut Connection) -> SqliteResult<T> + Send,
        T: Send,
    {
        let mut db = self.lock().await;
        f(&mut db)
    }
}

/// A simple query builder for constructing dynamic SQL queries.
///
/// This helps reduce code duplication for queries with optional filters.
///
/// # Example
/// ```ignore
/// let mut builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1");
/// if let Some(name) = name_filter {
///     builder.add_condition("name = ?", name);
/// }
/// if let Some(age) = age_filter {
///     builder.add_condition("age >= ?", age);
/// }
/// builder.add_order_by("created_at DESC");
/// builder.add_limit(10);
///
/// let (query, params) = builder.build();
/// ```
pub struct QueryBuilder {
    base_query: String,
    conditions: Vec<String>,
    params: Vec<Box<dyn ToSql + Send>>,
    order_by: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl QueryBuilder {
    /// Create a new query builder with the base SELECT clause.
    pub fn new(base_query: &str) -> Self {
        Self {
            base_query: base_query.to_string(),
            conditions: Vec::new(),
            params: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    /// Add a condition with a parameter.
    pub fn add_condition<T: ToSql + Send + 'static>(&mut self, condition: &str, param: T) {
        self.conditions.push(condition.to_string());
        self.params.push(Box::new(param));
    }

    /// Add a condition without a parameter (e.g., "status IS NOT NULL").
    pub fn add_raw_condition(&mut self, condition: &str) {
        self.conditions.push(condition.to_string());
    }

    /// Add an ORDER BY clause.
    pub fn add_order_by(&mut self, order: &str) {
        self.order_by = Some(order.to_string());
    }

    /// Add a LIMIT clause.
    pub fn add_limit(&mut self, limit: usize) {
        self.limit = Some(limit);
    }

    /// Add an OFFSET clause.
    pub fn add_offset(&mut self, offset: usize) {
        self.offset = Some(offset);
    }

    /// Build the final query string and parameters.
    pub fn build(self) -> (String, Vec<Box<dyn ToSql + Send>>) {
        let mut query = self.base_query;

        for condition in &self.conditions {
            query.push_str(" AND ");
            query.push_str(condition);
        }

        if let Some(order) = &self.order_by {
            query.push_str(" ORDER BY ");
            query.push_str(order);
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        (query, self.params)
    }

    /// Convert params to references for use with rusqlite.
    pub fn params_as_refs(params: &[Box<dyn ToSql + Send>]) -> Vec<&dyn ToSql> {
        params.iter().map(|p| p.as_ref() as &dyn ToSql).collect()
    }
}

/// Common column lists for frequently queried tables.
/// These constants help ensure consistency and reduce typos.
pub mod columns {
    /// Column list for agent_runs table queries.
    pub const AGENT_RUNS: &str =
        "id, agent_id, session_id, working_dir, github_url, github_context,
        source, status, started_at, ended_at, last_activity,
        initial_prompt, error_message, pipeline_id, total_prompts, total_tool_calls,
        total_output_bytes, total_tokens_used, total_cost_usd, model_usage,
        can_resume, resume_data";

    /// Column list for agent_prompts table queries.
    pub const AGENT_PROMPTS: &str = "id, agent_id, timestamp, prompt";

    /// Column list for log entries.
    pub const LOG_ENTRIES: &str = "id, timestamp, level, source, message, metadata";
}

/// Execute a database operation within a transaction.
///
/// Automatically commits on success or rolls back on error.
pub fn with_transaction<F, T>(conn: &mut Connection, f: F) -> SqliteResult<T>
where
    F: FnOnce(&rusqlite::Transaction) -> SqliteResult<T>,
{
    let tx = conn.transaction()?;
    let result = f(&tx)?;
    tx.commit()?;
    Ok(result)
}

/// Execute multiple statements in a batch.
///
/// Useful for running multiple INSERTs or UPDATEs efficiently.
pub fn execute_batch(conn: &Connection, statements: &[&str]) -> SqliteResult<()> {
    for stmt in statements {
        conn.execute(stmt, [])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_basic() {
        let mut builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1");
        builder.add_condition("name = ?", "test".to_string());
        builder.add_condition("age >= ?", 18);
        builder.add_order_by("created_at DESC");
        builder.add_limit(10);

        let (query, params) = builder.build();

        assert!(query.contains("name = ?"));
        assert!(query.contains("age >= ?"));
        assert!(query.contains("ORDER BY created_at DESC"));
        assert!(query.contains("LIMIT 10"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_query_builder_empty() {
        let builder = QueryBuilder::new("SELECT * FROM users");
        let (query, params) = builder.build();

        assert_eq!(query, "SELECT * FROM users");
        assert!(params.is_empty());
    }

    #[test]
    fn test_query_builder_with_offset() {
        let mut builder = QueryBuilder::new("SELECT * FROM items WHERE 1=1");
        builder.add_limit(20);
        builder.add_offset(40);

        let (query, _) = builder.build();

        assert!(query.contains("LIMIT 20"));
        assert!(query.contains("OFFSET 40"));
    }
}
