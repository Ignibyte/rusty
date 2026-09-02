# Rusty v3 roadmap

The tracker for the rewrite: a QML desktop app for [Omarchy](https://omarchy.org) with a
pure MCP back end, an Obsidian-compatible markdown brain, to-do lists, notes, memories,
editable skills, and native Claude Code and Codex terminals. Tick items as they land; when
something unplanned turns up, add it to the milestone it belongs to rather than a side list.
The shape is in `docs/architecture.md`.

## Where we are

| Milestone | State |
|---|---|
| M0 Foundation | done 2026-09-02 |
| M1 The back end, complete | next |
| M2 App shell, Terminals, Tasks | |
| M3 Knowledge tabs: Brain, Notes, Memory | |
| M4 Semantic search | |
| M5 Skills, Secrets, Settings | |
| M6 Omarchy packaging and cutover | |

## M0. Foundation (done 2026-09-02)

- [x] Public repo `Ignibyte/rusty`, MIT, Cargo workspace, clean history
- [x] `rusty-core` lifted from v2: tasks, notes, memories, brain vault and index, skills,
      secrets, settings, events, file watcher; web, auth, TLS and PTY code left behind
- [x] Gates green: rustfmt, clippy with warnings as errors, 190 tests, docs
- [x] `rusty-mcp` on rmcp 3: 19 tools over stdio, verified against a real store
- [x] `rusty-mcp --http`: Streamable HTTP at `127.0.0.1:4174/mcp`, verified with curl
- [x] Prototype: PySide6 + qmltermwidget, side rail, Claude and Codex tabs on tmux
      sessions, Omarchy theme colours and font, Ctrl+PgUp/PgDn tab cycling
- [x] Decision: agent terminals are always embedded; the Alacritty-window mode was tried
      and removed. The Settings page stays as a page.

## M1. The back end, complete

Done when every v2 GUI action has an MCP equivalent, the dev box's own `.mcp.json`
points at v3, and the v2 server is not needed.

Tools and resources

- [x] Tasks: rename and delete a list, update a task title, unarchive, delete
- [ ] Tasks: reorder
- [x] Notes: create, rename, delete
- [ ] Notes: daily note open-or-create
- [x] Memories: delete
- [ ] Memories: update
- [x] Brain: read timeline, links and backlinks, set page body, delete page, resolve a
      partial slug
- [ ] Brain: capture into the inbox, page types listed
- [x] Skills: create (active or staged), approve, reject, delete, safety scan on demand
- [ ] Skills: update frontmatter and body in place
- [x] Secrets: list names, set, delete; values are never returned
- [x] Settings: get, set
- [ ] Settings: list
- [ ] Resources: `rusty://tasks`, `rusty://brain/<slug>`, `rusty://notes/<path>`, with
      `resources/updated` notifications fed by the file watcher
- [x] Router tests: every tool advertised once, every tool described
- [ ] An rmcp-client smoke test that runs in CI

Transport and services

- [x] A `systemd --user` unit for `rusty-mcp --http` (`omarchy/rusty-mcp.service`; running on
      the dev box)
- [ ] The installer enables it
- [x] MCP config snippets for Claude Code and Codex (`omarchy/mcp-config.json`)
- [x] `rusty-cli` ported onto `rusty-core`, installed over the v2 binary

Obsidian and the vault

- [ ] Obsidian CLI bridge: `open_in_obsidian`, link-safe `rename_page`, `backlinks`;
      skip cleanly when the app is not running
- [ ] Vault rule: the timeline becomes a `## Timeline` section; the body `---` trick goes,
      with a one-time migration of existing pages
- [ ] Vault rule: one wikilink path style that Obsidian and Rusty both write
- [ ] Register `~/.rusty/brain` as an Obsidian vault; decide the `.obsidian/` policy

Cutover for the back end

- [x] Parity check against v2's tool list: all 29 present
- [x] Switch the dev box's `.mcp.json` to v3 and retire the v2 `rusty-mcp`

## M2. App shell, Terminals, Tasks

Done when the app replaces a terminal for daily Claude and Codex use.

- [ ] `rusty-app` crate: cxx-qt on Qt 6, one window, side rail, tab stack
- [ ] MCP client over Streamable HTTP with typed models for the tabs
- [ ] Terminals: qmltermwidget per tab, one tmux session per tab, new tab and rename, close,
      pick the agent (Claude, Codex, shell), list existing sessions
- [ ] Terminal colours from the Omarchy theme through a registered scheme directory, no
      root-owned file; font from the Alacritty config
- [ ] Live re-theme through `~/.config/omarchy/hooks/theme-set`
- [ ] Tasks: lists, quick add, toggle, archive, rename, reorder, keyboard first, live updates
      from notifications
- [ ] Settings page: paths, theme and terminal font shown; the first editable settings
- [ ] Desktop entry, icon, stable `app_id`, `omarchy launch or focus rusty`, a default key
- [ ] Delete the prototype

## M3. Knowledge tabs

- [ ] Brain: folder tree, full-text search, rendered markdown view, backlinks and timeline,
      open in Obsidian, capture and append
- [ ] Notes: daily notes living in the vault so Obsidian sees them
- [ ] Memory: list, add, edit, delete
- [ ] Obsidian themed by `omarchy theme set` where the theme provides it

## M4. Semantic search

- [ ] `sqlite-vec` inside `rusty.db`: an embedding per page, note and memory chunk
- [ ] Embedding provider trait; Ollama when it is running (default), OpenAI through the
      secrets vault otherwise; no provider means no vectors and everything else still works
- [ ] Hybrid retrieval, full text and vectors merged, behind `brain_search`
- [ ] Incremental indexing from the watcher; a re-embed command
- [ ] Settings entries for the provider and the model

## M5. Skills, Secrets, Settings

- [ ] Skills: editor inside the app, frontmatter as a form, body in an editor, staging and
      approval, the safety scan, delete
- [ ] Secrets: names listed, set and delete, a value never rendered after entry
- [ ] Settings: every setting the earlier milestones introduced, in one place

## M6. Omarchy packaging and cutover

- [ ] `omarchy/install.sh`: dependencies through `omarchy pkg add`, the desktop entry, a
      keybinding snippet, the theme hook, the MCP config snippet, the user service
- [ ] A package for the Omarchy or AUR channel once the installer is boring
- [ ] Retire the v2 web UI and server; swap the repo directories on the dev box
- [ ] Docs, screenshots, first release

## Later

- Background agents tab: dispatch, watch, results
- Phone access through Obsidian Sync of the vault
- A macOS build if anyone wants one; nothing in the core is Linux-only

## Principles

- **Omarchy is a dependency, not a target.** Rusty assumes Hyprland, uwsm, the `omarchy`
  CLI, its theme files and its launcher conventions, and uses them instead of reinventing
  them.
- **Files are the truth.** The brain is a folder of markdown Obsidian can open. SQLite holds
  the index, never the only copy of anything.
- **One back end process.** `rusty-mcp` serves the app over local HTTP and the agents over
  stdio. No web UI, no REST layer, no second protocol.
- **Native terminals.** Claude Code and Codex run in a real terminal emulator inside the app,
  attached to tmux sessions, so nothing is lost when the app closes.
- **Same gates as before.** rustfmt, clippy with warnings as errors, tests, docs, on every
  change.

## Non-goals

No browser UI, no voice, no cloud service, no multi-user. Rusty runs on one machine for one
person and talks to the agents that person already uses.
