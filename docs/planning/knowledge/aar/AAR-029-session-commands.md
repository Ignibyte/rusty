---
title: AAR-029-session-commands
ticket: TICKET-029
pipeline: c7987c16-45b5-40b6-b99d-46cb5f0d4915
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-029: Session commands

## 0. Recall log

- The register's `AD-rusty-app-as-session-service-001` made `rusty-session` the one entry
  point; this run keeps the units and moves the entry point into the binary, so the
  decision is amended, not reversed.
- TICKET-010 put store scripts first in the binary's dispatch; the new convention puts
  built-in nouns ahead of them, and the README has to say a script named like a noun is
  shadowed.
- The Omarchy 4 upgrade (2026-09-04) moved the theme to `~/.local/state/omarchy/current`;
  the app's five readers of the theme directory all go through `theme_dir()`, so one
  function carries the fix, and the watcher takes its directory from the same function.

## 1. Outcome

One way in. `rusty session start|stop|status|run` are verbs of the app binary, decided
before Qt starts; the `rusty-session` script is gone from the repo, the installer, the
unit, the desktop entry, the key snippet and the package, and a test keeps it gone.
Built-in nouns come before store scripts and an unknown word is an error, so a typo no
longer opens a window. The theme directory is Omarchy 4's, the Omarchy 3 path behind it,
through the one function every reader and the watcher already used. Eight tests in
`session.rs`, one in `omarchy.rs`; `GATE GREEN [diff]`; installed on the box, where the
unit runs `rusty session run` and the terminal pane has its Tokyo Night scheme back after
a day of black. The wiki, the docs and the register say `rusty session`.

## 2. What went well

- The port took a quarter of an hour: the script had one job per verb and the units carry
  the semantics, so the Rust is four small functions over `systemctl --user`, a
  hand-written `POST`, and `/proc`. The design's regression table was the test list.
- `theme_dir()` was already the one door to the theme, so Omarchy 4 was one function and
  one test with a scratch home.
- The lost session's transcript under `~/.claude/projects/` held the gate output, the
  installer's output and the smokes verbatim, with timestamps. Nothing had to be re-run
  to be claimed, and nothing was claimed that had not run.
- A stand-in `systemctl` first on `PATH` proved every branch of `start` and the whole of
  `stop` (the exact calls, in order) without touching the units Chad was using.

## 3. What went poorly

- The session closed between the gate run and the Phase 4 notes, the second loss this
  week, so Phase 4 was written by a later session from evidence it did not produce. The
  notes now carry the batch-and-write habit as `PR-rusty-notes-as-you-go-001`.
- The live `rusty session stop` was not run against the real unit: Chad was in the app,
  and the standing notice says his desktop is not taken for a probe. The verb's one call
  is proved; the unit's behaviour under `stop` is systemd's and was smoked in TICKET-009.

## 4. Surprises

- SUPER+ALT+R had been dead since the upgrade. Omarchy 4 configures Hyprland from
  `~/.config/hypr/*.lua` and the `.conf` files beside them are read by nothing, so the
  line in `bindings.conf` did not exist as far as the compositor knew. Moved to
  `bindings.lua` on the box; the repo's snippet is Omarchy 3 syntax and is TICKET-030.
- `/proc/<pid>/environ` shows the environment at exec, not after `std::env::set_var`; the
  app's tmux children were the witness that `run` completed `PATH`.

## 5. Lessons

- `AD-rusty-commands-are-nouns-and-verbs-001`: the binary answers `rusty <noun> <verb>`
  before Qt; nouns first, scripts second, dash-prefixed arguments Qt's, anything else an
  error; every future command is a noun of the binary.
- `PR-rusty-newest-desktop-path-first-001` and `BF-rusty-desktop-path-moved-001`: a path
  the desktop owns is read through one function, newest location first, with a test for
  both; after an Omarchy major, read the theme directory and the scheme file's mtime, and
  now also `omarchy menu keybindings --print` for the app's key.
- `PR-rusty-notes-as-you-go-001`: Phase 4 notes are written as each command finishes; a
  lost session's transcript is quotable evidence, named by file and timestamp.
- `PR-rusty-systemctl-stand-in-001`: a verb that drives `systemctl --user` is proved
  against a logging stand-in when the real unit is in use; the live run is the window
  owner's.

## 6. Time spent

From the transcripts' timestamps, two sessions.

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.4 h | 0.3 h (13:47–14:03Z) |
| 2 Design | 0.3 h | 0.1 h (14:03–14:08Z) |
| 3 Implement | 1.2 h | 0.1 h (14:08–14:15Z) |
| 3.5 Inspect | 0.3 h | 0.1 h (14:15–14:19Z) |
| 4 Validate | 0.6 h | 0.4 h (14:19–14:22Z, then 14:33–14:45Z after the loss) |
| 5 Complete | 0.5 h | 0.4 h (14:45Z on) |
