---
title: AAR-021-terminal-clipboard
ticket: TICKET-021
pipeline: 7034f494-16ee-4d43-bd21-8353ce4291f4
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-021: Terminal clipboard

## 0. Recall log

- `PR-rusty-signals-through-connections-001` and `AD-rusty-workspace-is-obsidian-001`
  both changed the design before a line was written: the first sends `copyAvailable`
  through `Connections`, the second rules out a window-level `Shortcut`.
- QMLTermWidget ships no `.qmltypes`, so its QML surface was read out of the shared
  object rather than guessed. That turned up `pasteSelection` and `copyAvailable`, which
  the ticket had not anticipated.

## 1. Outcome

Copy and paste in every agent terminal — the tabs and the pane, one component — with
Ctrl+Shift+C / Ctrl+Shift+V, middle-click paste of the primary selection, and a
right-click menu whose Copy is enabled by the widget's own selection state. One file,
thirty-odd lines. `qmllint` clean, `GATE GREEN [diff]`, and the rebuilt binary loaded the
component offscreen with no QML error in either instance.

## 2. What went well

- Reading the shared object before designing. The ticket assumed `copyClipboard` and
  `pasteClipboard`; the `.so` also exports `pasteSelection` and `copyAvailable`, which
  made middle click correct (primary selection, not clipboard) and the menu honest
  (Copy greyed without a selection) for no extra cost.
- The recall rules did their job: two register entries settled two design questions
  before implementation.

## 3. What went poorly

- The first implementation used a `MouseArea`. Inspect caught that Konsole's
  `TerminalDisplay` already handles the middle button, so a passive area above it would
  paste twice, and that a `MouseArea` leaves wheel propagation to scrollback in doubt.
  Both were fixed by switching to a `TapHandler` with an exclusive grab — but the fast
  gate had already run on the earlier file, so the `--diff` gate is the one that covers
  what shipped. Cheap, but it is the second time this pipeline family has shipped-then-
  fixed inside one phase; inspect before the fast gate would have saved a cargo cycle.

## 4. Surprises

- The gate's `cargo test` does not produce `target/debug/rusty`. The binary on disk was
  from the previous evening and predated the QML edit, and because the QML module is
  compiled in (`qrc:`), a runtime check against it would have tested the old code.
  A separate `cargo build -p rusty-app -p rusty-mcp` was needed first.
- `Keys.priority: Keys.BeforeItem` is the default. Kept as stated intent, since the
  ordering is the whole reason the chords work in a widget that eats keys.

## 5. Lessons

- `PR-rusty-exclusive-grab-over-native-handlers-001`: a pointer handler laid over a
  third-party widget that already handles the same button must take an exclusive grab
  or not exist. Read the widget's exported surface first.
- Run inspect before the fast gate when the change is one file; the gate is minutes, the
  read is seconds.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 10m |
| 2 Design | 15m | 10m |
| 3 Implement | 15m | 10m |
| 3.5 Inspect | 10m | 10m (one confirmed finding, F5) |
| 4 Validate | 20m | 25m (two gates, a rebuild, the offscreen run) |
| 5 Complete | 20m | — |
