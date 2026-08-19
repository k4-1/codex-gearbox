use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{Config, RateBand, Router, routing::exceeds_recommendation};

#[derive(Debug, Deserialize)]
pub struct HookInput {
    pub prompt: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub effort: String,
}

pub fn evaluate(input: &str, config: Config) -> Result<Value> {
    let input: HookInput = serde_json::from_str(input).context("invalid UserPromptSubmit input")?;
    let route = Router::new(config.clone(), vec![], RateBand::Normal).deterministic(&input.prompt);
    Ok(output(&input, &route, &config))
}

pub fn parse(input: &str) -> Result<HookInput> {
    serde_json::from_str(input).context("invalid UserPromptSubmit input")
}

pub fn output(input: &HookInput, route: &crate::RouteDecision, config: &Config) -> Value {
    let notice = format!(
        "Gearbox recommends {} · {} ({}%, {})",
        route.model, route.effort, route.confidence, route.reason
    );
    let selected_model = (!input.model.is_empty()).then_some(input.model.as_str());
    let selected_effort = (!input.effort.is_empty()).then_some(input.effort.as_str());
    if exceeds_recommendation(config, selected_model, selected_effort, route) {
        return overpowered_output(&notice, route, take_or_create_override(input));
    }
    clear_override(input);
    continue_output(notice, route)
}

fn overpowered_output(
    notice: &str,
    route: &crate::RouteDecision,
    override_state: Option<bool>,
) -> Value {
    match override_state {
        Some(true) => continue_output(
            format!("{notice}. Proceeding once with your selected settings."),
            route,
        ),
        Some(false) => json!({
            "decision": "block",
            "reason": format!(
                "{notice}. Your selected settings are stronger than this task needs. Change them and resend, or resend once unchanged to proceed anyway."
            )
        }),
        None => continue_output(
            format!(
                "{notice}. Gearbox could not create a one-shot override, so the prompt was not blocked."
            ),
            route,
        ),
    }
}

fn continue_output(message: String, route: &crate::RouteDecision) -> Value {
    json!({
        "continue": true,
        "systemMessage": message,
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

fn take_or_create_override(input: &HookInput) -> Option<bool> {
    // ponytail: marker covers the next submission, not prompt identity; use a host override token if Codex exposes one.
    let path = override_path(input)?;
    if path.exists() {
        fs::remove_file(path).ok()?;
        return Some(true);
    }
    fs::create_dir_all(path.parent()?).ok()?;
    fs::write(path, b"").ok()?;
    Some(false)
}

fn clear_override(input: &HookInput) {
    if let Some(path) = override_path(input) {
        let _ = fs::remove_file(path);
    }
}

fn override_path(input: &HookInput) -> Option<PathBuf> {
    let valid_session = !input.session_id.is_empty()
        && input.session_id.len() <= 64
        && input
            .session_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
    if !valid_session {
        return None;
    }
    Some(
        Config::path()?
            .parent()?
            .join("gearbox-overrides")
            .join(&input.session_id),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_when_desktop_model_is_stronger() {
        let route = Router::new(Config::default(), vec![], RateBand::Normal)
            .deterministic("Rename the variable");
        let output = overpowered_output("Gearbox recommends luna · low", &route, Some(false));
        assert_eq!(output["decision"], "block");
    }

    #[test]
    fn allows_one_desktop_override() {
        let route = Router::new(Config::default(), vec![], RateBand::Normal)
            .deterministic("Rename the variable");
        let output = overpowered_output("Gearbox recommends luna · low", &route, Some(true));
        assert_eq!(output["continue"], true);
    }

    #[test]
    fn continues_when_override_state_is_unavailable() {
        let output = evaluate(
            r#"{"prompt":"Rename the variable","model":"gpt-5.6-sol"}"#,
            Config::default(),
        )
        .unwrap();
        assert_eq!(output["continue"], true);
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
