# Contributing

Thanks for helping improve Codex Gearbox.

Before editing, read [`AGENTS.md`](AGENTS.md) and the detailed
[`AI contribution guide`](docs/CONTRIBUTING-AI.md). Use the maintainer skill and
only the rule files relevant to the change.

## Contributor workflow

1. Search existing issues, then open one for behavior changes, new routing
   signals, or protocol changes. Documentation-only fixes may go directly to
   a pull request. Report security vulnerabilities privately through
   [`SECURITY.md`](SECURITY.md).
2. Fork the repository, clone your fork, and create a focused branch from
   `main`. Use names such as `feat/routing-signal`, `fix/hook-timeout`, or
   `docs/contributing-flow`.
3. Make the smallest change that solves the issue. Keep the router
   deterministic, explainable, and independent of prompt length alone.
4. Add a focused test for every routing, protocol, security, privacy, or CLI
   change. Update documentation when a public contract changes.
5. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test --all-targets`, and `cargo build --release --locked`.
6. Validate `plugins/codex-gearbox/` after changing its manifest or hooks.
7. Push the branch and open a pull request using the repository template. Link
   the issue, use a Conventional Commit title, describe the behavior and risk,
   and list the checks you ran.
8. Keep the branch focused while review is in progress. Resolve conversations
   and update the PR when requested; maintainers merge only after required CI
   checks and approvals pass.

## Pull requests and releases

Pull requests target `main`; do not push directly to it. The maintainer merge
policy and branch-protection checklist are in
[`docs/MAINTAINER-GITHUB.md`](docs/MAINTAINER-GITHUB.md).

Releases are cut from a green `main` by merging the automated release PR. The
workflow calculates the version, updates the package and changelog, creates an
annotated `vX.Y.Z` tag, and publishes the cross-platform archives. Maintainers
verify the generated archives and release notes before announcing the release.
Release-please groups those notes using `release-please-config.json`, so
contributors should keep PR titles plain-language and maintainers should apply
one release label before merging.

Version selection is automated from Conventional Commits by
[`release-please`](https://github.com/googleapis/release-please):

- `fix:`, `perf:` → patch (`0.1.0` → `0.1.1`)
- `feat:` → minor (`0.1.0` → `0.2.0`)
- `feat!:`, `fix!:`, or `BREAKING CHANGE:` → breaking release (`0.1.0` → `0.2.0` while pre-`1.0.0`)
- `docs:`, `test:`, and `chore:` → no release

The action opens a release PR with the calculated version and changelog. Merge
that PR to create the `vX.Y.Z` tag and publish the platform archives.

Never report secrets or private prompts in issues or pull requests. Follow
[`SECURITY.md`](SECURITY.md) for vulnerabilities.

## Project invariants

- Do not add telemetry, prompt logging, or automatic credit actions.

Protocol changes should link to the relevant official Codex documentation or upstream schema.
