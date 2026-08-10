use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub fast_models: Vec<String>,
    pub balanced_models: Vec<String>,
    pub deep_models: Vec<String>,
    pub confidence_threshold: u8,
    pub judge_model: String,
    pub judge_effort: String,
    pub judge_timeout_seconds: u64,
    pub judge_enabled: bool,
    pub judge_api_key: bool,
    pub judge_disabled_plans: Vec<String>,
    pub min_effort: String,
    pub max_effort: String,
    pub conserve_at_percent: u8,
    pub critical_at_percent: u8,
    pub metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fast_models: vec![
                "gpt-5.6-luna".into(),
                "gpt-5.3-codex-spark".into(),
                "gpt-5.6-terra".into(),
            ],
            balanced_models: vec![
                "gpt-5.6-terra".into(),
                "gpt-5.6-sol".into(),
                "gpt-5.6-luna".into(),
            ],
            deep_models: vec!["gpt-5.6-sol".into(), "gpt-5.6-terra".into()],
            confidence_threshold: 80,
            judge_model: "gpt-5.6-luna".into(),
            judge_effort: "medium".into(),
            judge_timeout_seconds: 15,
            judge_enabled: true,
            judge_api_key: false,
            judge_disabled_plans: vec!["free".into()],
            min_effort: "low".into(),
            max_effort: "xhigh".into(),
            conserve_at_percent: 70,
            critical_at_percent: 90,
            metrics: true,
        }
    }
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        if let Some(home) = std::env::var_os("CODEX_HOME") {
            return Some(PathBuf::from(home).join("gearbox.json"));
        }
        BaseDirs::new().map(|dirs| dirs.home_dir().join(".codex/gearbox.json"))
    }

    pub fn load() -> Result<Self> {
        let Some(path) = Self::path() else {
            return Ok(Self::default());
        };
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read configuration at {}", path.display()))?;
        serde_json::from_slice(&bytes)
            .with_context(|| format!("invalid Gearbox configuration at {}", path.display()))
    }
}
