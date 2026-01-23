// Auto-pipeline prompt templates
//
// Consolidated prompts for the automated development pipeline:
// - Templates: Shared constants, default instructions, and common prompt builders
// - Planning: Planning phase prompts (initial planning, replanning)
// - Building: Build/execution phase prompts
// - Verification: Verification and decision-making prompts

mod building;
mod planning;
mod templates;
mod verification;

// Re-export from templates module
pub use templates::{
    build_initial_prompt, DEFAULT_CUSTOM_INSTRUCTIONS, JSON_FORMAT_INSTRUCTION,
    QNA_GENERATION_PROMPT, TASK_REFINEMENT_PROMPT, WORKING_DIR_CONSTRAINT,
};

// Re-export from planning module
pub use planning::{build_planning_prompt, PLANNING_PROMPT_TEMPLATE, REPLAN_PROMPT_TEMPLATE};

// Re-export from building module
pub use building::{build_builder_prompt, BUILDER_PROMPT_TEMPLATE};

// Re-export from verification module
pub use verification::{
    build_verification_prompt, VERIFICATION_DECISION_PROMPT, VERIFIER_PROMPT_TEMPLATE,
};
