use std::{process::Stdio, sync::Arc, time::Duration};

use anyhow::{Context, Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use getrandom::fill;
use serde_json::{Value, json};
use tokio::{net::TcpListener, process::Command, sync::Mutex, time};
use tokio_tungstenite::{
    accept_hdr_async, connect_async,
    tungstenite::{
        Message,
        handshake::server::{ErrorResponse, Request, Response},
        http::StatusCode,
    },
};

use crate::{
    Config, Router,
    app_server::{ControlClient, ManagedServer, ServerSnapshot},
    metrics,
};

const TOKEN_ENV: &str = "CODEX_GEARBOX_REMOTE_TOKEN";

pub async fn run(config: Config, codex_args: Vec<String>) -> Result<i32> {
    let mut server = ManagedServer::start().await?;
    let mut control = match ControlClient::connect_with_retry(&server.url).await {
        Ok(client) => client,
        Err(error) => {
            server.stop().await;
            eprintln!("Gearbox warning: {error:#}; launching Codex without automatic routing.");
            return run_plain_codex(codex_args).await;
        }
    };
    let snapshot = control.snapshot(&config).await?;
    let control = Arc::new(Mutex::new(control));
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let proxy_address = listener.local_addr()?;
    let token = random_token()?;
    let proxy_url = format!("ws://{proxy_address}");
    let upstream = server.url.clone();
    let proxy_config = config.clone();
    let proxy_token = token.clone();
    let proxy_task = tokio::spawn(async move {
        serve_one(
            listener,
            &upstream,
            &proxy_token,
            proxy_config,
            snapshot,
            control,
        )
        .await
    });

    let mut command = Command::new("codex");
    command
        .arg("--remote")
        .arg(proxy_url)
        .arg("--remote-auth-token-env")
        .arg(TOKEN_ENV)
        .args(codex_args)
        .env(TOKEN_ENV, token)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let status = command
        .status()
        .await
        .context("failed to launch the Codex terminal UI")?;
    proxy_task.abort();
    server.stop().await;
    Ok(status.code().unwrap_or(1))
}

async fn run_plain_codex(args: Vec<String>) -> Result<i32> {
    let status = Command::new("codex")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;
    Ok(status.code().unwrap_or(1))
}

#[allow(clippy::result_large_err)] // tungstenite's authentication callback fixes this error type.
async fn serve_one(
    listener: TcpListener,
    upstream_url: &str,
    expected_token: &str,
    config: Config,
    snapshot: ServerSnapshot,
    control: Arc<Mutex<ControlClient>>,
) -> Result<()> {
    let (stream, _) = listener.accept().await?;
    let expected_header = format!("Bearer {expected_token}");
    let client = accept_hdr_async(stream, move |request: &Request, response: Response| {
        let authorized = request
            .headers()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            == Some(expected_header.as_str());
        if authorized {
            Ok(response)
        } else {
            let mut error = ErrorResponse::new(Some("unauthorized".into()));
            *error.status_mut() = StatusCode::UNAUTHORIZED;
            Err(error)
        }
    })
    .await?;
    let (upstream, _) = connect_async(upstream_url).await?;
    relay(client, upstream, config, snapshot, control).await
}

async fn relay<Client, Upstream>(
    mut client: tokio_tungstenite::WebSocketStream<Client>,
    mut upstream: tokio_tungstenite::WebSocketStream<Upstream>,
    config: Config,
    snapshot: ServerSnapshot,
    control: Arc<Mutex<ControlClient>>,
) -> Result<()>
where
    Client: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    Upstream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let mut manually_pinned = false;
    let mut previous_route: Option<crate::RouteDecision> = None;
    loop {
        tokio::select! {
            incoming = client.next() => {
                let Some(incoming) = incoming else { break };
                let mut message = incoming?;
                if let Message::Text(text) = &message
                    && let Ok(mut value) = serde_json::from_str::<Value>(text.as_str())
                {
                    if value.get("method").and_then(Value::as_str) == Some("thread/settings/update") {
                        manually_pinned = has_model_setting(&value);
                    }
                    if !manually_pinned
                        && value.get("method").and_then(Value::as_str) == Some("turn/start")
                        && let Some(prompt) = prompt_from_turn(&value)
                    {
                        let router = Router::new(
                            config.clone(),
                            snapshot.models.clone(),
                            snapshot.rate_band,
                        );
                        let rules = router.deterministic(&prompt);
                        let route = if should_inherit(&prompt, &rules)
                            && let Some(previous) = &previous_route
                        {
                            crate::RouteDecision {
                                source: "inherited".into(),
                                confidence: 90,
                                reason: "short follow-up inherits the previous route".into(),
                                ..previous.clone()
                            }
                        } else if router.should_judge(
                            &rules,
                            snapshot.account_class,
                            snapshot.plan_type.as_deref(),
                        ) {
                            match time::timeout(
                                Duration::from_secs(config.judge_timeout_seconds),
                                control.lock().await.judge(&prompt, &config),
                            )
                            .await
                            {
                                Ok(Ok(judged)) if judged.confidence >= 70 => router.from_judge(
                                    judged.scores,
                                    judged.confidence,
                                    &judged.reason,
                                ),
                                _ => rules,
                            }
                        } else {
                            rules
                        };
                        previous_route = Some(route.clone());
                        inject_route(&mut value, &route)?;
                        if config.metrics {
                            let _ = metrics::append(
                                &route,
                                snapshot.account_class,
                                snapshot.rate_band,
                                snapshot.models.len(),
                            );
                        }
                        let thread_id = value.pointer("/params/threadId").cloned().unwrap_or(Value::Null);
                        let warning = json!({
                            "method": "warning",
                            "params": {
                                "threadId": thread_id,
                                "message": format!(
                                    "Gearbox: {} · {} ({}, {}%) — {}",
                                    route.model, route.effort, route.source, route.confidence, route.reason
                                )
                            }
                        });
                        client.send(Message::Text(warning.to_string().into())).await?;
                        message = Message::Text(value.to_string().into());
                    }
                }
                upstream.send(message).await?;
            }
            outgoing = upstream.next() => {
                let Some(outgoing) = outgoing else { break };
                client.send(outgoing?).await?;
            }
        }
    }
    Ok(())
}

