---
title: Notes into the vault
pipeline_id: 01151524-3b0d-40a5-abd2-8f53a3bff0e9
status: Phase 5 — Complete PASS
ticket: TICKET-014
ticket_doc: docs/planning/tickets/open/TICKET-014-notes-into-the-vault.md
aar: docs/planning/knowledge/aar/AAR-014-notes-into-the-vault.md
sealed: Chad, 2026-09-03 16:00, "lets start working on these" (relayed by the rustal session, covering tickets 011 to 018); the store change here (the notes folder folds into the vault, the default path moves) is reported to Chad at delivery, and the one-shot that moves his files is his to run
created: 2026-09-03
---

# Notes into the vault: spec

## Intent

The notes folder folds into the vault. The files under `~/.rusty/notes` move to `notes/`
inside the vault by a one-shot command, the notes tools point there by default, and the
explorer, search, links, graph and semantic index cover them like any page. Chad, at
15:40: "i dont see anywhere my files in ~./rusty/notes are?"

## Scope

- In: `rusty-cli notes adopt` (a move with a report, refusing on a clash, leaving a README
  behind), the `notes_path` default when unset, the notes tools' path resolution (unchanged
  code, a new default), the vault index picking up `notes/` pages as type `note`, the
  `/note` skill's fallback path in the store, docs and the installer's line.
- Out (named seams, not forgotten): merging the notes tools into the brain tools (a later
  cleanup); any Obsidian concern; moving Chad's real notes (his one-shot to run).

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `rusty-cli notes adopt` runs, it shall move every file under the notes path into `<vault>/notes/`, keeping names and folders, refusing on a name clash, and reporting what moved. | tests on a temporary vault |
| REQ-002 | WHEN `notes_path` is unset, the notes tools shall use `<vault>/notes`. | unit on the setting's default |
| REQ-003 | WHEN a note is created or appended through the notes tools or the `/note` skill, the vault index shall pick it up as a page of type `note`. | tool test; index test |
| REQ-004 | WHEN the adoption has run, the old folder shall be left with a README naming the new place, and nothing shall be deleted. | test |
| REQ-005 | WHEN the explorer shows the vault, `notes/` shall appear as an ordinary folder with no special casing. | screenshot |
| REQ-006 | WHEN the adoption ships, the README and the installer's output shall name the one-shot command. | doc review |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The notes move into the vault as files; the notes tools keep their names and shape and only their default root changes. | The vault already treats a file without frontmatter as a page typed by its top folder; nothing else in the app knows two stores. | A second explorer root for the notes folder (visible, still outside search, links and the index). |
| 2 | The adoption is a CLI one-shot that refuses clashes and deletes nothing; the old folder keeps a README. | Files are the truth (§10); a move a person runs once is reviewable, a background migration is not. | Moving at service start (silent, unreviewable). |
| 3 | An explicit `notes_path` setting keeps winning. | A user who put notes elsewhere on purpose loses nothing. | Forcing the vault path. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-014-notes-into-the-vault.md`
- Intake: none
- Design references: `crates/rusty-core/src/notes/mod.rs`, `crates/rusty-core/src/core.rs`
  (the path defaults), the notes tools in `crates/rusty-mcp/src/main.rs`, the vault's
  lenient page rule (`AD-rusty-lenient-pages-001`), the `/note` skill in the store
- Architecture: `docs/architecture.md` (files are the truth; vault rules since the workspace)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
