# Rusty v3 roadmap

Rusty is a local-first AI assistant built for [Omarchy](https://omarchy.org). It is a QML
desktop app with a pure MCP back end. Its wiki is an Obsidian-compatible markdown vault,
and its terminals run Claude Code and Codex natively. This file is the plan; the shape is in
`docs/architecture.md`.

## Principles

- **Omarchy is a dependency, not a target.** Rusty assumes Hyprland, uwsm, the `omarchy`
  CLI, its theme files and its launcher conventions, and uses them instead of reinventing
  them. It installs like an Omarchy web app or TUI would.
- **Files are the truth.** The brain is a folder of markdown that Obsidian can open. SQLite
  holds the index (full text and vectors), never the only copy of anything.
- **One back end process.** `rusty-mcp` serves the app over local HTTP and the agents over
  stdio. There is no web UI, no REST layer, no second protocol.
- **Native terminals.** Claude Code and Codex run in real terminal emulators inside the app,
  attached to tmux sessions, so nothing is lost when the app closes.
- **Same gates as before.** rustfmt, clippy with warnings as errors, tests, docs, on every
  change.

## Milestones

### M0. Foundation (now)

- Public repo, MIT, Cargo workspace.
- `rusty-core`: the manager layer lifted from v2 (tasks, notes, memories, brain, skills,
  secrets, settings, events, watcher) with the web, auth, TLS and PTY code removed. Gates green.
- `rusty-mcp`: an `rmcp` server over stdio exposing the first tool set (tasks, notes,
  memories, brain search and read, skills list). Usable from Claude Code today.
- A one-day PySide6 prototype proving the terminal item, the tmux attach flow and Omarchy
  theme colours before any Rust UI is written.

Done when: `cargo test` passes, Claude Code lists the tools over stdio, the prototype runs
`claude` in a tab that survives closing the window.

**Status 2026-09-02: done.** 190 tests green under fmt, clippy and docs; 19 tools over stdio
and, ahead of M1, over Streamable HTTP at `127.0.0.1:4174/mcp`; the prototype shows Claude
Code and Codex in tmux-backed tabs behind a side rail, themed from the Omarchy theme's own
Alacritty colours and font, with a Settings tab that switches the agent terminals between
the embedded widget and real Alacritty windows on the same sessions.

### M1. The back end, complete

- Full tool and resource surface: tasks and lists, notes, memories, brain pages and
  timeline, skills (read, write, approve), secrets (names only, set, delete), settings.
- Streamable HTTP transport on localhost next to stdio; change notifications driven by the
  file watcher replace the old WebSocket bus.
- `rusty-cli` ported onto the same core.
- Obsidian CLI bridge: `open_in_obsidian`, link-safe `rename_page`, `backlinks`, used when the
  app is running and skipped cleanly when it is not.
- Vault rules for v3: the timeline is a normal `## Timeline` section (no body `---` trick);
  wikilinks use one path style Obsidian and Rusty both write.

Done when: every v2 GUI action has an MCP equivalent and the old server is not needed.

### M2. The app shell and the first two tabs

- Qt Quick app through `cxx-qt`, one window, a tab bar, an MCP client to the local server.
- **Terminals**: `qmltermwidget` per tab, each attached to a tmux session
  (`rusty-<name>`), Claude or Codex per tab, new tab and rename, sessions listed from tmux.
  A per-agent terminal mode, embedded or an Alacritty window on the same session, since
  Wayland cannot embed a foreign window; the prototype already has the setting.
- **Tasks**: to-do lists with groups, quick add, toggle, archive, keyboard first.
- Omarchy fit: `.desktop` entry and icon, stable `app_id`, `omarchy launch or focus rusty`,
  a default key, theme colours from `~/.config/omarchy/current/theme/colors.toml`, live
  re-theme through the `theme-set` hook.

Done when: the app replaces a terminal for daily Claude and Codex use.

### M3. Knowledge tabs

- **Brain**: folder tree, full-text search, rendered read view, backlinks and timeline,
  open in Obsidian, capture and append through MCP.
- **Notes**: daily notes living in the vault so Obsidian sees them too.
- **Memory**: list, add, edit, delete.
- Vault registration: `~/.rusty/brain` opened as an Obsidian vault, `.obsidian/` policy
  decided, Obsidian themed by `omarchy theme set` where the theme provides it.

### M4. Semantic search

- `sqlite-vec` inside `rusty.db`: one embedding per page, note and memory chunk.
- Embedding providers behind one trait: Ollama locally when present (default), OpenAI
  through the secrets vault otherwise. No key, no vectors, and the app still works.
- Hybrid retrieval (FTS5 and vectors merged) behind `brain_search`, incremental indexing
  from the watcher, a re-embed command.

### M5. Skills, secrets, settings

- **Skills**: editable inside the app, frontmatter as a form, body in an editor, the
  staging and approval flow and the safety scan carried over from v2.
- **Secrets**: the vault as a list, set and delete, values never rendered twice.
- **Settings**: paths, providers, theme sync, keybinding hints.

### M6. Omarchy packaging and cutover

- `omarchy/install.sh`: dependencies through `omarchy pkg add`, the desktop entry, a
  keybinding snippet, the theme hook, the MCP config snippet for Claude Code and Codex.
- Packaging for the Omarchy or AUR channel once the install script is boring.
- Retire the v2 web UI and server. Docs, screenshots, release.

### Later

- Background agents tab (dispatch, watch, results).
- Phone access through Obsidian Sync of the vault.
- A macOS build, if anyone wants one; nothing in the core is Linux-only.

## Non-goals

No browser UI, no voice, no cloud service, no multi-user. Rusty runs on one machine for one
person and talks to the agents that person already uses.
