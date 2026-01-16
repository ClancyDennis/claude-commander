use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};

use crate::pool_manager::AgentPool;
use crate::meta_agent::MetaAgent;
use crate::pipeline_manager::FusionStrategy;

#[derive(Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub n: usize,                      // Number of agents (default: 3)
    pub fusion_strategy: FusionStrategy,
    pub confidence_threshold: f32,     // Minimum confidence (0.0-1.0, default: 0.8)
    pub timeout: std::time::Duration,  // Max time to wait for all agents
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            n: 3,
            fusion_strategy: FusionStrategy::WeightedConsensus,
            confidence_threshold: 0.8,
            timeout: std::time::Duration::from_secs(120),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct AgentResult {
    pub agent_id: String,
    pub output: String,
    pub confidence: Option<f32>,
    pub execution_time: f32,          // seconds
    pub validation_passed: bool,
}

#[derive(Clone, Serialize)]
pub struct VerificationResult {
    pub selected_result: AgentResult,
    pub confidence: f32,
    pub all_results: Vec<AgentResult>,
    pub fusion_reasoning: String,
    pub verification_time: f32,       // total seconds
}

pub struct VerificationEngine {
    agent_pool: Option<Arc<Mutex<AgentPool>>>,
    meta_agent: Arc<Mutex<MetaAgent>>,
}

impl VerificationEngine {
    pub fn new(
        agent_pool: Option<Arc<Mutex<AgentPool>>>,
        meta_agent: Arc<Mutex<MetaAgent>>
    ) -> Self {
        Self { agent_pool, meta_agent }
    }

    /// Run Best-of-N verification
    pub async fn best_of_n(
        &self,
        prompt: &str,
        config: VerificationConfig
    ) -> Result<VerificationResult, String> {
        let start_time = Instant::now();

        // Check if agent pool is available
        let pool = self.agent_pool.as_ref()
            .ok_or("Agent pool not initialized")?;

        // Acquire N agents from pool
        let mut agent_ids = Vec::new();
        let mut pool_lock = pool.lock().await;

        for _ in 0..config.n {
            match pool_lock.acquire_agent().await {
                Ok(id) => agent_ids.push(id),
                Err(e) => {
                    // Release already acquired agents
                    for id in &agent_ids {
                        pool_lock.release_agent(id.clone()).await;
                    }
                    return Err(format!("Failed to acquire {} agents: {}", config.n, e));
                }
            }
        }
        drop(pool_lock);

        // Send same prompt to all agents in parallel
        let mut handles = Vec::new();
        for agent_id in agent_ids.clone() {
            let prompt = prompt.to_string();
            let agent_id_clone = agent_id.clone();

            let handle = tokio::spawn(async move {
                let agent_start = Instant::now();

                // TODO: Send prompt to agent and get response
                // For now, simulate with placeholder
                // In real implementation, would use agent_manager.send_prompt()
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                let output = format!("Agent {} output for: {}", agent_id_clone, prompt);

                let execution_time = agent_start.elapsed().as_secs_f32();

                AgentResult {
                    agent_id: agent_id_clone,
                    output,
                    confidence: Some(0.9),
                    execution_time,
                    validation_passed: true,
                }
            });

            handles.push(handle);
        }

        // Wait for all agents to complete (with timeout)
        let timeout = tokio::time::timeout(
            config.timeout,
            futures::future::join_all(handles)
        );

        let results = match timeout.await {
            Ok(results) => {
                results.into_iter()
                    .filter_map(|r| r.ok())
                    .collect::<Vec<_>>()
            }
            Err(_) => {
                return Err("Verification timeout".to_string());
            }
        };

        // Release agents back to pool
        let mut pool_lock = pool.lock().await;
        for agent_id in agent_ids {
            pool_lock.release_agent(agent_id).await;
        }
        drop(pool_lock);

        // Fuse results based on strategy
        let selected = self.fuse_results(&results, &config).await?;

        let verification_time = start_time.elapsed().as_secs_f32();

        Ok(VerificationResult {
            selected_result: selected.clone(),
            confidence: selected.confidence.unwrap_or(0.5),
            all_results: results,
            fusion_reasoning: format!("Selected using {:?} strategy", config.fusion_strategy),
            verification_time,
        })
    }

    async fn fuse_results(
        &self,
        results: &[AgentResult],
        config: &VerificationConfig
    ) -> Result<AgentResult, String> {
        if results.is_empty() {
            return Err("No results to fuse".to_string());
        }

        match config.fusion_strategy {
            FusionStrategy::MajorityVote => {
                // Find most common output
                let mut counts = std::collections::HashMap::new();
                for result in results {
                    *counts.entry(&result.output).or_insert(0) += 1;
                }

                let most_common = counts.into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(output, _)| output)
                    .unwrap();

                Ok(results.iter()
                    .find(|r| &r.output == most_common)
                    .unwrap()
                    .clone())
            }

            FusionStrategy::WeightedConsensus => {
                // Pick result with highest confidence
                results.iter()
                    .max_by(|a, b| {
                        a.confidence.unwrap_or(0.0)
                            .partial_cmp(&b.confidence.unwrap_or(0.0))
                            .unwrap()
                    })
                    .cloned()
                    .ok_or_else(|| "No results with confidence".to_string())
            }

            FusionStrategy::MetaAgentReview => {
                // Use meta-agent to pick best result
                let _meta = self.meta_agent.lock().await;

                let review_prompt = format!(
                    "Review these {} results and pick the best one. Return just the agent_id of the best result.\n\nResults:\n{}",
                    results.len(),
                    serde_json::to_string_pretty(results).unwrap()
                );

                // TODO: Send to meta-agent properly
                // For now, just pick first result
                let _response = review_prompt; // Placeholder

                // Return first result as placeholder
                Ok(results[0].clone())
            }

            FusionStrategy::FirstCorrect => {
                // Return first result that passed validation
                results.iter()
                    .find(|r| r.validation_passed)
                    .cloned()
                    .ok_or_else(|| "No results passed validation".to_string())
            }
        }
    }

    /// Spawn a verification agent to check another agent's work
    pub async fn spawn_verification_agent(
        &self,
        target_agent_id: &str,
        _verification_prompt: &str
    ) -> Result<VerificationReport, String> {
        // Check if agent pool is available
        let pool = self.agent_pool.as_ref()
            .ok_or("Agent pool not initialized")?;

        // Acquire verification agent from pool
        let mut pool_lock = pool.lock().await;
        let verifier_id = pool_lock.acquire_agent().await.map_err(|e| e.to_string())?;
        drop(pool_lock);

        // TODO: Send verification prompt to verifier agent
        // For now, simulate verification
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let report = VerificationReport {
            target_agent_id: target_agent_id.to_string(),
            verifier_agent_id: verifier_id.clone(),
            passed: true,
            issues: Vec::new(),
            suggestions: Vec::new(),
            confidence: 0.95,
        };

        // Release verifier back to pool
        let mut pool_lock = pool.lock().await;
        pool_lock.release_agent(verifier_id).await;

        Ok(report)
    }
}

#[derive(Clone, Serialize)]
pub struct VerificationReport {
    pub target_agent_id: String,
    pub verifier_agent_id: String,
    pub passed: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub confidence: f32,
}
