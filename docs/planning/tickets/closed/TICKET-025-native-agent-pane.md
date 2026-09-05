---
title: TICKET-025-native-agent-pane
status: done
ticket_number: 025
type: feature
created: 2026-09-04
intake:
pipeline_spec: docs/planning/pipeline/active/native-agent-pane.spec.md
---

# TICKET-025-native-agent-pane

## Summary

Replace the terminal in the agent side pane with a native chat UI driven by a headless `claude -p` process over streaming JSON, so the assistant beside a note is not a second terminal emulator.

## Why

The agent pane is useful and its packaging is wrong. It mounts a whole `AgentTerminal` — tmux session, terminal widget, ANSI — to hold a conversation, which means no selectable message bubbles, no copy without TICKET-021, no way to render a tool call as anything but text, and a second full agent process beside the note.

Claude Code's print mode is built for exactly this, and the flags are on the box today:

```
claude -p --input-format stream-json --output-format stream-json \
        --include-partial-messages --resume <session-id> --model <model>
```

`--input-format stream-json` is the part that matters: with both formats set to `stream-json` the process is **long-lived and bidirectional**, not one-shot per message. Rusty writes a message in, reads structured events out, and renders them itself. `--session-id`, `--resume`, `--continue` and `--fork-session` give continuity across app restarts; `--mcp-config` points it at Rusty's own MCP server so the pane's agent has the brain, tasks and notes tools; `--permission-mode` and `--permission-prompts host` mean Rusty can render approvals as buttons instead of a terminal prompt.

This keeps Claude Code's harness — skills, tools, MCP, the existing auth — and drops only the terminal. It is not the Claude API and needs no API key.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the agent pane is opened for a page, it shall start one headless `claude -p` process with `--input-format stream-json --output-format stream-json` and render the conversation natively. | smoke; screenshot |
| REQ-002 | WHEN the assistant replies, the pane shall render text, tool calls and tool results as distinct, selectable elements, with partial messages streamed as they arrive. | smoke |
| REQ-003 | WHEN the app restarts, reopening the pane for the same page shall resume that page's session by id rather than start a fresh one. | smoke across a restart |
| REQ-004 | WHEN the agent asks for a permission, the pane shall render it as a prompt with buttons and return the answer to the process. | smoke |
| REQ-005 | WHEN the process exits or fails to start, the pane shall say so and offer to restart, and shall never leave a half-rendered conversation. | test; smoke |
| REQ-006 | WHEN the pane's agent uses a tool, it shall reach Rusty's own MCP server, so the brain, tasks and notes are available to it. | smoke: ask it to read the open page |

## Scope

- In: a native chat view in `RightPane.qml`; a process/session manager in the app or core; stream-json parsing; per-page session ids; MCP wiring through `--mcp-config`.
- Out: replacing the full terminal tabs — those stay a real terminal on purpose; the Claude API or an API key; the Claude Agent SDK (a different library, and Rust is not one of its languages); multi-agent orchestration.

## Notes

- Terminal tabs and this pane are deliberately different things: a tab is the real agent with a real terminal, the pane is a conversation about the open note.
- `--append-system-prompt` is the hook for telling the pane's agent which page is open.
- Codex has no equivalent print mode; the pane stays Claude-only until it does, and should say so rather than offer a broken choice.
- Pipeline spec: TBC.
