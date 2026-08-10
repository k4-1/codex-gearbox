# Codex Gearbox agent contract

Read this file before changing code. It is the short contract for humans and
AI agents; detailed procedures live in [`docs/CONTRIBUTING-AI.md`](docs/CONTRIBUTING-AI.md)
and the versioned maintainer skill at
[`.agents/skills/codex-gearbox-maintainer/SKILL.md`](.agents/skills/codex-gearbox-maintainer/SKILL.md).
For idiomatic Rust ownership, error handling, Clippy, performance, testing,
and documentation guidance, also apply the pinned Apollo GraphQL skill at
[`rust-best-practices`](.agents/skills/rust-best-practices/SKILL.md). Project
security, privacy, protocol, and CLI rules take precedence when they are more
specific.

## Repository map

- `src/routing.rs` — pure, explainable scoring and model/effort policy.
- `src/app_server.rs` — Codex App Server lifecycle, protocol, account/model
  discovery, and the optional Luna judge.
- `src/proxy.rs` — CLI proxy and `turn/start` route injection.
- `src/hook.rs` — desktop `UserPromptSubmit` adapter; it may advise or block,
  but must not pretend hooks can mutate model settings.
- `src/config.rs` and `src/metrics.rs` — local configuration and privacy-safe
  metrics only.
- `src/main.rs` / `src/shift.rs` — the `codex-gearbox` launcher and `shift`
  utility entry points.
- `plugins/codex-gearbox/` — desktop plugin manifest and hook declaration.
- `.github/workflows/` — cross-platform CI and tagged release packaging.

## Non-negotiable rules

1. Keep routing deterministic, explainable, and independent of prompt length
   alone. A model judge may advise; policy and availability make the final
   decision.
2. Treat App Server and hook payloads as untrusted protocol boundaries. Validate
   shapes, use timeouts, preserve native approval/sandbox behavior, and fail
   closed for unsafe or malformed data.
3. Free plans never invoke the Luna judge. Never bypass account, model
   availability, rate-limit, effort-cap, or high-risk safety-floor checks.
4. Never store or print prompt text, credentials, bearer tokens, or API keys.
   Metrics may contain only aggregate routing metadata.
5. Keep `src/routing.rs` pure where possible. Use traits only at real external
   boundaries; do not add abstractions for one implementation (YAGNI/KISS).
6. Preserve both CLI contracts: `codex-gearbox` launches Codex and reports its
   version; `shift route|account|report|hook` provides utilities.
7. Prefer feature-local changes and small diffs. Do not reorganize the project
   or add dependencies without a concrete need and verification.
8. Every routing, protocol, security, privacy, or CLI change needs a focused
   regression test and documentation update when the contract changes.

## Skill routing (avoid duplicate audits)

Use `codex-gearbox-maintainer` for every repository change and
`rust-best-practices` for Rust implementation or review. Add at most one
security audit skill for a given pass:

| Need | Skill |
| --- | --- |
| Fast changed-code check for secrets and `unsafe` usage | `general-security` |
| Rust dependency, advisory, fuzzing, or vulnerability-management review | `security-and-vulnerability-management` |
| Periodic whole-repository NIST/CWE hardening assessment | `harden` |
| Pre-publication secret, PII, and Git-history review | `open-source-checker` |

Do not stack these security skills “for completeness.” Choose the narrowest
one that matches the task. `harden` owns a comprehensive audit; the others
should only be added when their distinct release or history checks are needed.

## Required checks

Run the narrowest relevant check first, then the full local gate:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked
```

For plugin changes, also run the official plugin validator on
`plugins/codex-gearbox/`. Before release, verify both binaries and the desktop
hook from a clean installation path.

Do not report a change complete while a required check is failing. If a check
cannot run because an external service or tool is unavailable, state that
explicitly and keep the deterministic fallback intact.

## Definition of done

- The smallest correct change is implemented.
- New behavior has a focused test, including a failure or fallback path where
  relevant.
- Privacy, plan gating, protocol compatibility, and error handling are covered.
- Documentation and plugin metadata match the executable behavior.
- Local checks pass and `git diff --check` is clean.
