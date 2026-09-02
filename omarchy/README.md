# Omarchy integration

What makes Rusty an Omarchy app rather than a program that happens to run there. The
installer that ties these together lands with M6; until then each piece is applied by hand.

| File | Purpose |
|---|---|
| `rusty-mcp.service` | `systemd --user` unit running the back end over Streamable HTTP on localhost |
| `mcp-config.json` | the `mcpServers` entries for Claude Code and Codex (stdio) and for HTTP clients |

Coming with M2 and M6: the desktop entry and icon, a `bindings.conf` snippet for the launch
key, the `~/.config/omarchy/hooks/theme-set` hook that re-themes a running Rusty, and
`install.sh`, which installs dependencies through `omarchy pkg add` and wires all of it.

Conventions Rusty follows: apps launch through `uwsm-app`, the theme lives in
`~/.config/omarchy/current/theme/` (`colors.toml`, `alacritty.toml`), windows are found and
focused with `omarchy launch or focus`, and nothing under `~/.local/share/omarchy/` is ever
edited.
