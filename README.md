# Codex Gearbox

Codex Gearbox chooses a Codex model and reasoning effort from the task in front of you.

It has two modes:

| Surface | Mode | Behavior |
| --- | --- | --- |
| Codex CLI | Autopilot | Routes `turn/start` before execution and injects the model and effort. |
| Codex desktop | Advisor | Recommends a route and pauses when the selected model does not match. |

> [!WARNING]
> Autopilot is alpha software. It uses Codex App Server's experimental WebSocket transport. If the proxy cannot start, Gearbox warns and launches normal Codex.

## How routing works

Clear prompts use deterministic local rules. On subscribed ChatGPT plans, ambiguous prompts may be classified by a fresh GPT-5.6 Luna Medium judge when Luna is available and account usage is healthy. Free accounts never call the judge. Deterministic policy always makes the final choice and applies model availability, rate limits, safety floors, user caps, and manual overrides.

Gearbox does not store prompts. Optional local JSONL metrics contain only the selected model, effort, routing source, confidence, plan class, rate band, and timestamp.

## Install from source

Rust 1.85 or newer and Codex CLI are required.

```bash
cd codex-gearbox
cargo install --path . --locked --root "$HOME/.local"
```

Ensure `$HOME/.local/bin` is on `PATH` for both your shell and the ChatGPT desktop app.

### CLI Autopilot

Launch the normal Codex terminal UI through Gearbox:

```bash
codex-gearbox
```

Show the installed version:

```bash
codex-gearbox --version
```

All unrecognized arguments pass through to Codex. Other commands:

```bash
shift route "Investigate this authentication failure"
shift account
shift report
```

### Desktop Advisor plugin

Add this repository's local marketplace, install the plugin, and start a new Codex task:

```bash
codex plugin marketplace add "$(pwd)"
codex plugin add codex-gearbox@personal
```

In Codex desktop, the plugin's `UserPromptSubmit` hook runs `shift hook`. Current hooks cannot change model or effort, so Advisor mode blocks a mismatched model with a recommendation; select it and resend. Correctly routed prompts continue immediately.

## Configuration

Gearbox reads `$CODEX_HOME/gearbox.json`, or `~/.codex/gearbox.json` when `CODEX_HOME` is unset. Every field is optional:

```json
{
  "confidenceThreshold": 80,
  "judgeModel": "gpt-5.6-luna",
  "judgeEffort": "medium",
  "judgeTimeoutSeconds": 15,
  "judgeEnabled": true,
  "judgeApiKey": false,
  "judgeDisabledPlans": ["free"],
  "minEffort": "low",
  "maxEffort": "xhigh",
  "conserveAtPercent": 70,
  "criticalAtPercent": 90,
  "metrics": true
}
```

Automatic routing never selects Max or Ultra. High-risk work receives at least Sol-class/High routing even when a lower effort cap is configured.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked
python3 /path/to/plugin-creator/scripts/validate_plugin.py plugins/codex-gearbox
```

AI agents and contributors should read [`AGENTS.md`](AGENTS.md) first, then
[`docs/CONTRIBUTING-AI.md`](docs/CONTRIBUTING-AI.md) and the relevant rules in
`.agents/skills/codex-gearbox-maintainer/rules/`. Rust changes also use the
pinned Apollo GraphQL [`rust-best-practices`](.agents/skills/rust-best-practices/SKILL.md)
skill.

For the GitHub contribution flow, read [`CONTRIBUTING.md`](CONTRIBUTING.md).
Release history is maintained automatically on [GitHub Releases](https://github.com/k4-1/codex-gearbox/releases); see [`CHANGELOG.md`](CHANGELOG.md).

The protocol design follows the official OpenAI documentation for [Codex App Server](https://learn.chatgpt.com/docs/app-server), [hooks](https://learn.chatgpt.com/docs/hooks), [models](https://learn.chatgpt.com/docs/models), and [usage](https://learn.chatgpt.com/docs/pricing).

## License

MIT
