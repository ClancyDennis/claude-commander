// Helper functions for step execution

use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::auto_pipeline::orchestrator_agent::OrchestratorAgent;
use crate::auto_pipeline::types::{AutoPipeline, StepStatus};

/// Emit a step-related event
pub fn emit_step_event(
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
    event_name: &str,
    data: serde_json::Value,
) {
    let _ = app_handle.emit(event_name, data);
}

/// Emit a step status change event
pub fn emit_step_status(
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
    pipeline_id: &str,
    step_number: u8,
    status: &StepStatus,
) {
    emit_step_event(
        app_handle,
        "auto_pipeline:step_status",
        json!({
            "pipeline_id": pipeline_id,
            "step_number": step_number,
            "status": format!("{:?}", status),
        }),
    );
}

/// Emit a step completed event
pub fn emit_step_completed(
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
    pipeline_id: &str,
    step_number: u8,
    extra: Option<serde_json::Value>,
) {
    eprintln!(
        "[emit_step_completed] Emitting step_completed for pipeline={}, step={}",
        pipeline_id, step_number
    );

    let mut data = json!({
        "pipeline_id": pipeline_id,
        "step_number": step_number,
    });

    if let Some(extra_data) = extra {
        if let (Some(obj), Some(extra_obj)) = (data.as_object_mut(), extra_data.as_object()) {
            for (k, v) in extra_obj {
                obj.insert(k.clone(), v.clone());
            }
        }
    }

    emit_step_event(app_handle, "auto_pipeline:step_completed", data);
}

/// Emit a pipeline completed event
pub fn emit_pipeline_completed(
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
    pipeline_id: &str,
    status: &str,
    decision: &str,
    extra: Option<serde_json::Value>,
) {
    let mut data = json!({
        "pipeline_id": pipeline_id,
        "status": status,
        "decision": decision,
    });

    if let Some(extra_data) = extra {
        if let (Some(obj), Some(extra_obj)) = (data.as_object_mut(), extra_data.as_object()) {
            for (k, v) in extra_obj {
                obj.insert(k.clone(), v.clone());
            }
        }
    }

    emit_step_event(app_handle, "auto_pipeline:completed", data);
}

/// Get data from a pipeline with a closure
pub async fn with_pipeline<F, T>(
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    pipeline_id: &str,
    f: F,
) -> Result<T, String>
where
    F: FnOnce(&AutoPipeline) -> T,
{
    let pipelines_lock = pipelines.lock().await;
    let pipeline = pipelines_lock
        .get(pipeline_id)
        .ok_or("Pipeline not found")?;
    Ok(f(pipeline))
}

/// Mutate a pipeline with a closure
pub async fn with_pipeline_mut<F, T>(
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    pipeline_id: &str,
    f: F,
) -> Result<T, String>
where
    F: FnOnce(&mut AutoPipeline) -> T,
{
    let mut pipelines_lock = pipelines.lock().await;
    let pipeline = pipelines_lock
        .get_mut(pipeline_id)
        .ok_or("Pipeline not found")?;
    Ok(f(pipeline))
}

/// Take an orchestrator agent from storage
pub async fn take_orchestrator_agent(
    agents: &Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    pipeline_id: &str,
) -> Result<OrchestratorAgent, String> {
    let mut agents_lock = agents.lock().await;
    agents_lock
        .remove(pipeline_id)
        .ok_or_else(|| "OrchestratorAgent not found for pipeline".to_string())
}

/// Store an orchestrator agent back
pub async fn store_orchestrator_agent(
    agents: &Arc<Mutex<HashMap<String, OrchestratorAgent>>>,
    pipeline_id: &str,
    agent: OrchestratorAgent,
) {
    let mut agents_lock = agents.lock().await;
    agents_lock.insert(pipeline_id.to_string(), agent);
}

/// Stop an agent by ID
pub async fn stop_agent(agent_manager: &Arc<Mutex<AgentManager>>, agent_id: &str) {
    eprintln!("[auto_pipeline] Stopping agent {}", agent_id);
    let manager = agent_manager.lock().await;
    let _ = manager.stop_agent(agent_id).await;
    eprintln!("[auto_pipeline] Agent {} stopped", agent_id);
}

/// Stop agent from a specific step if it exists
pub async fn stop_step_agent(
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    pipeline_id: &str,
    step_number: u8,
    agent_manager: &Arc<Mutex<AgentManager>>,
) {
    eprintln!(
        "[auto_pipeline] stop_step_agent called for pipeline={}, step={}",
        pipeline_id, step_number
    );

    let agent_id = {
        let pipelines_lock = pipelines.lock().await;
        pipelines_lock
            .get(pipeline_id)
            .and_then(|p| p.steps.get((step_number - 1) as usize))
            .and_then(|s| s.agent_id.clone())
    };

    if let Some(ref agent_id) = agent_id {
        stop_agent(agent_manager, agent_id).await;
    } else {
        eprintln!("[auto_pipeline] No agent to stop for step {}", step_number);
    }
}

/// Stop all agents associated with a pipeline
pub async fn stop_all_pipeline_agents(
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    pipeline_id: &str,
    agent_manager: &Arc<Mutex<AgentManager>>,
) {
    eprintln!(
        "[auto_pipeline] stop_all_pipeline_agents called for pipeline={}",
        pipeline_id
    );

    let agent_ids: Vec<String> = {
        let pipelines_lock = pipelines.lock().await;
        pipelines_lock
            .get(pipeline_id)
            .map(|p| p.steps.iter().filter_map(|s| s.agent_id.clone()).collect())
            .unwrap_or_default()
    };

    eprintln!(
        "[auto_pipeline] Found {} agents to stop: {:?}",
        agent_ids.len(),
        agent_ids
    );

    let manager = agent_manager.lock().await;
    for agent_id in agent_ids {
        eprintln!("[auto_pipeline] Stopping agent {}", agent_id);
        let _ = manager.stop_agent(&agent_id).await;
        eprintln!("[auto_pipeline] Agent {} stopped", agent_id);
    }

    eprintln!("[auto_pipeline] stop_all_pipeline_agents completed");
}

/// Update step status and emit event
pub async fn update_step_status(
    pipelines: &Arc<Mutex<HashMap<String, AutoPipeline>>>,
    pipeline_id: &str,
    step_number: u8,
    status: StepStatus,
    app_handle: &Arc<dyn crate::events::AppEventEmitter>,
) {
    let mut pipelines_lock = pipelines.lock().await;
    if let Some(pipeline) = pipelines_lock.get_mut(pipeline_id) {
        let index = (step_number - 1) as usize;
        pipeline.steps[index].status = status.clone();
        emit_step_status(app_handle, pipeline_id, step_number, &status);
    }
}
