---
title: Live preview editing — notes
pipeline: 3c7a7dd7-ed6c-4903-8af3-75505c3b5475
ticket: TICKET-028
---

# Live preview editing: notes

## Recall (2026-09-05)

- Bulletins: bulletin 2 shapes validation; the scene enters a section by a function,
  never a synthetic click on Chad's desktop.
- Register: `AD-rusty-renderer-in-core-001` (rich text from `brain_render`, split at
  `HEADING_MARK` into one block per top-level heading), `AD-rusty-workspace-is-obsidian-001`.
- The ticket's own staging note: a setting for which view a page opens in, and
  click-to-edit — both are inside this design (the mode key and the section editor).
- Code read:
  - `NoteTab.qml`: `editing` flips the reading column (a `Repeater` of `Text` blocks
    over `chunks`, `onLinkActivated` → `onLink`) against the whole-file `editor`
    (`TextArea`, `MarkdownHighlighter` on its document, `autosave` 1500 ms, `save()`
    writes `editor.text` through `brain_write_page`, `Ctrl+S` and `Ctrl+E` on the
    editor); the render answer sets `raw`, `chunks` (`html.split("<!--h-->")`) and the
    editor's text when not dirty; `reload()` waits while dirty; `scrollToHeading`
    knows the preamble offset (`chunks[0]` not starting with `<h`).
  - `render.rs`: `HEADING_MARK` before a heading at depth 0 outside quotes, lists and
    footnotes — any level.
  - `markdown.rs`: a `cxx_qt::bridge` of `extern "Rust"` functions for C++ (the
    tokenizer), fences by ``` and `~~~`; `cpp/tools.*`: `Tools`, a `QML_ELEMENT` with one
    invokable, instantiated once in `Main.qml`; `highlighter.cpp` includes
    `rusty-app/src/markdown.cxxqt.h` and calls `rusty::highlight_line` with a `rust::Str`.
  - `Main.qml`: `ui` keys with load, save and write; `Ctrl+E` and "Toggle reading view"
    call `toggleEditing`; the `edit` scene sets `editing`.

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-app/src/markdown.rs` | `page_sections(raw) -> Vec<String>` in the bridge and in Rust: the frontmatter (or an empty string) first, then one part per heading line outside fences, the preamble its own part; a test |
| `crates/rusty-app/cpp/tools.h`, `tools.cpp` | `Q_INVOKABLE QStringList pageSections(const QString&)` over the bridge |
| `crates/rusty-app/qml/NoteTab.qml` | `editMode`, `tools`, `live`, `liveIndex`, `liveWhole`, `parts`, `partsMatch`, `renderHeld`; `splitParts`, `editSection`, `commitSection`, `assemble`, `ensureVisibleIn`; the delegate as a block with its rendered `Text` and a section `TextArea` with its own highlighter; `save`, `reload`, `toggleEditing`, `scrollToHeading`, the header label and the render branch adjusted |
| `crates/rusty-app/qml/Main.qml` | `ui.editMode`; `editMode` and `tools` on the note; three commands; the `live:` scene |

### The split

