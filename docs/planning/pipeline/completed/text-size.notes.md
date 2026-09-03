---
title: Text size: notes
pipeline_id: c796967f-73d9-4675-9107-40393f31f132
---

# Text size: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins (three notices, the hook one added today). Register:
  `PR-rusty-workspace-state-in-json-001` (state in the JSON files the Rust side owns),
  `PR-rusty-qml-component-scope-001`, `AD-rusty-skin-roles-001` (`Theme` derives every
  token). TICKET-008 notes: the application font is set before the engine loads, sizes
  were left as literals. Census on 2026-09-03: 187 literal `pixelSize` values across 15
  QML files (7 once, 9 twenty-two times, 10 nineteen, 11 twenty-five, 12 sixty, 13
  twenty-two, 14 twenty-four, 15 six, 16 once, 20 once, 22 four, 28 twice), no computed
  ones; three `pointSize` values (the terminal at 11, the note editor at 11, the skills
  editor at 10.5). `Theme` has no size property; `Style::size` in the renderer is the
  reading view's base in pixels; the workspace state is `ui` in `Main.qml`, saved through
  `terminals.saveState`.
- Decisions: the four locked decisions in the spec.

## Phase 2: Design

- Architecture and data flow: `Theme` gains `baseSize` (12 to 18, default 14, or
  `RUSTY_TEXT_SIZE` when set, which then wins over the state) and `scale`
  (`baseSize / 12`, the body size the mock set), plus the invokable `setTextSize(px)`,
  which clamps and sets both so every binding that reads `scale` re-evaluates. Every
  literal `pixelSize: n` in the QML becomes `Math.round(n * <owner>.theme.scale)`, the two
  editors' `pointSize` scale the same way, and the chrome containers that hold text (the
  top bar, its buttons, the ribbon and its buttons, the side tabs, the tab strip, the
  pane heads, the status bar) scale their fixed sizes. The note passes
  `Math.round(15 * scale)` as the render style's `size` and re-renders when `scale`
  changes. The window keeps `textSize` in the workspace state, applies it at load, and
  owns `setTextSize(n)`; three `Shortcut`s and three palette commands step it; Settings
  shows a `SpinBox` under "This machine" that raises `setTextSize` on the page.
- File manifest:
  - `crates/rusty-app/src/theme.rs`: the two properties, the invokable, the helpers
    (`DEFAULT_TEXT_SIZE`, `clamp_text_size`, `text_size_from_env`), the tests (helpers;
    the QML scan for literal sizes).
  - `crates/rusty-app/qml/*.qml` (every file but `AgentTerminal.qml`, `Icon.qml`,
    `Scanlines.qml`): the sweep.
  - `crates/rusty-app/qml/Main.qml`: `ui.textSize`, `setTextSize`, the keys, the
    commands, the chrome sizes, the Settings wiring.
  - `crates/rusty-app/qml/NoteTab.qml`: the style's `size`; a re-render on `scaleChanged`.
  - `crates/rusty-app/qml/SettingsPage.qml`: the stepper and its signal.
  - `crates/rusty-app/qml/TopBar.qml`: the bar height and the button height.
  - Phase 5: `README.md`, `ROADMAP.md`, `docs/architecture.md`, the wiki page
    `workspace-app.md`, the docs screenshots (every one shows text).
- Store consequences: one new key, `textSize`, in `~/.config/rusty/workspace.json`;
  absent means the default.
- Tool contract: none; `brain_render` already takes `size`.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001 | the Rust test `qml_text_sizes_derive_from_the_theme` scans `qml/` and fails on any literal `pixelSize`, or a literal `pointSize` outside `AgentTerminal.qml` |
  | REQ-002 | the Rust test on the helpers: the default is 14, the clamp holds 12 to 18 |
  | REQ-003 | the `view:settings` scene shows the stepper; the state key by reading `write()` |
  | REQ-004 | the three `Shortcut`s and commands by reading; the Hotkeys table lists them |
  | REQ-005 | `AgentTerminal.qml` untouched (`git diff --stat`) and the scan test allows its `pointSize` |
  | REQ-006 | the `reading` and `view:settings` scenes at `RUSTY_TEXT_SIZE=12` and `18` |
- Risks: data safety, none. A missing `scale` on a stale theme object cannot happen (a
  qproperty always has a value). Keyboard: the three keys stand down while a terminal has
  focus, as every workspace key does. Theme: no colour touched. Performance: 187 bindings
  re-evaluate on a change, a one-off relayout.
- CodeGraph evidence: `ThemeRust` is built in `theme.rs` alone (`Default`, `select`,
  `reload` set fields through setters); `Style::size` in `render.rs` is read by the
  renderer's heading and code scale and nowhere else in the app.

## Phase 3: Implement

