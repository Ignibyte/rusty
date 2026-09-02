---
title: INTAKE-knowledge-workspace
status: promoted
created: 2026-09-02
ticket: TICKET-002
---

# INTAKE-knowledge-workspace

## Problem or opportunity

Chad, 2026-09-02: "I want to skip the obsidian api altogether and basically remake obsidian
inside our system like we did before. lets leave the notes in there for now and then
migrate. … the html file is a design from replit i really like. We should try to get a full
feature list of obsidian stuff and look to start building it."

Today the brain is an Obsidian vault Rusty indexes, and the app reaches Obsidian's own
features (graph, link-safe rename, backlinks) through Obsidian's CLI. That keeps a second
program in the loop for the thing Rusty is for. The knowledge tabs in the app (Brain, Notes)
already render pages, search, follow links and capture; they stop short of an editor, a
graph, tags, properties, and the rest of what makes Obsidian a workspace.

## Proposed outcome

Rusty's app is the knowledge workspace: the vault is browsed, read, edited, linked, graphed
and searched inside it, in the layout of the Replit mock (`docs/design/rusty-omarchy.html`)
themed by Omarchy. Obsidian stays usable on the same folder while the workspace catches up;
when the must-have tier below is delivered, the Obsidian bridge is retired. No data moves:
the vault folder is already Rusty's, so "migration" is a change of tool, not of files.

## Design reference

`docs/design/rusty-omarchy.html`. What it fixes: a slim top bar (brand, workspaces, status,
clock), a narrow icon rail (files, find, graph, canvas, daily, setup), a vault tree with
counts and a footer ("indexed 42 notes · 3 links unresolved"), a main area with document
tabs, a breadcrumb toolbar with a read/edit toggle, the note (title, callouts, sections,
task lists, code blocks, backlinks), a local-graph overlay, a right pane for the assistant
with context loaded, and a command layer (Ctrl+K). Typography is monospace with
uppercase micro-labels. The palette in the mock (amber on black) is one theme; the app
takes colours from the current Omarchy theme.

## Obsidian's features, sorted by what Rusty needs

Legend: **have** (in the app now), **core** (rusty-core has it, no UI), **build**, **later**,
**never**.

### Workspace and navigation