`page_sections(raw)`: when `raw` starts with `---\n`, the frontmatter runs to the next
`\n---\n` (or `\n---` at the end) and is part 0, else part 0 is empty; the body is cut
before every line that is a heading (`#` × 1–6 then a space or the end) while not
inside a fence (a line starting with ``` or `~~~` toggles it); the text before the
first heading is a part when it is not empty. `parts.join("")` is `raw` again.

### The section editor

The block delegate holds both views. In live mode a tap on the rendered block (not on a
link) calls `editSection(index, y / height)`: an earlier section still open is
committed; `liveIndex` becomes the index (the parts line up) or the whole body goes to
the source editor (they do not — `liveWhole`); the block's `TextArea` takes the part's
text with `applying` set, focus, and the caret at the start of the line the height
fraction points at. Its `onTextChanged` marks the page dirty and restarts `autosave`;
`Ctrl+S` saves; `Escape` commits; losing focus commits. `assemble()` joins the parts
with the open section's text in place of its part; `save()` writes that (or
`editor.text` in source mode). A save or a change notification while a section is open
sets `renderHeld` instead of rendering; `commitSection()` clears the open section and
renders again when held. `toggleEditing()` commits first; `[ LIVE ]` is the header's
word for the mode.

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | reading of `editSection` (focus, the caret line); the `live:1` scene; Chad's click |
| REQ-002 | `page_sections_split_at_headings_outside_fences` (a fenced `#` line stays in its section, the preamble, six levels, the identity); the scene |
| REQ-003 | reading: one `autosave`, one `save`, the section editor's `Ctrl+S` |
| REQ-004 | reading of `ui.editMode` and the three commands; Chad's restart |
| REQ-005 | reading: the same `Text` blocks and `onLink`; a tap with `hoveredLink` set never enters a section; the scene's links |

### Risks

- **Data safety.** `assemble()` is the parts with one replaced; the identity test on
  the split is what keeps a save from dropping a byte; a mismatch falls back to the
  whole-file editor rather than a guess.
- **The re-render.** Held while a section is open; a save's answer and a change
  notification both wait for the commit.
- **Keyboard.** `Ctrl+E`, the three commands, `Escape` to commit, `Ctrl+S`; the caret
  lands by the click, then the keys are the editor's.
- **Theme.** The section editor is the source editor's face and tokens.

### CodeGraph

`codegraph_explore` over `highlight_line`, `HEADING_MARK`: the tokenizer has one C++
caller; the mark is pushed in `render.rs` at depth 0. A second pass after
implementation goes in the ledger.

## Phase 3: Implement

As the manifest said. `markdown.rs`: `page_sections` (the frontmatter first, then the
preamble, then one part per ATX heading outside a fence; the parts join back to the
page) and `is_heading_line`, in the bridge for C++; one test. `cpp/tools.h`, `tools.cpp`:
`pageSections` over the bridge, a `QStringList`. `NoteTab.qml`: `editMode`,
`sectionTools`, `live`, `parts`, `partsMatch`, `liveIndex`, `liveWhole`, `renderHeld`,
`sourceVisible`; `splitParts`, `assemble`, `editSection`, `commitSection`,
`ensureVisibleIn`; the block delegate with its rendered `Text` and its section
`TextArea` and highlighter; `save`, `reload`, `toggleEditing`, `scrollToHeading`, the
render and saved branches, the header label and tooltip. `Main.qml`: `ui.editMode`
(load, save, write), `sectionTools` and `editMode` on the note, three commands, the
`live:` scene. `qmllint` exit 0, `cargo fmt --all` ran; the C++ compiled in the gate.

Deviations: the note's split property is `sectionTools`, not `tools` (F1 below).

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | correctness | `tools: tools` on the `NoteTab` instance bound the tab's own `tools` property to itself — an unqualified name inside an inline component finds the component's property first (the wiki's own invariant) — so the split never ran, `partsMatch` stayed false and the first `live:1` scene fell back to the whole-file editor | **confirmed**; the property is `sectionTools`, bound to the window's `tools` |
| F2 | data safety | `assemble()` joins the parts with the open section's text; a byte lost here would be lost on save | rejected: `page_sections`' test asserts the parts concatenate to the page, and the open section replaces its own part alone |
| F3 | data safety | a re-render while a section is open would rebuild the blocks under the writer | rejected (designed): `reload` and the `saved` answer set `renderHeld` while `liveIndex >= 0`; `commitSection` renders when held |
| F4 | correctness | the parts and the blocks lining up: the renderer marks a heading at depth 0 outside quotes, lists and footnotes; the split takes an ATX line outside a fence | accepted: a setext heading or a heading inside a footnote definition would put them off by one, and the fallback edits the whole body; noted in the wiki |
| F5 | correctness | the caret lands on the line the click's height fraction points at, not the exact character | accepted, in the spec's Out: the renderer keeps no position map; the caret is then the editor's |
| F6 | correctness | a click on a link in live preview | rejected: the tap handler checks `hoveredLink` and leaves the link to `onLinkActivated` |
| F7 | correctness | `Escape` in the section editor commits; `Ctrl+E` leaves edit mode after a commit | rejected (read) |
| F8 | correctness | leaving a page (`open`, `jump`) while a section is open | rejected: both save when dirty and set `editing = false`; `liveIndex` stays but `live` is false, and `load` resets `chunks` and `parts`; the next `editSection` sets `liveIndex` afresh |
| F9 | keyboard first | reading → live → source reachable by keys | no finding: `Ctrl+E`, the three palette commands, `Escape`, `Ctrl+S` |
| F10 | theme | the section editor: the source editor's face, `line` for its frame, `accent` for the selection | tokens only |
| F11 | correctness | `scrollToHeading` in live preview | rejected: `sourceVisible` picks the editor branch only in source mode; live preview scrolls the blocks as reading does |
| F12 | prose | the header's `[ LIVE ]`, the tooltips, the commands | read against `no-ai-slop` |
| F13 | correctness | CodeGraph: `page_sections` has the bridge and its test as callers, `highlight_line` its one C++ caller; `HEADING_MARK` unchanged in `render.rs`; nothing outside `markdown.rs`, `tools.*`, `NoteTab.qml` and `Main.qml` moved | the blast radius matches the manifest |

## Phase 4: Validate

- `bin/gate.sh --fast` after implement: `GATE GREEN [fast]` on the first run, the C++
  compiled and `markdown::tests::page_sections_split_at_headings_outside_fences` passing
  (the frontmatter, the preamble, six levels, a fenced `#` kept in its section, `#` alone
  as a heading, the identity, no frontmatter, an open frontmatter).
- `cargo build -p rusty-app -p rusty-mcp` (00:27:09), then `scripts/screenshot.sh
  <scratch> "live:1" "edit"` with `RUSTY_SHOT_DELAY=4500`; the first `live:1` showed the
  whole-file editor (F1); after F1 the app was rebuilt (00:28:40) and both scenes shot
  again. Logs clean of `error|warning|TypeError|ReferenceError|Cannot assign|is not a
  type|Detected anchors|binding loop` both times.
  - `live-1.png`: `[ LIVE ]` in the header; the preamble (its paragraph and callout)
    and "Layout contract" rendered; "North stars" open as a source editor in the
    terminal face with the highlighter's marks and a frame — REQ-002's scene, REQ-005's
    links still rendered around it.
  - `edit.png`: the page in edit mode in the user's default mode, live preview.
- REQ-001 rests on the reading of `editSection` and `openSource` and Chad's click;
  REQ-003 on the one timer and one `save`; REQ-004 on `ui.editMode` and the commands,
  with Chad's restart to come.
- `bin/gate.sh --diff` after the last gated edit (F1): every step ok, receipt written, GATE GREEN [diff]

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-005 satisfied — REQ-001 by the reading of
  `editSection` and `openSource` and the scene (Chad's click to come), REQ-002 by the
  split's test and the scene, REQ-003 by the one timer and one `save`, REQ-004 by
  `ui.editMode` and the three commands (Chad's restart to come), REQ-005 by the reading
  of the tap handler and `onLinkActivated` and the scene's links. None split, none
  waived; the span-level grain is the named seam.
- Wiki: run `dbfe4e05-4fa9-48f1-ac16-8c8a10e6ccdc`, `openwiki_finish` → `complete`; the
  page-tab bullet says three views and describes live preview, a tests bullet added;
  six claims re-anchored or reworded, one added. The PostToolUse hook did not fire
  (twelfth sighting); bulletin 3's recovery with the pair under `active/`, then
  `bin/gate.sh --verify`.
- ROADMAP ticked under M8. `AD-rusty-live-preview-is-a-section-001` in the AAR and the
  register. Brain: timeline entry on `projects/rusty-v3`.
