---
title: TICKET-014-notes-into-the-vault
status: open
ticket_number: 014
type: migration
created: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/active/notes-into-the-vault.spec.md
---

# TICKET-014-notes-into-the-vault

## Summary

The notes folder folds into the vault: the files under `~/.rusty/notes` move to `notes/`
inside the vault, the notes tools point there by default, and the explorer, search, links,
graph and semantic index cover them like any page.

## Why

Rusty keeps two markdown stores. The vault at `~/.rusty/brain` is what the explorer, search,
backlinks, graph and the semantic index read; `~/.rusty/notes` has its own manager and tools
(`notes_path`) and none of that. The notes under `~/.rusty/notes` are invisible in the app for that reason.
A second explorer root would show them and still leave them outside everything else. The
vault already treats a file without frontmatter as a page typed by its top folder, so the
files can move as they are.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `rusty-cli notes adopt` runs, it shall move every file under the notes path into `<vault>/notes/`, keeping names and folders, refusing on a name clash, and reporting what moved. | tests on a temporary vault |
| REQ-002 | WHEN `notes_path` is unset, the notes tools shall use `<vault>/notes`. | unit on the setting's default |
| REQ-003 | WHEN a note is created or appended through the notes tools or the `/note` skill, the vault index shall pick it up as a page of type `note`. | tool test; index test |
| REQ-004 | WHEN the adoption has run, the old folder shall be left with a README naming the new place, and nothing shall be deleted. | test |
| REQ-005 | WHEN the explorer shows the vault, `notes/` shall appear as an ordinary folder with no special casing. | screenshot |
| REQ-006 | WHEN the adoption ships, the README and the installer's output shall name the one-shot command. | doc review |

## Scope

- In: the adoption command, the setting's default, the notes tools' path resolution, docs.
- Out: merging the notes tools into the brain tools (a later cleanup); any Obsidian concern.

## Notes

- Pipeline spec: docs/planning/pipeline/active/notes-into-the-vault.spec.md
- Related docs: `crates/rusty-core/src/notes/mod.rs`, the `notes_path` setting in `SettingsPage.qml`, `docs/architecture.md` (vault rules since the workspace), the `/note` skill in the store.
- Promoted from intake: none; drafted by the rustal session on 2026-09-03 from Chad's words at 15:40: "i dont see anywhere my files in ~./rusty/notes are?".
- Follow-ups opened: none.
