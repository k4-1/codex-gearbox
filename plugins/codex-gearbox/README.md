# Codex Gearbox desktop advisor

This plugin runs the locally installed `shift hook` command before each desktop prompt. When the selected model is stronger than the recommendation, it pauses the first submission. Change the model and resend, or resend unchanged once to proceed anyway. The hook resolves the standard `$HOME/.local/bin` and `$HOME/.cargo/bin` install paths because desktop Codex may not inherit the terminal `PATH`.

Install the native helper first. The plugin cannot automatically change desktop model settings, and the current hook payload does not expose selected reasoning effort, so effort remains recommendation-only on desktop.

The hook is intentionally a stable launcher. Gearbox routing behavior updates
through the verified `shift` binary updater, so users normally do not need to
reinstall the plugin for routing changes. Refresh the Git marketplace when the
plugin manifest or hook definition itself changes.
