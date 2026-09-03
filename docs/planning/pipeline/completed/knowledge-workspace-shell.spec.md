---
title: Knowledge workspace shell
pipeline_id: 6b0c5d3e-2c0f-4b7e-9f0a-1d2e3f4a5b6c
status: Phase 5 — Complete PASS
ticket: TICKET-002
ticket_doc: docs/planning/tickets/closed/TICKET-002-knowledge-workspace-shell.md
aar: docs/planning/knowledge/aar/AAR-002-knowledge-workspace-shell.md
sealed: 2026-09-02, Chad: "lets work ticket 2 through 6 auto approved until finished. make sure we refer to the html file, the screen shots. I want near identical to obsidian except that we have the shell built in and an MCP in which the agent can interact with"
created: 2026-09-02
---

# Knowledge workspace shell: spec

## Intent

Make Rusty's app the place the vault is read and written, in the layout of
`docs/design/rusty-omarchy.html`, so Obsidian is no longer needed for the everyday path.
This pipeline delivers the frame and the read/write path; the intake lists the tiers that
follow. Chad, 2026-09-02: "remake obsidian inside our system like we did before … the html
file is a design from replit i really like."

## Scope

- In: the rail, the explorer over the real folder tree with file operations and link-safe
  rename in the core, document tabs, the breadcrumb toolbar with read/edit, the Rust
  markdown renderer for Obsidian's flavour, the source editor with highlighting and
  Ctrl+S, the backlinks/outgoing/unresolved/outline panes, the quick switcher and the
  command palette, the right pane with an agent terminal, theme tokens throughout,
  replacement of the Brain and Notes tabs.
- Out (named seams): tags and properties (TICKET-003), global and local graph
  (TICKET-004), search operators and bookmarks (TICKET-005), Obsidian bridge retirement
  (TICKET-006); live preview, split panes, canvas, bases, math, mermaid, attachments.

## Acceptance criteria (EARS)

REQ-001 to REQ-009 as in the ticket.

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | Markdown is rendered by a Rust module in `rusty-core` (`brain::render`) from Obsidian's flavour to the HTML subset Qt's rich text understands; QML shows it with `Text.RichText`. | One renderer serves the app, the CLI and any future export; unit-testable without Qt; no Chromium in the app. | QtWebEngine (heavy, a second engine to theme); Qt's built-in `MarkdownText` (no callouts, embeds, footnotes, wikilinks) |
| 2 | The editor is a QML `TextArea` with a Rust `QSyntaxHighlighter` subclass through cxx-qt; two modes, source and read, toggled with Ctrl+E. | Live preview is a large editor project; source plus read covers the everyday path and matches the mock's `[ READ ]` toggle. | live preview now; an embedded web editor |
| 3 | The explorer shows the vault's real folder tree from the filesystem; the core's vault manager learns nested folders and file operations, and rename rewrites links in every page (the Obsidian bridge's `move` is no longer the only way). | Obsidian users think in folders; the type folders remain the top level; migration keeps every file where it is. | keeping the type tree only |
| 4 | Pages open in document tabs inside the workspace; agent terminals stay top-level tabs in the rail and can also be shown as the right pane of the workspace. | The mock's "AI is a pane in the workspace" with what Rusty already has (tmux-backed terminals). | a chat pane with a model call |
| 5 | Colours come only from the Omarchy theme tokens; the mock's structure and typography are adopted, its amber palette is not. | Constitution §10. | shipping the mock's palette as a theme |
| 6 | The Brain and Notes tabs are replaced by the workspace; their actions (capture, daily, search, timeline append) move into it. | One place for the vault. | keeping both |

## Amendments at seal (2026-09-02)

Chad's seal asks for Obsidian, near identical, plus the built-in shell and the MCP. That
sharpens three of the locked decisions and adds two rules:

- Decision 4 becomes: everything is a tab in the main area, as in Obsidian. Pages, the
  graph, Tasks, Memory, Skills, Secrets, Settings and agent terminals all open as tabs;
  the right sidebar holds Backlinks, Outgoing links, Outline and an Agent pane that shows
  a terminal beside the note (the mock's right pane). The old rail of pages goes.
- Decision 5 becomes: the layout, spacing and typography are Obsidian's (proportional UI
  font, mono for code and the source editor, inline title, properties block, status bar
  counts, ribbon and sidebars); the mock keeps two contributions, the agent as a pane and
  the command layer. Colours still come only from the Omarchy theme: its `obsidian.css`
  tokens and its Alacritty palette.
- Keys follow Obsidian's defaults (Ctrl+P palette, Ctrl+O switcher, Ctrl+E reading
  toggle, Ctrl+N new note, Ctrl+W close, Ctrl+Shift+F search, Ctrl+, settings) and are
  suspended while a terminal has focus, because Claude Code and the shell use the same
  keys.
- Rule: a vault file without frontmatter is a page too (title from the file name, type
  from its top folder or `note`), so anything Obsidian wrote shows up and opens.
- Rule: the source editor edits the whole file, frontmatter and timeline included, and
  autosaves, as Obsidian does; REQ-004's "byte for byte" means untouched text is written
  back unchanged.

## Linked artifacts

- Ticket: TICKET-002
- Intake: `docs/planning/intake/INTAKE-knowledge-workspace.md` (the full Obsidian inventory)
- Design references: `docs/design/rusty-omarchy.html`
- Architecture: `docs/architecture.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | this spec, the ticket, the intake, the AAR opened | Chad seals this spec (it changes what Rusty is) |
| 2 Design | architecture of `brain::render`, `brain::vault` file operations and link rewrite, the QML workspace files, the highlighter bridge; file manifest; regression table; CodeGraph over `brain`, `vault`, `frontmatter`, the QML pages | design actionable |
| 3 Implement | the manifest, in three steps: core (renderer, vault ops, rewrite), app (workspace, tabs, editor), replacement of the old tabs | `bin/gate.sh --fast` green |
| 3.5 Inspect | ledger through the lenses; post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | renderer and vault tests, scratch-vault smoke, screenshots in two themes, `bin/gate.sh --diff` and receipt | receipt matches |
| 5 Complete | requirement audit, docs, AAR, register, brain capture, archive | pair archived |
