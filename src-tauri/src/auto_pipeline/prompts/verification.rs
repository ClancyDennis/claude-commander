// Verification-related prompts
//
// Prompts used during the verification phase and decision-making of the automated development pipeline.

/// Verifier step prompt template (legacy mode)
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

/// Build the verification agent prompt (v2 mode - OrchestratorAgent)
///
/// Used when spawning a Claude Code verification agent via the orchestrator.
pub fn build_verification_prompt(
    user_request: &str,
    current_plan: &str,
    current_implementation: &str,
    focus_section: &str,
    skills_section: &str,
    subagents_section: &str,
) -> String {
    format!(
        r#"You are verifying an implementation.

## USER REQUEST
{user_request}

## PLAN
{current_plan}

## IMPLEMENTATION SUMMARY
{current_implementation}
{focus_section}
{skills_section}
{subagents_section}

## YOUR TASK
Review the implementation and verify it works:
1. Read the created files
2. Check if the plan was followed
3. Test the implementation if possible (run it, check syntax, etc.)
4. Identify any issues or missing functionality
5. Verify that skill requirements (API keys, endpoints, configuration) are correctly used
6. Verify that subagent integrations work correctly if applicable

## OUTPUT FORMAT
Respond with valid JSON:
{{
  "overall_status": "success" | "partial" | "failed",
  "files_reviewed": ["file1.py", "file2.py"],
  "tests_performed": ["test 1 result", "test 2 result"],
  "issues_found": [
    {{
      "severity": "critical" | "major" | "minor",
      "description": "Issue description",
      "location": "file:line"
    }}
  ],
  "recommendations": ["recommendation 1", "recommendation 2"],
  "summary": "Overall assessment"
}}"#
    )
}
