// Agent runs database module
//
// Manages persistence of agent run records including statistics,
// cost tracking, and prompt history.
//
// Submodules:
// - crud.rs: Create/read/update/delete operations for runs and prompts
// - queries.rs: Complex queries and statistics
// - cost.rs: Cost aggregation and reporting
// - orchestrator_events.rs: Orchestrator event persistence
// - models.rs: Data structures
// - schema.rs: Database schema and migrations

mod cost;
mod crud;
mod models;
mod orchestrator_events;
mod queries;
mod schema;

use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub use models::{
    AgentOutputRecord, AgentRun, CostSummary, DailyCost, DatabaseStats, DateRangeCostSummary,
    EventQueryFilters, ModelCostBreakdown, OrchestratorDecisionRecord,
    OrchestratorStateChangeRecord, OrchestratorToolCallRecord, PipelineHistoryBundle,
    RunQueryFilters, RunStats, RunStatus, SessionCostRecord,
};

use cost::CostOperations;
use crud::CrudOperations;
use orchestrator_events::OrchestratorEventOps;
use queries::QueryOperations;

/// Main database interface for agent runs
///
/// This struct coordinates access to the underlying SQLite database
/// and delegates operations to specialized submodules.
pub struct AgentRunsDB {
    db: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl AgentRunsDB {
    /// Create a new AgentRunsDB instance, initializing the schema if needed
    pub fn new(db_path: PathBuf) -> SqliteResult<Self> {
        let conn = Connection::open(&db_path)?;

        // Initialize all database tables and run migrations
        schema::initialize_schema(&conn)?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    // ========================================================================
    // CRUD Operations - delegated to CrudOperations
    // ========================================================================

    /// Create a new run record when an agent starts
    pub async fn create_run(&self, run: &AgentRun) -> SqliteResult<i64> {
        CrudOperations::new(&self.db).create_run(run).await
    }

    /// Update an existing run record
    pub async fn update_run(&self, run: &AgentRun) -> SqliteResult<()> {
        CrudOperations::new(&self.db).update_run(run).await
    }

    /// Get a specific run by agent_id
    pub async fn get_run(&self, agent_id: &str) -> SqliteResult<Option<AgentRun>> {
        CrudOperations::new(&self.db).get_run(agent_id).await
    }

    /// Query runs with filters
    pub async fn query_runs(&self, filters: RunQueryFilters) -> SqliteResult<Vec<AgentRun>> {
        CrudOperations::new(&self.db).query_runs(filters).await
    }

    /// Get all crashed runs that can be resumed
    pub async fn get_resumable_runs(&self) -> SqliteResult<Vec<AgentRun>> {
        CrudOperations::new(&self.db).get_resumable_runs().await
    }

    /// Record a prompt sent to an agent
    pub async fn record_prompt(
        &self,
        agent_id: &str,
        prompt: &str,
        timestamp: i64,
    ) -> SqliteResult<()> {
        CrudOperations::new(&self.db)
            .record_prompt(agent_id, prompt, timestamp)
            .await
    }

    /// Get prompt history for an agent
    pub async fn get_prompts(&self, agent_id: &str) -> SqliteResult<Vec<(String, i64)>> {
        CrudOperations::new(&self.db).get_prompts(agent_id).await
    }

    /// Clean up old runs (older than days_to_keep)
    pub async fn cleanup_old_runs(&self, days_to_keep: i64) -> SqliteResult<usize> {
        CrudOperations::new(&self.db)
            .cleanup_old_runs(days_to_keep)
            .await
    }

    /// Mark all "running" or "waiting_input" agents as "crashed" - called on app startup
    pub async fn reconcile_stale_runs(&self) -> SqliteResult<usize> {
        CrudOperations::new(&self.db).reconcile_stale_runs().await
    }

    // ========================================================================
    // Query/Statistics Operations - delegated to QueryOperations
    // ========================================================================

    /// Get statistics about all runs
    pub async fn get_stats(&self) -> SqliteResult<RunStats> {
        QueryOperations::new(&self.db, &self.db_path)
            .get_stats()
            .await
    }

    /// Get database statistics including size and record counts
    pub async fn get_database_stats(&self) -> SqliteResult<DatabaseStats> {
        QueryOperations::new(&self.db, &self.db_path)
            .get_database_stats()
            .await
    }

    // ========================================================================
    // Cost Operations - delegated to CostOperations
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
