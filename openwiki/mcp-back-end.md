---
type: "Reference"
title: "MCP back end: one server for the app and the agents"
openwiki_generated: true
sources:
  - id: openwiki-source-c7501cab00d475ec77094adb
    resource: repo://crates/rusty-core/src/brain/mod.rs
  - id: openwiki-source-087a3c8d2ec2da0b0f978302
    resource: repo://crates/rusty-mcp/src/main.rs
  - id: openwiki-source-84acb13abf83511312610cd3
    resource: repo://crates/rusty-mcp/tests/smoke.rs
generated: {by: "claude-code", at: "2026-09-03T05:07:54.592Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-09-03T05:22:20.038Z
---

# MCP back end: one server for the app and the agents

## Purpose

`rusty-mcp` is the only way into the store. Claude Code and Codex spawn it over stdio
(`.mcp.json`, `.codex/config.toml`); the app talks to one long-running instance over
Streamable HTTP at `127.0.0.1:4174/mcp`, kept up by the `rusty-mcp.service` user unit.
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
- Notes: list, read, write, create, rename, delete (the notes folder, not the vault).
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
  page's neighbourhood). `brain_search` understands `tag:<name>` terms.
- Skills: list, view, create, update, scan, approve, reject, delete.
- Secrets: list names, set, delete; a value is never returned.
- Settings: get, set, list with credential-looking values masked.
- Obsidian bridge: status, open, backlinks, links, unresolved, rename through
  Obsidian's CLI when the app is running; scheduled for retirement once the workspace
  has replaced it in daily use.

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

## Invariants

- No tool reaches the database directly; managers do.
- A renamed or removed tool is a versioned break; new tools are additive.
- Long work runs in `spawn_blocking` so the server keeps answering.

## Failure modes

- Without Obsidian the six bridge tools answer with a clear error and nothing else
  changes.
- Without an embedding provider `brain_reembed` errors and search stays full text.
- A lost HTTP session makes the app reconnect every three seconds.

## Extension points

- A new tool: a parameter struct with doc comments, a `#[tool]` method that calls a
  manager, `mutate()` for writes, its name in `EXPECTED`, a line in the README.
- A new resource: a `ResourceUri` variant, `parse_resource_uri`, `resource_text`.

## Tests

- `cargo test -p rusty-mcp`: the router tests and the smoke test (list tools, a task
  group, the Obsidian status without Obsidian, resources, a new page, a whole-file write,
  a render with a style, a rename with its link rewrite, the tree, no unresolved links).

## Primary sources

- `crates/rusty-mcp/src/main.rs`, `crates/rusty-mcp/tests/smoke.rs`
- `omarchy/rusty-mcp.service`, `omarchy/mcp-config.json`
