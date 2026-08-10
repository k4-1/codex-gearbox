# Testing and release discipline

## Why it matters

The project ships a native binary and a desktop plugin across operating
systems. A local macOS pass alone cannot validate the release contract.

## Required local gate

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --locked
```

## Rules

- Add focused unit tests beside routing/protocol logic; test behavior rather
  than implementation details.
- Include a failure, timeout, unavailable-model, or privacy assertion when the
  changed branch has one.
- Keep the GitHub matrix green on Linux, macOS, and Windows.
- Validate `plugins/codex-gearbox/` with the official plugin validator after
  manifest or hook changes.
- Ensure release bundles contain both `codex-gearbox` and `shift` binaries.
- Never call a live judge or App Server test successful when only a local
  deterministic fallback was exercised.
- Avoid tests that mutate real user settings, send real coding tasks, consume
  unnecessary model credits, or expose prompt text.

## Completion

Report the exact checks run, their result, and any external test that could not
be performed.
