# Codex Gearbox

<p align="center">
  <img src="plugins/codex-gearbox/assets/gear-shift.webp" alt="Codex Gearbox icon" width="180">
</p>

The right Codex model depends on the work, not on your muscle memory.

You ask Codex to rename a variable. A fast model with low effort is enough. A
few minutes later you are investigating an authentication failure, comparing
architecture choices, or touching production data. That work needs more
reasoning, more verification, and a higher safety floor. Manually changing the
model for every turn is its own small chore.

Codex Gearbox makes that choice for you. It reads the task, checks the models
and account state that Codex actually exposes, and selects a model and
reasoning effort that fit the work in front of you.

> [!WARNING]
> Autopilot is alpha software. It uses Codex App Server's experimental WebSocket transport. If the proxy cannot start, Gearbox warns and launches normal Codex.

## Before and after

Without Gearbox, every task starts with the same question: “Which model should
I use?” You either spend time choosing or leave the same expensive setting on
for work that does not need it.

With Gearbox, the route follows the task:

| Task shape | Typical route | Why |
| --- | --- | --- |
| Clear, focused change | Luna + low effort | Little ambiguity and a small reasoning surface |
| Multi-step repository work | Terra + medium effort | Some planning, files, or tools are involved |
| Architecture or debugging | Terra/Sol + medium or high effort | The task needs deeper analysis or tradeoffs |
| Security, credentials, production, or irreversible work | Sol + high effort minimum | Risk takes priority over conservation |

These are the default preferences, not hard-coded promises. Available models,
your configuration, rate limits, and manual choices still win.

## How it works

Gearbox follows one short path for each prompt:

```text
prompt
  ↓
local task scoring
  ↓
account + plan + rate limits + available models
  ↓
optional Luna classification for eligible ambiguous work
  ↓
deterministic policy
  ↓
model + reasoning effort
```

The important part is that Luna advises; policy decides. A judge cannot bypass
model availability, rate limits, effort caps, risk floors, or a manual model
selection.

### 1. Read the task

The CLI proxy reads the text inputs in `turn/start`. The desktop plugin receives
the prompt through `UserPromptSubmit`. Gearbox does not need the whole
conversation to make its first decision, and it never treats prompt length as
the only signal.

### 2. Score the work

Fast local rules classify six useful signals, each from 0 to 2:

| Signal | Question |
| --- | --- |
| Ambiguity | Is the request underspecified or conflicting? |
| Scope | How broad or multi-part is the change? |
| Reasoning depth | Does it need diagnosis, design, or tradeoff analysis? |
| Tool breadth | Does it span repositories, APIs, browsers, or other systems? |
| Verification burden | How much testing, review, or evidence is needed? |
| Risk | Could it affect security, data, money, production, or irreversible state? |

The result becomes a route role: `fast`, `balanced`, or `deep`, plus an effort
level from `low` through `xhigh`.

### 3. Add live context

When Codex App Server is available, Gearbox reads the account class, plan,
rate-limit usage, and available models. This prevents a good-looking route from
choosing a model that is unavailable or spending a scarce rate window
carelessly.

On subscribed plans, an ambiguous or near-threshold task may receive a fresh
GPT-5.6 Luna Medium classification when Luna is available, usage is in the
normal band, and judging is enabled. Free plans never invoke the judge. API-key
judging is opt-in.

The judge runs as a read-only, tool-free, time-limited classification. If it
times out, returns an unusable result, or cannot be started, Gearbox keeps the
local route.

### 4. Apply policy

The final route applies the boring rules that should never be left to a model:

- use an available model from the configured preference list;
- clamp effort to what that model supports and to the configured minimum and maximum;
- conserve non-risky balanced work when the rate band is high;
- keep risky work at high effort or above;
- inherit a previous route for short, low-risk follow-ups when appropriate;
- preserve an explicit model or effort selection.

### 5. Continue normally

CLI Autopilot injects the selected model and effort into the Codex turn. Desktop
Advisor shows the recommendation but leaves the selected settings active because
the current hook API cannot mutate them. If routing infrastructure fails, the
normal Codex path remains available.

## What data is used

| Data | Used for | Stored? |
| --- | --- | --- |
| Current prompt text | Local scoring and, only when eligible, judge classification | No |
| Account and plan state | Judge eligibility and plan policy | No |
| Rate-limit usage | Conservation and critical routing bands | No |
| Available model metadata | Availability and supported-effort checks | No |
| `gearbox.json` | Local routing preferences and limits | You control it |
| Optional JSONL metrics | Aggregate routing reports | Yes, without prompt text |

Optional metrics contain only routing metadata: selected model, effort, role,
source, confidence, account class, rate band, available-model count, and a
timestamp. Gearbox does not store prompts, judge text, credentials, bearer
tokens, or API keys.

## Two ways to use it

| Surface | Mode | Behavior |
| --- | --- | --- |
| Codex CLI | Autopilot | Routes `turn/start` before execution and injects the model and effort. |
| Codex desktop | Advisor | Recommends a route while preserving the selected model and effort. |

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
gearbox-shift route "Investigate this authentication failure"
gearbox-shift account
gearbox-shift report
```

`gearbox-shift` avoids the POSIX shell `shift` builtin. The `shift` binary is
kept for existing scripts and hooks; invoke it as `env shift …` from a shell.

### Package-manager install

Homebrew users on macOS or Linux can install the latest native CLI with:

```bash
brew tap k4-1/codex-gearbox https://github.com/k4-1/codex-gearbox
brew install k4-1/codex-gearbox/codex-gearbox
```

Windows users can install it from the repository's Scoop bucket:

```powershell
scoop bucket add codex-gearbox https://github.com/k4-1/codex-gearbox
scoop install codex-gearbox
```

Both package manifests are refreshed after every GitHub release. The desktop
plugin remains optional; CLI Autopilot is enabled by launching `codex-gearbox`.

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
