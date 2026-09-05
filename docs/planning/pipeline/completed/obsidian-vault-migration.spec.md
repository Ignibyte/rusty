---
title: Obsidian vault migration
pipeline_id: bd3dd522-2b7b-4d64-ab22-2c7148c62a14
status: Phase 5 — Complete PASS
ticket: TICKET-026
ticket_doc: docs/planning/tickets/open/TICKET-026-obsidian-vault-migration.md
aar: docs/planning/knowledge/aar/AAR-026-obsidian-vault-migration.md
sealed: two tools (82), one CLI subcommand, one dialog; no new table, dependency or tab
created: 2026-09-05
---

# Obsidian vault migration: spec

## Intent

The brain is an Obsidian vault by format; what is missing is the front door. Someone
with years of notes points Rusty at the vault, reads what will happen, imports it, and
keeps what Obsidian's own files hold. The source is read and never written.

## Scope

- In: a plan (`brain_import_plan`, `rusty-cli brain import --dry-run`, the app's review
  dialog) naming pages, folders, attachments, tags, collisions, unresolved links and the
  bookmarks it would carry; the import (`brain_import`, `rusty-cli brain import`, the
  dialog's Import): pages and attachments copied at their own paths, bare-name links
  rewritten to vault paths as `brain migrate` does, a report page under `inbox/`, the
  index rebuilt, one git commit; `.obsidian/bookmarks.json` read into Rusty bookmarks
  (the app merges them into its state); a rollback that removes what the run created
  when a write fails; a seeded vault and an `import:` scene in the screenshot script.
- Out (named seams, not forgotten): two-way sync; watching a foreign vault; plugin
  settings and community-plugin data; canvas files; Obsidian's daily-notes folder
  mapped onto `daily/`; a `tags:` string with commas read as several tags; renaming a
  colliding page during the import (it is skipped and named, the decision is the
  user's).

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN an Obsidian vault folder is chosen, the app shall report what it found — pages, folders, attachments, tags, unresolved links — and what it will do, before changing anything. | `import_plan_reports_what_it_found`; the `import:` scene; smoke on a real vault by Chad |
| REQ-002 | WHEN an import runs, pages, folders and attachments shall land in the brain and be indexed for full text, tags, links and the graph. | `import_vault_brings_pages_attachments_and_a_report` (search, tags, links after the run) |
| REQ-003 | WHEN a page's slug collides with one already in the brain, the import shall neither overwrite nor silently rename, and shall report the collision for a decision. | `import_plan_reports_what_it_found` (the collision listed, the page left as it was) |
| REQ-004 | WHEN `.obsidian/bookmarks.json` is present, its bookmarks shall come across as Rusty bookmarks. | `parse_bookmarks_flattens_groups_and_maps_kinds`; the plan test (bookmarks kept and skipped); reading of the app's merge |
| REQ-005 | WHEN an import finishes, it shall write a report naming what came in, what was skipped and why. | the import test (the `inbox/` page and its sections); reading |
| REQ-006 | WHEN an import fails part way, the vault it read from shall be unchanged, and the brain shall be left consistent. | `import_vault_rolls_back_when_a_write_fails`; the import test's source snapshot |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | Pages keep their paths: a page's slug is its path in the source without `.md`, folders as they were; nothing is sorted into type folders | The vault rules allow a page in any folder and a page without frontmatter; Obsidian's own links name those paths; a re-sort would be guessing | mapping folders onto `TYPE_DIRS`; renaming to slugs |
| 2 | A collision (a slug or an attachment path already in the brain) is skipped and named in the plan and the report; the import never overwrites and never renames | The ticket's word; `notes::adopt` refuses clashes the same way | a `-2` suffix; a prompt per page |
| 3 | Links are rewritten with `migrate_vault`'s `LinkIndex` built over the brain's pages and the incoming ones together, body only, frontmatter byte for byte; what does not resolve is reported | One resolver for the vault (`AD-rusty-vault-rules-001`); the plan can say what will not resolve before anything is written | leaving bare names for the renderer to resolve at read time |
| 4 | The plan and the import are the same read; the import writes pages first, attachments next, the report page last, records every path it creates, and on any error removes them, rebuilds the index and answers with the error | REQ-006 without a transaction: the source is never written, and the brain is either the import whole or as it was | a staging folder; a git reset |
| 5 | Bookmarks are parsed in core (groups flattened; file, folder, search and heading kinds; url and graph skipped) and travel in the plan; the app merges the ones whose target exists into `ui.bookmarks`, the CLI lists them | Bookmarks live in the app's state (`AD-rusty-bookmarks-in-state-001`), so core reports and the app writes; nothing new is stored | writing the app's state file from core |
| 6 | The report is a page under `inbox/` (`import-<date>-<name>`), written by the import itself, so it is searchable, in the tree and in the commit | A file the user can find where the rest lives; the CLI and the tool both leave it | a file under `~/.rusty` |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-026-obsidian-vault-migration.md`
- Register: `AD-rusty-vault-rules-001`, `AD-rusty-bridge-retired-whole-001`,
  `AD-rusty-bookmarks-in-state-001`; `notes::adopt` (TICKET-014) for the tone
- Architecture: `openwiki/vault-and-brain.md` (the vault, migration), `openwiki/mcp-back-end.md`
  (the tools), `openwiki/workspace-app.md` (the dialog, bookmarks)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled |
| 2 Design | Manifest, the plan, the rollback, the bookmarks, regression table | design actionable |
| 3 Implement | `brain/import.rs`, `brain/mod.rs`, `rusty-mcp`, `rusty-cli`, `Main.qml`, the script, the counts | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger; CodeGraph over the import path | confirmed findings resolved |
| 4 Validate | The tests, the scene, `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, AAR, register, brain, archive | pair archived |