- Built: `theme.rs` (`baseSize`, `scale`, `setTextSize`, the helpers, two tests);
  the sweep over fifteen QML files (189 sizes: 187 `pixelSize`, the two editors'
  `pointSize`) by a script that used each file's own theme reference (`page.theme`,
  `pane.theme`, `note.theme`, `bar.theme`, `pop.theme`, `explorer.theme`, `view.theme`,
  bare `theme` in `Main.qml`); the chrome sizes in `TopBar.qml` (the bar, its buttons)
  and `Main.qml` (the ribbon and its buttons, the side tabs, the tab strip, the pane
  heads, the status bar, the avatar); `ui.textSize` loaded, saved and applied;
  `win.setTextSize`; three `Shortcut`s and three palette commands; the Settings
  `SpinBox`; the note's render `size` and a re-render on `scaleChanged`.
- Deviations: the first form of the scan test flagged the editors' `pointSize: 11 *
  note.theme.scale` because the value starts with a digit; the test now calls a size
  derived when its value reads `theme.scale`. The skin note under the Settings chips
  clipped at the larger sizes; fixed in Phase 3.5.
- Fast gate: `bin/gate.sh --fast` on 2026-09-03: `GATE GREEN [fast]` after the test fix
  (20 tests in `rusty-app`, the two new ones included).

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | the scan test's first rule counted the editors' derived `pointSize` as literal | low | fixed: a value that reads `theme.scale` is derived |
| 2 | layout | the skin note under the Settings chips overflowed its column at 14 and 18 | low | fixed in the page's container (see Phase 4) |
| 3 | correctness | `setTextSize` under `RUSTY_TEXT_SIZE` is ignored, so the stepper shows the forced value and does nothing | ok | by design, a screenshot and test knob; the doc comment says so |
| 4 | data safety | the state gains one key; an old file without it means the default | ok | by reading `load()` |
| 5 | keyboard | Ctrl with plus, minus and zero stand down while a terminal has focus, like every workspace key; the palette lists all three | ok | the `view:settings` scene shows the Hotkeys rows |
| 6 | scope | `Icon` sizes are not text and stay; the terminal keeps `pointSize: 11` | ok | REQ-005; the scan test allows it |
| 7 | performance | 189 bindings and one render re-evaluate on a change | ok | a one-off relayout on a rare action |
| 8 | prose | the Settings strings, the doc comments, the README sentence | ok | zero em dashes; no banned words |

- Post-implementation CodeGraph: `ThemeRust` and its setters are used in `theme.rs`
  alone; `set_text_size` has no Rust caller (QML calls it); `Style::size` unchanged in
  `render.rs`.

## Phase 4: Validate

- Tests run (commands and output): `cargo test -p rusty-app theme::tests`: the two new
  tests green (`the_base_size_defaults_to_fourteen_and_stays_between_twelve_and_eighteen`,
  `qml_text_sizes_derive_from_the_theme`), and the gate ran every test; `grep -c
  'pixelSize: [0-9]' crates/rusty-app/qml/*.qml` prints 0 for every file; `git diff
  --stat` shows `AgentTerminal.qml` untouched; `scripts/screenshot.sh` with
  `SHOT_ENV=RUSTY_TEXT_SIZE=12|14|18` wrote `reading` and `view:settings` at each size
  with no QML error; the ten docs scenes wrote at the default.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: `GATE GREEN [diff]`, `receipt written:
  .git/rusty-gate-receipt`.
- Smoke evidence: at 12 the window is the pre-ticket look; at 18 the top bar, the ribbon
  and its labels, the explorer, the tab strip, the note body, the properties block, the
  backlinks pane and the status bar all grow together; at 14 the Settings scene shows the
  Text size stepper at 14 and the Hotkeys rows "View: Larger text Ctrl+=" and "View:
  Smaller text Ctrl+-". No synthetic input on the desktop.
- Skips or pre-existing failures: the keys are verified by reading the `Shortcut`s and
  by the Hotkeys table; the state round trip by reading `load()` and `write()`; the
  terminal's unchanged size by the untouched file and the scan test's allowance.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (the scan test); REQ-002 PASS (the helpers test);
  REQ-003 PASS (the Settings scene at 14; `write()` by reading); REQ-004 PASS (the three
  `Shortcut`s and commands; the Hotkeys rows); REQ-005 PASS (`AgentTerminal.qml`
  untouched); REQ-006 PASS (the scenes at 12 and 18).
- Docs: `README.md` (the Settings sentence), `docs/architecture.md` (the skinned
  bullet), `ROADMAP.md` (M7 line), the ten docs screenshots re-rendered at the default.
- Wiki: `update` run `bca8fbbd` through the lifecycle: five claims on `workspace-app.md`
  refreshed after the sweep moved their lines, two claims added (the base size and its
  scale; the state, the keys, the stepper and the scan test), the theme bullet extended;
  `openwiki_finish` returned `status: complete`; the PostToolUse hook stayed silent and
  the genuine result was fed to `record-pipeline-tool-use.sh` (the bulletin's path).
- AAR: `docs/planning/knowledge/aar/AAR-012-text-size.md`; no new register IDs.
- Brain capture: timeline entry on `projects/rusty-v3` at delivery.
- Archive: this pair lives in `completed/`; the ticket in `closed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
