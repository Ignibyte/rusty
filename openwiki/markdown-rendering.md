---
type: "Reference"
title: "Markdown rendering: Obsidian's flavour to Qt rich text"
openwiki_generated: true
sources:
  - id: openwiki-source-b07f311845b35eb2b2bd8c5b
    resource: repo://crates/rusty-app/cpp/highlighter.cpp
  - id: openwiki-source-d678395ec2ca71c73018a3fd
    resource: repo://crates/rusty-app/qml/NoteTab.qml
  - id: openwiki-source-659b16aac0bb21abcdfa4b6f
    resource: repo://crates/rusty-app/src/markdown.rs
  - id: openwiki-source-b0fccdb632d2710022a80345
    resource: repo://crates/rusty-core/src/brain/render.rs
generated: {by: "claude-code", at: "2026-09-03T13:09:51.830Z"}
---

# Markdown rendering: Obsidian's flavour to Qt rich text

## Purpose

The app shows a page the way Obsidian's reading view does, without a web engine. One
renderer in the core turns Obsidian-flavoured markdown into the HTML subset Qt's rich
text engine understands, and the source editor colours the raw text with a highlighter
whose rules live in Rust.

## Ownership

- `crates/rusty-core/src/brain/render.rs`: `render(body, &Style, &dyn Resolver,
  self_slug)` on pulldown-cmark 0.13 with tables, footnotes, strikethrough, task lists,
  wikilinks and math enabled; `Style` (colours, fonts and the base size, the skin's
  roles the page is painted with, and two switches, `marks` and `code_head`, every
  field with a default so a partial JSON fills the rest); the `Resolver` trait (link targets to
  slugs, page text for embeds, file URLs for images); `Rendered` (html, outline, links,
  unresolved targets, task count, word and character counts).
- `crates/rusty-app/src/markdown.rs`: `highlight_line(line, prev_state)`, the per-line
  tokenizer (frontmatter, fences, comments, headings, lists, tasks, quotes, callout
  heads, tables, inline code, links, URLs, tags, emphasis, strong, strike, highlight,
  math, HTML), returning spans in UTF-16 units and a block state.
- `crates/rusty-app/cpp/highlighter.{h,cpp}`: `MarkdownHighlighter`, a
  `QSyntaxHighlighter` on the editor's `QQuickTextDocument`, mapping span kinds to
  formats from the theme's tokens.

## Runtime flow

1. `BrainManager::render_page` strips the frontmatter (`body_of`), builds a
   `DbResolver` over the vault and the index, and calls `render`.
2. Two pre-passes: `%% comments %%` are stripped outside fenced code, and every
   `> [!kind]` head is rewritten with private-use markers, because pulldown-cmark
   splits `[!kind]` into several text events.
3. The writer walks the events and emits HTML with inline styles: headings with
   Obsidian's size scale and the theme's heading colours (with `marks`, a `#` per
   level in the accent before the text and a rule under a section title), callouts
   and quotes as tables with a coloured bar (with `marks`, the label uppercase and
   small), code blocks as tables with the code background (with `code_head`, a header
   strip naming the fenced language), task boxes in the line colour and the alive
   colour when `marks` is on, tables with
   header cells, footnotes collected after an `<hr>`, task boxes as
   `rusty:task/<n>` links (done items struck through), `==highlights==` as background
   spans, `#tags` as `rusty:tag/<tag>` links, images with `file://` URLs from the
   resolver, page embeds rendered inline to a depth of two and never into themselves.
4. Wikilinks resolve through the resolver: a page becomes `rusty:page/<slug>[#frag]` in
   the link colour; a missing page becomes `rusty:new/<target>` in the unresolved colour.
5. A marker (`<!--h-->`) precedes every top-level heading so the app can split the
   reading view into blocks and scroll the outline to one.
6. The app (`NoteTab.qml`) routes the `rusty:` links: page navigation in the tab, page
   creation, task toggling by index in the raw source, tag search.

## Invariants

- The renderer needs no Qt; every construct has a unit test.
- Colours are parameters: the app sends the theme's tokens as the `style` argument of
  `brain_render`, never the renderer's defaults.
- Code, fenced or inline, is never scanned for links, tags or highlights.
- Task indexes in the HTML count the same items the source-side toggle counts
  (list markers, including inside quotes; never inside fences).

## Failure modes

- Rich text has no stylesheet, so a construct without an inline style falls back to
  the `Text` item's font and colour.
- Anchors inside a page (`#footnote`, `[[page#heading]]`) render but do not scroll yet.

## Extension points

- A new construct: handle its event in `Writer::event`, add a test in `render.rs`,
  and a kind in `markdown.rs` for the editor if it needs colour.
- A new style token: add it to `Style`, to the app's `style()` in `NoteTab.qml`, and to
  the highlighter's format table if the editor shows it.

## Tests

- `cargo test -p rusty-core brain::render` and `brain::links`.
- `cargo test -p rusty-app` for the tokenizer (kinds, states, UTF-16 positions).
- The screenshot script renders a fixture page with every construct.

## Primary sources

- `crates/rusty-core/src/brain/render.rs`, `crates/rusty-core/src/brain/links.rs`
- `crates/rusty-app/src/markdown.rs`, `crates/rusty-app/cpp/highlighter.cpp`
- `crates/rusty-app/qml/NoteTab.qml`
