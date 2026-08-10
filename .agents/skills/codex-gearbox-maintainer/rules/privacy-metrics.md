# Privacy, credentials, and local security

## Why it matters

Gearbox sits beside a coding agent and sees task prompts. The default must be
data minimization, not retrospective cleanup.

## Rules

- Never persist or print prompt text, judge input, Codex credentials, API keys,
  bearer tokens, or full protocol payloads.
- Metrics may contain timestamp, account class, rate band, model count, source,
  model, effort, role, and confidence only.
- Keep the proxy bound to `127.0.0.1`; authenticate local connections with a
  random token and do not expose a configurable remote bind by default.
- Do not read Codex credential files. Use the supported App Server interface.
- Do not add telemetry, remote logging, analytics, or prompt hashing without an
  explicit privacy decision and documentation.
- Use secure cleanup for temporary protocol resources and avoid leaking values
  in errors.

## Verification

Privacy/security changes require a test or inspection proving prompt text is
absent from metrics and that loopback/authentication behavior remains intact.
