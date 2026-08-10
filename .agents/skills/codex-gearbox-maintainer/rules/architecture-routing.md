# Architecture and routing boundaries

## Why it matters

Routing is the product's policy engine. Keeping it pure and explainable makes
free-plan behavior auditable, tests cheap, and judge failures recoverable.

## Rules

- Keep prompt scoring and route resolution in `src/routing.rs`.
- Prefer pure functions and explicit value types (`RouteDecision`, scores,
  model metadata, account/rate bands).
- Keep App Server I/O in `src/app_server.rs`; keep process/socket forwarding in
  `src/proxy.rs`; keep hook serialization in `src/hook.rs`.
- Route through one policy path. Do not reimplement model selection in the
  proxy, hook, CLI, or plugin.
- Apply SOLID where it reduces coupling at a real boundary. Do not wrap every
  function in a trait or add a factory for one implementation.
- Treat YAGNI/KISS as design constraints: no speculative provider registry,
  database, background daemon, or prompt cache.

## Avoid

- Selecting a model from prompt length alone.
- Letting a judge directly override availability, account, rate, risk, or user
  effort policy.
- Adding state to the router when the decision can remain a value calculation.
