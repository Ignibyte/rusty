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
| M1 The back end, complete | done 2026-09-02 |
| M2 App shell, Terminals, Tasks | done 2026-09-02 |
| M3 Knowledge tabs: Brain, Notes, Memory | done 2026-09-02 |
| M4 Semantic search | done 2026-09-02 |
| M5 Skills, Secrets, Settings | done 2026-09-02 |
| M6 Omarchy packaging and cutover | next |

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

## M1. The back end, complete (done 2026-09-02)

Done when every v2 GUI action has an MCP equivalent, the dev box's own `.mcp.json`
points at v3, and the v2 server is not needed.

Tools and resources

- [x] Tasks: rename and delete a list, update a task title, unarchive, delete
- [x] Tasks: reorder (`reorder_tasks`)
- [x] Notes: create, rename, delete
- [x] Notes: daily note open-or-create (`brain_daily_note`, the daily page in the vault)
- [x] Memories: delete
- [x] Memories: update (`update_memory`)
- [x] Brain: read timeline, links and backlinks, set page body, delete page, resolve a
      partial slug
- [x] Brain: capture into the daily page or the inbox, page types listed (`brain_capture`,
      `brain_page_types`)
- [x] Skills: create (active or staged), approve, reject, delete, safety scan on demand
- [x] Skills: update description and body in place, other keys kept (`skill_update`)
- [x] Secrets: list names, set, delete; values are never returned
- [x] Settings: get, set
- [x] Settings: list, credential-looking values masked (`settings_list`)
- [x] Resources: `rusty://tasks`, `memories`, `skills`, `notes`, `brain` and the templates
      `tasks/{group_id}`, `brain/{slug}`, `notes/{path}`; `resources/list_changed` fires on
      every tool mutation and every watched-file change (verified over HTTP)
- [x] Router tests: every tool advertised once, every tool described
- [x] An rmcp-client smoke test that runs in CI (`crates/rusty-mcp/tests/smoke.rs`,
      `.github/workflows/ci.yml`)

Transport and services

- [x] A `systemd --user` unit for `rusty-mcp --http` (`omarchy/rusty-mcp.service`; running on
      the dev box)
- [x] The installer enables it (`omarchy/install.sh`: binaries, unit, Obsidian registration)
- [x] MCP config snippets for Claude Code and Codex (`omarchy/mcp-config.json`)
- [x] `rusty-cli` ported onto `rusty-core`, installed over the v2 binary

Obsidian and the vault

- [x] Obsidian CLI bridge: `open_in_obsidian`, link-safe `rename_page`, `backlinks`;
      skip cleanly when the app is not running
- [x] Vault rule: the timeline is a `## Timeline` section; the bare `---` rule is only read,
      never written. `rusty-cli brain migrate` converted the dev box's 22 legacy pages on
      2026-09-02
