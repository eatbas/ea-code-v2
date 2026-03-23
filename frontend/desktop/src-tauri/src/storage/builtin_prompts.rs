/// Shared prompt templates used by built-in pipeline definitions.

pub const ANALYSE_PROMPT: &str = "\
Analyse the codebase for {{task}}. Workspace: {{workspace_path}}\n\n\
{{#if code_context}}Relevant code:\n{{code_context}}{{/if}}\n\n\
Provide a detailed analysis with specific file paths and line numbers.";

pub const REVIEW_PROMPT: &str = "\
Review the analysis and refine the implementation plan.\n\n\
Previous analysis:\n{{previous_output}}\n\n\
{{#if test_results}}Previous test results:\n{{test_results}}{{/if}}\n\n\
{{#if judge_feedback}}Judge feedback:\n{{judge_feedback}}{{/if}}";

pub const IMPLEMENT_PROMPT: &str = "\
Implement the changes for: {{task}}\n\n\
Plan from review:\n{{previous_output}}\n\n\
Workspace: {{workspace_path}}\n\
{{#if git_branch}}Branch: {{git_branch}}{{/if}}\n\n\
Write clean, tested code. Follow existing patterns.";

pub const TEST_PROMPT: &str = "\
Run the test suite and verify the implementation.\n\n\
{{#if previous_output}}Implementation summary:\n{{previous_output}}{{/if}}\n\n\
{{#if git_diff}}Changes made:\n{{git_diff}}{{/if}}";

pub const QF_IMPLEMENT_PROMPT: &str = "\
Fix this issue: {{task}}\n\n\
Workspace: {{workspace_path}}\n\
{{#if code_context}}Context:\n{{code_context}}{{/if}}\n\n\
Make minimal, focused changes.";

pub const QF_TEST_PROMPT: &str = "\
Verify the fix works.\n\n\
{{#if git_diff}}Changes:\n{{git_diff}}{{/if}}";

pub const RO_ANALYSE_PROMPT: &str = "\
Research and analyse: {{task}}\n\n\
Workspace: {{workspace_path}}\n\
{{#if file_list}}Files:\n{{file_list}}{{/if}}\n\
{{#if code_context}}Code:\n{{code_context}}{{/if}}";

pub const RO_REVIEW_PROMPT: &str = "\
Refine the analysis.\n\n\
Previous findings:\n{{previous_output}}\n\n\
Provide actionable recommendations.";

pub const MBR_GEMINI_REVIEW: &str = "\
Provide an independent code review perspective.\n\n\
Analysis:\n{{previous_output}}\n\n\
Focus on patterns the first reviewer may have missed.";

pub const MBR_CODEX_REVIEW: &str = "\
Provide a third perspective on this codebase.\n\n\
Previous reviews:\n{{previous_output}}\n\n\
Focus on edge cases, error handling, and testing gaps.";

pub const SECURITY_REVIEW_PROMPT: &str = "\
Perform a security audit focused on OWASP Top 10.\n\n\
Analysis:\n{{previous_output}}\n\n\
Check for: injection, auth flaws, XSS, insecure deserialization, misconfigurations.";
