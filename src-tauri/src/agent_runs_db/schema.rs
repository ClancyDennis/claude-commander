// Database schema creation and migrations
//
// Handles all SQLite table creation, index creation, and schema migrations
// for the agent runs database.

use rusqlite::{Connection, Result as SqliteResult};

/// Create the main agent_runs table
pub fn create_agent_runs_table(conn: &Connection) -> SqliteResult<()> {
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
    Ok(())
}

/// Create indexes for common queries on agent_runs
pub fn create_agent_runs_indexes(conn: &Connection) -> SqliteResult<()> {
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
    Ok(())
}

/// Create the agent_prompts table for tracking conversation history
pub fn create_prompts_table(conn: &Connection) -> SqliteResult<()> {
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

    Ok(())
}

/// Create orchestrator event tables for hybrid persistence
pub fn create_orchestrator_tables(conn: &Connection) -> SqliteResult<()> {
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

    Ok(())
}

/// Run schema migrations to add new columns
pub fn run_migrations(conn: &Connection) -> SqliteResult<()> {
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(agent_runs)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    // Migration: Add model_usage column if it doesn't exist
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

    Ok(())
}

/// Initialize all database tables and indexes
pub fn initialize_schema(conn: &Connection) -> SqliteResult<()> {
    create_agent_runs_table(conn)?;
    create_agent_runs_indexes(conn)?;
    create_prompts_table(conn)?;
    run_migrations(conn)?;
    create_orchestrator_tables(conn)?;
    Ok(())
}
