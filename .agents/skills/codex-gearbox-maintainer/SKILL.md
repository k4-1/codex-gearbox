---
name: codex-gearbox-maintainer
description: Maintain Codex Gearbox safely across its Rust router, Codex App Server protocol, CLI proxy, desktop hook plugin, privacy metrics, and release workflow. Use when implementing, reviewing, testing, or releasing changes in this repository.
---

# Codex Gearbox Maintainer

Use this skill for any code, protocol, plugin, security, or release change in
Codex Gearbox. Read [`AGENTS.md`](../../../AGENTS.md) first, then load only the
relevant rules below.

For Rust implementation details, also apply the pinned upstream
[`rust-best-practices`](../rust-best-practices/SKILL.md) skill and read only the
relevant chapter references. The Gearbox rules below override it when they are
more specific about protocol safety, privacy, plan policy, or CLI compatibility.
Choose any security audit companion using the non-overlapping routing table in
[`AGENTS.md`](../../../AGENTS.md#skill-routing-avoid-duplicate-audits).

## Rule index

1. [`architecture-routing.md`](rules/architecture-routing.md) — pure policy,
   boundaries, SOLID/KISS/YAGNI.
2. [`protocol-safety.md`](rules/protocol-safety.md) — App Server, proxy, hook,
   validation, timeouts, and safe fallback.
3. [`account-plan-policy.md`](rules/account-plan-policy.md) — free/subscribed/
   API-key gating, availability, rate limits, and safety floors.
4. [`privacy-metrics.md`](rules/privacy-metrics.md) — prompt minimization,
   credentials, loopback security, and metrics.
5. [`cli-plugin-contract.md`](rules/cli-plugin-contract.md) — binary names,
   command behavior, plugin hooks, and compatibility.
6. [`testing-release.md`](rules/testing-release.md) — focused tests, required
   checks, plugin validation, and release packaging.

## Workflow

1. Identify the owning module before editing.
2. Read the relevant rule file(s) and existing tests.
3. Implement the smallest change that preserves the existing contract.
4. Add a focused regression test, including a failure/fallback path when the
   change crosses a trust or protocol boundary.
5. Run formatting, Clippy, tests, release build, and plugin validation when
   applicable.
6. Update README/plugin metadata if a public command or hook contract changed.
7. Report checks and limitations accurately.

Do not introduce telemetry, prompt storage, speculative abstractions, model
aliases, or automatic credit actions without an explicit product decision.