| Feature | State |
|---|---|
| File explorer: folders and files, create, rename, move, delete, collapse, sort, reveal | build (Brain tab has a type tree; the vault's real folder tree, nested, with actions) |
| Document tabs, pinned tabs, close others | build |
| Split panes (vertical, horizontal), linked panes | later |
| Quick switcher (fuzzy open by name, create when missing) | build |
| Command palette | build |
| Bookmarks (files, folders, searches, headings) | later |
| Workspaces (saved layouts) | later |
| Hover preview of a link | build |
| Random note | later |
| Vault switcher | never (one vault: the brain) |

### Editing and reading

| Feature | State |
|---|---|
| Source editor with markdown highlighting | build (QML `TextArea` plus a Rust highlighter) |
| Reading view (rendered) | have, partial (Qt markdown; no callouts, embeds, footnotes, math) |
| Live preview (inline rendering while typing) | later |
| Wikilinks `[[page]]`, aliases `[[page\|text]]`, heading links `[[page#h]]` | have (render and navigate) / build (autocomplete while typing) |
| Block references `[[page^id]]` | later |
| Embeds `![[page]]`, `![[image.png]]` | build |
| Callouts `> [!note]` | build (Rust renderer) |
| Tags, inline `#tag` and frontmatter, tags pane with counts | build (core indexes frontmatter tags; inline tags need parsing) |
| Properties (typed frontmatter editor) | build |
| Tasks `- [ ]`, tables, footnotes, highlights `==x==`, comments `%% %%`, code blocks | build (renderer + highlighter) |
| Math (MathJax), Mermaid | later (optional external renderer) |
| Slash commands, autocomplete for links and tags | build |
| Paste or drop an image into an attachments folder | later |
| Find and replace in a note | build |
| Note composer (extract selection to a new note, merge) | later |
| Word count | build (trivial) |
| Templates with `{{date}}` and `{{title}}`, template folder | later (core has per-type templates) |
| Daily notes (open today, folder, template) | have |
| Unique note (Zettelkasten prefix) | never |
| File recovery (snapshots) | core (`brain_versions` table; no UI) |

### Links and structure

| Feature | State |
|---|---|
| Backlinks pane (linked mentions) | have (page view) / build (a pane with context lines) |
| Unlinked mentions | later |
| Outgoing links pane, unresolved links, create a page from an unresolved link | build |
| Rename or move with every link rewritten | build in core (today only through Obsidian's move) |
| Outline pane (headings) | build |
| Global graph: nodes and edges, filters (search, tags, orphans), groups by query with colours, display (arrows, node size, link thickness, text fade), forces | build (Rust layout, QML canvas) |
| Local graph with depth | build |

### Search

| Feature | State |
|---|---|
| Full-text search with context | have (FTS, plus semantic) |
| Operators (`path:`, `file:`, `tag:`, `line:`, `section:`, regex, case) | build (a subset: path, tag, type, regex) |
| Search and replace across the vault | later |

### Beyond notes

| Feature | State |
|---|---|
| Canvas (infinite board of cards, files, groups, edges) | later |
| Bases (table views over properties with filters and formulas) | later |
| Web viewer, slides, audio recorder | never |
| Sync, Publish | never (git and the box's backups) |
| Community plugins | never (Rusty's skills and MCP tools are the extension surface) |
| Themes and CSS snippets | never as such (the Omarchy theme is the theme) |
| Hotkeys, customisable | build (a table in Settings; defaults first) |

### What Rusty has that Obsidian does not

The agents in the same window (Claude Code, Codex as tabs or a pane beside the note), the
timeline per page with capture from anywhere, memories, tasks, skills, semantic search with
a provider setting, and the MCP surface every tool and agent shares.

## Candidate requirements (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The app shall show the vault's real folder tree, nested, with page counts and create, rename, move and delete on files and folders, every rename rewriting links vault-wide. | unit tests in core; smoke against a scratch vault |
| REQ-002 | WHEN a page is opened, the app shall show it in a document tab; several tabs shall stay open and one can be pinned. | screenshot review |
| REQ-003 | The app shall render Obsidian-flavoured markdown: wikilinks with aliases and headings, embeds of pages and images, callouts, tasks, tables, footnotes, highlights, code blocks. | unit tests on the renderer; screenshot review |
| REQ-004 | WHEN the read/edit toggle is used, the app shall edit the page's source with markdown highlighting and save with Ctrl+S, keeping frontmatter and timeline intact. | core round-trip tests; smoke |
| REQ-005 | The app shall show backlinks with context, outgoing links, unresolved links (a click creates the page), and an outline of headings for the open page. | unit tests on the index; screenshot review |
| REQ-006 | The app shall index inline and frontmatter tags and show a tags pane with counts; a click searches by tag. | unit tests; screenshot |
| REQ-007 | The app shall show a properties editor for the page's frontmatter with typed values. | screenshot; round-trip test |
| REQ-008 | The app shall draw a global graph of pages and links with filters, groups by type, and forces, and a local graph around the open page; a node click opens the page. | screenshot; layout unit test |
| REQ-009 | WHEN Ctrl+O is pressed, the app shall open a quick switcher that fuzzy-matches page titles and paths and creates a page for a name with no match; WHEN Ctrl+K is pressed, a command palette shall list every command with its key. | smoke; screenshot |
| REQ-010 | The app shall use the Omarchy theme's tokens for every colour and read a monospace micro-label style from the mock, in dark and light themes. | screenshot in two themes |
| REQ-011 | WHILE the workspace is in use, an agent terminal shall be openable beside the note as a right pane. | screenshot |
| REQ-012 | WHEN the must-have tier is delivered and Chad says so, the Obsidian bridge tools shall be removed and the docs updated. | doc review |

## Scope notes

- In: everything marked build, in tiers; the renderer and the editor in Rust and QML; the
  vault folder tree as the truth for the explorer.
- Out: everything marked later or never; QtWebEngine as a renderer (heavy; the Rust
  renderer to Qt rich text is the path); live preview in the first tiers.

## Promotion

- [x] Ticket created: `docs/planning/tickets/open/TICKET-002-knowledge-workspace-shell.md`
- [x] Spec/notes pair: `docs/planning/pipeline/active/knowledge-workspace-shell.{spec,notes}.md`
- [x] `ticket:` set; `status:` promoted.
