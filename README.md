# Codex Gearbox

Codex Gearbox chooses a Codex model and reasoning effort from the task in front of you.

It has two modes:

| Surface | Mode | Behavior |
| --- | --- | --- |
| Codex CLI | Autopilot | Routes `turn/start` before execution and injects the model and effort. |
| Codex desktop | Advisor | Recommends a route while preserving the selected model and effort. |

> [!WARNING]
> Autopilot is alpha software. It uses Codex App Server's experimental WebSocket transport. If the proxy cannot start, Gearbox warns and launches normal Codex.

## How routing works

Clear prompts use deterministic local rules. On subscribed ChatGPT plans,
ambiguous or near-threshold prompts may be classified by a fresh GPT-5.6 Luna
Medium judge when Luna is available and account usage is healthy. Free accounts
never call the judge, and API-key judging is opt-in. The judge returns scores and
confidence; deterministic policy always makes the final choice and applies model
availability, rate limits, safety floors, user caps, and manual overrides.

Gearbox does not store prompts. Optional local JSONL metrics contain only the selected model, effort, routing source, confidence, plan class, rate band, and timestamp.

## Install from source

Rust 1.85 or newer and Codex CLI are required.

```bash
cd codex-gearbox
cargo install --path . --locked --root "$HOME/.local"
```

Ensure `$HOME/.local/bin` is on `PATH` for both your shell and the ChatGPT desktop app.

After installation, Gearbox checks the pinned GitHub release channel in the
background once per day. If a newer matching binary is available, it downloads
it, verifies the release asset digest, and uses it on the next invocation. Set
`CODEX_GEARBOX_DISABLE_UPDATE=1` to disable this behavior. Routing and hook
behavior lives in the binary, so normal releases reach installed plugins
automatically. Plugin manifest and other plugin-file changes require refreshing
the Git marketplace snapshot.

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

Add the GitHub marketplace, install the plugin, and start a new Codex task:

```bash
codex plugin marketplace add k4-1/codex-gearbox --ref main
codex plugin add codex-gearbox@personal
```

In Codex desktop, the plugin's `UserPromptSubmit` hook runs `env shift hook` so
the `shift` shell builtin cannot shadow the installed helper. Current hooks
cannot change model or effort, so Advisor mode displays a recommendation while
every prompt continues with the user's selected settings.

The marketplace points to this repository's `main` branch. To refresh cached
plugin files manually:

```bash
codex plugin marketplace upgrade personal
codex plugin remove codex-gearbox@personal
codex plugin add codex-gearbox@personal
```

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

### Feedback without code

You can contribute without changing code:

- [Ask a question or share feedback](https://github.com/k4-1/codex-gearbox/discussions)
- [Report a bug](https://github.com/k4-1/codex-gearbox/issues/new?template=bug_report.yml)
- [Request a feature](https://github.com/k4-1/codex-gearbox/issues/new?template=feature_request.yml)
- [Report a security vulnerability privately](SECURITY.md)

The protocol design follows the official OpenAI documentation for [Codex App Server](https://learn.chatgpt.com/docs/app-server), [hooks](https://learn.chatgpt.com/docs/hooks), [models](https://learn.chatgpt.com/docs/models), and [usage](https://learn.chatgpt.com/docs/pricing).

## License

MIT