- [x] Vault rule: one wikilink path style that Obsidian and Rusty both write: `[[folder/slug]]`;
      the migration rewrote 161 bare-name links, leaving 14 it could not resolve as written
      (Obsidian's side is set on 2026-09-02: vault-path wikilinks, `newLinkFormat: absolute`,
      `useMarkdownLinks: false`; Rusty's writers already use `[[folder/slug]]`)
- [x] Register `~/.rusty/brain` as an Obsidian vault; decide the `.obsidian/` policy
      (2026-09-02: `rusty-cli obsidian register` writes the vault and the CLI toggle into
      Obsidian's config; `.obsidian/` is gitignored in the vault, so it stays per machine)

Cutover for the back end

- [x] Parity check against v2's tool list: all 29 present
- [x] Switch the dev box's `.mcp.json` to v3 and retire the v2 `rusty-mcp`

## M2. App shell, Terminals, Tasks (done 2026-09-02)

Done when the app replaces a terminal for daily Claude and Codex use.

- [x] `rusty-app` crate: cxx-qt 0.10 on Qt 6.11, one window, side rail, tab stack; the
      prototype's QML ported, `Theme` served from Rust (binary `rusty`, app_id
      `com.ignibyte.rusty`)
- [x] MCP client over Streamable HTTP: the `Backend` type keeps one session on a tokio
      runtime, reconnects every 3s, forwards tool calls and answers as JSON through a
      `result` signal, and turns `resources/list_changed` into `dataChanged` (the tabs parse
      the JSON in QML; typed Rust models can come when a tab needs one)
- [x] Terminals: qmltermwidget per tab, one tmux session per tab, new tab and rename, close
      (keeping or ending the session), pick the agent (Claude, Codex, shell), attach an
      existing session; tabs persist in `~/.config/rusty/tabs.json`; Ctrl+Shift+T/W, F2
- [x] Terminal colours from the Omarchy theme: the scheme is generated into
      `~/.config/rusty/color-schemes` and the widget is pointed there with `COLORSCHEMES_DIR`,
      no root-owned file; font from the Alacritty config
- [x] Live re-theme: the app watches `~/.config/omarchy/current` and reloads when
      `omarchy theme set` repoints the theme link (no hook script to install)
- [x] Tasks: lists (add, rename, delete with confirm), quick add, toggle, archive and
      restore, inline rename (F2), reorder (Ctrl+Up/Down), delete; keyboard first (Enter,
      arrows, Space, Delete, Escape); live refresh on `dataChanged`; verified against the
      running service on the box
- [x] Settings page: theme, font, scheme, tabs file and back end shown; every setting
      `rusty-mcp` stores is listed and editable in place (Enter saves), new keys can be added
- [x] Desktop entry, icon, stable `app_id` (`com.ignibyte.rusty`), `omarchy-launch-or-focus
      rusty`, and a key: `omarchy/hyprland-bindings.conf` binds SUPER+ALT+R (the installer
      points at it rather than editing your bindings)
- [x] Delete the prototype (2026-09-02, superseded by `crates/rusty-app`)

## M3. Knowledge tabs (done 2026-09-02)

- [x] Brain: type tree with counts, full-text search, rendered markdown with wikilinks that
      navigate, edit and save, timeline with append, backlinks and outbound links, open in
      Obsidian, capture to the daily page or the inbox
- [x] Notes: daily pages in the vault (so Obsidian sees them); Today opens or creates, older
      days listed newest first, same page view as the Brain tab
- [x] Memory: list with category and importance, add with Enter, edit and delete in a side
      panel, filter by category
- [x] Obsidian themed by `omarchy theme set`: the theme's `obsidian.css` becomes the vault's
      `omarchy` CSS snippet (`rusty-cli obsidian configure`, run again by the app on every
      theme change); a theme without the file removes the snippet

## M4. Semantic search (done 2026-09-02)

- [x] `sqlite-vec` inside `rusty.db`: an embedding per page chunk (`brain_chunks` + the `vec0`
      table `brain_vec`, created at the model's width); notes and memories can follow the same
      path when a tab needs them
- [x] Embedding provider trait; Ollama when it is running (default), OpenAI only when the
      setting says so and the vault has the key; no provider means no vectors and everything
      else still works (verified with unit tests and a mock Ollama; the dev box has no provider
      yet)
- [x] Hybrid retrieval, full text and vectors merged by reciprocal rank fusion, behind
      `brain_search`
- [x] Incremental indexing: the server embeds stale pages after each burst of changes and
      every ten minutes; `brain_reembed` and `rusty-cli brain embed [--all]` on demand
- [x] Settings entries: `embedding_provider`, `embedding_model`, `ollama_url`; the key lives in
      the secrets vault as `openai_api_key`

## M5. Skills, Secrets, Settings (done 2026-09-02)

- [x] Skills: list with status and origin, description and body edited in place (Ctrl+S saves),
      new skill (active or staged), safety scan with findings shown, approve (or approve
      anyway), reject, delete
- [x] Secrets: names listed, set or replace (typed into a password field), delete with confirm;
      no value is ever rendered or returned
- [x] Settings: every key the back end reads, with what it means and its default, editable in
      place; other stored keys below; the machine's theme, font, scheme, tabs file and back end

## M6. Omarchy packaging and cutover

- [x] `omarchy/install.sh`: dependencies through `omarchy pkg add`, the binaries, the desktop
      entry and icon, the keybinding snippet (pointed at, not applied), the MCP config snippet,
      the user service, the Obsidian registration; no theme hook is needed, the app watches
      the theme link itself
- [ ] A package for the Omarchy or AUR channel once the installer is boring
- [ ] Retire the v2 web UI and server; swap the repo directories on the dev box (Chad's call:
      the private repo keeps the ops handbook and machine commands either way)
- [ ] Docs, screenshots, first release (docs are current; screenshots need a scratch vault so
      no real data ships; the release tag is Chad's to cut)

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
