//! Runtime model selection shared by model discovery and explicit launch checks.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::acp::types::{
    SessionConfigKindInfo, SessionConfigOptionInfo, SessionConfigSelectOptionInfo,
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModelSelection {
    pub model: Option<String>,
    pub reasoning: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub slug: String,
    /// None means the runtime did not advertise levels for this model.
    pub reasoning: Option<Vec<String>>,
    /// Never infer a context window from a model's name.
    pub context_window: Option<u64>,
}

fn is_model(option: &SessionConfigOptionInfo) -> bool {
    option.category.as_deref() == Some("model") || option.id == "model"
}

fn is_reasoning(option: &SessionConfigOptionInfo) -> bool {
    matches!(
        option.category.as_deref(),
        Some("thought_level" | "reasoning_effort" | "effort")
    ) || matches!(
        option.id.as_str(),
        "thought_level" | "reasoning_effort" | "effort"
    )
}

fn choices(option: &SessionConfigOptionInfo) -> Vec<&SessionConfigSelectOptionInfo> {
    match &option.kind {
        SessionConfigKindInfo::Select(select) => select
            .options
            .iter()
            .chain(select.groups.iter().flat_map(|group| group.options.iter()))
            .collect(),
        SessionConfigKindInfo::Boolean(_) => Vec::new(),
    }
}

fn current(option: &SessionConfigOptionInfo) -> String {
    match &option.kind {
        SessionConfigKindInfo::Select(select) => select.current_value.clone(),
        SessionConfigKindInfo::Boolean(value) => value.current_value.to_string(),
    }
}

pub fn model_catalog(
    options: &[SessionConfigOptionInfo],
    selected_model: Option<&str>,
) -> Vec<ModelInfo> {
    let Some(model) = options.iter().find(|option| is_model(option)) else {
        return Vec::new();
    };
    let current_model = selected_model
        .map(str::to_owned)
        .unwrap_or_else(|| current(model));
    let current_reasoning = options.iter().find(|option| is_reasoning(option));
    choices(model)
        .into_iter()
        .map(|choice| ModelInfo {
            name: choice.name.clone(),
            slug: choice.value.clone(),
            reasoning: choice
                .reasoning_efforts
                .as_ref()
                .map(|effort| effort.values.clone())
                .or_else(|| {
                    (choice.value == current_model)
                        .then(|| {
                            current_reasoning.map(|option| {
                                choices(option)
                                    .iter()
                                    .map(|choice| choice.value.clone())
                                    .collect()
                            })
                        })
                        .flatten()
                }),
            context_window: None,
        })
        .collect()
}

impl ModelSelection {
    pub fn is_empty(&self) -> bool {
        self.model.is_none() && self.reasoning.is_none()
    }

    /// Resolve public model/reasoning arguments to the agent's actual option ids.
    /// Probe with the model first: reasoning choices can change with the model.
    pub fn resolve(
        &self,
        options: &[SessionConfigOptionInfo],
    ) -> Result<BTreeMap<String, String>, String> {
        let mut result = BTreeMap::new();
        for (name, value, predicate) in [
            (
                "model",
                &self.model,
                is_model as fn(&SessionConfigOptionInfo) -> bool,
            ),
            (
                "reasoning",
                &self.reasoning,
                is_reasoning as fn(&SessionConfigOptionInfo) -> bool,
            ),
        ] {
            let Some(value) = value else {
                continue;
            };
            let option = options
                .iter()
                .find(|option| predicate(option))
                .ok_or_else(|| {
                    format!("{name} is not supported by this agent; available options: []")
                })?;
            if !choices(option).iter().any(|choice| &choice.value == value) {
                return Err(format!(
                    "invalid {name} {value:?}; available options: {}",
                    available(option)
                ));
            }
            result.insert(option.id.clone(), value.clone());
        }
        Ok(result)
    }
}

fn available(option: &SessionConfigOptionInfo) -> String {
    match &option.kind {
        SessionConfigKindInfo::Boolean(_) => "true, false".into(),
        SessionConfigKindInfo::Select(_) => choices(option)
            .iter()
            .map(|choice| format!("{} ({})", choice.name, choice.value))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// Shared post-start check for explicit selections. The caller must wait for
/// session readiness before validating; a rejected override must never reach a prompt.
pub fn validate_config_values(
    options: &[SessionConfigOptionInfo],
    requested: &BTreeMap<String, String>,
) -> Result<(), String> {
    let actual = crate::acp::connection::current_config_option_values(options);
    for (key, value) in requested {
        if actual.get(key) != Some(value) {
            let option = options.iter().find(|option| &option.id == key);
            return Err(format!(
                "could not apply config {key}={value:?}; actual: {:?}; available options: {}",
                actual.get(key),
                option.map(available).unwrap_or_else(|| "[]".into())
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn options() -> Vec<SessionConfigOptionInfo> {
        serde_json::from_value(json!([
            {"id":"provider/model", "name":"Model", "category":"model", "description":null,
             "kind":{"type":"select", "current_value":"a", "options":[
                 {"value":"a","name":"Alpha","description":null,"reasoning_efforts":{"values":["low","high"],"recommended":"high"}},
                 {"value":"b","name":"Beta","description":null}],"groups":[]}},
            {"id":"thinking", "name":"Thinking", "category":"thought_level", "description":null,
             "kind":{"type":"select", "current_value":"low", "options":[
                 {"value":"low","name":"Low","description":null},
                 {"value":"high","name":"High","description":null}],"groups":[]}}
        ])).unwrap()
    }

    #[test]
    fn lists_actual_model_names_efforts_and_unknown_context() {
        let catalog = model_catalog(&options(), None);
        assert_eq!(catalog[0].slug, "a");
        assert_eq!(catalog[0].name, "Alpha");
        assert_eq!(
            catalog[0].reasoning,
            Some(vec!["low".into(), "high".into()])
        );
        assert_eq!(catalog[0].context_window, None);
        // A model without per-model metadata cannot borrow another model's levels.
        assert_eq!(catalog[1].reasoning, None);
    }

    #[test]
    fn selection_uses_runtime_option_ids_and_rejects_unknown_values() {
        let selected = ModelSelection {
            model: Some("a".into()),
            reasoning: Some("high".into()),
        };
        let values = selected.resolve(&options()).unwrap();
        assert_eq!(values.get("provider/model").map(String::as_str), Some("a"));
        assert_eq!(values.get("thinking").map(String::as_str), Some("high"));
        let bad = ModelSelection {
            model: Some("missing".into()),
            reasoning: None,
        };
        let error = bad.resolve(&options()).unwrap_err();
        assert!(error.contains("Alpha"));
        assert!(error.contains("a"));
        assert!(error.contains("b"));
    }

    #[test]
    fn model_dependent_levels_belong_to_applied_model_not_probe_default() {
        let mut options = options();
        if let SessionConfigKindInfo::Select(select) = &mut options[0].kind {
            select.options[0].reasoning_efforts = None;
        }
        let catalog = model_catalog(&options, Some("b"));
        assert_eq!(catalog[0].reasoning, None);
        assert_eq!(
            catalog[1].reasoning,
            Some(vec!["low".into(), "high".into()])
        );
    }

    #[test]
    fn explicit_configuration_must_match_actual_values() {
        let requested = BTreeMap::from([("thinking".into(), "high".into())]);
        let error = validate_config_values(&options(), &requested).unwrap_err();
        assert!(error.contains("thinking"));
        assert!(error.contains("low"));
        assert!(error.contains("high"));
    }
}
