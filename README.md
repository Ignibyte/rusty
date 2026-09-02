# Rusty

A local-first AI assistant for [Omarchy](https://omarchy.org): a QML desktop app with a
pure MCP back end, an Obsidian-compatible markdown brain, to-do lists, notes, memories and
skills, and terminals that run Claude Code and Codex natively in tabs.

**Status: v3 rewrite in progress.** The manager layer and the MCP server are being lifted
from the earlier web version; the QML app is next. See `ROADMAP.md` for the plan and
`docs/architecture.md` for the shape.

## Layout

```
crates/rusty-core   the manager layer: tasks, notes, memories, brain vault + index, skills, secrets, settings
crates/rusty-mcp    the back end: an MCP server (stdio now, local HTTP next) built on rmcp
prototype/          a PySide6 spike for the terminal tab and Omarchy theming; deleted once the app exists
docs/               architecture and vault rules
omarchy/            install script, desktop entry, hooks (arriving with M6)
```

## Build

```bash
cargo build
cargo test
```

Data lives in `~/.rusty/`: `rusty.db` (SQLite), `brain/` (the vault), `notes/`, `skills/`,
`.secret` (the vault of secrets, mode 600).

## Use the MCP server

From Claude Code or Codex over stdio:

```json
{ "mcpServers": { "rusty": { "type": "stdio", "command": "rusty-mcp" } } }
```

For the app, or any HTTP client, one shared process on localhost:

```bash
rusty-mcp --http                 # Streamable HTTP at http://127.0.0.1:4174/mcp
```

## Prototype

`python3 prototype/main.py` under Omarchy (needs `pyside6`, `qmltermwidget`, `qt6-wayland`,
`tmux`). Claude and Codex tabs attach to tmux sessions; the Settings tab chooses between the
embedded terminal, coloured from the Omarchy theme's Alacritty palette, and real Alacritty
windows on the same sessions. See `prototype/README.md`.

## License

MIT. The bundled `no-ai-slop` skill text is vendored from
[petergyang/no-ai-slop](https://github.com/petergyang/no-ai-slop) (MIT).
