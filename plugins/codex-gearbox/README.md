# Codex Gearbox desktop advisor

This plugin runs the locally installed `shift hook` command before each desktop prompt. It recommends the best model and reasoning effort, but remains advisory: prompts continue with the user's selected model and effort. The hook resolves the standard `$HOME/.local/bin` and `$HOME/.cargo/bin` install paths because desktop Codex may not inherit the terminal `PATH`.

Install the native helper first. The plugin cannot automatically change desktop model settings until Codex exposes a before-turn model mutation API.
