use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use crate::agent_manager::AgentManager;

#[derive(Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub min_size: usize,              // Minimum agents to keep alive (default: 3)
    pub max_size: usize,              // Maximum agents allowed (default: 10)
    pub default_size: usize,          // Initial pool size (default: 5)
    pub auto_scale: bool,             // Enable auto-scaling (default: true)
    pub scale_up_threshold: f32,      // Trigger scale-up at X% utilization (default: 0.8)
    pub scale_down_threshold: f32,    // Trigger scale-down at X% utilization (default: 0.2)
    pub idle_timeout: Duration,       // Remove idle agents after X (default: 5 min)
    pub default_working_dir: String,  // Default dir for pool agents
    pub queue_timeout: Duration,      // Max wait time for agent (default: 30s)
    pub max_queue_size: usize,        // Max pending requests (default: 50)
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 3,
            max_size: 10,
            default_size: 5,
            auto_scale: true,
            scale_up_threshold: 0.8,
            scale_down_threshold: 0.2,
            idle_timeout: Duration::from_secs(300),
            default_working_dir: std::env::temp_dir().to_string_lossy().to_string(),
            queue_timeout: Duration::from_secs(30),
            max_queue_size: 50,
        }
    }
}

#[derive(Clone, Serialize)]
pub struct TaskAssignment {
    pub task_id: String,
    pub assigned_at: SystemTime,
    pub task_description: String,
}

#[derive(Clone, Serialize, Default)]
pub struct PoolStats {
    pub total_agents: usize,
    pub idle_agents: usize,
    pub busy_agents: usize,
    pub utilization: f32,              // busy / total
    pub tasks_completed: u64,
    pub average_task_time: f32,        // seconds
    pub uptime: Duration,
}

#[derive(Debug)]
pub enum PoolError {
    QueueFull,
    Timeout,
    ChannelClosed,
    SpawnFailed(String),
    AgentCrashed(String),
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::QueueFull => write!(f, "Request queue is full. Pool is overloaded."),
            PoolError::Timeout => write!(f, "Request timed out waiting for available agent."),
            PoolError::ChannelClosed => write!(f, "Internal channel closed unexpectedly."),
            PoolError::SpawnFailed(e) => write!(f, "Failed to spawn agent: {}", e),
            PoolError::AgentCrashed(id) => write!(f, "Agent {} crashed unexpectedly.", id),
        }
    }
}

impl std::error::Error for PoolError {}

pub struct AgentPool {
    config: PoolConfig,
    idle_agents: VecDeque<String>,              // Queue of available agent IDs
    busy_agents: HashMap<String, TaskAssignment>, // agent_id -> task info
    agent_manager: Arc<Mutex<AgentManager>>,
    stats: Arc<Mutex<PoolStats>>,
    auto_scaler: Option<tokio::task::JoinHandle<()>>,
    waiting_queue: VecDeque<tokio::sync::oneshot::Sender<String>>,
    start_time: Instant,
    app_handle: Option<tauri::AppHandle>,
}

