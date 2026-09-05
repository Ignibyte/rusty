---
title: Native agent pane
pipeline_id: aac23d69-4814-4079-b26b-c299897d6741
status: Phase 5 — Complete PASS
ticket: TICKET-025
ticket_doc: docs/planning/tickets/open/TICKET-025-native-agent-pane.md
aar: docs/planning/knowledge/aar/AAR-025-native-agent-pane.md
sealed: no new tab, tool, table or dependency; one new Rust module and QML in the pane; the process is Claude Code's own, already on the box
created: 2026-09-05
---

# Native agent pane: spec

## Intent

The agent pane holds a conversation about the open note inside a whole terminal
emulator: a tmux session, ANSI, no selectable text, a tool call as nothing but text.
Claude Code's print mode over stream-json is long-lived and bidirectional; Rusty writes
messages in, reads structured events out and renders them itself, keeping Claude Code's
harness — skills, tools, MCP, the box's auth — and dropping only the terminal. The
terminal tabs stay real terminals; the pane becomes a conversation.

## Scope

- In: an `Assistant` QML type in the app (the process, a line parser, the messages out,
  signals in); the pane as a chat list (user, text, tool call, tool result, permission
  prompt, notice) with a multi-line input, Send/Stop and Allow/Deny; a session id per
  page in the workspace state, resumed with `--resume`; Rusty's server through
  `--mcp-config` over HTTP; read-only Rusty tools pre-allowed, everything else prompting;
  the page named in `--append-system-prompt`; a fake `claude` for the tests and the
  screenshot script; an `agent:ask:` scene.
- Out (named seams, not forgotten): replacing the terminal tabs; the Claude API or an API
  key; the Claude Agent SDK; Codex in the pane (no print mode; the pane says so); a
  replay of a resumed session's earlier messages (Claude Code keeps the transcript, the
  pane shows the continuation); markdown rendering of the assistant's text through
  `brain_render` (plain text now); a model picker.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the agent pane is opened for a page, it shall start one headless `claude -p` process with `--input-format stream-json --output-format stream-json` and render the conversation natively. | Rust test of `build_args`; the `agent:ask:` scene against the fake |
| REQ-002 | WHEN the assistant replies, the pane shall render text, tool calls and tool results as distinct, selectable elements, with partial messages streamed as they arrive. | Rust tests of `parse_line` on the probe's lines; the scene (bubbles for text, a tool call, its result) |
| REQ-003 | WHEN the app restarts, reopening the pane for the same page shall resume that page's session by id rather than start a fresh one. | Rust test (`--resume <id>` in the arguments when an id is given); reading of `ui.agentSessions`; smoke by Chad across a restart |
| REQ-004 | WHEN the agent asks for a permission, the pane shall render it as a prompt with buttons and return the answer to the process. | Rust tests of the `control_request` parse and the `control_response` shape (the probe's exact wire format); the scene shows the prompt |
| REQ-005 | WHEN the process exits or fails to start, the pane shall say so and offer to restart, and shall never leave a half-rendered conversation. | Rust test: a fake that exits 3 reports `Exit(3)` after its lines; a missing binary is an error at spawn; reading of `onExited` |
| REQ-006 | WHEN the pane's agent uses a tool, it shall reach Rusty's own MCP server, so the brain, tasks and notes are available to it. | Rust test of the `--mcp-config` argument; smoke by Chad ("read the open page") |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The pane's agent is Claude Code itself, headless: `claude -p --input-format stream-json --output-format stream-json --include-partial-messages --verbose`, one long-lived process per open page, spawned and read by the app's own `Assistant` type on std threads (a reader thread hands lines to the Qt thread through `qt_thread().queue`, as `Backend` does with tool answers) | The probe on this box (`claude` 2.1.260) showed the process is bidirectional and streams every event; no API key, no SDK, the box's auth and skills; std threads need no new crate or tokio feature | the Claude API (a key, a second harness); the Agent SDK (not Rust); tokio's process feature |
| 2 | Permissions go through `--permission-prompt-tool stdio` under `--permission-mode default`: the CLI writes `control_request` (`can_use_tool`) and the pane answers `control_response` (`allow` with the input as given, or `deny` with a message); Rusty's read-only tools are pre-allowed by name, everything else prompts | The probe showed `--permission-prompts host` denies unless a host announces itself and `--permission-prompt-tool stdio` is the wire the SDK uses; a read of the store is what the app itself does freely, a write deserves a click | `bypassPermissions`; prompting for every read |
| 3 | A session id per page lives under `agentSessions` in the workspace state (`{slug: id}`), taken from the `init` event and passed back as `--resume`; "New" forgets it | The window already keeps per-page JSON keys this way; the id is Claude Code's own, so no uuid crate | a table in the store; one session for all pages |
| 4 | Rusty reaches the pane's agent as an HTTP MCP server (`--mcp-config` with the app's own `Backend.url`, `--strict-mcp-config`) | One store, one running server, no second `rusty-mcp` process; the app already knows the URL | a stdio `rusty-mcp` per pane |
| 5 | The working directory is `HOME` and the system prompt names the page and Rusty's tools; the vault is reached through the tools, not the agent's file tools | The index, the links and the git auto-commit stay consistent (a file tool on the vault bypasses the commit); the constitution's "the app touches nothing under `~/.rusty`" | cwd in the vault |
| 6 | Rendering is the pane's: a `ListModel` of items (user, text, tool, result, permission, notice) built from typed signals; text in a read-only `TextEdit` that selects; the assistant's text plain for now | Selectable, themable, a tool call as its own item — the ticket's point; a later ticket may render the final text through `brain_render` | rendering inside Rust; markdown now |
| 7 | The tabs stay tmux terminals: `AD-rusty-agents-are-terminals-001` is qualified, not reversed — "there is no in-process chat with a model" becomes "the pane is a headless Claude Code, the tabs are terminals" | Chad's own words: a tab is the real agent with a real terminal, the pane is a conversation about the note | a chat everywhere |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-025-native-agent-pane.md`
- Register: `AD-rusty-agents-are-terminals-001` (qualified), `AD-rusty-mcp-only-back-end-001`,
  `PR-rusty-lazy-pane-terminals-001` (the pane no longer holds a terminal)
- Architecture: `docs/architecture.md` (the app's objects), `openwiki/workspace-app.md`
- The probes: `scratchpad/claude-probe.ndjson` (one turn), `perm-probe-tool.ndjson` (a
  permission asked and answered), kept for this session only

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled; no seal needed |
| 2 Design | Manifest, the wire, the parser, the pane, the state, regression table | design actionable |
| 3 Implement | `assistant.rs`, `RightPane.qml`, `Main.qml`, `build.rs`, `main.rs`, `scripts/screenshot.sh` | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger; CodeGraph over the new module | confirmed findings resolved |
| 4 Validate | The tests, the scene against the fake, `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, architecture, AAR, register, brain, archive | pair archived |
