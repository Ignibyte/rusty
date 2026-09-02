# Rusty

A local-first AI assistant for [Omarchy](https://omarchy.org): a QML desktop app with a
pure MCP back end, an Obsidian-compatible markdown brain, to-do lists, notes, memories and
skills, and terminals that run Claude Code and Codex natively in tabs.

**Status: v3 rewrite in progress.** The manager layer and the MCP server are being lifted
from the earlier web version; the QML app is next. See `ROADMAP.md` for the plan and
`docs/architecture.md` for the shape.

## Status

Milestones M0 to M5 of `ROADMAP.md` landed on 2026-09-02: the back end with 59 tools, the
desktop app with agent terminals and the Tasks, Brain, Notes, Memory, Skills, Secrets and
Settings tabs, the Obsidian bridge, and semantic search behind a provider setting. What is
left in M6 is packaging for the Omarchy or AUR channel, retiring the v2 code, screenshots and
a first release.

## Run it

```bash
rusty                    # the app; or `omarchy-launch-or-focus rusty`
rusty-mcp                # the back end over stdio (agents); the user service serves HTTP
rusty-cli brain search "orbit"
```

In the app: the bar across the top has one button per agent CLI found on the machine (Claude
Code, Codex, Gemini, Aider, OpenCode) plus a shell; a click opens it in a new tab, each a tmux
session that outlives the window. Ctrl+Shift+T opens a custom tab (name, session, working
directory), Ctrl+Shift+W closes one, F2
renames, Ctrl+PgUp/PgDn (or Ctrl+Tab and Ctrl+Shift+Tab) switch tabs, Ctrl+Shift+PgUp/PgDn
move one, and tabs and tasks can be dragged into a new order. A tab that gets output
while another is showing gets a mark; a tab whose title asks for attention raises a desktop
notification. Under Omarchy the app's messages go to the journal: `journalctl --user -t rusty`;
`RUSTY_DEBUG=1` adds a line per tab event.
Tasks, Brain, Notes, Memory, Skills, Secrets and Settings talk to the back end over
`http://127.0.0.1:4174/mcp`, so a Claude session changing a task shows up in the app as it
happens.

## Layout

```
crates/rusty-core   the manager layer: tasks, notes, memories, brain vault + index, skills, secrets, settings
crates/rusty-mcp    the back end: an MCP server on rmcp, 59 tools, stdio and local Streamable HTTP
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

## Semantic search

`brain_search` merges full-text hits with nearest neighbours from `sqlite-vec` when an embedding
provider is configured; without one it stays full-text and nothing else changes. The settings
(`settings_set`, or the Settings tab): `embedding_provider` is `auto` (the default: Ollama when
it answers on this machine), `ollama`, `openai`, or `off`; `embedding_model` overrides the
provider's default (`nomic-embed-text`, `text-embedding-3-small`); `ollama_url` defaults to
`http://127.0.0.1:11434`. OpenAI needs `openai_api_key` (or `OPENAI_API_KEY`) in the secrets vault and sends page
text to OpenAI, so it is never picked by itself. The server embeds new and changed pages a few
seconds after they change; `rusty-cli brain embed --all` or the `brain_reembed` tool rebuilds,
and `rusty-cli brain semantic` shows the state. Changing the provider or model rebuilds the
index, because vectors from different models do not compare.

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
