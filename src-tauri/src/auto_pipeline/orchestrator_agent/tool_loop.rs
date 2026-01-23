// Tool Calling Loop
//
// The main orchestrator loop that processes AI responses and executes tools.

use serde_json::{json, Value};

use crate::auto_pipeline::orchestrator_tools::{
    CompleteInput, GiveUpInput, IterateInput, ReplanInput, ToolResult,
};

use super::context_builders::send_to_ai;
use super::types::{
    ContentBlockValue, ConversationContent, ConversationMessage, OrchestratorAction,
};
use super::OrchestratorAgent;

impl OrchestratorAgent {
    /// Run the orchestrator until it requests a phase transition or decision
    pub async fn run_until_action(&mut self) -> Result<OrchestratorAction, String> {
        loop {
            // Send message to AI
            let response = send_to_ai(&self.ai_client, &self.messages, &self.tools).await?;

            // Check for tool calls
            let mut tool_uses = Vec::new();
            let mut has_text = false;

            for block in &response {
                match block {
                    ContentBlockValue::ToolUse { id, name, input } => {
                        tool_uses.push((id.clone(), name.clone(), input.clone()));
                    }
                    ContentBlockValue::Text { .. } => {
                        has_text = true;
                    }
                    _ => {}
                }
            }

            // Add assistant response to messages
            self.messages.push(ConversationMessage {
                role: "assistant".to_string(),
                content: ConversationContent::Blocks(response),
            });

            // If no tool calls, the AI is done thinking
            if tool_uses.is_empty() {
                if has_text {
                    // AI provided text but no action - prompt it to take action
                    self.add_context("user", "Please use one of the available tools to proceed.");
                    continue;
                }
                return Err("AI did not provide any response".to_string());
            }

            // Process each tool call
            let mut tool_results = Vec::new();
            let mut action: Option<OrchestratorAction> = None;

            for (tool_id, tool_name, tool_input) in tool_uses {
                // Emit tool start event
                if let Some(ref emitter) = self.event_emitter {
                    let _ = emitter.emit(
                        "orchestrator:tool_start",
                        json!({
                            "tool_name": tool_name,
                            "tool_input": tool_input,
                            "current_state": format!("{:?}", self.current_state),
                            "iteration": self.current_iteration,
                            "step_number": self.get_current_step_number(),
                        }),
                    );
                }

                let result = self.execute_tool(&tool_name, &tool_input).await;

                // Emit tool complete event
                if let Some(ref emitter) = self.event_emitter {
                    // Truncate content for event (avoid sending huge outputs)
                    let summary = if result.content.len() > 200 {
                        format!("{}...", &result.content[..200])
                    } else {
                        result.content.clone()
                    };
                    let _ = emitter.emit(
                        "orchestrator:tool_complete",
                        json!({
                            "tool_name": tool_name,
                            "is_error": result.is_error,
                            "summary": summary,
                            "current_state": format!("{:?}", self.current_state),
                            "step_number": self.get_current_step_number(),
                        }),
                    );
                }

                // Check if this is a terminal action (decision tools)
                // Only set action if the tool succeeded - don't exit loop on error
                if !result.is_error {
                    match &tool_name as &str {
                        "complete" => {
                            if let Ok(input) = serde_json::from_value::<CompleteInput>(tool_input) {
                                action = Some(OrchestratorAction::Complete {
                                    summary: input.summary,
                                });
                            }
                        }
                        "iterate" => {
                            if let Ok(input) = serde_json::from_value::<IterateInput>(tool_input) {
                                action = Some(OrchestratorAction::Iterate {
                                    issues: input.issues,
                                    suggestions: input.suggestions,
                                });
                            }
                        }
                        "replan" => {
                            if let Ok(input) = serde_json::from_value::<ReplanInput>(tool_input) {
                                action = Some(OrchestratorAction::Replan {
                                    reason: input.reason,
                                    issues: input.issues,
                                    suggestions: input.suggestions,
                                });
                            }
                        }
                        "give_up" => {
                            if let Ok(input) = serde_json::from_value::<GiveUpInput>(tool_input) {
                                action = Some(OrchestratorAction::GiveUp {
                                    reason: input.reason,
                                });
                            }
                        }
                        _ => {
                            // Other tools (start_planning, start_execution, start_verification, etc.)
                            // are handled internally and continue the loop
                        }
                    }
                }

                tool_results.push(ContentBlockValue::ToolResult {
                    tool_use_id: tool_id,
                    content: result.content,
                    is_error: if result.is_error { Some(true) } else { None },
                });
            }

            // Add tool results to messages
            self.messages.push(ConversationMessage {
                role: "user".to_string(),
                content: ConversationContent::Blocks(tool_results),
            });

            // If we got a terminal action (decision), get final summary and return
            if let Some(act) = action {
                // For Complete action, do one more AI turn to get a final summary
                if matches!(act, OrchestratorAction::Complete { .. }) {
                    let final_summary = self.get_final_summary().await;
                    if let OrchestratorAction::Complete { summary } = act {
                        return Ok(OrchestratorAction::Complete {
                            summary: if final_summary.is_empty() {
                                summary
                            } else {
                                final_summary
                            },
                        });
                    }
                }
                return Ok(act);
            }

            // Otherwise continue the loop
        }
    }

