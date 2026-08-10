# Contributing

Thanks for helping improve Codex Gearbox.

1. Open an issue for behavior changes or new routing signals.
2. Keep the router deterministic, explainable, and independent of prompt length alone.
3. Add a focused test for every routing or protocol change.
4. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
5. Do not add telemetry, prompt logging, or automatic credit actions.

Protocol changes should link to the relevant official Codex documentation or upstream schema.

