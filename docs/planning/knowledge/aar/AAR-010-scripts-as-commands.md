---
title: AAR-010-scripts-as-commands
pipeline_id: 0cbefd28-f640-49c3-a2cf-78bc996d66c2
ticket: TICKET-010
submitted: 2026-09-03
---

# AAR-010-scripts-as-commands

## Recall log

- As in the notes; sealed at 17:20 (relayed) with the four answers.
- At Phase 3.5 the register was read again for the store and hook rules
  (`PR-rusty-gate-the-repository-not-the-tool-001`, `AD-rusty-mcp-only-back-end-001`).

## 1. Outcomes

A `*.sh` beside a skill is a command. `rusty usb-reset` runs it with its arguments and its
exit status, before Qt starts; `rusty-cli scripts list|view|new|rm|path|edit|run` manages
them; four tools serve agents, `script_run` only for an approved script and only for sixty
seconds; the Skills tab lists scripts under the skills with view, edit, scan and a Run that
opens a terminal tab which keeps a shell after the script. A pending skill's script does not
run, and the scan that reads a skill reads a script.

Two sessions delivered it. A Fable session wrote Phase 1 to Phase 3 and stopped on a usage
limit at 18:52, mid-file. An Opus session reviewed the whole diff, fixed four defects, wrote
the tests and the evidence, and delivered.

## 2. What went well

- The design's own regression plan was specific enough to run months later by a different
  model: a scratch store, a script that exits 3, an unknown name under an offscreen Qt. Two
  of the four defects were found by trying to satisfy it rather than by reading.
- The resolver landing in `rusty-core` kept the app thin: the app decides "is there a file",
  and everything with a rule behind it (pending, clash, exec) has one home.

## 3. What went poorly

- The handover cost a review. An interrupted session leaves a tree that looks finished:
  every file written, nothing compiled. The first useful act was `cargo build`, not reading.
- The store had two resolvers. The app read `RUSTY_SKILLS`, the CLI read the setting, and
  nothing in the diff connected them; the design's own test would have failed on the box.

## 4. Surprises

- The broken Rust literal in `terminals.rs` was the *only* thing standing between the tree
  and a green build. Everything else compiled on the first try.
- `command_for`'s result goes to tmux as one argument, which hands it to `sh -c`. The
  original `sh -c '…'` wrapper survived only because adjacent shell quotes concatenate, and
  a path with a space would have split it.

## 5. Lessons

- `PR-rusty-one-store-one-resolver-001`: when two binaries can open the same store, one
  function resolves its path and both call it; an environment override that only one of them
  honours makes the found script and the run script different files.
- `PR-rusty-build-before-you-read-a-handover-001`: the first act on an interrupted tree is a
  build, not a review. A file written but never compiled can hold a syntax error that costs
  nothing to find and a whole review pass to miss.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 20m | 15m (Fable) |
| 2 Design | 30m | 25m (Fable) |
| 3 Implement | 1h | 1h 10m (Fable, interrupted) |
| 3.5 Inspect | 20m | 25m (Opus: four defects, two rejections) |
| 4 Validate | 30m | 30m (Opus) |
| 5 Complete | 20m | 20m (Opus; the wiki step outstanding) |