    /// Get a final summary from the orchestrator after completion
    async fn get_final_summary(&mut self) -> String {
        // Emit event that we're generating final summary
        if let Some(ref emitter) = self.event_emitter {
            let _ = emitter.emit(
                "orchestrator:generating_summary",
                json!({
                    "current_state": format!("{:?}", self.current_state),
                }),
            );
        }

        // Add a prompt asking for a final summary
        self.add_context(
            "user",
            "Pipeline complete. Please provide a brief final summary of what was accomplished, \
             including any key decisions made and the final state of the implementation. \
             Respond with text only, no tool calls.",
        );

        // Make one final AI call with no tools available
        let response = send_to_ai(&self.ai_client, &self.messages, &[]).await;

        match response {
            Ok(blocks) => {
                // Extract text from response
                let mut summary = String::new();
                for block in blocks {
                    if let ContentBlockValue::Text { text } = block {
                        if !summary.is_empty() {
                            summary.push('\n');
                        }
                        summary.push_str(&text);
                    }
                }
                summary
            }
            Err(e) => {
                eprintln!("[ORCHESTRATOR] Failed to get final summary: {}", e);
                String::new()
            }
        }
    }

    /// Run the orchestrator through its complete workflow until final completion or give up
    /// Handles planning, execution, verification, iteration, and replanning internally
    pub async fn run_to_completion(&mut self) -> Result<OrchestratorAction, String> {
        let max_iterations = self.max_iterations;
        let mut iteration_count = 0;

        loop {
            // Run until we get a decision
            let action = self.run_until_action().await?;

            match action {
                // Terminal success
                OrchestratorAction::Complete { summary } => {
                    eprintln!("[ORCHESTRATOR] run_to_completion: Pipeline completed successfully");
                    return Ok(OrchestratorAction::Complete { summary });
                }

                // Terminal failure
                OrchestratorAction::GiveUp { reason } => {
                    eprintln!(
                        "[ORCHESTRATOR] run_to_completion: Pipeline gave up: {}",
                        reason
                    );
                    return Ok(OrchestratorAction::GiveUp { reason });
                }

                // Iteration - reset and continue
                OrchestratorAction::Iterate {
                    issues: _,
                    suggestions: _,
                } => {
                    iteration_count += 1;
                    eprintln!(
                        "[ORCHESTRATOR] run_to_completion: Iteration {} of {} requested",
                        iteration_count, max_iterations
                    );

                    if iteration_count >= max_iterations {
                        let reason = format!("Maximum iterations ({}) reached", max_iterations);
                        eprintln!("[ORCHESTRATOR] run_to_completion: {}", reason);
                        return Ok(OrchestratorAction::GiveUp { reason });
                    }

                    // Reset state and continue
                    self.current_iteration += 1;
                    // The iterate tool already reset state to ReadyForExecution
                    // Just continue the loop
                    eprintln!("[ORCHESTRATOR] run_to_completion: Continuing to next iteration");
                    continue;
                }

                // Replan - reset and continue
                OrchestratorAction::Replan {
                    reason,
                    issues: _,
                    suggestions: _,
                } => {
                    iteration_count += 1;
                    eprintln!(
                        "[ORCHESTRATOR] run_to_completion: Replan {} of {} requested: {}",
                        iteration_count, max_iterations, reason
                    );

                    if iteration_count >= max_iterations {
                        let reason = format!(
                            "Maximum iterations ({}) reached during replan",
                            max_iterations
                        );
                        eprintln!("[ORCHESTRATOR] run_to_completion: {}", reason);
                        return Ok(OrchestratorAction::GiveUp { reason });
                    }

                    // Reset state and continue
                    self.current_iteration += 1;
                    // The replan tool already reset state to Planning
                    // Just continue the loop
                    eprintln!("[ORCHESTRATOR] run_to_completion: Continuing to replan");
                    continue;
                }

                // Should not reach here - these are internal actions handled by run_until_action
                _ => {
                    eprintln!(
                        "[ORCHESTRATOR] run_to_completion: Unexpected action: {:?}",
                        action
                    );
                    return Err(format!("Unexpected action from orchestrator: {:?}", action));
                }
            }
        }
    }

    /// Execute a single tool
    pub(crate) async fn execute_tool(&mut self, tool_name: &str, input: &Value) -> ToolResult {
        match tool_name {
            "read_instruction_file" => self.tool_read_instruction_file(input).await,
            "create_skill" => self.tool_create_skill(input).await,
            "create_subagent" => self.tool_create_subagent(input).await,
            "generate_claudemd" => self.tool_generate_claudemd(input).await,
            "start_planning" => self.tool_start_planning(input).await,
            "approve_plan" => self.tool_approve_plan().await,
            "start_execution" => self.tool_start_execution(input).await,
            "start_verification" => self.tool_start_verification(input).await,
            "complete" => self.tool_complete().await,
            "iterate" => self.tool_iterate().await,
            "replan" => self.tool_replan().await,
            "give_up" => ToolResult::success("".to_string(), "Pipeline abandoned.".to_string()),
            _ => ToolResult::error("".to_string(), format!("Unknown tool: {}", tool_name)),
        }
    }
}
