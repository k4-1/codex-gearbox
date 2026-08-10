use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountClass {
    Free,
    Subscribed,
    ApiKey,
    Unknown,
}

impl AccountClass {
    pub fn from_account(account: Option<&serde_json::Value>) -> Self {
        let Some(account) = account else {
            return Self::Unknown;
        };
        if account.get("type").and_then(|v| v.as_str()) == Some("apiKey") {
            return Self::ApiKey;
        }
        match account
            .get("planType")
            .or_else(|| account.get("plan_type"))
            .and_then(|v| v.as_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("free") => Self::Free,
            Some(_) => Self::Subscribed,
            None => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateBand {
    Normal,
    Conservation,
    Critical,
    Reached,
}

impl RateBand {
    pub fn from_percent(percent: f64, config: &Config) -> Self {
        if percent >= 100.0 {
            Self::Reached
        } else if percent >= f64::from(config.critical_at_percent) {
            Self::Critical
        } else if percent >= f64::from(config.conserve_at_percent) {
            Self::Conservation
        } else {
            Self::Normal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    Low,
    Medium,
    High,
    Xhigh,
}

impl Effort {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            "xhigh" | "extra_high" | "extra-high" => Some(Self::Xhigh),
            _ => None,
        }
    }
}

impl fmt::Display for Effort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Fast,
    Balanced,
    Deep,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureScores {
    pub ambiguity: u8,
    pub scope: u8,
    pub reasoning_depth: u8,
    pub tool_breadth: u8,
    pub verification_burden: u8,
    pub risk: u8,
}

impl FeatureScores {
    pub fn total(&self) -> u8 {
        self.ambiguity
            + self.scope
            + self.reasoning_depth
            + self.tool_breadth
            + self.verification_burden
            + self.risk
    }

    fn clamp(mut self) -> Self {
        self.ambiguity = self.ambiguity.min(2);
        self.scope = self.scope.min(2);
        self.reasoning_depth = self.reasoning_depth.min(2);
        self.tool_breadth = self.tool_breadth.min(2);
        self.verification_burden = self.verification_burden.min(2);
        self.risk = self.risk.min(2);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub default_reasoning_effort: Option<String>,
    #[serde(default)]
    pub supported_reasoning_efforts: Vec<SupportedEffort>,
    #[serde(default)]
    pub input_modalities: Vec<String>,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedEffort {
    pub reasoning_effort: String,
    #[serde(default)]
    pub description: String,
}

impl ModelInfo {
    pub fn synthetic(model: &str) -> Self {
        Self {
            id: model.into(),
            model: model.into(),
            display_name: model.into(),
            default_reasoning_effort: Some("medium".into()),
            supported_reasoning_efforts: ["low", "medium", "high", "xhigh"]
                .into_iter()
                .map(|effort| SupportedEffort {
                    reasoning_effort: effort.into(),
                    description: String::new(),
                })
                .collect(),
            input_modalities: vec!["text".into(), "image".into()],
            is_default: model.ends_with("terra"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteDecision {
    pub model: String,
    pub effort: Effort,
    pub role: Role,
    pub confidence: u8,
    pub source: String,
    pub reason: String,
    pub scores: FeatureScores,
}

pub struct Router {
    config: Config,
    models: Vec<ModelInfo>,
    rate_band: RateBand,
}

impl Router {
    pub fn new(config: Config, models: Vec<ModelInfo>, rate_band: RateBand) -> Self {
        let models = if models.is_empty() {
            ["gpt-5.6-sol", "gpt-5.6-terra", "gpt-5.6-luna"]
                .into_iter()
                .map(ModelInfo::synthetic)
                .collect()
        } else {
            models
        };
        Self {
            config,
            models,
            rate_band,
        }
    }

    pub fn deterministic(&self, prompt: &str) -> RouteDecision {
        self.resolve(score_prompt(prompt), "rules")
    }

    pub fn from_judge(&self, scores: FeatureScores, confidence: u8, reason: &str) -> RouteDecision {
        let mut route = self.resolve(scores.clamp(), "luna-judge");
        route.confidence = confidence.min(100);
        if !reason.trim().is_empty() {
            route.reason = reason
                .split_whitespace()
                .take(12)
                .collect::<Vec<_>>()
                .join(" ");
        }
        route
    }

    pub fn should_judge(
        &self,
        route: &RouteDecision,
        account: AccountClass,
        plan_type: Option<&str>,
    ) -> bool {
        let account_allows = account == AccountClass::Subscribed
            || (account == AccountClass::ApiKey && self.config.judge_api_key);
        let plan_disabled = plan_type.is_some_and(|plan| {
            self.config
                .judge_disabled_plans
                .iter()
                .any(|disabled| disabled.eq_ignore_ascii_case(plan))
        });
        self.config.judge_enabled
            && account_allows
            && !plan_disabled
            && self.rate_band == RateBand::Normal
            && (route.confidence < self.config.confidence_threshold || needs_judgment(route))
            && self.models.iter().any(|m| {
                m.model == self.config.judge_model && supports_effort(m, &self.config.judge_effort)
            })
    }

    fn resolve(&self, scores: FeatureScores, source: &str) -> RouteDecision {
        let total = scores.total();
        let mut role = if scores.risk >= 1
            || (scores.reasoning_depth >= 2 && (scores.scope >= 1 || scores.ambiguity >= 1))
        {
            Role::Deep
        } else if scores.reasoning_depth >= 2 {
            Role::Balanced
        } else if total <= 3 && scores.ambiguity == 0 {
            Role::Fast
        } else {
            Role::Balanced
        };

        if matches!(
            self.rate_band,
            RateBand::Conservation | RateBand::Critical | RateBand::Reached
        ) && role == Role::Balanced
            && scores.risk == 0
        {
            role = Role::Fast;
        }

        let mut effort = match total {
            0..=2 => Effort::Low,
            3..=5 => Effort::Medium,
            6..=8 => Effort::High,
            _ => Effort::Xhigh,
        };
        if scores.reasoning_depth >= 2 {
            effort = effort.max(Effort::Medium);
        }
        if let Some(minimum) = Effort::from_str(&self.config.min_effort) {
            effort = effort.max(minimum);
        }
        if let Some(maximum) = Effort::from_str(&self.config.max_effort) {
            effort = effort.min(maximum);
        }
        if scores.risk >= 1 {
            effort = effort.max(Effort::High);
        }

        let model = self.pick_model(role);
        effort = clamp_effort(effort, self.models.iter().find(|m| m.model == model));
        let confidence = confidence(&scores);
        let reason = reason(&scores, role);
        RouteDecision {
            model,
            effort,
            role,
            confidence,
            source: source.into(),
            reason,
            scores,
        }
    }

    fn pick_model(&self, role: Role) -> String {
        let preferences = match role {
            Role::Fast => &self.config.fast_models,
            Role::Balanced => &self.config.balanced_models,
            Role::Deep => &self.config.deep_models,
        };
        preferences
            .iter()
            .find(|wanted| self.models.iter().any(|m| m.model == **wanted))
            .cloned()
            .or_else(|| {
                self.models
                    .iter()
                    .find(|m| m.is_default)
                    .map(|m| m.model.clone())
            })
            .unwrap_or_else(|| self.models[0].model.clone())
    }
}

fn supports_effort(model: &ModelInfo, effort: &str) -> bool {
    model.supported_reasoning_efforts.is_empty()
        || model
            .supported_reasoning_efforts
            .iter()
            .any(|item| item.reasoning_effort.eq_ignore_ascii_case(effort))
}

fn clamp_effort(requested: Effort, model: Option<&ModelInfo>) -> Effort {
    let Some(model) = model else {
        return requested;
    };
    let supported: Vec<Effort> = model
        .supported_reasoning_efforts
        .iter()
        .filter_map(|item| Effort::from_str(&item.reasoning_effort))
        .collect();
    if supported.is_empty() || supported.contains(&requested) {
        return requested;
    }
    supported
        .iter()
        .copied()
        .filter(|candidate| *candidate <= requested)
        .max()
        .or_else(|| supported.iter().copied().min())
        .unwrap_or(requested)
}

fn score_prompt(prompt: &str) -> FeatureScores {
    let text = prompt.to_ascii_lowercase();
    let words = text.split_whitespace().count();
    let tokens: Vec<&str> = text.split_whitespace().collect();
    let count = |terms: &[&str]| terms.iter().filter(|term| text.contains(**term)).count();

    let ambiguity =
        (usize::from(
            words < 5
                && tokens
                    .iter()
                    .any(|token| matches!(*token, "it" | "this" | "that" | "thing")),
        ) + usize::from(count(&["fix it", "make it better", "somehow", "whatever", "etc."]) > 0))
        .min(2) as u8;
    let scope = (usize::from(words > 80 || count(&["entire", "whole", "across", "every"]) > 0)
        + usize::from(
            count(&[
                "implement",
                "refactor",
                "migrate",
                "document",
                "deploy",
                "test",
                "release",
            ]) >= 3,
        ))
    .min(2) as u8;
    let reasoning_depth =
        (usize::from(
            count(&[
                "architecture",
                "root cause",
                "tradeoff",
                "strategy",
                "investigate",
                "diagnose",
            ]) > 0,
        ) + usize::from(count(&["compare", "why", "design", "complex", "optimize"]) >= 2))
        .min(2) as u8;
    let tool_breadth =
        (usize::from(
            count(&[
                "repository",
                "files",
                "database",
                "web",
                "api",
                "github",
                "browser",
            ]) >= 2,
        ) + usize::from(count(&["build", "test", "deploy", "search", "run", "inspect"]) >= 3))
        .min(2) as u8;
    let verification_burden =
        (usize::from(count(&["test", "verify", "validate", "benchmark", "review", "prove"]) > 0)
            + usize::from(count(&["regression", "cross-platform", "production", "acceptance"]) > 0))
        .min(2) as u8;
    let risk = (usize::from(
        count(&[
            "delete",
            "drop",
            "overwrite",
            "reset",
            "migration",
            "production",
            "payment",
        ]) > 0,
    ) + usize::from(
        count(&[
            "security",
            "credential",
            "authentication",
            "permission",
            "private key",
        ]) > 0,
    ))
    .min(2) as u8;

    FeatureScores {
        ambiguity,
        scope,
        reasoning_depth,
        tool_breadth,
        verification_burden,
        risk,
    }
}

fn confidence(scores: &FeatureScores) -> u8 {
    let boundary_penalty = matches!(scores.total(), 3 | 5 | 6 | 8 | 9) as u8 * 8;
    94u8.saturating_sub(scores.ambiguity * 12)
        .saturating_sub(boundary_penalty)
        .max(40)
}

fn needs_judgment(route: &RouteDecision) -> bool {
    route.scores.ambiguity > 0 || matches!(route.scores.total(), 2 | 3 | 5 | 6)
}

fn reason(scores: &FeatureScores, role: Role) -> String {
    if scores.risk > 0 {
        return "risk-sensitive task".into();
    }
    if scores.ambiguity > 0 && scores.reasoning_depth > 0 {
        return "ambiguous task requiring judgment".into();
    }
    if scores.scope > 0 || scores.tool_breadth > 0 {
        return "multi-step repository task".into();
    }
    match role {
        Role::Fast => "clear, focused task".into(),
        Role::Balanced => "everyday task requiring some planning".into(),
        Role::Deep => "complex task requiring deeper analysis".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> Router {
        Router::new(Config::default(), vec![], RateBand::Normal)
    }

    #[test]
    fn routes_clear_mechanical_work_to_luna_low() {
        let route = router().deterministic("Rename the variable user_id to account_id");
        assert_eq!(route.model, "gpt-5.6-luna");
        assert_eq!(route.effort, Effort::Low);
    }

    #[test]
    fn applies_deep_high_floor_to_security_work() {
        let route = router().deterministic(
            "Investigate the authentication vulnerability and verify the production fix",
        );
        assert_eq!(route.model, "gpt-5.6-sol");
        assert!(route.effort >= Effort::High);
    }

    #[test]
    fn routes_architecture_reasoning_to_terra_medium() {
        let route = router()
            .deterministic("Investigate the architecture tradeoffs and compare design choices");
        assert_eq!(route.model, "gpt-5.6-terra");
        assert_eq!(route.effort, Effort::Medium);
    }

    #[test]
    fn applies_high_floor_to_single_category_security_work() {
        let route = router().deterministic("Review this security design for vulnerabilities");
        assert_eq!(route.model, "gpt-5.6-sol");
        assert_eq!(route.effort, Effort::High);
    }

    #[test]
    fn never_selects_an_unavailable_model() {
        let models = vec![ModelInfo::synthetic("only-model")];
        let route = Router::new(Config::default(), models, RateBand::Normal)
            .deterministic("Design the whole architecture and compare the security tradeoffs");
        assert_eq!(route.model, "only-model");
    }

    #[test]
    fn free_accounts_never_use_judge() {
        let route = router().deterministic("Fix it");
        assert!(!router().should_judge(&route, AccountClass::Free, Some("free")));
    }

    #[test]
    fn conservation_avoids_judge() {
        let router = Router::new(Config::default(), vec![], RateBand::Conservation);
        let route = router.deterministic("Fix it");
        assert!(!router.should_judge(&route, AccountClass::Subscribed, Some("plus")));
    }

    #[test]
    fn subscribed_ambiguous_prompts_can_use_luna_judge() {
        let route = router().deterministic("Fix it");
        assert!(router().should_judge(&route, AccountClass::Subscribed, Some("plus")));
    }

    #[test]
    fn clear_prompts_skip_luna_judge() {
        let route = router().deterministic("Rename the variable");
        assert!(!router().should_judge(&route, AccountClass::Subscribed, Some("plus")));
    }

    #[test]
    fn boundary_prompts_use_luna_judge_even_with_high_rule_confidence() {
        let route = router()
            .deterministic("Investigate the architecture tradeoffs and compare design choices");
        assert!(route.confidence >= Config::default().confidence_threshold);
        assert!(router().should_judge(&route, AccountClass::Subscribed, Some("plus")));
    }

    #[test]
    fn api_key_judging_is_opt_in() {
        let route = router().deterministic("Fix it");
        assert!(!router().should_judge(&route, AccountClass::ApiKey, None));
    }

    #[test]
    fn risk_floor_overrides_a_low_user_effort_cap() {
        let config = Config {
            max_effort: "low".into(),
            ..Config::default()
        };
        let route = Router::new(config, vec![], RateBand::Normal)
            .deterministic("Delete production credentials and reset authentication");
        assert!(route.effort >= Effort::High);
    }
}