impl AgentPool {
    /// Initialize pool with default_size agents
    pub async fn new(
        config: PoolConfig,
        agent_manager: Arc<Mutex<AgentManager>>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<Arc<Mutex<Self>>, String> {
        let pool = Arc::new(Mutex::new(Self {
            config: config.clone(),
            idle_agents: VecDeque::new(),
            busy_agents: HashMap::new(),
            agent_manager: agent_manager.clone(),
            stats: Arc::new(Mutex::new(PoolStats::default())),
            auto_scaler: None,
            waiting_queue: VecDeque::new(),
            start_time: Instant::now(),
            app_handle,
        }));

        // Spawn initial agents
        let pool_clone = pool.clone();
        for _ in 0..config.default_size {
            let mut pool_lock = pool_clone.lock().await;
            if let Err(e) = pool_lock.spawn_agent().await {
                eprintln!("Warning: Failed to spawn initial agent: {}", e);
            }
        }

        // Start auto-scaler
        if config.auto_scale {
            let pool_clone = pool.clone();
            let scaler_handle = tokio::spawn(Self::auto_scale_loop(pool_clone));
            pool.lock().await.auto_scaler = Some(scaler_handle);
        }

        Ok(pool)
    }

    /// Get an idle agent from the pool (or spawn if none available)
    pub async fn acquire_agent(&mut self) -> Result<String, PoolError> {
        // Try to get idle agent
        if let Some(agent_id) = self.idle_agents.pop_front() {
            self.update_stats().await;
            return Ok(agent_id);
        }

        // Try to scale up
        if self.total_agents() < self.config.max_size {
            return self.spawn_agent().await.map_err(|e| PoolError::SpawnFailed(e));
        }

        // Queue the request with timeout
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.waiting_queue.push_back(tx);

        if self.waiting_queue.len() > self.config.max_queue_size {
            return Err(PoolError::QueueFull);
        }

        let timeout_duration = self.config.queue_timeout;

        match tokio::time::timeout(timeout_duration, rx).await {
            Ok(Ok(agent_id)) => Ok(agent_id),
            Ok(Err(_)) => Err(PoolError::ChannelClosed),
            Err(_) => Err(PoolError::Timeout),
        }
    }

    /// Mark agent as busy with task
    pub async fn assign_task(&mut self, agent_id: String, task: TaskAssignment) {
        self.busy_agents.insert(agent_id, task);
        self.update_stats().await;
    }

    /// Release agent back to idle pool
    pub async fn release_agent(&mut self, agent_id: String) {
        self.busy_agents.remove(&agent_id);

        // First check if anyone is waiting
        if let Some(tx) = self.waiting_queue.pop_front() {
            let _ = tx.send(agent_id); // Assign directly to waiter
        } else {
            self.idle_agents.push_back(agent_id);
        }

        self.stats.lock().await.tasks_completed += 1;
        self.update_stats().await;
    }

    /// Spawn a new agent in the pool
    async fn spawn_agent(&mut self) -> Result<String, String> {
        let working_dir = self.config.default_working_dir.clone();
        let manager = self.agent_manager.lock().await;

        let agent_id = if let Some(app_handle) = &self.app_handle {
            manager.create_agent(working_dir, None, app_handle.clone()).await?
        } else {
            return Err("Pool requires app_handle to spawn agents".to_string());
        };

        drop(manager);

        // Health check is handled by agent_manager internally
        // We just track the agent in our pool

        self.idle_agents.push_back(agent_id.clone());
        self.update_stats().await;
        Ok(agent_id)
    }

    /// Remove an idle agent from the pool
    async fn remove_idle_agent(&mut self) -> Result<(), String> {
        if let Some(agent_id) = self.idle_agents.pop_back() {
            let manager = self.agent_manager.lock().await;
            manager.stop_agent(&agent_id).await?;
            drop(manager);
            self.update_stats().await;
        }
        Ok(())
    }

    /// Auto-scaling background loop
    async fn auto_scale_loop(pool: Arc<Mutex<Self>>) {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;

            let mut pool_lock = pool.lock().await;
            let stats = pool_lock.stats.lock().await.clone();

            // Scale up if utilization high
            if stats.utilization > pool_lock.config.scale_up_threshold
                && pool_lock.total_agents() < pool_lock.config.max_size
            {
                if let Err(e) = pool_lock.spawn_agent().await {
                    eprintln!("Failed to scale up pool: {}", e);
                }
            }

            // Scale down if utilization low
            if stats.utilization < pool_lock.config.scale_down_threshold
                && pool_lock.total_agents() > pool_lock.config.min_size
            {
                if let Err(e) = pool_lock.remove_idle_agent().await {
                    eprintln!("Failed to scale down pool: {}", e);
                }
            }
        }
    }

    fn total_agents(&self) -> usize {
        self.idle_agents.len() + self.busy_agents.len()
    }

    async fn update_stats(&mut self) {
        let total = self.total_agents();
        let busy = self.busy_agents.len();
        let idle = self.idle_agents.len();

        let mut stats = self.stats.lock().await;
        stats.total_agents = total;
        stats.busy_agents = busy;
        stats.idle_agents = idle;
        stats.utilization = if total > 0 { busy as f32 / total as f32 } else { 0.0 };
        stats.uptime = self.start_time.elapsed();
    }

    pub async fn get_stats(&self) -> PoolStats {
        self.stats.lock().await.clone()
    }

    /// Graceful shutdown
    pub async fn shutdown(&mut self, timeout: Duration) -> Result<(), String> {
        // Stop accepting new requests
        self.config.auto_scale = false;

        // Cancel auto-scaler
        if let Some(handle) = self.auto_scaler.take() {
            handle.abort();
        }

        // Wait for busy agents to finish (with timeout)
        let start = Instant::now();
        while !self.busy_agents.is_empty() && start.elapsed() < timeout {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Stop all agents
        let all_agents: Vec<String> = self.idle_agents.iter()
            .chain(self.busy_agents.keys())
            .cloned()
            .collect();

        let manager = self.agent_manager.lock().await;
        for agent_id in all_agents {
            let _ = manager.stop_agent(&agent_id).await;
        }

        Ok(())
    }
}
