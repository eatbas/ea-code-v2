use std::collections::HashMap;

use crate::models::templates::TemplateNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionCandidate {
    pub source_node_id: String,
    pub session_group: String,
    pub provider: String,
    pub model: String,
    pub provider_session_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionDecision {
    New {
        warning: Option<String>,
    },
    Resume(String),
}

#[derive(Debug, Clone)]
struct SessionEntry {
    provider: String,
    model: String,
    provider_session_ref: String,
}

#[derive(Debug, Default, Clone)]
pub struct SessionManager {
    refs_by_group: HashMap<String, SessionEntry>,
}

impl SessionManager {
    pub fn decide_mode(&self, node: &TemplateNode, inbound: &[SessionCandidate]) -> SessionDecision {
        for candidate in inbound {
            if candidate.session_group == node.session_group
                && candidate.provider == node.provider
                && candidate.model == node.model
            {
                return SessionDecision::Resume(candidate.provider_session_ref.clone());
            }
        }

        let inbound_warning = inbound.iter().find_map(|candidate| {
            if candidate.session_group != node.session_group {
                return None;
            }
            if candidate.provider != node.provider || candidate.model != node.model {
                Some(format!(
                    "Session fallback to new: inbound '{}' uses {}/{} but node '{}' requires {}/{}.",
                    candidate.source_node_id,
                    candidate.provider,
                    candidate.model,
                    node.id,
                    node.provider,
                    node.model
                ))
            } else {
                None
            }
        });

        if let Some(entry) = self.refs_by_group.get(&node.session_group) {
            if entry.provider == node.provider && entry.model == node.model {
                return SessionDecision::Resume(entry.provider_session_ref.clone());
            }
            return SessionDecision::New {
                warning: Some(format!(
                    "Session fallback to new: group '{}' stored {}/{} but node '{}' requires {}/{}.",
                    node.session_group, entry.provider, entry.model, node.id, node.provider, node.model
                )),
            };
        }

        SessionDecision::New {
            warning: inbound_warning,
        }
    }

    pub fn remember(&mut self, node: &TemplateNode, provider_session_ref: Option<&str>) {
        let Some(reference) = provider_session_ref else {
            return;
        };
        if reference.trim().is_empty() {
            return;
        }

        self.refs_by_group.insert(
            node.session_group.clone(),
            SessionEntry {
                provider: node.provider.clone(),
                model: node.model.clone(),
                provider_session_ref: reference.to_string(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::templates::{TemplateNode, UiPosition};

    fn node(id: &str, provider: &str, model: &str, session_group: &str) -> TemplateNode {
        TemplateNode {
            id: id.into(),
            label: id.into(),
            stage_type: "review".into(),
            handler: "chat".into(),
            provider: provider.into(),
            model: model.into(),
            session_group: session_group.into(),
            prompt_template: "{{task}}".into(),
            enabled: true,
            execution_intent: "text".into(),
            config: None,
            ui_position: UiPosition { x: 0.0, y: 0.0 },
        }
    }

    #[test]
    fn resumes_with_first_compatible_inbound_candidate() {
        let manager = SessionManager::default();
        let target = node("n2", "claude", "sonnet", "A");
        let inbound = vec![
            SessionCandidate {
                source_node_id: "n0".into(),
                session_group: "A".into(),
                provider: "claude".into(),
                model: "sonnet".into(),
                provider_session_ref: "sess-1".into(),
            },
            SessionCandidate {
                source_node_id: "n1".into(),
                session_group: "A".into(),
                provider: "claude".into(),
                model: "sonnet".into(),
                provider_session_ref: "sess-2".into(),
            },
        ];

        let decision = manager.decide_mode(&target, &inbound);
        assert_eq!(decision, SessionDecision::Resume("sess-1".into()));
    }

    #[test]
    fn falls_back_to_new_on_provider_model_mismatch() {
        let mut manager = SessionManager::default();
        let source = node("n1", "claude", "opus", "A");
        manager.remember(&source, Some("sess-opus"));

        let target = node("n2", "claude", "sonnet", "A");
        let decision = manager.decide_mode(&target, &[]);
        match decision {
            SessionDecision::New { warning } => {
                assert!(warning.is_some());
            }
            SessionDecision::Resume(_) => panic!("expected new mode"),
        }
    }

    #[test]
    fn reuses_stored_group_ref_when_compatible() {
        let mut manager = SessionManager::default();
        let source = node("n1", "claude", "sonnet", "B");
        manager.remember(&source, Some("sess-b"));

        let target = node("n2", "claude", "sonnet", "B");
        let decision = manager.decide_mode(&target, &[]);
        assert_eq!(decision, SessionDecision::Resume("sess-b".into()));
    }
}
