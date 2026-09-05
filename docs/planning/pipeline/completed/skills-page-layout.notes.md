---
title: Skills page layout — notes
pipeline: 2e5afa0a-3edb-46f2-aa18-f2c0a1d69379
ticket: TICKET-023
---

# Skills page layout: notes

## Recall (2026-09-05)

- Bulletins: none critical; bulletin 2 shapes validation as before.
- Register: `BF-rusty-moving-frame-delta-001` (TICKET-022) is the reason this page takes
  the corrected splitter rather than growing its own. `AD-rusty-workspace-is-obsidian-001`
  and the wiki's invariant: page state lives in the JSON the Rust side owns, never in
  QtCore `Settings`.
- Wiki: `workspace-app.md` describes the workspace state and, since 022, how the
  splitters measure a drag. The Skills page is described only as the Scripts section's
  host.
- Code read:
  - `SkillsPage.qml`: a `RowLayout` — a 300px `Rectangle` holding a `ColumnLayout`
    (header row, the skills `ListView` with `fillHeight`, a "Scripts" `Text`, the scripts
    `ListView` with a capped `preferredHeight`, the notice), a 1px divider, the detail.
    No handle, no collapse. The page takes `backend` and `theme` and emits `runScript`.
  - `Main.qml` `ui`: typed properties, each with `onXChanged: save()`, loaded by
    `if (typeof s.X === "string") ui.X = s.X`, written by `terminals.saveState(JSON)`
    with an explicit key list. Structured page state is kept as JSON strings (`graph`,
    `bookmarks`, `expanded`, `roots`).
  - `Splitter` is `component Splitter: Item` inside `Main.qml`, used twice. A
    `component` is file-local; the page cannot import it.
  - `build.rs` lists every QML file in `qml_files([...])`; a new file needs a line there.

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-app/qml/Splitter.qml` | new: the component from `Main.qml`, verbatim, with `isLeft` replaced by a generic contract: `property real value`, `property real min`, `property real max`, `property bool invert`, `signal moved(real value)` |
| `crates/rusty-app/build.rs` | `"qml/Splitter.qml"` in `qml_files` |
| `crates/rusty-app/qml/Main.qml` | the inline `component Splitter` removed; the two call sites bind `value`/`min`/`max`/`invert` and set `ui.leftWidth` / `ui.rightWidth` in `onMoved`; `ui.skillsLayout` declared, loaded, saved; `skillsComp` binds `savedLayout: ui.skillsLayout` and `onLayoutChanged: ui.skillsLayout = json` |
| `crates/rusty-app/qml/SkillsPage.qml` | `listWidth`, `skillsOpen`, `scriptsOpen`; a `Splitter` between list and detail; two `SectionHeader` rows (chevron, label, focusable, Enter/Space); the lists' `fillHeight` follows which is open; `savedLayout` in, `layoutChanged` out |

### The splitter's contract

`Splitter { value; min; max; invert; onMoved }`. The sidebars: left binds
`value: ui.leftWidth, min: 180, max: 600`; right binds `value: ui.rightWidth, min: 200,
max: 700, invert: true` (a drag left grows it). The page binds `value: page.listWidth,
min: 200, max: 600`. Scene coordinates inside, as TICKET-022 left it.

### The state key

`ui.skillsLayout` — a JSON string `{"width": 300, "skills": true, "scripts": true}`,
default `""` (the page uses its defaults). The page parses it once on `savedLayout`
changing and emits `layoutChanged(JSON)` whenever any of the three changes.

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | reading: the page's `Splitter` binds 200–600 and sets `listWidth` in `onMoved` |
| REQ-002 | `RUSTY_SHOT_SCENE=view:skills,skills:collapse-scripts` — a scene that collapses one section — photographed offscreen |
| REQ-003 | reading of declare/load/save; a scratch state file showing `skillsLayout` after a change |
| REQ-004 | reading: both sidebar call sites carry the same clamps as before and the same scene-coordinate MouseArea |
| REQ-005 | reading: the header's `Keys.onPressed` handles Return and Space |

### Risks

- Lifting `Splitter` touches the sidebars: the two call sites must carry exactly the
  clamps and the direction they have now (REQ-004). The right sidebar grows as the pointer
  moves *left*, which is the `invert` flag.
- `build.rs` is Rust: the gate's clippy and the doc build cover it; the module's
  `qml_files` list is the only edit.
- A collapsed Scripts section with zero scripts: the header hides as the list does now
  (`visible: page.scripts.length > 0`), so nothing toggles what is not there.
- No back end: the layout keys are the app's; nothing here calls a tool.

### CodeGraph

`build.rs` is the only Rust touched and it holds no symbols the graph tracks; QML by hand.

## Phase 3: Implement

Four files, as the manifest said: `Splitter.qml` new, one line in `build.rs`, the lift and
the state key in `Main.qml`, the split and the sections in `SkillsPage.qml`. `qmllint` exit
0 on all three QML files.

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | correctness | `SectionHeader` was a `RowLayout` holding its own focus ring as an `anchors.fill` child; a Layout manages its children's geometry, warns about anchors on them, and would have given the ring a cell | **confirmed**; the header is an `Item` sized by an inner `RowLayout`, the ring a sibling of the row |
| F2 | prose/whitespace | lifting the inline component left a double blank line | **confirmed**; collapsed |
| F3 | correctness | the scripts list's `fillHeight`/`preferredHeight` across the four open/closed combinations | rejected: both open keeps the cap; skills closed fills; scripts closed hides; both closed leaves the spacer to fill |
| F4 | data safety | `applyLayout` echoing back through `reportLayout` while applying | rejected: `applyingLayout` guards the three change handlers; `finally` clears it even when the JSON is bad |
| F5 | correctness | the `skills:` scene runs before the page exists | rejected: it sets `ui.skillsLayout`; the page binds `savedLayout` at creation and applies it in `Component.onCompleted` |
| F6 | keyboard first | the header reachable by Tab and toggled by Enter/Space | no finding after F1: `activeFocusOnTab` on the `Item`, `Keys.onPressed` handles Return, Enter, Space |
| F7 | theme | the page's divider was `accent` at 0.25; the shared handle draws `theme.line` | accepted change: the page now has the app's standard handle, one token |
| F8 | keyboard first | the split itself is mouse-only | noted, not fixed: the sidebars are the same, and it predates this ticket |
| F9 | correctness | `Splitter.theme` is `required`; every call site passes it | rejected: three sites, all bind it |
| F10 | correctness | `build.rs` change | rejected as a risk: one string in a list the gate's clippy and doc build compile |
| F11 | correctness | `Math.round(v)` into an `int` property at every `onMoved` | rejected: rounded before assignment at all three sites |

CodeGraph: `build.rs` holds no tracked symbols; QML by hand.

## Phase 4: Validate

- `bin/gate.sh --fast` after inspect: `GATE GREEN [fast]`.
- `cargo build -p rusty-app -p rusty-mcp` (22:37:21), then `SHOT_KEEP=1 scripts/screenshot.sh
  <scratch> "view:skills" "skills:collapse-scripts,view:skills"`, offscreen against a scratch
  vault. Both logs clean of `error|warning|TypeError|ReferenceError|Cannot assign|is not a
  type|Detected anchors` — the last is the Layout-managed-anchors warning F1 was about, and
  it did not appear.
  - `view-skills.png`: the list pane with "▾ 1 skills" and New, the skill row, "▾ Scripts"
    and its one row at the bottom, the handle between list and detail — REQ-002's open state.
  - `skills-collapse-scripts-view-skills.png`: "▸ Scripts" collapsed, its list gone, the
    skills list holding the height — REQ-002.
  - The kept scratch `workspace.json` after that scene: `skillsLayout =
    {"width":300,"skills":true,"scripts":false}` — REQ-003's file evidence; the load path
    is the same shape as every other key.
- REQ-001 and REQ-004 rest on the ledger's readings (F9, F11) and Chad's use; REQ-005 on F6.
- The gold-linker warning printed again from `cargo build`; box configuration, unchanged.
- One thing seen, not caused here: the scratch server listed the box's real skill
  (`dev-box-usb` / `usb-reset`), so the screenshot script's scratch store resolves to the
  real skills path. TICKET-010's own evidence showed the same. The scene only lists; noted
  in the AAR for a look at the script's `RUSTY_SKILLS` seeding, out of this ticket's scope.

- `bin/gate.sh --diff` after the last gated edit: every step ok, `GATE GREEN [diff]`, receipt written.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-005 satisfied — REQ-002 and REQ-003 by the offscreen
  scenes and the kept state file, the rest by the ledger's readings and Chad's use. None
  split, none waived.
- Wiki: run `a67b6ecc-9100-47f8-beed-10dcb5f46a8b`, `openwiki_finish` → `complete`; the
  splitter bullet rewritten around `Splitter.qml`, the Skills paragraph extended, six claims
  re-anchored by verified text, two added. The PostToolUse hook did not fire (fifth
  sighting); bulletin 3's recovery with the pair under `active/`.
- ROADMAP ticked under M8. `AD-rusty-one-splitter-owner-clamps-001` in the AAR and the
  register. Brain: timeline entry on `projects/rusty-v3`.
