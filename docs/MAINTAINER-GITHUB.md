# GitHub maintainer workflow

This document describes the repository settings and release flow that cannot
be enforced by files alone.

Before enabling automated releases, set Actions workflow permissions to
**Read and write permissions** and enable **Allow GitHub Actions to create and
approve pull requests** in the repository's Actions settings. The workflow
still creates a reviewable release PR; it does not merge that PR automatically.

## Protect `main`

In GitHub repository settings, create a branch rule for `main` with:

- Pull requests required before merging.
- At least one approving review; dismiss stale approvals after new commits.
- Required status checks for the Ubuntu, macOS, and Windows `CI / test` jobs.
- All review conversations resolved before merge.
- Branches required to be up to date before merging when practical.
- Force pushes and branch deletion blocked.

Keep administrator bypass for emergencies only, and record any emergency
change in an issue or release note afterward.

## Review and merge policy

1. A maintainer confirms the PR has a linked issue or a clear documentation
   rationale.
2. CI must pass on the exact commit being merged.
3. Review checks correctness, protocol compatibility, privacy, plan gating,
   and the fallback path—not just formatting.
4. The author resolves conversations and rebases or updates the branch when
   requested.
5. Use squash merge for focused changes; keep the issue and PR references in
   the resulting commit message.

## Automated releases

1. Merge normal Conventional Commit pull requests into a green `main`.
2. The release workflow opens or updates a release PR with the calculated
   semantic version, `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md` changes.
3. Review the release PR and generated notes, then merge it when ready.
4. The workflow creates and pushes the annotated `vX.Y.Z` tag automatically
   after the release PR is merged.

5. The tag-triggered release-assets workflow builds all command binaries for Linux,
   Intel and Apple Silicon macOS, and Windows, then attaches them to the GitHub
   release. The updater selects the matching asset by target triple.
6. Release-please generates categorized release notes using
   [`release-please-config.json`](../release-please-config.json).
   Review the wording, remove internal details, and add a short context paragraph
   when the generated notes need clarification.
7. Review the binaries, source archives, and generated notes, test the versioned source
   checkout with `cargo build --release --locked` when needed, and announce the
   release only after it is usable.

The generated GitHub release history is the canonical changelog. Keep
[`CHANGELOG.md`](../CHANGELOG.md) as the short pointer and contributor guide;
do not duplicate every release entry there.

Never put credentials, prompts, or private account data in release notes,
issues, commits, or pull requests.
