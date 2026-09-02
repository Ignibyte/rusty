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
crates/rusty-mcp    the back end: an MCP server on rmcp, 57 tools, stdio and local Streamable HTTP
crates/rusty-app    the desktop app: a QML shell on cxx-qt with native agent terminals (binary `rusty`)
crates/rusty-cli    terminal access to the same store: brain, tasks, notes, refresh, conversation ingest
docs/               architecture and vault rules
omarchy/            the user service unit and MCP config snippets; installer, desktop entry and hooks arrive with M6
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

On a machine that will run Rusty for real, `omarchy/install.sh` builds both binaries into
`~/.local/bin`, installs and starts the `rusty-mcp` user service, and registers the brain with
Obsidian when it is installed. Run it again after pulling; every step is idempotent.

## Obsidian

The brain folder is a plain Obsidian vault, and six tools (`obsidian_status`, `obsidian_open`,
`obsidian_backlinks`, `obsidian_links`, `obsidian_unresolved`, `obsidian_rename_page`) reach the
running app through Obsidian's own command-line interface (Obsidian 1.12.4 or newer). The app has
to know the vault and have the CLI switched on; `rusty-cli obsidian register` writes both into
Obsidian's config while the app is closed (and sets the vault to rewrite links on rename without
asking, so `obsidian_rename_page` never waits on a dialog), and `rusty-cli obsidian open <slug>` starts the app
when it is not running. Obsidian's per-machine state in `.obsidian/` stays out of the vault's git
history. Without Obsidian, those six tools answer with a clear error and nothing else changes.

Two vault rules keep the two writers agreeing. A page's timeline is its `## Timeline` section,
and wikilinks are vault paths (`[[projects/orbit]]`). `rusty-cli brain migrate --dry-run` shows what
an older vault would change; without the flag it rewrites the pages, reindexes, and commits.

On Arch the package launcher passes `~/.config/obsidian/user-flags.conf` to every invocation,
including CLI calls. A single-dash flag there (`-disable-gpu`, which Omarchy ships) reaches the
CLI as a command; write it as `--disable-gpu` and the CLI drops it.

## License

MIT. The bundled `no-ai-slop` skill text is vendored from
[petergyang/no-ai-slop](https://github.com/petergyang/no-ai-slop) (MIT).
