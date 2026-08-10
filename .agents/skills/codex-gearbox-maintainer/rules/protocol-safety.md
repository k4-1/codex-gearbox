# Protocol and trust-boundary safety

## Why it matters

Codex App Server, CLI output, hook input, and judge output are external
protocols. They can change, be malformed, or fail while a user's task is in
flight.

## Rules

- Deserialize into typed structs where a stable shape is required; validate
  required fields before use.
- Bound every process, socket, judge, and cleanup wait with a timeout or a
  terminating fallback.
- Keep the proxy loopback-only and preserve the native Codex approval and
  sandbox fields when forwarding turns.
- The Luna judge is read-only, has no tools, uses strict structured output, and
  is advisory. Invalid, low-confidence, timed-out, or unavailable judge output
  must fall back to deterministic routing.
- Hook output must use the documented decision shape. Hooks may continue,
  provide context, warn, or block; they must not claim to mutate model/effort.
- Never panic on user/protocol input. Return a useful error or safe fallback.

## Verification

Protocol changes require tests for valid input, malformed input, unavailable
server, timeout, and preserved safety/approval fields.
