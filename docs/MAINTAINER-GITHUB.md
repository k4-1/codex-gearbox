# GitHub maintainer workflow

This document describes the repository settings and release flow that cannot
be enforced by files alone.

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

## Release tags

1. Merge the release preparation change into a green `main`.
2. Update `Cargo.toml` and `Cargo.lock` when the package version changes.
3. Run the full local gate from `CONTRIBUTING.md`.
4. Create and push an annotated semantic-version tag:

   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```

5. The tag starts `.github/workflows/release.yml`, which builds Linux, macOS,
   and Windows archives.
6. GitHub generates categorized release notes using [`.github/release.yml`](../.github/release.yml).
   Review the wording, remove internal details, and add a short context paragraph
   when the generated notes need clarification.
7. Verify both `codex-gearbox` and `shift` are present in each archive, test
   `codex-gearbox --version`, review generated release notes, and announce the
   release only after the artifacts are usable.

The generated GitHub release history is the canonical changelog. Keep
[`CHANGELOG.md`](../CHANGELOG.md) as the short pointer and contributor guide;
do not duplicate every release entry there.

Never put credentials, prompts, or private account data in release notes,
issues, commits, or pull requests.
