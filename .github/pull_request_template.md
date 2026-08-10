## Summary

<!--
Use a Conventional Commit title: `feat: ...`, `fix: ...`, `perf: ...`,
`fix(security): ...`, `docs: ...`, or `chore: ...`. Add `!` for a breaking change.
Link the issue with `Closes #123` when applicable.
-->

## User-facing release note

<!--
Write one or two plain-language sentences describing the user benefit. Avoid
implementation terms. If there is no user-visible change, write: No user-facing change.
The PR title is included in generated notes; maintainers can use this note to
polish the release body.
-->

No user-facing change.

## Validation

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test --all-targets`
- [ ] `cargo build --release --locked`
- [ ] Plugin validation ran, if `plugins/codex-gearbox/` changed

## Review checklist

- [ ] The change is focused and avoids unrelated refactoring.
- [ ] Routing remains deterministic and preserves plan, availability, rate-limit, and safety-floor rules.
- [ ] No prompts, credentials, bearer tokens, or API keys are logged or persisted.
- [ ] A fallback or failure path is tested when a trust boundary changed.
- [ ] Documentation and public CLI/plugin contracts are updated when needed.
- [ ] Any breaking change or release impact is called out above.
- [ ] The PR title is concise and understandable to a non-technical user.
