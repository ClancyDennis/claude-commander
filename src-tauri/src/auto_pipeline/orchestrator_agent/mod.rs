// Orchestrator Agent
//
// Persistent tool-calling agent for pipeline orchestration.
// This agent maintains a consistent message buffer throughout the entire pipeline,
// using tools to:
// - Read and create skills from instruction files
// - Spawn planning, build, and verification agents
// - Make final decisions (complete, iterate, replan, give_up)

mod context_builders;
mod tool_loop;
mod tools;
mod types;

pub use types::OrchestratorAction;

use context_builders::build_system_context;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::agent_manager::AgentManager;
use crate::ai_client::{AIClient, Tool};
use crate::auto_pipeline::advisor::Advisor;
use crate::events::AppEventEmitter;
use crate::instruction_manager::{list_instruction_files, InstructionFileInfo};

use super::orchestrator_tools::get_tools_for_state;
use super::prompts::build_initial_prompt;
use super::state_machine::PipelineState;

use types::{ConversationContent, ConversationMessage};

/// The persistent orchestrator agent
pub struct OrchestratorAgent {
    pub(crate) ai_client: AIClient,
    /// The conversation history - maintained throughout the pipeline
    pub(crate) messages: Vec<ConversationMessage>,
    /// Available tools
    pub(crate) tools: Vec<Tool>,
    /// Working directory for the pipeline
    pub(crate) working_dir: String,
    /// Original user request
    pub(crate) user_request: String,
    /// Available instruction files (cached)
    pub(crate) instruction_files: Vec<InstructionFileInfo>,
    /// Skills that have been generated
    pub(crate) generated_skills: Vec<String>,
    /// Subagents that have been generated
    pub(crate) generated_subagents: Vec<String>,
    /// Whether CLAUDE.md has been generated
    pub(crate) claudemd_generated: bool,
    /// Current pipeline state
    pub(crate) current_state: PipelineState,
    /// Custom instructions for the orchestrator
    #[allow(dead_code)]
    pub(crate) custom_instructions: Option<String>,
    /// Maximum iterations
    pub(crate) max_iterations: u8,
    /// Current iteration count
    pub(crate) current_iteration: u8,
    /// Agent manager for spawning Claude Code agents
    pub(crate) agent_manager: Option<Arc<Mutex<AgentManager>>>,
    /// Event emitter for agent events
    pub(crate) event_emitter: Option<Arc<dyn AppEventEmitter>>,
    /// Current plan output (stored for execution phase)
    pub(crate) current_plan: String,
    /// Current Q&A (stored for execution phase)
    pub(crate) current_qna: String,
    /// Current implementation output (stored for verification phase)
    pub(crate) current_implementation: String,
    /// Agent outputs from the planning phase
    pub(crate) planning_agent_outputs: Vec<crate::types::AgentOutputEvent>,
    /// Agent outputs from the building phase
    pub(crate) building_agent_outputs: Vec<crate::types::AgentOutputEvent>,
    /// Agent outputs from the verification phase
    pub(crate) verification_agent_outputs: Vec<crate::types::AgentOutputEvent>,
    /// Number of times replan has been used during the planning phase
    pub(crate) planning_replan_count: u8,
    /// Maximum allowed replans during planning phase (0 = unlimited)
    pub(crate) max_planning_replans: u8,
    /// Pipeline ID for linking spawned agents to this pipeline
    pub(crate) pipeline_id: String,
    /// Spawned agent IDs for tracking: [planning, building, verification]
    pub(crate) spawned_agents: [Option<String>; 3],
    /// Optional external advisor (e.g., Gemini) that reviews each step
    pub(crate) advisor: Option<Advisor>,
}

impl OrchestratorAgent {
    /// Create a new orchestrator agent (without agent spawning - for testing)
    pub fn new(
        working_dir: String,
        user_request: String,
        custom_instructions: Option<String>,
        max_iterations: u8,
    ) -> Result<Self, String> {
        Self::create(
            working_dir,
            user_request,
            custom_instructions,
            max_iterations,
            None,
            None,
            None,
        )
    }

    /// Create a new orchestrator agent with agent manager (full integration)
    pub fn with_agent_manager(
        working_dir: String,
        user_request: String,
        custom_instructions: Option<String>,
        max_iterations: u8,
        agent_manager: Arc<Mutex<AgentManager>>,
        event_emitter: Arc<dyn AppEventEmitter>,
        pipeline_id: String,
    ) -> Result<Self, String> {
        Self::create(
            working_dir,
            user_request,
            custom_instructions,
            max_iterations,
            Some(agent_manager),
            Some(event_emitter),
            Some(pipeline_id),
        )
    }

