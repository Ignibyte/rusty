---
title: Amber phosphor and themes: notes
pipeline_id: c2bb8a6f-7da2-489c-87ed-d30218b1b8fe
---

# Amber phosphor and themes: running notes

## Phase 1: Plan

- Recall: register `AD-rusty-workspace-is-obsidian-001`, `PR-rusty-workspace-state-in-json-001`,
  `PR-rusty-qml-component-scope-001`, `PR-rusty-qml-signal-names-001`,
  `PR-rusty-offscreen-shots-grab-window-001`; completed notes of TICKET-002 (theme
  tokens from `obsidian.css` and Alacritty, the state file) and TICKET-004 (the graph
  panel's styling); wiki `workspace-app.md` (theme ownership, the extension point for a
  token), `markdown-rendering.md` (the `Style` struct); the mock read whole and rendered
  with headless Chromium (`chromium --headless=new --screenshot`).
- Decisions: the six in the spec.
- Seal: Chad's words of 2026-09-03 and his three answers.

## Phase 2: Design

- Architecture and data flow: `skin::Roles` is the look; `skin::resolve(choice,
  palette, ansi)` produces it from a preset (`PRESETS`, fourteen colours each, the
  rest filled by `fill`), from the Omarchy palette and its ANSI colours
  (`from_omarchy`), or from a TOML file (`parse_theme`: `bg`, `text` and `accent`
  required, the rest derived). `skin::tokens` derives every colour the shell binds to,
  the older names (`surface`, `hover`, `active`, `h1`..`h6`, the graph colours)
  included, so no QML binding changed its name. `Theme` computes the tokens for the
  choice read from the workspace state at start (`startup_choice`, or `RUSTY_THEME`),
  exposes the roles as properties, and `select()` recomputes; `main.rs` sets the
  application font from `startup_font` before the engine loads. The shell keeps
  `{source, name, scanlines}` under `theme` in the state; Settings lists
  `Theme.choices`. `Desk` reads memory, CPU, the clock and Hyprland's workspaces on a
  two-second timer for `TopBar`. The renderer's `Style` gains the roles it paints with
  and two switches (`marks`, `code_head`), fed by the note's `style()`.
- File manifest:

| File | Purpose |
|---|---|
| `crates/rusty-app/src/skin.rs` | roles, presets, the Omarchy mapping, theme files, tokens, tests |
| `crates/rusty-app/src/theme.rs` | the properties, `select`, `reload`, the startup choice and font |
| `crates/rusty-app/src/desk.rs` | the top bar's readings, `switch_workspace` |
| `crates/rusty-app/src/omarchy.rs`, `main.rs`, `build.rs`, `Cargo.toml` | the token readers the skin replaced go; the font; the new files; `chrono`, `serde` |
| `crates/rusty-core/src/brain/render.rs` | `Style` roles and switches, heading marks, the rule, code header strips, task colours, uppercase callout labels |
| `qml/TopBar.qml`, `qml/Scanlines.qml` | the top bar; the overlay |
| `qml/Main.qml` | the state key, `selectTheme`, `Desk`, the top bar, the rail, pane heads, the footer, the tab strip, the status bar, the toast, the scene |
| `qml/Explorer.qml`, `NoteTab.qml`, `RightPane.qml`, `SearchPane.qml`, `SettingsPage.qml`, `QuickSwitcher.qml`, `CommandPalette.qml` | the surfaces |
| `scripts/screenshot.sh` | a theme file in the scratch config, `theme:` scenes, slashes in scene names |
| `README.md`, `ROADMAP.md`, `docs/architecture.md`, `docs/screenshots/`, `openwiki/` | the tier |

- Store consequences: none.
- Tool contract: `brain_render`'s `style` accepts the new keys; the rest unchanged.
- Regression plan: REQ-001 `grep` and the skin tests; REQ-002 the skin tests and the
  three `theme:` scenes; REQ-003 to REQ-007 screenshots and reading; REQ-008 the
  scenes. The renderer test covers the marks.
- Risks: the application font cannot change items already built, so a face change
  waits for the next launch (named in Settings); `hyprctl` needs the session's runtime
  dir, so the strip is static offscreen; the `<hr>` under a section title sits below
  the text rather than beside it, as rich text allows.