fn prompt_from_turn(value: &Value) -> Option<String> {
    let texts: Vec<&str> = value
        .pointer("/params/input")?
        .as_array()?
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|item| item.get("text").and_then(Value::as_str))
        .collect();
    (!texts.is_empty()).then(|| texts.join("\n"))
}

fn inject_route(value: &mut Value, route: &crate::RouteDecision) -> Result<()> {
    let params = value
        .get_mut("params")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow!("turn/start params are not an object"))?;
    params.insert("model".into(), Value::String(route.model.clone()));
    params.insert("effort".into(), Value::String(route.effort.to_string()));
    Ok(())
}

fn has_model_setting(value: &Value) -> bool {
    value.pointer("/params/model").is_some()
        || value.pointer("/params/effort").is_some()
        || value.pointer("/params/settings/model").is_some()
        || value.pointer("/params/settings/effort").is_some()
}

fn should_inherit(prompt: &str, route: &crate::RouteDecision) -> bool {
    let text = prompt.trim().to_ascii_lowercase();
    let short = text.split_whitespace().count() <= 10;
    let follow_up = [
        "also ",
        "and ",
        "now ",
        "then ",
        "continue",
        "do that",
        "fix it",
        "what about",
    ]
    .iter()
    .any(|prefix| text.starts_with(prefix));
    short && follow_up && route.scores.risk == 0 && route.scores.reasoning_depth == 0
}

fn random_token() -> Result<String> {
    let mut bytes = [0u8; 24];
    fill(&mut bytes).map_err(|error| anyhow!("failed to obtain OS randomness: {error}"))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_only_text_inputs() {
        let value = json!({
            "params": { "input": [
                { "type": "text", "text": "Fix the bug" },
                { "type": "image", "url": "x" }
            ]}
        });
        assert_eq!(prompt_from_turn(&value).as_deref(), Some("Fix the bug"));
    }

    #[test]
    fn injects_model_and_effort_without_touching_input() {
        let mut value = json!({ "params": { "input": [{ "type": "text", "text": "x" }] } });
        let route = Router::new(Config::default(), vec![], crate::RateBand::Normal)
            .deterministic("Rename a variable");
        inject_route(&mut value, &route).unwrap();
        assert_eq!(value.pointer("/params/model"), Some(&json!("gpt-5.6-luna")));
        assert_eq!(value.pointer("/params/effort"), Some(&json!("low")));
        assert_eq!(value.pointer("/params/input/0/text"), Some(&json!("x")));
    }

    #[test]
    fn inherits_short_followups_but_not_new_risk() {
        let router = Router::new(Config::default(), vec![], crate::RateBand::Normal);
        assert!(should_inherit(
            "Also update that",
            &router.deterministic("Also update that")
        ));
        assert!(!should_inherit(
            "Now delete the production database",
            &router.deterministic("Now delete the production database")
        ));
    }
}
