---
type: "Reference"
title: "MCP back end: one server for the app and the agents"
openwiki_generated: true
sources:
  - id: openwiki-source-1de5221fd140fd89f39f87cd
    resource: repo://crates/rusty-cli/src/main.rs
  - id: openwiki-source-d2476bcfdf1c1072b66eb52b
    resource: repo://crates/rusty-core/src/brain/decisions.rs
  - id: openwiki-source-c7501cab00d475ec77094adb
    resource: repo://crates/rusty-core/src/brain/mod.rs
  - id: openwiki-source-705d180fc941297b1e844397
    resource: repo://crates/rusty-core/src/core.rs
  - id: openwiki-source-8f342262c76136dc27154aaf
    resource: repo://crates/rusty-core/src/engine/db.rs
  - id: openwiki-source-bb352c1ae3d0e8267aac9d76
    resource: repo://crates/rusty-core/src/engine/pin_lock.rs
  - id: openwiki-source-5097c4ef41727eee45d8c689
    resource: repo://crates/rusty-core/src/lib.rs
  - id: openwiki-source-2bac0135ef08343388f2c7a1
    resource: repo://crates/rusty-core/src/notes/mod.rs
  - id: openwiki-source-38142a1a317c38546fd7b1f4
    resource: repo://crates/rusty-core/src/skills/scripts.rs
  - id: openwiki-source-087a3c8d2ec2da0b0f978302
    resource: repo://crates/rusty-mcp/src/main.rs
  - id: openwiki-source-84acb13abf83511312610cd3
    resource: repo://crates/rusty-mcp/tests/smoke.rs
  - id: openwiki-source-f47a49d22d041953f356ca04
    resource: repo://omarchy/rusty-mcp.service