- CodeGraph evidence: `omarchy::tokens` and `obsidian_tokens` had one caller each
  (`Look::gather`), so they went with the skin.

## Phase 3: Implement

- Built: the manifest. Rust: `skin.rs` with four presets and four tests, `theme.rs`
  rewritten around `compute` and `apply`, `desk.rs`, the font in `main.rs`, the
  renderer's marks with a test. QML: `TopBar` (brand, workspace strip, vault state,
  memory, CPU, clock, quit), `Scanlines`, rail buttons with labels and the mock's
  selected state, the om-mark and the avatar, pane heads as micro-labels, the tree's
  glyphs, counts and active bar, the tab strip's glyphs and amber underline, the
  breadcrumb bar with `[ READ ]`, the meta line, the `#` before the title, the
  "linked from" footer, the legend card, the assistant header and context card, the
  skin picker and the scanline switch, the toast, the palette and switcher frames.
- Deviations: the "distant nodes" count comes from a depth-3 neighbourhood, one
  breadth-first walk in QML; the mock's numbers were static.
- Fast gate: `cargo build --workspace` clean; `cargo test -p rusty-core marks_add`
  passed; eleven scenes rendered without a QML error.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | the first screenshots showed no heading marks: `cargo build -p rusty-app` had not rebuilt `rusty-mcp`, and the screenshot script runs both | medium | fixed: build the workspace before shooting (PR-rusty-build-workspace-before-shots-001) |
| 2 | layout | the assistant's state overlapped the right sidebar's collapse button, which Main.qml draws over the pane | low | fixed: a right margin on the head |
| 3 | layout | the left footer's text elided beside the two buttons | low | fixed: shorter words |
| 4 | correctness | a scene name with a slash made a path the script could not write | low | fixed: slashes become dashes |
| 5 | data safety | the skin choice is a JSON string in the state; a bad one parses to the default choice, and an unknown name resolves to the first preset | ok | by reading and the skin test |
| 6 | performance | `hyprctl` runs twice every two seconds on the Qt thread; a few milliseconds | ok | accepted |
| 7 | theme | every colour in the new QML is a token; `grep -c '"#'` over the QML finds none outside `Icon.qml`'s paths | ok | by `grep` |
| 8 | prose | the README paragraph, the Settings text and the wiki against `no-ai-slop` | ok | clean |

## Phase 4: Validate

- Tests run (commands and output): the gate ran `cargo test --workspace`: the four
  skin tests (presets resolve and fall back, the Omarchy mapping, theme files need
  three colours, tokens carry the old names and the new), the colour math, the desk
  readings, the renderer's marks; everything older unchanged.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: GATE GREEN [diff].
- Smoke evidence: `scripts/screenshot.sh` scenes `reading`, `edit`, `switcher`,
  `palette`, `right:tags`, `graph`, `left:bookmarks`, `view:settings` in Amber
  phosphor, `theme:omarchy` (Tokyo Night mapped), `theme:file:ember` (a theme file),
  `theme:preset:paper` (the light preset); the docs screenshots are these.
- Skips or pre-existing failures: the workspace strip's click and the font at the next
  launch are verified by reading; no synthetic input on the desktop.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (`grep`, the tokens test); REQ-002 PASS (skin tests,
  three scenes, the state key by reading); REQ-003 PASS (screenshots); REQ-004 PASS
  (screenshots, the strip by reading); REQ-005 PASS (renderer test, screenshots);
  REQ-006 PASS (by reading; the card shows when the note has room); REQ-007 PASS
  (screenshot); REQ-008 PASS (the script's default scenes).
- Wiki: `update` run through the lifecycle, openwiki_finish returned status complete; `workspace-app` and
  `markdown-rendering` reconciled.
- Docs: README (the skin paragraph, two screenshots, every screenshot re-rendered),
  ROADMAP (M8 line), `docs/architecture.md` (a dated bullet), this pair.
- AAR: `docs/planning/knowledge/aar/AAR-008-amber-phosphor-and-themes.md`.
- Brain capture: timeline entry on `projects/rusty-v3`.
- Archive: this pair lives in `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-03 | a stale server binary behind fresh screenshots | PR-rusty-build-workspace-before-shots-001 |
| 2026-09-03 | the look as roles with three sources | AD-rusty-skin-roles-001 |
