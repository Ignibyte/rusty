---
title: First-class tagging — notes
pipeline: dd540fb5-d193-4833-99fe-842d6f6b55bc
ticket: TICKET-024
---

# First-class tagging: notes

## Recall (2026-09-05)

- Bulletins: none touch tags; bulletin 2 (no synthetic input on Chad's desktop) shapes
  validation: the scenes run offscreen against the scratch vault.
- Register: `AD-rusty-tags-one-index-001` — frontmatter and inline `#tags` share
  `brain_tags`, stored as first written and compared without case; `tag:` is part of
  `brain_search`; a property edit touches only the frontmatter mapping. Every decision
  here rests on it.
- TICKET-005's locked decisions 1 to 5 stand: one index, `tag:` inside search, a property
  edit rewrites the mapping only, the properties block is the editor, the Tags pane is a
  right-sidebar tab that searches on click.
- Wiki: `workspace-app.md` describes the properties block ("tag chips also open a `tag:`
  search") and the Tags pane; `vault-and-brain.md` the index and `tags()`.
- Brain: `brain_search "rusty tags"` ranked `projects/rusty-v3` and the Hermes/Obsidian
  research page; nothing on tagging beyond what the ticket says.
- Code read:
  - `NoteTab.qml`: `setProperty`/`removeProperty` call the property tools; `addProperty`
    maps the type list (`Text, List, Number, Checkbox, Date`) to a JSON value; a list
    renders as chips (`PropertyRow`), each with ×, and a `chipAdd` field whose Enter pushes
    the text; a `tags` chip opens a `tag:` search. The page re-renders on the change
    notification, so `properties` is fresh after a write.
  - `RightPane.qml`: `tags` (the window's `brain_tags` answer) → `tagRows` (tree rows with
    depth and counts); the Tags pane's rows have a `TapHandler` that searches and nothing
    else; `pane.note` is the open note (title, links, outline, `scrollToHeading`).
  - `Main.qml`: `win.tags` is re-asked on the change notification (`ask("brain_tags")`),
    passed to the right pane; the palette has "Tags: Show tags"; the `NoteTab` instance
    binds `backend`, `theme`, `isCurrent` and the signals; scenes are one chain.
  - `rusty-core`: `set_property` → `write_edited` re-indexes (the test
    `tags_index_search_and_properties` already asserts `tags()` holds a tag set through the
    property); `tags()` folds nested tags under their parents; `links::tags` scans inline.

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-app/qml/NoteTab.qml` | `tags`, `tagAdd`, `pendingTagFocus`, `pendingTagText`; `propertyValue`, `tagRowReady`, `tagCompletions`, `tagThePage`, `focusTagAdd`; `Tags` in the type list and in `addProperty`; the tags row's field completes (a `Popup` under it, Up/Down/Enter/Tab/Escape); `PropertyRow` registers its field |
| `crates/rusty-app/qml/RightPane.qml` | `signal tagPage`; the Tags list keyboard-navigable (Enter searches, `T` tags); a `+` on hover, a right-click menu |
| `crates/rusty-app/qml/Main.qml` | `tags: win.tags` on the note; `onTagPage`; the palette command; the `tagfield:` scene |
| `crates/rusty-core/src/brain/mod.rs` | `tags_index_search_and_properties` extended: `tag:` search after the property write, and the count and search after a tag is removed |

### The completion list

`tagCompletions(q)`: `q` trimmed, a leading `#` dropped, lowered; the page's own tags
(from `propertyValue("tags")`) lowered; `tags.filter(t => t.tag includes q && not own)`
sorted by count descending then name, the first eight. An empty `q` lists the most used.
The field: `refresh()` on text and on focus opens the popup when there is something to
show, `pick` starts at 0; Down/Up move within bounds; Enter takes the pick when the
popup is open, else the text; Tab takes the pick and is otherwise left to focus
navigation; Escape closes the popup first; a click on a row adds it. The popup has no
focus of its own (`focus: false`), so the field keeps the keyboard.

### The writer

`tagThePage(tag)`: trim, drop `#`, refuse empty; the current list; refuse a duplicate
without case; push; `setProperty("tags", list)`. `focusTagAdd(prefill)`: leave edit mode;
when the page has no `tags`, remember the wish (`pendingTagFocus`, `pendingTagText`) and
add the property, and the row's `tagRowReady` focuses the new field when the re-render
creates it; else focus the field and set the text.

### The pane

`tagPage(tag)` → the window → `currentNote.tagThePage(tag)`. The list gets
`keyNavigationEnabled`, a current row, Enter and `T`; the delegate a `+` (a `Text`, no
new icon) visible on hover when a note is open, and a right-button `TapHandler` that pops
`tagMenu` (Tag the open page — enabled with a note open; Search #tag).

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | `tags_index_search_and_properties` (the `tags` list through `set_property`, the file and the counts); reading of `addProperty`; the `reading` scene |
| REQ-002 | scene `tagfield:r` (the popup with the vault's tags holding `r`, ranked by count); reading of `tagCompletions` |
| REQ-003 | the `reading` scene (chips with ×); the chip's remove path is TICKET-005's, unchanged |
| REQ-004 | reading of `tagPage` → `tagThePage` → `setProperty`, and of `ask("brain_tags")` on the change notification; Chad's smoke |
| REQ-005 | the extended core test: `tag:next` found after the property write; after the tag is removed through the property the count is gone and the search is empty |

### Risks

- **Data safety.** Only the frontmatter mapping is rewritten (TICKET-005's `set_property`);
  the tests run in a temporary vault; the scenes against the scratch vault.
- **A stale field reference.** `tagAdd` is a delegate's item; the row's `onDestruction`
  clears it, so a focus after a re-render waits for `tagRowReady`.
- **Theme.** `tag`, `hover`, `active`, `panel3`, `line`, `faint`: existing tokens.
- **Keyboard.** The palette command, the field's keys, the pane's Enter and `T`.
- **Tags typed with `#`.** `tagThePage` and the field both drop a leading `#`; the index
  compares without case, so `Rust` and `rust` are one tag and the duplicate check agrees.

### CodeGraph

`codegraph_explore` over `set_property`, `sync_tags`, `tags`: the property write path is
`BrainManager::set_property → frontmatter::set_property → write_edited`, with
`sync_tags` on the create/update path; the app calls `brain_set_property` from
`NoteTab.qml` alone. A second pass after implementation goes in the ledger.

## Phase 3: Implement

Four files, as the manifest said. `NoteTab.qml`: `tags`, `tagAdd`, the pending focus,
`propertyValue`, `tagList`, `ownTags`, `tagRowReady`, `tagCompletions`, `tagThePage`,
`focusTagAdd`; `Tags` in the type list (the key field reads `tags` and is disabled while
it is chosen), `addProperty` routing it to `focusTagAdd`; the tags row's field with its
completions (`refresh`, `add`, the keys, the `Popup`); `PropertyRow` registering and
clearing the field. `RightPane.qml`: `tagPage`, `focusTags`, the keyboard list, the `+`,
the right-click menu. `Main.qml`: `tags: win.tags`, `onTagPage`, "Tags: Tag this page",
"Tags: Show tags" focusing the list, the `tagfield:` scene. `rusty-core`: the test.
`qmllint` exit 0 on the three QML files.

Deviations: the pick starts at `-1`, not `0` (F3 below); a scalar `tags:` counts as one
tag (F1).

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | data safety | a page with a scalar `tags: x` (a lenient page) would have had it replaced by `[new]` in `tagThePage`, since `listOf` reads only sequences | **confirmed**; `tagList` counts a scalar as one tag, so the write keeps it |
| F2 | correctness | a click on a completion row under `CloseOnPressOutsideParent` | rejected: the policy closes on a press outside the popup that is also outside its parent; a press inside the popup reaches the row's `TapHandler` |
| F3 | keyboard first | the first completion was picked on every refresh, so Enter after typing a new tag in full (`rusty` beside `rust-lang`) would have taken the completion | **confirmed**; nothing is picked until Down, Enter adds the text as typed, Tab takes the pick or the first |
| F4 | data safety | `focusTagAdd` on a page without `tags` writes `tags: []` before anything is typed | accepted: it is "add property" with a fixed key, which writes the same way (TICKET-005); an empty list is harmless to the index |
| F5 | correctness | the popup closing when focus leaves the field, before a click on a row lands | rejected: a `TapHandler` on a `Rectangle` takes no focus; the field keeps it |
| F6 | theme | `panel3`, `line`, `hover`, `active`, `tag`, `faint`, `foreground` | tokens only |
| F7 | keyboard first | the Tags pane's list had `keyNavigationEnabled` and keys but nothing ever gave it focus | **confirmed**; a click on a row focuses the list, and "Tags: Show tags" calls `focusTags` |
| F8 | correctness | the list's default `currentIndex` of 0 painted the first row as chosen (seen in the first `right:tags` scene) | **confirmed**; `currentIndex: -1` until a click or a key |
| F9 | correctness | `tagCompletions` reads `win.tags` as `[{tag, count}]` | rejected: the same array the pane reads (`t.tag`, `t.count`) |
| F10 | correctness | Tab with `event.accepted = false` | rejected: the event falls through to focus navigation when the list is closed |
| F11 | complexity | the key field keeps `tags` after the type is switched away from Tags | accepted: the field is enabled again and editable; a placeholder swap is not worth a handler |
| F12 | prose | "Tag the open page", "Search #x", the tooltip, the comments | read against `no-ai-slop` |
| F13 | correctness | CodeGraph: `brain_set_property` → `BrainManager::set_property` → `frontmatter::set_property` → `write_edited`; the only app call site is `NoteTab.setProperty`, which `tagThePage` and the chip field share | the blast radius matches the manifest; no core code changed, one test grew |

## Phase 4: Validate

- `bin/gate.sh --fast` after implement: `GATE GREEN [fast]` on the first run;
  `brain::tests::tags_index_search_and_properties` passed with its new assertions
  (`tag:next` found after the property write, gone with the count after the tag is
  removed through the property).
- `cargo build -p rusty-app -p rusty-mcp` (23:21:15), then `scripts/screenshot.sh
  <scratch> "tagfield:r" "right:tags"` with `RUSTY_SHOT_DELAY=4500`, offscreen against the
  scratch vault; after F1, F3, F7 and F8 the app was rebuilt and both scenes shot again.
  Logs clean of `error|warning|TypeError|ReferenceError|Cannot assign|is not a
  type|Detected anchors|binding loop` both times.
  - `tagfield-r.png`: Orbit's properties, the tags row with `launcher` and `rust` as chips
    and the field holding `r`, the list under it with `#person 1` and
    `#person/engineering 1` — the vault's tags holding `r` with the page's own `rust` and
    `launcher` left out — REQ-002's scene; the chips carry their × — REQ-003.
  - `right-tags.png`: the Tags pane with its nine rows and counts; after F8 no row is
    painted as chosen until one is picked.
- REQ-001 rests on the core test (the `tags` list through `set_property`) and the reading
  of `addProperty`; REQ-004 on the reading of `tagPage` → `tagThePage` → `setProperty`
  and the `brain_tags` re-read on the change notification; REQ-005 on the test.
- `bin/gate.sh --diff` after the last gated edit (the F1/F3/F7/F8 fixes): fmt, clippy,
  test, doc, shell-syntax, secrets, whitespace all ok, `receipt written:
  .git/rusty-gate-receipt`, `GATE GREEN [diff]`. The second shoot of the two scenes ran
  after the gate (a `&&` chain had stopped at the build step the first time — the
  scenes are not part of the gate); its logs were clean.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-005 satisfied — REQ-001 by the core test and the
  reading of `addProperty`, REQ-002 by the `tagfield:r` scene and the reading of
  `tagCompletions`, REQ-003 by the scene's chips (TICKET-005's remove path, unchanged),
  REQ-004 by the readings of `tagPage` → `tagThePage` and the `brain_tags` re-read (Chad's
  smoke to come), REQ-005 by the test's new assertions. None split, none waived.
- Wiki: run `0151396f-14ee-4a0d-9d60-76451485d8dd`, `openwiki_finish` → `complete`; the
  page-tab bullet gained the deliberate path, a tests bullet added; three claims
  re-anchored and rewritten (the Tags pane, the properties block, the scenes), one added.
  The PostToolUse hook did not fire (eighth sighting); bulletin 3's recovery with the
  pair under `active/`, then `bin/gate.sh --verify`.
- ROADMAP ticked under M8. `AD-rusty-tag-one-writer-001` in the AAR and the register.
  Brain: timeline entry on `projects/rusty-v3`.
