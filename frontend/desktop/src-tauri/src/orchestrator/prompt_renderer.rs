use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct UpstreamOutput {
    pub node_id: String,
    pub output: String,
}

#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    pub task: String,
    pub workspace_path: String,
    pub iteration_number: u32,
    pub max_iterations: u32,
    pub upstream_outputs: Vec<UpstreamOutput>,
    pub edge_inputs: HashMap<String, String>,
    pub extra_vars: HashMap<String, String>,
}

pub fn render_node_prompt(template: &str, context: &PromptContext) -> String {
    let vars = build_vars(context);
    crate::prompts::renderer::render_prompt(template, &vars)
}

fn build_vars(context: &PromptContext) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    vars.insert("task".into(), context.task.clone());
    vars.insert("workspace_path".into(), context.workspace_path.clone());
    vars.insert("iteration_number".into(), context.iteration_number.to_string());
    vars.insert("max_iterations".into(), context.max_iterations.to_string());

    let mut joined_outputs: Vec<String> = Vec::with_capacity(context.upstream_outputs.len());
    for upstream in &context.upstream_outputs {
        vars.insert(
            format!("upstream_output.{}", upstream.node_id),
            upstream.output.clone(),
        );
        joined_outputs.push(format!("[{}]\n{}", upstream.node_id, upstream.output));
    }
    vars.insert("upstream_outputs".into(), joined_outputs.join("\n\n"));

    if context.upstream_outputs.len() == 1 {
        vars.insert(
            "previous_output".into(),
            context.upstream_outputs[0].output.clone(),
        );
    } else {
        vars.insert("previous_output".into(), String::new());
    }

    for (key, value) in &context.edge_inputs {
        vars.insert(format!("edge_input.{}", key), value.clone());
    }

    for (key, value) in &context.extra_vars {
        vars.insert(key.clone(), value.clone());
    }

    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_upstream_output_and_previous_output_for_single_parent() {
        let context = PromptContext {
            task: "Fix auth bug".into(),
            workspace_path: "/repo".into(),
            iteration_number: 1,
            max_iterations: 3,
            upstream_outputs: vec![UpstreamOutput {
                node_id: "analyse".into(),
                output: "Root cause is token expiry handling".into(),
            }],
            edge_inputs: HashMap::new(),
            extra_vars: HashMap::new(),
        };

        let template = "Task: {{task}}\nPrev: {{previous_output}}\nByNode: {{upstream_output.analyse}}";
        let rendered = render_node_prompt(template, &context);
        assert!(rendered.contains("Task: Fix auth bug"));
        assert!(rendered.contains("Prev: Root cause is token expiry handling"));
        assert!(rendered.contains("ByNode: Root cause is token expiry handling"));
    }

    #[test]
    fn omits_previous_output_for_fan_in_and_renders_edge_inputs() {
        let mut edge_inputs = HashMap::new();
        edge_inputs.insert("security".into(), "Input from security reviewer".into());

        let context = PromptContext {
            task: "Harden login".into(),
            workspace_path: "/repo".into(),
            iteration_number: 2,
            max_iterations: 3,
            upstream_outputs: vec![
                UpstreamOutput {
                    node_id: "review-a".into(),
                    output: "Add CSRF validation".into(),
                },
                UpstreamOutput {
                    node_id: "review-b".into(),
                    output: "Rate-limit auth endpoint".into(),
                },
            ],
            edge_inputs,
            extra_vars: HashMap::new(),
        };

        let template = "Prev: {{previous_output}}\nAll: {{upstream_outputs}}\nSec: {{edge_input.security}}";
        let rendered = render_node_prompt(template, &context);
        assert!(rendered.contains("Prev: "));
        assert!(rendered.contains("[review-a]"));
        assert!(rendered.contains("[review-b]"));
        assert!(rendered.contains("Sec: Input from security reviewer"));
    }
}
