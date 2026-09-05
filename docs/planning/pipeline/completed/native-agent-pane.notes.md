---
title: Native agent pane — notes
pipeline: aac23d69-4814-4079-b26b-c299897d6741
ticket: TICKET-025
---

# Native agent pane: notes

## Recall (2026-09-05)

- Bulletins: bulletin 2 (no synthetic input on Chad's desktop) — the scenes run
  offscreen against a fake `claude`; the real one was probed from the terminal with two
  one-turn prompts (haiku), which is not the app.
- Register: `AD-rusty-agents-are-terminals-001` ("no in-process chat with a model") is
  the decision this ticket qualifies; `AD-rusty-mcp-only-back-end-001` holds — the pane's
  agent reaches the store through `rusty-mcp` like any agent; `PR-rusty-lazy-pane-terminals-001`
  loses its subject in the pane (the tabs keep it).
- Wiki: `workspace-app.md` describes the pane as "one terminal that stays with the
  sidebar" and the right-click on a top-bar agent opening it; `AgentTerminal` stays for
  the tabs.
- Code read:
  - `backend.rs`: the pattern — a `#[cxx_qt::bridge]` with properties, invokables and
    signals, `impl cxx_qt::Threading`, work on another thread, results back through
    `qt_thread().queue(|mut obj| …)`. Copied for the process.
  - `RightPane.qml`: the agent section is a context card, a program `ComboBox` and an
    `AgentTerminal` on a `rusty-pane-<program>` tmux session; `pane.note` is the open
    note (`slug`, `title`, `words`, `backlinkCount`); `focusAgent` focuses the terminal.
  - `Main.qml`: `ui` keeps per-page JSON keys (`expanded`, `graph`, `bookmarks`,
    `skillsLayout`) with `load`/`write`; `paneProgram` fed the combo; the top bar's
    `agentPaneRequested(p)` set the program and showed the pane; `Backend` is declared
    once and passed down; `backend.url` is the server's address.
  - `terminals.rs`: `AGENT_CANDIDATES` and the tmux command; untouched.
  - `rusty-core`'s `engine/agent_manager.rs` and `output_parser.rs` are the v2 one-shot
    dispatch (`--output-format json`); a precedent for spawning `claude`, not reused.
- The wire, from the probes on this box (`claude` 2.1.260):
  - in: `{"type":"user","message":{"role":"user","content":[{"type":"text","text":…}]}}`
    per line; a `control_response` as `{"type":"control_response","response":{"subtype":"success","request_id":…,"response":{"behavior":"allow","updatedInput":…}}}`
    (or `{"behavior":"deny","message":…}`); a `control_request` with `{"subtype":"interrupt"}`
    stops a turn.
  - out: `system init` (`session_id`, `tools`, `mcp_servers`); `stream_event` with
    `content_block_start` (`text`, `tool_use` with `id` and `name`, `thinking`),
    `content_block_delta` (`text_delta`), `content_block_stop`, `message_*`;
    `assistant` (the whole message, tool_use blocks with their input); `user` (a
    `tool_result` with `tool_use_id`, `content` as a string or text blocks, `is_error`);
    `control_request` (`can_use_tool`: `request_id`, `tool_name`, `input`,
    `description`, `tool_use_id`); `result` (`subtype`, `is_error`, `total_cost_usd`,
    `num_turns`, `result`); `system permission_denied` when nobody answers;
    `rate_limit_event` and `system status|thinking_tokens` to ignore.
  - `--permission-prompts host` without a host announcement denies; `--permission-prompt-tool stdio`
    prompts on stdout. `--allowedTools Bash` pre-allows; `echo` never asks anyway.

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-app/src/assistant.rs` | new: the bridge (`available`, `running`, `busy`, `status`, `sessionId`; `start`, `send`, `answer`, `interrupt`, `stop`; the signals), `Event` and `parse_line`, `user_message`, `control_response`, `interrupt_request`, `build_args`, `mcp_config`, `READ_TOOLS`, `claude_binary`, `spawn` (std threads: a reader on stdout, a collector on stderr, `Output::Line`/`Output::Exit`), tests |
| `crates/rusty-app/build.rs`, `src/main.rs` | the module |
| `crates/rusty-app/qml/RightPane.qml` | the agent section: header (page, status, New), the chat `ListView` over a `ListModel`, the empty and not-installed states, the input row; `openAgent`, `newConversation`, `sendMessage`, `askAgent`, `answerPermission`, the `Connections` to the assistant; the combo and the terminal go |
| `crates/rusty-app/qml/Main.qml` | `Assistant {}`; `ui.agentSessions` (load, save, write); the pane's `assistant`, `sessions`, `onSessionStarted`, `onForgetSession`; the program wiring goes; `agent:ask:` scene |
| `scripts/screenshot.sh` | a fake `claude` under the scratch (one canned turn: a tool call, its result, streamed text, a permission prompt, a result) and `RUSTY_CLAUDE_BIN` in the scene environment |

### The process

`claude_binary()`: `RUSTY_CLAUDE_BIN` when set, else the first `claude` on `PATH`, else
`bash -lc 'command -v claude'` once (the app runs as a user service whose `PATH` may
not carry mise's shims). `spawn(binary, args, cwd, on_output)`: stdin piped, stdout
piped and read line by line on a named thread, stderr collected to a bounded buffer on a
second; at stdout's end the child is waited and `Output::Exit(code, stderr tail)` is
handed over. The child sits behind `Arc<Mutex>` so `stop` (kill and wait) and the
reader's wait cannot race. A generation counter on the Qt side drops lines from a
process that was replaced.

### The parser

`parse_line(&str) -> Vec<Event>`: `Init`, `BlockStart {kind, name, id}`, `TextDelta`,
`TextFinal` (the text of an `assistant` message, so the bubble ends with the
authoritative text even if a delta was missed), `ToolInput {id, name, input}`,
`ToolResult {id, text, is_error}`, `Permission {request_id, tool, input, description}`,
`TurnDone {ok, cost_usd, num_turns, text}`, `Notice`. Everything else is nothing.

### The pane

Items `{kind, name, text, extra, input, answered}`: `user`, `text`, `tool` (`extra` the
tool_use id; `text` the input once the `assistant` message names it), `result`,
`permission` (`extra` the request id, `input` the JSON to hand back), `notice`.
`openAgent` runs when the pane shows or the note changes: the same page keeps its
process; another page stops the old one, clears the list, and starts with the page's
session id when there is one. Enter sends (Shift+Enter a newline); Send becomes Stop
while a turn runs and sends an interrupt; Allow and Deny answer once and show what was
chosen; an exit appends a notice that says how to start again (any message restarts).

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | `build_args_carry_the_wire` (both formats, partial messages, the prompt tool); the scene |
| REQ-002 | `parse_line_reads_the_probe` (init, block start, deltas, the assistant's tool input, a tool result, a result); the scene's bubbles |
| REQ-003 | `build_args_carry_the_wire` (`--resume <id>` present with an id, absent without); reading of `agentSessions`; Chad's restart |
| REQ-004 | `parse_line_reads_the_probe` (the `control_request`); `control_response_matches_the_probe`; the scene's prompt |
| REQ-005 | `spawn_reports_lines_then_the_exit` (a fake that answers and exits 3); `spawn_refuses_a_missing_binary`; reading of `onExited` |
| REQ-006 | `build_args_carry_the_wire` (the `--mcp-config` JSON names `rusty` at the URL, `--strict-mcp-config`); Chad's "read the open page" |

### Risks

- **Data safety.** The agent can write through Rusty's tools and its own after a click;
  reads need none. Tests spawn only a fake script under a temporary directory; the
  scenes use the fake under the scratch `HOME`.
- **Secrets.** Nothing new leaves the machine beyond what Claude Code sends as it does in
  a terminal; `secret_reveal` is not pre-allowed.
- **A hung process.** Stop kills it; an exit is always reported (the reader thread ends
  with the wait).
- **Keyboard.** Enter sends, Shift+Enter breaks a line, the pane's tab and the palette
  focus the input; Allow and Deny are buttons reachable by Tab.
- **Theme.** `active`, `hover`, `panel3`, `line`, `accent`, `muted`, `faint`, `foreground`,
  `termFont`.
- **Old pane sessions.** `rusty-pane-<program>` tmux sessions from before stay until they
  end on their own; nothing here kills them. Noted in the wiki.

### CodeGraph

`codegraph_explore` over `Backend`, `qt_thread`, `AgentTerminal`: `Backend` is the one
threaded bridge and its `call` the pattern; `AgentTerminal.qml` is used by the tab host
and the pane, and keeps the tab host after this. A second pass after implementation goes
in the ledger.

## Phase 3: Implement

Six files, as the manifest said. `assistant.rs` (new): the bridge with five properties,
five invokables and ten signals; `READ_TOOLS`; `Event` and `Output`; `parse_line`,
`user_message`, `control_response`, `interrupt_request`, `mcp_config`, `build_args`,
`claude_binary`; `Process` (`write_line`, `kill`) and `spawn` (a reader thread, a stderr
collector bounded to two kilobytes, the exit after the wait); `handle` and `emit` on the
Qt thread; six tests. `build.rs` and `main.rs` list the module. `RightPane.qml`: the
agent section as a conversation; `Main.qml`: the `Assistant`, `ui.agentSessions`, the
pane's wiring, the `agent:ask:` scene; `scripts/screenshot.sh`: the fake `claude` and
`RUSTY_CLAUDE_BIN` in every scene's environment. `qmllint` exit 0, `bash -n` on the
script, `cargo fmt --all` ran.

Deviations: two clippy rounds — `QString::new()` does not exist (`default()` does), and
the delta arm's inner `if` became a match guard. The pane forgets a session id when the
process exits before announcing itself (F6 below), which the design did not say.

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | correctness | the permission item's label clipped at the pane's edge (seen in the first scene) | **confirmed**; the label fills the row and elides in the middle |
| F2 | correctness | the list followed new items (`onCountChanged`) but not text growing inside the last one, so a streamed answer scrolled out of view | **confirmed**; `onContentHeightChanged` follows too |
| F3 | data safety | Allow hands the input back unchanged as `updatedInput`, the wire the probe showed | rejected: no edit of the input is offered, so nothing can drift |
| F4 | correctness | switching pages while a turn runs kills the process mid-turn | accepted, by design: one process per page, the list is cleared, and `--resume` brings the conversation back |
| F5 | correctness | `sendMessage` writes before `init` arrives when it has just started the process | rejected: stdin is open from the spawn and the CLI queues the message; both probes sent before `init` |
| F6 | data safety | a stale session id (the transcript gone) fails the start, and the next message would start the same way forever | **confirmed**; an exit before `init` with an id to resume forgets it, and the notice says the next message starts anew |
| F7 | keyboard first | Enter sends, Shift+Enter breaks, Tab reaches Send, Allow and Deny; the pane's tab and the palette focus the input | no finding |
| F8 | theme | `active`, `hover`, `panel3`, `line`, `accent`, `muted`, `faint`, `foreground`, `termFont`, the application face for prose | tokens only |
| F9 | secrets | `READ_TOOLS` carries no `secret_*` tool; the system prompt names the slug and the title alone | no finding |
| F10 | performance | a delta appends to the item's text through `setProperty` (a copy each time) | accepted: a reply is kilobytes |
| F11 | correctness | the generation counter: `stop` bumps it, `start` bumps it again after `stop`, so a replaced process's late lines are dropped on the Qt thread | rejected (read) |
| F12 | correctness | `kill` and the reader both lock the child | rejected: the reader locks only after stdout closes, which the kill causes; no wait is held across the other |
| F13 | performance | `claude_binary` may spawn `bash -lc` once at construction when `PATH` lacks `claude` | accepted: once, at startup, for a service whose `PATH` is not a shell's; `RUSTY_CLAUDE_BIN` skips it |
| F14 | complexity | `ui.paneProgram` stays in the state, written and never read | accepted: an older state file still loads; a later sweep can drop it |
| F15 | correctness | `rusty-pane-<program>` tmux sessions from before this ticket linger | accepted: they end on their own; in the wiki |
| F16 | prose | the pane's strings, the notice wording, the system prompt | read against `no-ai-slop` |
| F17 | correctness | CodeGraph: the module's callers are its bridge and its tests; `Backend` stays the only other threaded bridge; nothing outside `assistant.rs`, `RightPane.qml` and `Main.qml` moved | the blast radius matches the manifest |

## Phase 4: Validate

- `bin/gate.sh --fast` after implement: `GATE RED [fast] at clippy` twice (the
  constructor, the collapsible match), then `GATE GREEN [fast]` with the six
  `assistant::tests` passing: `parse_line_reads_the_probe`, `messages_out_match_the_probe`,
  `build_args_carry_the_wire`, `spawn_reports_lines_then_the_exit` (a fake that answers
  and exits 3: `Init`, `TurnDone`, `Exit(3, "oops")`, then stdin refused),
  `spawn_refuses_a_missing_binary`, `kill_ends_a_waiting_process`.
- `cargo build -p rusty-app -p rusty-mcp` (23:40:50), then `scripts/screenshot.sh
  <scratch> "right:agent,agent:ask:What is this page about?" "right:agent"` with
  `RUSTY_SHOT_DELAY=5000`, offscreen against the scratch vault and the fake `claude`;
  after F1, F2 and F6 the app was rebuilt and both scenes shot again. Logs clean of
  `error|warning|TypeError|ReferenceError|Cannot assign|is not a type|Detected
  anchors|binding loop` both times.
  - `right-agent-agent-ask-….png`: the header (Orbit, `ready`, New), the user bubble,
    `⚙ mcp__rusty__brain_read_page` with `{"slug":"projects/orbit"}`, `↳ result` with the
    page's text in the terminal face, the streamed answer, the permission item with
    Allow and Deny, the input row — REQ-001, REQ-002 and REQ-004's scene.
  - `right-agent.png` (the second launch, the scratch state carried over): "Continuing
    this page's conversation." — the pane found the session id the first scene stored
    and passed it as `--resume` — REQ-003's shape.
- REQ-005 rests on `spawn_reports_lines_then_the_exit`, `spawn_refuses_a_missing_binary`
  and the reading of `onExited` (F6); REQ-006 on `build_args_carry_the_wire`'s
  `--mcp-config` assertion and Chad's smoke.
- `bin/gate.sh --diff` after the last gated edit (the F1/F2/F6 fixes): fmt, clippy,
  test, doc, shell-syntax, secrets, whitespace all ok, `receipt written:
  .git/rusty-gate-receipt`, `GATE GREEN [diff]`.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-006 satisfied — REQ-001 by `build_args_carry_the_wire`
  and the scene, REQ-002 by `parse_line_reads_the_probe` and the scene's items, REQ-003
  by the `--resume` assertion, the reading of `agentSessions` and the second scene's
  "Continuing" (Chad's restart to come), REQ-004 by the parse and response tests and the
  scene's prompt, REQ-005 by the two spawn tests and `onExited` (F6), REQ-006 by the
  `--mcp-config` assertion (Chad's "read the open page" to come). None split, none
  waived.
- Wiki: run `6c6a34c4-cdf8-4822-a396-9caf3eaaa26f`, `openwiki_finish` → `complete`; an
  agent-pane bullet, an invariant, a failure mode, a tests bullet, a source; five claims
  re-anchored (the pane and `Main.qml` moved them), the state and top-bar claims
  reworded, one added. `docs/architecture.md` names `Assistant` among the app's objects.
  The PostToolUse hook did not fire (ninth sighting); bulletin 3's recovery with the pair
  under `active/`, then `bin/gate.sh --verify`.
- ROADMAP ticked under M8. `AD-rusty-pane-agent-is-headless-claude-001` in the AAR and
  the register, qualifying `AD-rusty-agents-are-terminals-001`. Brain: timeline entry on
  `projects/rusty-v3`.
