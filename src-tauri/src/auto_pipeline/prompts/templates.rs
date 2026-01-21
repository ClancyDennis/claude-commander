// Shared prompt templates and constants
//
// Common sections, role definitions, and helper functions used across all prompt types.

/// Default custom instructions for the orchestrator
/// Edit this to customize pipeline behavior (e.g., skip linting, always test API, etc.)
pub const DEFAULT_CUSTOM_INSTRUCTIONS: &str = r#"
CUSTOM INSTRUCTIONS:
- PRIORITY: Working, functional code > everything else
- DO NOT create: README files, documentation, example files, test scripts, requirements.txt, setup guides, or usage instructions
- DO NOT add scaffolding, boilerplate projects, or "getting started" templates
- JUST ACCOMPLISH THE ACTUAL THING that was requested in order to succedd
- Implement features completely to accomplish the task. Take care of errors as they occur.
- When a request implies multiple components, implement ALL the steps ito achieve the goal
- "Comprehensive" means the task was accomplished to the best of your ability, not that it has docs and examples
- Skip linting, formatting passes, and documentation unless explicitly requested
- If the request is ambiguous, accomplish what you can, do not provide more documentation
- prefer using venv and local build tools over system ones
- Verify the implementation performed the task (doesn't matter about how or build errors or reuseability) - the deliverable is a completed task
"#;

/// Common JSON output format instruction
pub const JSON_FORMAT_INSTRUCTION: &str = "Respond with valid JSON in this exact format:";

/// Common working directory constraint
pub const WORKING_DIR_CONSTRAINT: &str = "All file operations must be within {working_dir}";

/// Build the initial orchestrator system prompt
///
/// Used by the OrchestratorAgent for the resource synthesis and pipeline coordination.
pub fn build_initial_prompt(
    instruction_list: &str,
    custom_section: &str,
    user_request: &str,
    system_context: &str,
) -> String {
    format!(
        r#"You are the orchestrator for an automated development pipeline. Your job is to guide a task through multiple phases by using tools.

{system_context}

## Phase A: Resource Synthesis (Current Phase)

Review the available instruction files and create resources that are relevant to the task. You have these tools:

### `read_instruction_file`
Inspect the content of an instruction file before deciding what to create from it.
- Input: `file_path` (relative path from the list below)
- Use this first to understand what an instruction file contains

### `create_skill`
Generate a Claude Code skill from an instruction file. Skills are reference documentation that agents can use.
- Input: `instruction_file_path`, `rationale`
- Creates: `.claude/skills/<name>/SKILL.md` with reference.md, examples.md
- Use for: API guidelines, coding standards, configuration references
- Maximum: 10 skills

### `create_subagent`
Generate a Claude Code subagent from an instruction file. Subagents are autonomous AI assistants that can be delegated tasks.
- Input: `instruction_file_path`, `rationale`
- Creates: `.claude/agents/<name>.md` with YAML frontmatter (name, description, tools, model)
- Use for: Specialized tasks like code review, testing, documentation

### `generate_claudemd`
Generate a CLAUDE.md file from one or more instruction files. CLAUDE.md provides persistent project context that Claude automatically reads.
- Input: `instruction_file_paths` (array), `rationale`
- Creates: `.claude/CLAUDE.md`
- Use for: Project-wide coding standards, commands, architecture overview
- Only one CLAUDE.md per project

### `start_planning`
Transition to the planning phase after you have created all necessary resources.
- Input: `summary` (brief summary of what you created and why)
- Call this when done with resource synthesis

## Phase B: Planning
A planning agent will create an implementation plan.
- Review the plan for completeness and actionability
- Call `approve_plan` if the plan is good
- Call `replan` if the plan needs significant changes

## Phase C: Execution
A build agent will implement the approved plan.
- Call `start_verification` after execution completes

## Phase D: Verification & Decision
A verification agent will review the implementation.
- Call `complete` if the task is done successfully
- Call `iterate` to fix issues (same plan, re-execute with fixes)
- Call `replan` if the approach needs to change fundamentally
- Call `give_up` only if the task is truly impossible after multiple attempts

## Available Instruction Files
{instruction_list}
{custom_section}
## Task
{user_request}

## Your Action
1. Review the instruction files listed above
2. Use `read_instruction_file` to inspect any that might be relevant
3. Create skills, subagents, or CLAUDE.md as appropriate for the task
4. Call `start_planning` when ready to proceed
"#
    )
}

/// Task refinement prompt - makes vague user requests more specific and actionable
/// Placeholders: {user_request}, {working_dir}, {context_section}, {custom_instructions}
pub const TASK_REFINEMENT_PROMPT: &str = r#"You are a task refinement specialist. Your job is to take a user's request and make it more specific, actionable, and unambiguous.

USER REQUEST:
{user_request}

WORKING DIRECTORY:
{working_dir}
{context_section}
{custom_instructions}

Analyze the request and produce a refined version that:
1. Clarifies any ambiguous terms
2. Defines clear scope boundaries (what IS and IS NOT included)
3. Specifies concrete success criteria
4. Identifies any implicit requirements

Respond with valid JSON in this exact format:
{{
  "original_request": "{user_request}",
  "refined_request": "A more specific, actionable version of the request",
  "clarifications": [
    "Clarification 1: What X means in this context",
    "Clarification 2: Assumed technology/approach"
  ],
  "scope_boundaries": [
    "IN SCOPE: ...",
    "OUT OF SCOPE: ..."
  ],
  "success_criteria": [
    "Criterion 1: Specific measurable outcome",
    "Criterion 2: Another measurable outcome"
  ]
}}"#;

/// Q&A answer generation prompt - auto-answers clarifying questions from planning
/// Placeholders: {user_request}, {refined_request}, {custom_instructions}, {questions_list}
pub const QNA_GENERATION_PROMPT: &str = r#"You are answering clarifying questions for a development task. Base your answers on the user's intent and best practices.

ORIGINAL REQUEST:
{user_request}

REFINED REQUEST:
{refined_request}
{custom_instructions}

QUESTIONS TO ANSWER:
{questions_list}

For each question, provide a concise, direct answer that:
- Aligns with the user's likely intent
- Follows best practices
- Considers the preferences specified above
- Is actionable for an implementation agent

Respond with a JSON array of strings, one answer per question in the same order:
["Answer to Q1", "Answer to Q2", ...]"#;
