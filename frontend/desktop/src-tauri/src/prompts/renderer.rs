use std::collections::HashMap;

/// Renders a prompt template by substituting `{{variable}}` placeholders and
/// evaluating `{{#if variable}}...{{/if}}` conditional blocks.
///
/// - Unresolved variables are replaced with an empty string.
/// - Conditional blocks are omitted if the variable is empty or missing.
/// - Unmatched `{{#if ...}}` without a closing `{{/if}}` is left as-is.
pub fn render_prompt(template: &str, vars: &HashMap<String, String>) -> String {
    let after_conditionals = resolve_conditionals(template, vars);
    resolve_variables(&after_conditionals, vars)
}

/// Resolves `{{#if var}}...{{/if}}` blocks.
fn resolve_conditionals(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let mut remaining = template;

    while let Some(if_start) = remaining.find("{{#if ") {
        let after_tag = &remaining[if_start + 6..];
        let Some(tag_end) = after_tag.find("}}") else {
            result.push_str(&remaining[..if_start + 6]);
            remaining = after_tag;
            continue;
        };

        let var_name = after_tag[..tag_end].trim();
        let block_start = if_start + 6 + tag_end + 2;

        let Some(endif_pos) = remaining[block_start..].find("{{/if}}") else {
            // No closing tag — leave entire remainder as-is (graceful)
            result.push_str(remaining);
            return result;
        };
        let endif_abs = block_start + endif_pos;
        let after_endif = endif_abs + 7;

        result.push_str(&remaining[..if_start]);

        let value = vars.get(var_name).map(|s| s.as_str()).unwrap_or("");
        if !value.is_empty() {
            result.push_str(&remaining[block_start..endif_abs]);
        }

        remaining = &remaining[after_endif..];
    }

    result.push_str(remaining);
    result
}

/// Replaces `{{variable}}` placeholders with values from the map.
/// Missing variables resolve to empty string.
fn resolve_variables(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let mut remaining = template;

    while let Some(start) = remaining.find("{{") {
        result.push_str(&remaining[..start]);
        let after_open = &remaining[start + 2..];

        if let Some(end) = after_open.find("}}") {
            let var_name = after_open[..end].trim();
            // Leave leftover conditional tags as-is
            if var_name.starts_with("#if ") || var_name.starts_with("/if") {
                result.push_str("{{");
                result.push_str(&after_open[..end + 2]);
                remaining = &after_open[end + 2..];
                continue;
            }
            let value = vars.get(var_name).map(|s| s.as_str()).unwrap_or("");
            result.push_str(value);
            remaining = &after_open[end + 2..];
        } else {
            result.push_str("{{");
            remaining = after_open;
        }
    }

    result.push_str(remaining);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn simple_variable_substitution() {
        let result = render_prompt("Hello {{name}}!", &vars(&[("name", "World")]));
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn multiple_variables() {
        let template = "Task: {{task}}\nPath: {{workspace_path}}";
        let v = vars(&[("task", "fix bug"), ("workspace_path", "/home/user/project")]);
        let result = render_prompt(template, &v);
        assert_eq!(result, "Task: fix bug\nPath: /home/user/project");
    }

    #[test]
    fn missing_variable_resolves_to_empty() {
        let result = render_prompt("Value: {{missing}}", &HashMap::new());
        assert_eq!(result, "Value: ");
    }

    #[test]
    fn conditional_with_populated_variable() {
        let template = "Start\n{{#if code_context}}Code: {{code_context}}{{/if}}\nEnd";
        let v = vars(&[("code_context", "fn main() {}")]);
        let result = render_prompt(template, &v);
        assert_eq!(result, "Start\nCode: fn main() {}\nEnd");
    }

    #[test]
    fn conditional_with_missing_variable_omits_block() {
        let template = "Start\n{{#if code_context}}Code: {{code_context}}{{/if}}\nEnd";
        let result = render_prompt(template, &HashMap::new());
        assert_eq!(result, "Start\n\nEnd");
    }

    #[test]
    fn conditional_with_empty_variable_omits_block() {
        let template = "Start\n{{#if test_results}}Results: {{test_results}}{{/if}}\nEnd";
        let v = vars(&[("test_results", "")]);
        let result = render_prompt(template, &v);
        assert_eq!(result, "Start\n\nEnd");
    }

    #[test]
    fn mixed_variables_and_conditionals() {
        let template =
            "Task: {{task}}\n{{#if git_branch}}Branch: {{git_branch}}\n{{/if}}Path: {{workspace_path}}";
        let v = vars(&[
            ("task", "add feature"),
            ("git_branch", "main"),
            ("workspace_path", "/project"),
        ]);
        let result = render_prompt(template, &v);
        assert_eq!(result, "Task: add feature\nBranch: main\nPath: /project");
    }

    #[test]
    fn unmatched_if_without_endif_left_as_is() {
        let template = "Before {{#if var}}content without endif";
        let v = vars(&[("var", "yes")]);
        let result = render_prompt(template, &v);
        assert_eq!(result, "Before {{#if var}}content without endif");
    }

    #[test]
    fn handles_empty_template() {
        assert_eq!(render_prompt("", &HashMap::new()), "");
    }
}
