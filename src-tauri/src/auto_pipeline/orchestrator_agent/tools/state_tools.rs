// State Transition Tools
//
// Tools that manage pipeline state transitions: approve_plan, complete, iterate, replan.

use crate::auto_pipeline::orchestrator_tools::ToolResult;
use crate::auto_pipeline::state_machine::PipelineState;

use super::super::OrchestratorAgent;

impl OrchestratorAgent {
    /// Approve plan tool - validates plan exists and transitions to ReadyForExecution
    pub(crate) async fn tool_approve_plan(&mut self) -> ToolResult {
        if self.current_plan.is_empty() {
            return ToolResult::error(
                "".to_string(),
                "Cannot approve plan: no plan exists. Call start_planning first.".to_string(),
            );
        }
        // Transition to ReadyForExecution state
        self.set_state(PipelineState::ReadyForExecution);

        ToolResult::success(
            "".to_string(),
            "Plan approved. Now call start_execution to begin implementation.".to_string(),
        )
    }

    /// Complete tool - validates we've gone through verification and transitions to Completed state
    pub(crate) async fn tool_complete(&mut self) -> ToolResult {
        if self.current_state != PipelineState::Verifying {
            return ToolResult::error(
                "".to_string(),
                "Cannot complete: must complete verification first. Call start_verification after execution.".to_string(),
            );
        }
        // Transition to terminal Completed state
        self.set_state(PipelineState::Completed);

        ToolResult::success("".to_string(), "Pipeline marked as complete.".to_string())
    }

    /// Iterate tool - clears implementation and goes back to execution
    pub(crate) async fn tool_iterate(&mut self) -> ToolResult {
        if self.current_state != PipelineState::Verifying {
            return ToolResult::error(
                "".to_string(),
                "Cannot iterate: must complete verification first. Call start_verification after execution.".to_string(),
            );
        }
        // Clear implementation but keep plan
        self.current_implementation.clear();

        // Go back to ReadyForExecution state
        self.current_state = PipelineState::ReadyForExecution;
        self.refresh_tools();

        ToolResult::success(
            "".to_string(),
            "Iterating. Call start_execution to re-implement with fixes.".to_string(),
        )
    }

    /// Replan tool - clears current plan and goes back to planning phase
    /// Keeps generated skills intact so we don't lose work
    pub(crate) async fn tool_replan(&mut self) -> ToolResult {
        // Only allow replan from Verifying state or Planning states
        // Not allowed from terminal states or during execution
        let is_planning_phase = matches!(
            self.current_state,
            PipelineState::Planning
                | PipelineState::PlanReady
                | PipelineState::PlanRevisionRequired
        );
        let allowed = is_planning_phase || self.current_state == PipelineState::Verifying;

        if !allowed {
            return ToolResult::error(
                "".to_string(),
                format!(
                    "Cannot replan from {:?} state. Replan is only available during planning or verification phases.",
                    self.current_state
                ),
            );
        }

        // Track replan usage during planning phase
        if is_planning_phase {
            self.planning_replan_count += 1;
        }

        // Clear current plan and implementation, but keep skills
        self.current_plan.clear();
        self.current_implementation.clear();

        // Go back to Planning state (not ReceivedTask - we want to keep generated skills)
        self.set_state(PipelineState::Planning);

        ToolResult::success(
            "".to_string(),
            "Replanning. Create a new plan with start_planning, then approve_plan when ready.".to_string(),
        )
    }
}
