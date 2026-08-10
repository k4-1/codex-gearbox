# Security Policy

Please do not report vulnerabilities in public issues or pull requests.

Use GitHub's private vulnerability reporting form:

<https://github.com/k4-1/codex-gearbox/security/advisories/new>

Include a concise description, affected version or commit, safe reproduction
steps, and impact. Remove prompts, credentials, API keys, bearer tokens, and
personal data from all evidence.

Codex Gearbox binds its proxy to loopback, authenticates the local TUI
connection with a random bearer token, never reads Codex credential files, and
never records prompt text. It forwards the native Codex approval and sandbox
protocol unchanged.

The App Server WebSocket transport is experimental. Do not expose the Gearbox
proxy port to another machine.

Security reports should cover issues such as credential exposure, proxy
authentication bypass, unsafe hook behavior, plan-gating bypasses, or a route
that weakens the configured safety floor. The maintainer will acknowledge and
triage reports privately before deciding on disclosure and a patched release.