    /// Internal constructor
    fn create(
        working_dir: String,
        user_request: String,
        custom_instructions: Option<String>,
        max_iterations: u8,
        agent_manager: Option<Arc<Mutex<AgentManager>>>,
        event_emitter: Option<Arc<dyn AppEventEmitter>>,
        pipeline_id: Option<String>,
    ) -> Result<Self, String> {
        // Prefer OpenAI for orchestration (falls back to any available provider)
        let ai_client = AIClient::openai_from_env()
            .or_else(|_| AIClient::from_env())
            .map_err(|e| format!("Failed to create AI client: {}", e))?;

        // Initialize the external advisor (Gemini by default) if configured
        let advisor = Advisor::from_env();

        // Load available instruction files
        let instruction_files = list_instruction_files(&working_dir).unwrap_or_default();

        // Start with tools for the initial state (ReceivedTask)
        let initial_state = PipelineState::ReceivedTask;
        let tool_defs = get_tools_for_state(&initial_state);
        let tool_names: Vec<&str> = tool_defs.iter().map(|t| t.name.as_str()).collect();
        eprintln!(
            "[ORCHESTRATOR] Initial state: {:?} -> Available tools: {:?}",
            initial_state, tool_names
        );

        let tools: Vec<Tool> = tool_defs
            .into_iter()
            .map(|t| Tool {
                name: t.name,
                description: t.description,
                input_schema: t.input_schema,
            })
            .collect();

        // Build initial system context
        let instruction_list = if instruction_files.is_empty() {
            "No instruction files available.".to_string()
        } else {
            instruction_files
                .iter()
                .map(|f| format!("- {} ({})", f.relative_path, f.file_type))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let custom_section = custom_instructions
            .as_ref()
            .map(|i| format!("\n## Custom Instructions\n{}\n", i))
            .unwrap_or_default();

        let system_context = build_system_context();
        let initial_prompt = build_initial_prompt(
            &instruction_list,
            &custom_section,
            &user_request,
            &system_context,
        );

        let messages = vec![ConversationMessage {
            role: "user".to_string(),
            content: ConversationContent::Text(initial_prompt),
        }];

        Ok(Self {
            ai_client,
            messages,
            tools,
            working_dir,
            user_request,
            instruction_files,
            generated_skills: Vec::new(),
            generated_subagents: Vec::new(),
            claudemd_generated: false,
            current_state: PipelineState::ReceivedTask,
            custom_instructions,
            max_iterations,
            current_iteration: 1,
            agent_manager,
            event_emitter,
            current_plan: String::new(),
            current_qna: String::new(),
            current_implementation: String::new(),
            planning_agent_outputs: Vec::new(),
            building_agent_outputs: Vec::new(),
            verification_agent_outputs: Vec::new(),
            planning_replan_count: 0,
            max_planning_replans: 1, // Default: allow 1 replan during planning
            pipeline_id: pipeline_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            spawned_agents: [None, None, None],
            advisor,
        })
    }

    /// Get the current state
    pub fn current_state(&self) -> &PipelineState {
        &self.current_state
    }

    /// Get generated skills
    pub fn generated_skills(&self) -> &[String] {
        &self.generated_skills
    }

    /// Get generated subagents
    pub fn generated_subagents(&self) -> &[String] {
        &self.generated_subagents
    }

    /// Get current iteration
    pub fn current_iteration(&self) -> u8 {
        self.current_iteration
    }

    /// Increment iteration counter
    pub fn increment_iteration(&mut self) {
        self.current_iteration += 1;
    }

    /// Check if at max iterations
    pub fn at_max_iterations(&self) -> bool {
        self.current_iteration >= self.max_iterations
    }

    /// Get the current step number based on pipeline state (1=Planning, 2=Building, 3=Verifying)
    pub fn get_current_step_number(&self) -> u8 {
        match &self.current_state {
            PipelineState::ReceivedTask
            | PipelineState::AnalyzingTask
            | PipelineState::SelectingInstructions
            | PipelineState::GeneratingSkills
            | PipelineState::Planning
            | PipelineState::PlanReady
            | PipelineState::PlanRevisionRequired => 1,
            PipelineState::ReadyForExecution | PipelineState::Executing => 2,
            PipelineState::Verifying
            | PipelineState::VerificationPassed
            | PipelineState::VerificationFailed => 3,
            PipelineState::Completed | PipelineState::Failed | PipelineState::GaveUp => 0,
        }
    }

    /// Consult the external advisor (if configured) and inject its advice into the conversation.
    /// Called after each major step (planning, execution, verification) completes.
    pub async fn consult_advisor(&mut self, step_output: &str) {
        let advisor = match &self.advisor {
            Some(a) => a,
            None => return,
        };

        let phase_name = self.current_state.phase_name();
        let available_actions: Vec<&str> = self
            .tools
            .iter()
            .map(|t| t.name.as_str())
            .collect();

        // Emit event that we're consulting the advisor
        if let Some(ref emitter) = self.event_emitter {
            let (provider, model) = advisor.model_info();
            let _ = emitter.emit(
                "orchestrator:advisor_consulting",
                serde_json::json!({
                    "phase": phase_name,
                    "provider": provider,
                    "model": model,
                }),
            );
        }

        eprintln!(
            "[ADVISOR] Consulting advisor after {} phase (iteration {}/{})",
            phase_name, self.current_iteration, self.max_iterations
        );

        // Truncate step output for advisor to avoid excessive token usage
        let truncated_output = if step_output.len() > 8000 {
            format!("{}...\n[truncated, {} total chars]", &step_output[..8000], step_output.len())
        } else {
            step_output.to_string()
        };

        match advisor
            .consult(
                &self.user_request,
                phase_name,
                &truncated_output,
                &available_actions,
                self.current_iteration,
                self.max_iterations,
            )
            .await
        {
            Ok(advice) => {
                eprintln!(
                    "[ADVISOR] Received advice from {} ({}+{} tokens): {}",
                    advice.model,
                    advice.input_tokens,
                    advice.output_tokens,
                    &advice.recommendation[..advice.recommendation.len().min(200)]
                );

                // Inject advisor recommendation into the orchestrator's conversation
                let advisor_context = format!(
                    "[EXTERNAL ADVISOR RECOMMENDATION]\nThe following advice comes from an external AI advisor ({}) reviewing the {} phase output:\n\n{}\n\n[END ADVISOR RECOMMENDATION]\n\nConsider this advice when deciding your next action, but use your own judgment.",
                    advice.model,
                    phase_name,
                    advice.recommendation
                );

                self.messages.push(ConversationMessage {
                    role: "user".to_string(),
                    content: ConversationContent::Text(advisor_context),
                });

                // Emit event with the advice
                if let Some(ref emitter) = self.event_emitter {
                    let _ = emitter.emit(
                        "orchestrator:advisor_advice",
                        serde_json::json!({
                            "phase": phase_name,
                            "model": advice.model,
                            "recommendation": advice.recommendation,
                            "input_tokens": advice.input_tokens,
                            "output_tokens": advice.output_tokens,
                        }),
                    );
                }
            }
            Err(e) => {
                eprintln!("[ADVISOR] Consultation failed: {}", e);
                // Non-fatal - the orchestrator continues without advice
                if let Some(ref emitter) = self.event_emitter {
                    let _ = emitter.emit(
                        "orchestrator:advisor_error",
                        serde_json::json!({
                            "phase": phase_name,
                            "error": e,
                        }),
                    );
                }
            }
        }
    }

    /// Add a message to the conversation (e.g., agent output)
    pub fn add_context(&mut self, role: &str, content: &str) {
        self.messages.push(ConversationMessage {
            role: role.to_string(),
            content: ConversationContent::Text(content.to_string()),
        });
    }

    /// Update the current state and refresh available tools
    pub fn set_state(&mut self, state: PipelineState) {
        let old_state = self.current_state.clone();
        self.current_state = state.clone();
        self.refresh_tools();

        // Emit state change event
        if let Some(ref emitter) = self.event_emitter {
            let _ = emitter.emit(
                "orchestrator:state_changed",
                serde_json::json!({
                    "old_state": format!("{:?}", old_state),
                    "new_state": format!("{:?}", state),
                    "iteration": self.current_iteration,
                    "generated_skills": self.generated_skills.len(),
                    "generated_subagents": self.generated_subagents.len(),
                    "claudemd_generated": self.claudemd_generated,
                }),
            );
        }
    }

    /// Refresh the available tools based on current state
    pub(crate) fn refresh_tools(&mut self) {
        let tool_defs = get_tools_for_state(&self.current_state);

        // Filter out replan if max replans reached during planning phase
        let is_planning_phase = matches!(
            self.current_state,
            PipelineState::Planning
                | PipelineState::PlanReady
                | PipelineState::PlanRevisionRequired
        );
        let replan_limit_reached = self.max_planning_replans > 0
            && self.planning_replan_count >= self.max_planning_replans;
        let tool_defs: Vec<_> = if is_planning_phase && replan_limit_reached {
            tool_defs
                .into_iter()
                .filter(|t| t.name != "replan")
                .collect()
        } else {
            tool_defs
        };

        let tool_names: Vec<&str> = tool_defs.iter().map(|t| t.name.as_str()).collect();
        eprintln!(
            "[ORCHESTRATOR] State: {:?} -> Available tools: {:?}",
            self.current_state, tool_names
        );

        self.tools = tool_defs
            .into_iter()
            .map(|t| Tool {
                name: t.name,
                description: t.description,
                input_schema: t.input_schema,
            })
            .collect();
    }
}