generated: {by: "claude-code", at: "2026-09-05T04:56:03.642Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-09-05T04:56:03.642Z
---

# MCP back end: one server for the app and the agents

## Purpose

`rusty-mcp` is the only way into the store. Claude Code and Codex spawn it over stdio
(`.mcp.json`, `.codex/config.toml`); the app talks to one long-running instance over
Streamable HTTP at `127.0.0.1:4174/mcp`, kept up by the `rusty-mcp.service` user unit,
which is wanted by `default.target`, restarts two seconds after any exit but a stop, and
sits at OOM score 100 (see [Development and validation](development-and-validation.md),
running as services).
Every tool is a thin wrapper around a manager call in `rusty-core`; the managers own the
rules.

## Ownership

`crates/rusty-mcp/src/main.rs`: the `Rusty` server (an rmcp `ToolRouter` over a shared
`Core`), the parameter types, `mutate()` (announce a change on success), resources under
`rusty://`, the indexer loop, the change notifier, and `main` with the two transports.
`crates/rusty-mcp/tests/smoke.rs` drives the built binary over stdio in a scratch `HOME`
the way an agent would.

## Tool families

- Tasks: lists, create, toggle, archive, unarchive, retitle, reorder, delete.
- Notes: list, read, write, create, rename, delete, over the notes folder: the vault's
  `notes/` since TICKET-014 (an explicit `notes_path` setting still wins), so a note is
  also a page of type `note`; `rusty-cli notes adopt` moves an older folder in once.
- Memories: list, store, update, delete; `search_conversations`.
- Brain: `brain_search` (hybrid when an embedder exists), read, list, create, update,
  delete, timeline read and append, links, resolve a slug, stats, daily note, capture,
  page types, semantic status, reembed.
- Workspace: `brain_tree`, `brain_render` (rich text plus outline, links, unresolved
  targets, counts, properties, raw), `brain_write_page` (the whole file),
  `brain_new_page`, `brain_new_folder`, `brain_delete_folder`, `brain_rename`,
  `brain_unresolved`, `brain_tags` (every tag with its count), `brain_set_property` and
  `brain_remove_property` (one frontmatter key, typed, the body untouched), and
  `brain_graph` (page nodes with title, type, folder and tags, edges from resolved
  links; tags and unresolved targets as nodes on request; `around` and `depth` for one
  page's neighbourhood). `brain_search` takes the operators of `parse_query`
  (`tag:`, `path:`, `file:`, `type:`, `-` excluding) in the query and the two text modes
  as `case_sensitive` and `regex`.
- Skills: list, view, create, update, scan, approve, reject, delete.
- Scripts (TICKET-010): `script_list`, `script_view`, `script_update` and `script_run`.
  A script is not an object of its own — it is a `*.sh` file *inside* a skill directory,
  so it inherits that skill's approval state, and `resolve_script` finds it by basename
  without the extension, taking `skill/name` when two skills share a basename. That one
  decision carries the safety story: `script_run` is the only tool here that executes
  anything, and it refuses a script whose skill is still pending, so approving the skill
  is the only route to running it.
- Secrets: list names, set, delete; and, since TICKET-015, the PIN behind the Secrets
  tab, which the server owns. `PinLock` (`rusty-core::engine::pin_lock`) keeps an
  argon2id hash at `~/.rusty/.pin` (mode 0600) and one in-memory token. `secret_pin_status`
  reports set, unlocked and any lockout; `secret_pin_set` sets the PIN (six characters or
  more) and needs the live token once one exists; `secret_unlock` verifies the PIN,
  counts five wrong tries in a row into a one-minute lockout, and returns a token good
  for `pin_timeout_minutes` (default five); `secret_reveal` and `secret_update` require
  that token; `secret_lock`, a new unlock and a server restart end it. `secret_list`
  stays name-only, so no tool returns a value without a live unlock. The PIN protects
  the screen: the secrets file's format and permissions do not change, because the back
  end reads it headless for the embeddings key.
- Settings: get, set, list with credential-looking values masked.
- The brain loop (TICKET-018): `brain_ask` runs the hybrid search (text alone without
  a provider), lists the decisions touching the question with their status and the
  follow-ups due, and records a consultation (`brain_consultations`: id, question, hits,
  outcome) whose id the next step needs. `brain_decide` writes a `decision` page under
  `decisions/` (question, choice, rationale, alternatives, a wikilink per consulted page,
  `status: decided`, `decided`, `follow_up_by`, `consulted`, `supersedes`), adds a
  timeline entry to every consulted page, marks a superseded decision, and sets the
  consultation's outcome. `brain_follow_up` appends a dated outcome and sets kept, revised
  or superseded (with the successor); `brain_no_decision` records the reason nothing was
  decided; `brain_due` lists the follow-ups due within `days` and every decision.
  `brain_graph` edges carry a `kind`: `link`, or a decision's `consulted`, `supersedes`
  and `follows_up`, read from its frontmatter. The record is `docs/architecture/brain-loop.md`.
- Text that is not a page: `brain_render` given `markdown` renders that text with the
  page renderer (links resolve against the vault as in a page; the slug may be empty),
  which is how the app shows a markdown file from a folder root.

The Obsidian bridge (six `obsidian_*` tools over Obsidian's CLI) was retired on
2026-09-03; `brain_get_links`, `brain_unresolved` and `brain_rename` cover what it did,
and the app opens pages.

`EXPECTED` in the router test lists every name; a tool missing from it or from the
router fails the test, and every tool must carry a description.

## Runtime flow

- A write goes through `mutate()`, which emits `AppEvent::DataChanged` on success. The
  change notifier forwards every such event to every connected client as a
  `resources/list_changed` notification, dropping peers that went away; the app turns
  it into `dataChanged` and refreshes.
- `start_data_watcher` watches the notes, brain and skills folders (and a sentinel
  `~/.rusty/.changed` that `rusty-cli refresh` touches) and emits `DataChanged` after a
  burst settles, `.git` excluded.
- The indexer loop runs at start and after every burst: `brain_manager.sync_all()`
  (files changed by other programs reach the index), then `index_stale` with the
  configured embedder when there is one; failures are logged to stderr, never fatal.
- Resources: `rusty://tasks`, `memories`, `skills`, `notes`, `brain` and the templates
  `tasks/{group_id}`, `brain/{slug}`, `notes/{path}`, all JSON except a note's markdown.
- Diagnostics go to stderr only; stdout is the protocol.
- The Obsidian import (TICKET-026) is two tools over `BrainManager`: `brain_import_plan`
  answers what an import of a vault path would do and writes nothing;
  `brain_import` does it through `mutate`, so every client sees `DataChanged` once the
  pages, attachments and the report page are in and the index is rebuilt. Neither writes
  the source vault. `rusty-cli brain import <vault> [--dry-run]` calls the same two
  methods and prints the plan and the report. The surface is 82 tools.

## Invariants

- No tool reaches the database directly; managers do.
- A renamed or removed tool is a versioned break; new tools are additive.
- Long work runs in `spawn_blocking` so the server keeps answering.
- A secret's value leaves the server only against the live PIN token, and no call
  logs a PIN, a token or a value.
- A script runs only from an approved skill. Both execution paths check the status, so
  neither the tool nor the command line can run a script waiting in staging.

## Failure modes

- Without an embedding provider `brain_reembed` errors and search stays full text.
- A lost HTTP session makes the app reconnect every three seconds.
- Five wrong PINs in a row refuse every unlock for a minute, the right PIN included;
  the lockout and the token live in memory, so a restart clears both.
- A script that does not finish inside the caller's cap is killed and reported as status
  124 with a `timed_out` flag rather than left running; each stream is truncated at
  64 KiB, so a script that prints forever cannot exhaust the server. The `rusty <name>`
  path has neither cap: `exec_script` replaces the process, so the script becomes the
  command and lives as long as the terminal lets it.

## Extension points

- A new tool: a parameter struct with doc comments, a `#[tool]` method that calls a
  manager, `mutate()` for writes, its name in `EXPECTED`, a line in the README.
- A new resource: a `ResourceUri` variant, `parse_resource_uri`, `resource_text`.

## Tests

- `cargo test -p rusty-mcp`: the router tests and the smoke test (list tools, a task
  group, resources, a new page, a whole-file write,
  a render with a style, a rename with its link rewrite, the tree, no unresolved links).
- `cargo test -p rusty-core pin_lock`: set, unlock, check and lock; the short and the
  wrong PIN; the lockout; the expiry; a PIN change needing the token; the file mode.
- `cargo test -p rusty-core decisions`: the consultation record, the decision page with
  its links and timeline entries, the follow-up's status and date, the due list, the
  typed edges; the smoke test walks the loop over stdio.
- The router test's `EXPECTED` list names `brain_import_plan` and `brain_import`; the
  import itself is tested in `rusty-core` (`vault-and-brain.md`).

## Primary sources

- `crates/rusty-mcp/src/main.rs`, `crates/rusty-mcp/tests/smoke.rs`
- `omarchy/rusty-mcp.service`, `omarchy/mcp-config.json`
