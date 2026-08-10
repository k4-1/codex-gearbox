# CLI and desktop plugin contract

## Why it matters

Users and the Codex desktop plugin invoke the binaries by name. A small command
rename can silently disable desktop routing or break scripts.

## Canonical commands

```text
codex-gearbox              launch Codex Autopilot
codex-gearbox --version    print the product version
shift route <prompt>       preview a route
shift account              inspect account/model/rate data
shift report               inspect aggregate metrics
shift hook                 handle UserPromptSubmit JSON
```

## Rules

- Keep the two executable entry points installed by Cargo and synchronized.
- Preserve the native Codex arguments passed through the main launcher.
- When changing a command, update `src/main.rs`, `src/shift.rs`, README
  examples, plugin hook commands, Windows variants, and release packaging.
- Keep desktop behavior honest: a hook can recommend or block a model mismatch;
  it cannot change active effort unless the host API explicitly supports it.
- After plugin changes, bump the local cachebuster with the plugin-creator
  update script, reinstall from the configured marketplace, and use a new Codex
  task to test pickup.
- Keep marketplace paths relative and plugin manifests valid.

## Compatibility

Prefer preserving a harmless legacy alias when it costs little, but make the
canonical command and documentation unambiguous.
