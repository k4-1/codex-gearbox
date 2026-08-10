use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use directories::BaseDirs;
use serde_json::{Value, json};

use crate::{AccountClass, RateBand, RouteDecision};

pub fn path() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("CODEX_HOME") {
        return Some(PathBuf::from(home).join("gearbox-metrics.jsonl"));
    }
    BaseDirs::new().map(|dirs| dirs.home_dir().join(".codex/gearbox-metrics.jsonl"))
}

pub fn append(
    route: &RouteDecision,
    account: AccountClass,
    rate_band: RateBand,
    available_models: usize,
) -> Result<()> {
    let Some(path) = path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    writeln!(
        file,
        "{}",
        json!({
            "timestamp": timestamp,
            "accountClass": account,
            "rateBand": rate_band,
            "availableModels": available_models,
            "source": route.source,
            "model": route.model,
            "effort": route.effort,
            "role": route.role,
            "confidence": route.confidence
        })
    )?;
    Ok(())
}

pub fn report() -> Result<Value> {
    let Some(path) = path() else {
        return Ok(json!({ "routes": 0 }));
    };
    if !path.exists() {
        return Ok(json!({ "routes": 0, "path": path }));
    }
    let file =
        fs::File::open(&path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut total = 0u64;
    let mut models = BTreeMap::<String, u64>::new();
    let mut sources = BTreeMap::<String, u64>::new();
    for line in BufReader::new(file).lines() {
        let Ok(value) = serde_json::from_str::<Value>(&line?) else {
            continue;
        };
        total += 1;
        if let Some(model) = value.get("model").and_then(Value::as_str) {
            *models.entry(model.into()).or_default() += 1;
        }
        if let Some(source) = value.get("source").and_then(Value::as_str) {
            *sources.entry(source.into()).or_default() += 1;
        }
    }
    Ok(json!({ "routes": total, "models": models, "sources": sources, "path": path }))
}
