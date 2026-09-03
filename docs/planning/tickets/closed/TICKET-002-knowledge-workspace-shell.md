---
title: TICKET-002-knowledge-workspace-shell
status: done
ticket_number: 002
type: feature
created: 2026-09-02
closed: 2026-09-02
intake: docs/planning/intake/INTAKE-knowledge-workspace.md
pipeline_spec: docs/planning/pipeline/completed/knowledge-workspace-shell.spec.md
---

# TICKET-002-knowledge-workspace-shell

## Summary

The first slice of the knowledge workspace: the mock's layout inside the app (icon rail,
vault folder tree, document tabs, breadcrumb toolbar, read/edit toggle, right pane), the
Rust markdown renderer that reads Obsidian's flavour, source editing with highlighting,
and link-safe rename in the core. Enough to read and write the vault in Rusty without
Obsidian for the everyday cases.

## Why

Chad wants Obsidian remade inside Rusty and the Obsidian bridge gone; the intake holds the
full inventory. This slice is the frame everything else hangs on: without tabs, a real
explorer, a renderer and an editor, nothing later has a place to live.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The app shall show the vault's real folder tree, nested, with page counts per folder, and create, rename, move and delete on pages and folders; every rename or move shall rewrite links vault-wide in the core. | core unit tests (rewrite of `[[a/b]]`, `[[a/b\|x]]`, `[[a/b#h]]`, embeds); smoke against a scratch vault |
| REQ-002 | WHEN a page is opened from the tree, search, a link or the switcher, the app shall show it in a document tab in the main area; tabs shall stay open across restarts, close with Ctrl+W, cycle with Ctrl+Tab, and one can be pinned. | screenshots (offscreen, scratch vault); keyboard walk |
| REQ-003 | The app shall render Obsidian-flavoured markdown through a Rust renderer: wikilinks with aliases and heading targets, page and image embeds, callouts, tasks, tables, footnotes, highlights, comments hidden, fenced code. | renderer unit tests per construct; screenshot review |
| REQ-004 | WHEN the read/edit toggle (Ctrl+E) is used, the app shall edit the page's whole source with markdown highlighting, autosave after a pause and on Ctrl+S, and write untouched text (frontmatter, timeline) back byte for byte. | core round-trip test; smoke |
| REQ-005 | The app shall show for the open page: backlinks with a context line, outgoing links, unresolved links (a click creates the page), and an outline of headings. | index unit tests; screenshot |
| REQ-006 | WHEN Ctrl+O is pressed, the app shall open a quick switcher that fuzzy-matches titles and paths and offers to create a page for a name with no match; WHEN Ctrl+K is pressed, the app shall open a command palette listing every command with its key. | smoke; screenshot |
| REQ-007 | The app shall take every colour from the Omarchy theme (its `obsidian.css` tokens and Alacritty palette), lay the workspace out as Obsidian does (ribbon, sidebars, tabs, inline title, properties, status bar) with Obsidian's proportional typography, and hold the layout at 1280 px wide. | screenshots in two themes and two widths |
| REQ-008 | The app shall open an agent terminal (Claude Code, Codex, or any agent found on PATH) as a tab in the main area from the ribbon, and the right sidebar shall offer an Agent pane that shows a terminal beside the open page. | screenshot |
| REQ-009 | The Brain and Notes tabs shall be replaced by the workspace without losing capture, daily pages, search (full text and semantic) or the timeline append. | smoke of each action |

## Scope

- In: the frame and the everyday read/write path (this ticket). Tiers after it: tags and
  properties (003), graph views (004), search operators and bookmarks (005), retirement of
  the Obsidian bridge (006).
- Out: live preview, split panes, canvas, bases, math, mermaid, attachments (later or never
  per the intake).

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/knowledge-workspace-shell.spec.md`
- Related docs: `docs/design/rusty-omarchy.html`, `docs/architecture.md`
- Promoted from intake: `INTAKE-knowledge-workspace`
- Follow-ups opened: TICKET-003 (tags and properties), TICKET-004 (graph views), TICKET-005 (search operators, bookmarks, hotkeys), TICKET-006 (retire the Obsidian bridge), as the roadmap lists them
