// Auto-pipeline prompt templates
//
// These prompts define the behavior of the 3-step automated development pipeline:
// 1. Planning - Creates implementation plan and generates clarifying questions
// 2. Building - Implements the solution based on the plan and Q&A
// 3. Verifying - Reviews implementation and generates verification report
//
// Orchestrator prompts (used by the Orchestrator LLM for decisions):
// - Task refinement - Make vague requests specific
// - Q&A generation - Auto-answer clarifying questions
// - Verification decision - Decide complete/iterate/replan/give_up

/// Planning step prompt template
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

/// Builder step prompt template
/// Placeholders: {user_request}, {plan}, {qna}, {working_dir}
pub const BUILDER_PROMPT_TEMPLATE: &str = r#"You are an expert builder agent in a 3-step automated development pipeline.

Your role is to complete the task based on the plan and clarifying question answers.

ORIGINAL USER REQUEST:
{user_request}

WORKING DIRECTORY:
{working_dir}

ACTION PLAN (from Planning Agent):
{plan}

CLARIFYING QUESTIONS & ANSWERS:
{qna}

Your responsibilities:
1. Execute ALL steps from the plan in the correct order
2. Follow the guidance from the Q&A answers
3. Write production-quality code with proper error handling
4. Ensure code is well-structured and follows project conventions
5. Create or modify files as specified in the plan
6. Add comments only where logic is non-obvious

Execution Guidelines:
- All file operations must be within {working_dir}
- Read existing files first to understand patterns and conventions
- Follow the plan sequentially - complete each step before moving to the next
- Use the Q&A answers to guide technical decisions
- Write clean, maintainable code without over-engineering
- Don't add features beyond what was requested
- Test as you go (read outputs, check syntax)
- If you encounter issues, adapt intelligently

Output Format:
After completing the task, provide a brief summary:

```
TASK COMPLETE

Files Modified:
- [list each file you created or modified]

Summary:
[2-3 sentence summary of what was accomplished]

Key Decisions:
- [Decision 1 and rationale]
- [Decision 2 and rationale]

Verification Notes:
[Any manual testing you performed or suggestions for testing]
```

Important:
- Focus on completing the task, not perfecting the code
- The plan is your guide - follow it closely
- Use the Q&A to resolve any ambiguities
- Write code that works, not code that's perfect
- If something in the plan doesn't fit the codebase, use your judgment"#;

/// Verifier step prompt template
/// Placeholders: {user_request}, {plan}, {qna}, {implementation}
pub const VERIFIER_PROMPT_TEMPLATE: &str = r#"You are an expert verification agent in a 3-step automated development pipeline.

Your role is to verify that the task was completed successfully and generate a verification report.

ORIGINAL USER REQUEST:
{user_request}

ACTION PLAN (from Planning Agent):
{plan}

CLARIFYING Q&A:
{qna}

BUILD SUMMARY (from Builder Agent):
{implementation}

Your responsibilities:
1. Review all files that were created or modified
2. Verify the work matches what the plan specified
3. Check for code quality issues
4. Identify any gaps or missing functionality
5. Test if possible
6. Provide actionable recommendations

Verification Process:
1. Read all modified files to understand what was built
2. Cross-reference against the plan (check each step was completed)
3. Review code quality: error handling, edge cases, maintainability
4. Check if Q&A answers were properly incorporated
5. Look for potential bugs or issues
6. Verify files compile/run (use appropriate tools: cargo check, npm run build, etc.)
7. Consider testing: does it work as expected?

Output Format (MUST be valid JSON):
{{
  "overall_status": "success" | "partial" | "failed",
  "plan_adherence": {{
    "steps_completed": ["Step 1", "Step 2"],
    "steps_incomplete": ["Step X"],
    "steps_modified": ["Step Y was adapted because..."]
  }},
  "code_quality": {{
    "strengths": ["Good error handling", "Clean abstractions"],
    "weaknesses": ["Missing edge case handling for X", "Could improve Y"]
  }},
  "issues_found": [
    {{
      "severity": "critical" | "major" | "minor",
      "description": "Detailed issue description",
      "location": "file.rs:123",
      "impact": "What this affects"
    }}
  ],
  "testing_performed": {{
    "compilation": "pass" | "fail" | "not_tested",
    "manual_tests": ["Test 1: result", "Test 2: result"],
    "suggestions": ["How to test this feature"]
  }},
  "qna_incorporation": {{
    "q1": "Answer was addressed by...",
    "q2": "Answer was addressed through..."
  }},
  "recommendations": [
    {{
      "priority": "high" | "medium" | "low",
      "category": "bug_fix" | "enhancement" | "documentation" | "testing",
      "description": "Specific actionable recommendation"
    }}
  ],
  "files_reviewed": ["file1.rs", "file2.ts"],
  "summary": "2-3 sentence overall assessment of whether the task was completed successfully"
}}

Guidelines:
- Be thorough but fair in your assessment
- Focus on correctness and completeness first, style second
- Provide specific, actionable recommendations
- Include file paths and line numbers where relevant
- Test compilation if possible (use cargo check, npm build, etc.)
- Consider both what was done and what might be missing
- Acknowledge good work, but don't hesitate to flag real issues"#;

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

// =============================================================================
// ORCHESTRATOR PROMPTS - Used by the Orchestrator LLM for pipeline decisions
// =============================================================================

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

/// Verification decision prompt - decides what to do after verification step
/// Placeholders: {user_request}, {plan}, {qna}, {build_output}, {verification_output},
///               {iteration_count}, {max_iterations}, {previous_issues_section}, {custom_instructions}
pub const VERIFICATION_DECISION_PROMPT: &str = r#"You are the orchestrator deciding the next action for a development pipeline.

TASK:
{user_request}

PLAN:
{plan}

Q&A:
{qna}

BUILD OUTPUT:
{build_output}

VERIFICATION RESULT:
{verification_output}

ITERATION: {iteration_count} of {max_iterations}
{previous_issues_section}
{custom_instructions}

Analyze the verification result and decide the next action:

1. **complete** - Task is done successfully. Use when:
   - All success criteria are met
   - Tests pass (if applicable)
   - No critical issues remain

2. **iterate** - Need minor fixes, try building again. Use when:
   - Small bugs or issues found
   - Implementation is mostly correct but needs tweaks
   - Errors are fixable without changing the approach

3. **replan** - Fundamental approach is wrong, need new plan. Use when:
   - Core architecture doesn't work
   - Same errors keep repeating (thrashing)
   - Approach conflicts with codebase patterns

4. **give_up** - Cannot complete, need human intervention. Use when:
   - Max iterations reached with no progress
   - Requires external dependencies/access not available
   - Conflicting requirements that can't be resolved

Consider:
- Are we making progress or thrashing?
- Is this a minor fix or fundamental issue?
- Have we seen the same errors before?
- Are we within iteration limits?

Respond with valid JSON:
{{
  "decision": "complete" | "iterate" | "replan" | "give_up",
  "reasoning": "Detailed explanation of why this decision was made",
  "issues_to_fix": ["Issue 1 to address", "Issue 2 to address"],
  "suggestions": ["Suggestion for next iteration", "Another suggestion"]
}}"#;
