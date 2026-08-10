# AI-assisted contribution guide

This guide explains how an agent should work in Codex Gearbox. It is a Rust
CLI and Codex desktop plugin, not a generic model router service.

## 1. Orient before editing

| Question | Source |
| --- | --- |
| What is the short contract? | [`AGENTS.md`](../AGENTS.md) |
| What skill should guide the change? | [`codex-gearbox-maintainer`](../.agents/skills/codex-gearbox-maintainer/SKILL.md) |
| What Rust baseline applies? | [`rust-best-practices`](../.agents/skills/rust-best-practices/SKILL.md) |
| Which security audit skill applies? | [`AGENTS.md` skill routing](../AGENTS.md#skill-routing-avoid-duplicate-audits) |
| Which rule applies? | [`rules/`](../.agents/skills/codex-gearbox-maintainer/rules/) |
| Where is routing policy? | [`src/routing.rs`](../src/routing.rs) |
| Where is App Server protocol code? | [`src/app_server.rs`](../src/app_server.rs) |
| Where is CLI proxy behavior? | [`src/proxy.rs`](../src/proxy.rs) |
| Where is desktop hook behavior? | [`src/hook.rs`](../src/hook.rs) |
| Where is configuration documented? | [`README.md`](../README.md) and [`src/config.rs`](../src/config.rs) |

Read only the relevant rule files after orientation. For a Rust change, read
the relevant Apollo chapter(s) from `rust-best-practices/references/` in the
same turn. Project-specific security, privacy, protocol, and CLI rules override
generic guidance when they are more specific. Do not load or recreate a large
generic framework guide for a small change.

## 2. Choose the correct change boundary

Use the smallest boundary that owns the behavior:

- **Scoring or thresholds:** `src/routing.rs`; keep it pure and table-testable.
- **Live account/model/rate-limit data:** `src/app_server.rs`; retain the
  deterministic fallback if the server, protocol, or judge fails.
- **CLI turn forwarding:** `src/proxy.rs`; do not duplicate routing logic.
- **Desktop hook JSON:** `src/hook.rs` and the plugin hook manifest together.
- **Local settings or metrics:** `src/config.rs` / `src/metrics.rs`; preserve
  the no-prompt-storage guarantee.
- **Public command behavior:** `src/main.rs`, `src/shift.rs`, README, and
  focused CLI checks.

Avoid new cross-cutting abstractions unless at least two real callers need the
same policy and the abstraction improves testability or safety.

## 3. Implementation principles

Apply SOLID selectively:

- Single responsibility: the router scores, the App Server client speaks the
  protocol, the proxy forwards turns, and the hook formats hook decisions.
- Dependency inversion: isolate external protocol/process boundaries when a
  test needs a substitute; do not invent interfaces around pure functions.
- Interface segregation and Liskov: keep boundary types narrow and preserve
  native Codex semantics when forwarding messages.

Apply KISS/YAGNI deliberately:

- Prefer the standard library and existing dependencies.
- Prefer a pure function and a focused test over a new service or registry.
- Do not add speculative model aliases, telemetry, caching, or persistence.
- Keep comments for security/protocol rationale, not for obvious syntax.

For Rust-specific ownership, error, Clippy, performance, testing, dispatch,
type-state, pointer, and documentation decisions, use the pinned Apollo
GraphQL [`rust-best-practices`](../.agents/skills/rust-best-practices/SKILL.md)
skill and its chapter references. Do not cargo-cult a recommendation that
conflicts with an explicit Gearbox protocol or privacy boundary.

## 4. Verification workflow

Run the smallest useful check immediately after editing, then run the complete
gate before handoff:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked
```

At minimum, tests should cover the changed branch plus one failure/fallback
path. Routing tests should assert model, effort, source, and relevant safety
floors. Protocol tests should assert malformed responses, timeouts, and that
native approval/sandbox fields are not weakened. Hook tests should assert both
`continue` and `block` decisions. Privacy changes must prove prompt text does
not enter metrics.

For plugin changes, validate the plugin directory with the official
`plugin-creator` validator, reinstall through the configured marketplace, and
start a new Codex task before testing hook pickup.

## 5. Security and privacy review

Before completion, check that the change does not:

- expose the loopback proxy beyond `127.0.0.1`;
- weaken bearer-token authentication;
- read Codex credential files or print secrets;
- send tools, write access, or approvals to the judge;
- allow a free plan to invoke the judge;
- bypass model availability, rate-limit conservation, effort caps, or the
  high-risk safety floor;
- persist prompt text, judge prompt text, or user-identifying content.

When uncertain, fail closed and fall back to deterministic routing.

## 6. Release and compatibility

The two executable contracts are intentional:

```text
codex-gearbox              launch Codex Autopilot
codex-gearbox --version    print the product version
shift route <prompt>       inspect a route
shift account              inspect live account/model state
shift report               inspect aggregate local metrics
shift hook                 serve the desktop hook protocol
```

Keep plugin hook command names, README examples, release bundles, and Windows
variants synchronized. App Server WebSocket behavior is experimental; link to
the relevant official OpenAI documentation when changing protocol methods.

## 7. Handoff format

Summarize the behavior changed, files touched, checks run, and any known
limitation. Never claim a live judge or App Server path was tested if the
external Codex process was unavailable; report the deterministic fallback test
instead.

## 8. GitHub handoff

Work on a focused branch from `main`; do not push directly to `main`. Before
opening a pull request, use the repository template, link the issue or explain
why the change is documentation-only, list every check that ran, and call out
security, privacy, compatibility, or release impact. Keep review fixes in the
same branch and resolve conversations before handoff. Vulnerabilities belong
in the private channel documented by [`SECURITY.md`](../SECURITY.md), never in
public issue or PR text.
