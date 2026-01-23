use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::orchestrator::TaskOrchestrator;
use crate::pool_manager::AgentPool;

/// Central controller for all thread patterns
pub struct ThreadController {
    config: Arc<Mutex<ThreadConfig>>,
    pool: Option<Arc<Mutex<AgentPool>>>,
    orchestrator: Arc<Mutex<TaskOrchestrator>>,
    stats: Arc<Mutex<ThreadStats>>,
}

/// Configuration for thread behavior
#[derive(Clone, Serialize, Deserialize)]
pub struct ThreadConfig {
    pub p_thread_enabled: bool, // Enable agent pool
    pub b_thread_enabled: bool, // Enable task orchestration
    pub f_thread_enabled: bool, // Enable verification (Phase C)
    pub c_thread_enabled: bool, // Enable checkpoints (Phase D)

    pub max_concurrent_workflows: usize, // Limit parallel workflows
    pub max_concurrent_verifications: usize, // Limit parallel verifications
    pub backpressure_threshold: f32,     // Stop accepting new work at X% utilization
}

impl Default for ThreadConfig {
    fn default() -> Self {
        Self {
            p_thread_enabled: true,
            b_thread_enabled: true,
            f_thread_enabled: true,
            c_thread_enabled: true,
            max_concurrent_workflows: 10,
            max_concurrent_verifications: 3,
            backpressure_threshold: 0.9,
        }
    }
}

/// Cross-thread statistics
#[derive(Clone, Serialize, Default)]
pub struct ThreadStats {
    pub active_workflows: usize,
    pub active_verifications: usize,
    pub pool_utilization: f32,
    pub total_agents: usize,
    pub system_load: f32, // 0.0 - 1.0
}

impl ThreadController {
    pub fn new(
        pool: Option<Arc<Mutex<AgentPool>>>,
        orchestrator: Arc<Mutex<TaskOrchestrator>>,
    ) -> Self {
        Self {
            config: Arc::new(Mutex::new(ThreadConfig::default())),
            pool,
            orchestrator,
            stats: Arc::new(Mutex::new(ThreadStats::default())),
        }
    }

    /// Check if system can accept new work
    pub async fn can_accept_work(&self) -> Result<(), String> {
        let config = self.config.lock().await;
        let stats = self.stats.lock().await;

        if stats.system_load > config.backpressure_threshold {
            return Err(format!(
                "System at {}% capacity (threshold: {}%). Rejecting new work.",
                (stats.system_load * 100.0) as u32,
                (config.backpressure_threshold * 100.0) as u32
            ));
        }

        if stats.active_workflows >= config.max_concurrent_workflows {
            return Err(format!(
                "Maximum concurrent workflows reached ({}/{})",
                stats.active_workflows, config.max_concurrent_workflows
            ));
        }

        Ok(())
    }

    /// Request agent from pool with backpressure check
    pub async fn request_agent(&self) -> Result<String, String> {
        self.can_accept_work().await?;

        let config = self.config.lock().await;
        if !config.p_thread_enabled {
            return Err("P-Thread (agent pool) is disabled".to_string());
        }
        drop(config);

        if let Some(pool) = &self.pool {
            let mut pool_lock = pool.lock().await;
            pool_lock.acquire_agent().await.map_err(|e| e.to_string())
        } else {
            Err("Agent pool not initialized".to_string())
        }
    }

    /// Create workflow with B-Thread check
    pub async fn create_workflow(&self, request: String) -> Result<String, String> {
        self.can_accept_work().await?;

        let config = self.config.lock().await;
        if !config.b_thread_enabled {
            return Err("B-Thread (task orchestration) is disabled".to_string());
        }
        drop(config);

        let orchestrator = self.orchestrator.lock().await;
        let workflow_id = orchestrator.create_workflow_from_request(&request).await?;

        // Update stats
        let mut stats = self.stats.lock().await;
        stats.active_workflows += 1;
        drop(stats);

        Ok(workflow_id)
    }

    /// Execute workflow with tracking
    pub async fn execute_workflow(&self, workflow_id: String) -> Result<(), String> {
        let orchestrator = self.orchestrator.lock().await;
        orchestrator.execute_workflow(&workflow_id).await?;
        Ok(())
    }

    /// Update system statistics (call periodically)
    pub async fn update_stats(&self) {
        let mut stats = self.stats.lock().await;

        // Update pool stats
        if let Some(pool) = &self.pool {
            let pool_lock = pool.lock().await;
            let pool_stats = pool_lock.get_stats().await;
            stats.pool_utilization = pool_stats.utilization;
            stats.total_agents = pool_stats.total_agents;
        }

        // Update workflow count
        let orchestrator = self.orchestrator.lock().await;
        let workflows = orchestrator.list_workflows().await;
        stats.active_workflows = workflows
            .iter()
            .filter(|w| w.status == crate::orchestrator::WorkflowStatus::Running)
            .count();

        // Calculate overall system load
        stats.system_load = (stats.pool_utilization + (stats.active_workflows as f32 / 10.0)) / 2.0;
    }

    /// Get current configuration
    pub async fn get_config(&self) -> ThreadConfig {
        self.config.lock().await.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, new_config: ThreadConfig) {
        let mut config = self.config.lock().await;
        *config = new_config;
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> ThreadStats {
        self.stats.lock().await.clone()
    }

    /// Emergency shutdown all threads
    pub async fn emergency_shutdown(&self) -> Result<(), String> {
        println!("⚠️ Emergency shutdown initiated");

        // Stop accepting new work
        let mut config = self.config.lock().await;
        config.p_thread_enabled = false;
        config.b_thread_enabled = false;
        config.f_thread_enabled = false;
        config.c_thread_enabled = false;
        drop(config);

        // Shutdown pool
        if let Some(pool) = &self.pool {
            let mut pool_lock = pool.lock().await;
            pool_lock
                .shutdown(std::time::Duration::from_secs(10))
                .await?;
        }

        println!("✓ Emergency shutdown complete");
        Ok(())
    }
}
