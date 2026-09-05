---
title: AAR-025-native-agent-pane
ticket: TICKET-025
pipeline: aac23d69-4814-4079-b26b-c299897d6741
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-025: Native agent pane

## 0. Recall log

- The register said agents are terminals; the ticket qualifies it for the pane alone.
- Two probes on the box settled the wire before a line was written: one turn over
  stream-json, and a permission asked and answered — `--permission-prompt-tool stdio`
  is the flag that works; `--permission-prompts host` denies unless a host announces.

## 1. Outcome

The pane is a conversation: a headless Claude Code over stream-json, rendered as items,
permissions as buttons, a session per page resumed by id, Rusty reached as an HTTP MCP
server with its reads pre-allowed. One new module with six tests, the pane rebuilt, one
state key, a fake `claude` for the scenes. `GATE GREEN [diff]`; two scenes photographed
twice.

## 2. What went well

- Two probes before a line of code: the wire was known, and `parse_line`'s fixtures are
  the probe's own lines. The permission flag that works (`--permission-prompt-tool
  stdio`) was found by trying, not by reading.
- std threads and `qt_thread().queue`, the `Backend` pattern: no new crate, no tokio
  feature, and the reader thread's exit is the one place the process's end is reported.
- The second scene showed the resume path for free: the scratch state carried the first
  scene's session id, and the pane said "Continuing this page's conversation".

## 3. What went poorly

- Two probe runs learned nothing: the first pre-allowed the very tool it meant to watch
  (`--allowedTools Bash`), the second asked for `echo`, which Claude Code never prompts
  for. Probe the case you will need — a write — not a proxy.
- Two clippy rounds on trivia (`QString::new` does not exist; a collapsible match).

## 4. Surprises

- `--permission-prompts host` is in the help and denies everything without a host that
  announces itself; the SDK's `--permission-prompt-tool stdio` is the wire that prompts.
- `system permission_denied` is what the CLI prints when nobody answers, then the model
  asks the user in prose — the pane would have shown a polite dead end.
- A `ListView` follows new items on `countChanged` but not text growing inside the last
  one; `contentHeightChanged` is the signal a streamed answer needs.

## 5. Lessons

- `AD-rusty-pane-agent-is-headless-claude-001`: the pane is Claude Code's print mode
  over stream-json, the tabs stay terminals.
- Probe a wire with the case you will build on, and read the probe's lines back into the
  tests as fixtures.
- Anything that talks to a service gets a fake the screenshot script seeds; the scene is
  then a test of the rendering alone.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 20m | 15m |
| 2 Design | 45m | 40m |
