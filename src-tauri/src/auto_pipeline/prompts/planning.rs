// Planning-related prompts
//
// Prompts used during the planning phase of the automated development pipeline.

/// Planning step prompt template (legacy mode)
/// Placeholders: {user_request}, {working_dir}
pub const PLANNING_PROMPT_TEMPLATE: &str = r#"You are coordinating a 3-step automated development pipeline.

IMPORTANT: Use the Plan agent (Task tool with subagent_type="Plan") to determine how to accomplish this task.

USER REQUEST:
{user_request}

WORKING DIRECTORY:
{working_dir}

Your responsibilities:
1. Launch a Plan agent to analyze the request and determine what needs to be done
2. The Plan agent will break down the task into concrete action steps
3. The Plan agent will generate clarifying questions
4. Once the Plan agent completes, extract its output and format as JSON

Use the Task tool like this:
- subagent_type: "Plan"
- prompt: "Determine how to accomplish this task: {user_request}. All work MUST be contained within {working_dir}. Break it into concrete action steps and generate clarifying questions about requirements, approach, and constraints."
- description: "Planning task approach"

After the Plan agent completes, format its output as JSON:
{{
  "plan": ["Step 1: ...", "Step 2: ..."],
  "questions": ["Question 1?", "Question 2?"]
}}
"#;

/// Replan step prompt template - used when verification decides to go back to planning
/// Placeholders: {user_request}, {previous_plan}, {qna}, {build_output}, {verification_output}, {issues_to_fix}, {suggestions}, {working_dir}
pub const REPLAN_PROMPT_TEMPLATE: &str = r#"You are coordinating a 3-step automated development pipeline. The previous attempt failed and we need to REPLAN.

IMPORTANT: Use the Plan agent (Task tool with subagent_type="Plan") to create a NEW implementation plan.

USER REQUEST:
{user_request}

WORKING DIRECTORY:
{working_dir}

=== PREVIOUS ATTEMPT (FAILED) ===

PREVIOUS PLAN:
{previous_plan}

Q&A FROM PREVIOUS ATTEMPT:
{qna}

BUILD OUTPUT:
{build_output}

VERIFICATION RESULT:
{verification_output}

ISSUES IDENTIFIED:
{issues_to_fix}

SUGGESTIONS FOR NEW APPROACH:
{suggestions}

=== END PREVIOUS ATTEMPT ===

Your responsibilities:
1. Analyze WHY the previous approach failed
2. Launch a Plan agent to create a NEW implementation plan that addresses the issues
3. The new plan should take a DIFFERENT approach - don't repeat the same mistakes
4. Generate new clarifying questions if needed

Use the Task tool like this:
- subagent_type: "Plan"
- prompt: "The previous implementation attempt failed. Create a NEW implementation plan for: {user_request}. The implementation MUST be contained within {working_dir}.

PREVIOUS ISSUES:
{issues_to_fix}

SUGGESTIONS:
{suggestions}

Create a plan that addresses these issues with a different approach. Break it into concrete steps and generate clarifying questions."
- description: "Replanning after failed attempt"

After the Plan agent completes, format its output as JSON:
{{
  "plan": ["Step 1: ...", "Step 2: ..."],
  "questions": ["Question 1?", "Question 2?"],
  "changes_from_previous": ["What's different: ..."]
}}"#;

/// Build the planning agent prompt (v2 mode - OrchestratorAgent)
///
/// Used when spawning a Claude Code planning agent via the orchestrator.
pub fn build_planning_prompt(
    user_request: &str,
    working_dir: &str,
    skills_section: &str,
    subagents_section: &str,
) -> String {
    format!(
        r#"You are planning an implementation for the following task.

## USER REQUEST
{user_request}

## WORKING DIRECTORY
{working_dir}
{skills_section}
{subagents_section}

## YOUR TASK
Create a detailed, actionable implementation plan. The plan must include:

1. **Directory Structure**: Exact files and folders to create
2. **Dependencies**: Required libraries, packages, or tools
3. **Implementation Steps**: Concrete steps with code locations
4. **Technical Approach**: Specific technologies, APIs, patterns to use
5. **Error Handling**: How to handle edge cases and errors
6. **Testing Plan**: How to verify the implementation works

## OUTPUT FORMAT
Respond with valid JSON:
{{
  "plan": [
    "Step 1: Create directory structure...",
    "Step 2: Install dependencies...",
    "Step 3: Implement core functionality..."
  ],
  "files_to_create": ["path/to/file1.py", "path/to/file2.py"],
  "dependencies": ["library1", "library2"],
  "technical_approach": "Description of the technical approach",
  "questions": [
    "Clarifying question 1?",
    "Clarifying question 2?"
  ]
}}

Be specific and actionable - each step should be clear enough that another agent can implement it."#
    )
}
