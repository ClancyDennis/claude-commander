// Agent-Spawning Tools
//
// Tools that spawn Claude Code agents: start_planning, start_execution, start_verification.

use serde_json::Value;

use crate::auto_pipeline::agent_utils::{extract_agent_output, wait_for_agent_completion};
use crate::auto_pipeline::orchestrator_tools::{
    StartExecutionInput, StartPlanningInput, StartVerificationInput, ToolResult,
};
use crate::auto_pipeline::state_machine::PipelineState;
use crate::types::AgentSource;

use super::super::context_builders::{build_full_skills_section, build_full_subagents_section};
use super::super::OrchestratorAgent;
use crate::auto_pipeline::prompts::{
    build_builder_prompt, build_planning_prompt, build_verification_prompt,
};

impl OrchestratorAgent {
    /// Start planning tool - spawns a Claude Code planning agent
    pub(crate) async fn tool_start_planning(&mut self, input: &Value) -> ToolResult {
        let parsed: StartPlanningInput = match serde_json::from_value(input.clone()) {
            Ok(p) => p,
            Err(e) => return ToolResult::error("".to_string(), format!("Invalid input: {}", e)),
        };

        self.set_state(PipelineState::Planning);

        // Check if we have an agent manager
        let (agent_manager, event_emitter) = match (&self.agent_manager, &self.event_emitter) {
            (Some(am), Some(ee)) => (am.clone(), ee.clone()),
            _ => {
                // No agent manager - return placeholder for testing
                return ToolResult::success(
                    "".to_string(),
                    format!(
                        "Planning phase started. Summary: {}\n\n[Agent manager not configured - planning agent not spawned. Add plan output manually via add_context.]",
                        parsed.summary
                    ),
                );
            }
        };

        // Build skills and subagents sections with FULL content (not just names)
        let skills_section = build_full_skills_section(&self.generated_skills, &self.working_dir);
        let subagents_section =
            build_full_subagents_section(&self.generated_subagents, &self.working_dir);

        let planning_prompt = build_planning_prompt(
            &self.user_request,
            &self.working_dir,
            &skills_section,
            &subagents_section,
        );

        // Spawn the planning agent (linked to this pipeline for historical queries)
        let agent_id = {
            let manager = agent_manager.lock().await;
            match manager
                .create_agent_with_pipeline(
                    self.working_dir.clone(),
                    None,
                    None,
                    Vec::new(), // No pre-generated skills for pipeline agents
                    AgentSource::Pipeline,
                    event_emitter.clone(),
                    Some(self.pipeline_id.clone()),
                    Some("Planning".to_string()),
                    None, // No model override
                    None, // No complexity
                )
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    return ToolResult::error(
                        "".to_string(),
                        format!("Failed to create planning agent: {}", e),
                    )
                }
            }
        };

        // Track the spawned agent for later cleanup
        self.spawned_agents[0] = Some(agent_id.clone());

        // Send the prompt
        // Note: No security_monitor for pipeline automated prompts
        {
            let manager = agent_manager.lock().await;
            if let Err(e) = manager
                .send_prompt(
                    &agent_id,
                    &planning_prompt,
                    Some(event_emitter.clone()),
                    None,
                )
                .await
            {
                return ToolResult::error(
                    "".to_string(),
                    format!("Failed to send planning prompt: {}", e),
                );
            }
        }

        // Wait for completion
        if let Err(e) = wait_for_agent_completion(&agent_id, agent_manager.clone()).await {
            return ToolResult::error("".to_string(), format!("Planning agent failed: {}", e));
        }

        // Extract output
        let output = match extract_agent_output(&agent_id, agent_manager.clone()).await {
            Ok(o) => o,
            Err(e) => {
                return ToolResult::error(
                    "".to_string(),
                    format!("Failed to get planning output: {}", e),
                )
            }
        };

        // Store the plan and agent outputs
        self.current_plan = output.raw_text.clone();
        self.planning_agent_outputs = output.agent_outputs.clone();

        ToolResult::success(
            "".to_string(),
            format!(
                "Planning agent completed. Here is the plan:\n\n{}\n\nReview this plan. If it looks good and actionable, call approve_plan. If it needs changes, call replan with specific issues.",
                output.raw_text
            ),
        )
    }

    /// Start execution tool - spawns a Claude Code build agent
    pub(crate) async fn tool_start_execution(&mut self, input: &Value) -> ToolResult {
        // Validate state: must have a plan before execution
        if self.current_plan.is_empty() {
            return ToolResult::error(
                "".to_string(),
                "Cannot start execution: no plan exists. Call start_planning first to create a plan.".to_string(),
            );
        }

        let parsed: StartExecutionInput =
            serde_json::from_value(input.clone()).unwrap_or(StartExecutionInput { notes: None });

        self.set_state(PipelineState::Executing);

        // Check if we have an agent manager
        let (agent_manager, event_emitter) = match (&self.agent_manager, &self.event_emitter) {
            (Some(am), Some(ee)) => (am.clone(), ee.clone()),
            _ => {
                return ToolResult::success(
                    "".to_string(),
                    "[Agent manager not configured - build agent not spawned. Add build output manually via add_context.]".to_string(),
                );
            }
        };

        // Build skills and subagents sections with FULL content (not just names)
        let skills_section = build_full_skills_section(&self.generated_skills, &self.working_dir);
        let subagents_section =
            build_full_subagents_section(&self.generated_subagents, &self.working_dir);

        let notes_section = parsed
            .notes
            .map(|n| format!("\n## ADDITIONAL NOTES\n{}\n", n))
            .unwrap_or_default();

        let builder_prompt = build_builder_prompt(
            &self.user_request,
            &self.working_dir,
            &self.current_plan,
            &self.current_qna,
            &skills_section,
            &subagents_section,
            &notes_section,
        );

        // Spawn the build agent (linked to this pipeline for historical queries)
        let agent_id = {
            let manager = agent_manager.lock().await;
            match manager
                .create_agent_with_pipeline(
                    self.working_dir.clone(),
                    None,
                    None,
                    Vec::new(), // No pre-generated skills for pipeline agents
                    AgentSource::Pipeline,
                    event_emitter.clone(),
                    Some(self.pipeline_id.clone()),
                    Some("Building".to_string()),
                    None, // No model override
                    None, // No complexity
                )
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    return ToolResult::error(
                        "".to_string(),
                        format!("Failed to create build agent: {}", e),
                    )
                }
            }
        };

        // Track the spawned agent for later cleanup
        self.spawned_agents[1] = Some(agent_id.clone());

        // Send the prompt
        // Note: No security_monitor for pipeline automated prompts
        {
            let manager = agent_manager.lock().await;
            if let Err(e) = manager
                .send_prompt(
                    &agent_id,
                    &builder_prompt,
                    Some(event_emitter.clone()),
                    None,
                )
                .await
            {
                return ToolResult::error(
                    "".to_string(),
                    format!("Failed to send build prompt: {}", e),
                );
            }
        }

        // Wait for completion
        if let Err(e) = wait_for_agent_completion(&agent_id, agent_manager.clone()).await {
            return ToolResult::error("".to_string(), format!("Build agent failed: {}", e));
        }

        // Extract output
        let output = match extract_agent_output(&agent_id, agent_manager.clone()).await {
            Ok(o) => o,
            Err(e) => {
                return ToolResult::error(
                    "".to_string(),
                    format!("Failed to get build output: {}", e),
                )
            }
        };

        // Store the implementation and agent outputs
        self.current_implementation = output.raw_text.clone();
        self.building_agent_outputs = output.agent_outputs.clone();

        ToolResult::success(
            "".to_string(),
            format!(
                "Build agent completed. Here is the output:\n\n{}\n\nCall start_verification to verify the implementation.",
                output.raw_text
            ),
        )
    }

    /// Start verification tool - spawns a Claude Code verification agent
    pub(crate) async fn tool_start_verification(&mut self, input: &Value) -> ToolResult {
        // Validate state: must have completed execution before verification
        if self.current_implementation.is_empty() {
            return ToolResult::error(
                "".to_string(),
                "Cannot start verification: no implementation exists. Call start_execution first to implement the plan.".to_string(),
            );
        }

        let parsed: StartVerificationInput =
            serde_json::from_value(input.clone()).unwrap_or(StartVerificationInput {
                focus_areas: Vec::new(),
            });

        self.set_state(PipelineState::Verifying);

        // Check if we have an agent manager
        let (agent_manager, event_emitter) = match (&self.agent_manager, &self.event_emitter) {
            (Some(am), Some(ee)) => (am.clone(), ee.clone()),
            _ => {
                return ToolResult::success(
                    "".to_string(),
                    "[Agent manager not configured - verification agent not spawned. Add verification output manually via add_context.]".to_string(),
                );
            }
        };

        let focus_section = if parsed.focus_areas.is_empty() {
            String::new()
        } else {
            format!(
                "\n## FOCUS AREAS\n{}\n",
                parsed
                    .focus_areas
                    .iter()
                    .map(|a| format!("- {}", a))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        // Build skills and subagents sections with FULL content for verification context
        let skills_section = build_full_skills_section(&self.generated_skills, &self.working_dir);
        let subagents_section =
            build_full_subagents_section(&self.generated_subagents, &self.working_dir);

        let verification_prompt = build_verification_prompt(
            &self.user_request,
            &self.current_plan,
            &self.current_implementation,
            &focus_section,
            &skills_section,
            &subagents_section,
        );

        // Spawn the verification agent (linked to this pipeline for historical queries)
        let agent_id = {
            let manager = agent_manager.lock().await;
            match manager
                .create_agent_with_pipeline(
                    self.working_dir.clone(),
                    None,
                    None,
                    Vec::new(), // No pre-generated skills for pipeline agents
                    AgentSource::Pipeline,
                    event_emitter.clone(),
                    Some(self.pipeline_id.clone()),
                    Some("Verification".to_string()),
                    None, // No model override
                    None, // No complexity
                )
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    return ToolResult::error(
                        "".to_string(),
                        format!("Failed to create verification agent: {}", e),
                    )
                }
            }
        };

        // Track the spawned agent for later cleanup
        self.spawned_agents[2] = Some(agent_id.clone());

        // Send the prompt
        // Note: No security_monitor for pipeline automated prompts
        {
            let manager = agent_manager.lock().await;
            if let Err(e) = manager
                .send_prompt(
                    &agent_id,
                    &verification_prompt,
                    Some(event_emitter.clone()),
                    None,
                )
                .await
            {
                return ToolResult::error(
                    "".to_string(),
                    format!("Failed to send verification prompt: {}", e),
                );
            }
        }

        // Wait for completion
        if let Err(e) = wait_for_agent_completion(&agent_id, agent_manager.clone()).await {
            return ToolResult::error("".to_string(), format!("Verification agent failed: {}", e));
        }

        // Extract output
        let output = match extract_agent_output(&agent_id, agent_manager.clone()).await {
            Ok(o) => o,
            Err(e) => {
                return ToolResult::error(
                    "".to_string(),
                    format!("Failed to get verification output: {}", e),
                )
            }
        };

        // Store agent outputs
        self.verification_agent_outputs = output.agent_outputs.clone();

        ToolResult::success(
            "".to_string(),
            format!(
                "Verification completed:\n\n{}\n\nBased on these results, decide:\n- Call `complete` if the task is done successfully\n- Call `iterate` to fix issues (provides notes to build agent)\n- Call `replan` if the approach needs to change\n- Call `give_up` only if the task is impossible",
                output.raw_text
            ),
        )
    }
}
