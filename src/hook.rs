use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{Config, RateBand, Router};

#[derive(Debug, Deserialize)]
pub struct HookInput {
    pub prompt: String,
    #[serde(default)]
    pub model: String,
}

pub fn evaluate(input: &str, config: Config) -> Result<Value> {
    let input: HookInput = serde_json::from_str(input).context("invalid UserPromptSubmit input")?;
    let route = Router::new(config, vec![], RateBand::Normal).deterministic(&input.prompt);
    Ok(output(&input, &route))
}

pub fn parse(input: &str) -> Result<HookInput> {
    serde_json::from_str(input).context("invalid UserPromptSubmit input")
}

pub fn output(input: &HookInput, route: &crate::RouteDecision) -> Value {
    let notice = format!(
        "Gearbox recommends {} · {} ({}%, {})",
        route.model, route.effort, route.confidence, route.reason
    );
    let model_notice = if !input.model.is_empty()
        && canonical_model(&input.model) != canonical_model(&route.model)
    {
        format!(
            "{notice}. Continuing with your selected model {}.",
            input.model
        )
    } else {
        notice
    };
    json!({
        "continue": true,
        "systemMessage": model_notice,
        "hookSpecificOutput": {
            "hookEventName": "UserPromptSubmit",
            "additionalContext": format!(
                "Codex Gearbox recommends model '{}' with reasoning effort '{}'. This is advisory; the selected model and effort remain active.",
                route.model,
                route.effort
            )
        }
    })
}

fn canonical_model(model: &str) -> &str {
    if model == "gpt-5.6" {
        "gpt-5.6-sol"
    } else {
        model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continues_when_desktop_model_is_wrong() {
        let output = evaluate(
            r#"{"prompt":"Investigate the security architecture tradeoffs","model":"gpt-5.6-luna"}"#,
            Config::default(),
        )
        .unwrap();
        assert_eq!(output["continue"], true);
        assert!(
            output["systemMessage"]
                .as_str()
                .is_some_and(|message| message.contains("Continuing with your selected model"))
        );
    }

    #[test]
    fn continues_when_desktop_model_matches() {
        let output = evaluate(
            r#"{"prompt":"Rename the variable","model":"gpt-5.6-luna"}"#,
            Config::default(),
        )
        .unwrap();
        assert_eq!(output["continue"], true);
    }
}
