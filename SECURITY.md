# Security Policy

Please report vulnerabilities privately to the repository maintainers rather than opening a public issue.

Codex Gearbox binds its proxy to loopback, authenticates the local TUI connection with a random bearer token, never reads Codex credential files, and never records prompt text. It forwards the native Codex approval and sandbox protocol unchanged.

The App Server WebSocket transport is experimental. Do not expose the Gearbox proxy port to another machine.

