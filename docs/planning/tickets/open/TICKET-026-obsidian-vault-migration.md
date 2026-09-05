---
title: TICKET-026-obsidian-vault-migration
status: open
ticket_number: 026
type: migration
created: 2026-09-04
intake:
pipeline_spec: docs/planning/pipeline/active/obsidian-vault-migration.spec.md
---

# TICKET-026-obsidian-vault-migration

## Summary

Bring an existing Obsidian vault into Rusty: point at a vault, see what will happen, import it, and keep what Obsidian's own files hold.

## Why

Rusty's brain is already an Obsidian vault by format — that was the point of retiring the bridge in TICKET-006. What is missing is the front door. Someone with years of notes has no way in except copying folders by hand and hoping the index catches up, and Chad will want his own vault in here.

The format compatibility does the hard part. The work is in the edges: `.obsidian/` holds bookmarks, starred files, graph settings and hotkeys that map onto features Rusty already has; attachments live wherever that vault configured them; and a real vault will have slug collisions and links Rusty's resolver reads differently.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN an Obsidian vault folder is chosen, the app shall report what it found — pages, folders, attachments, tags, unresolved links — and what it will do, before changing anything. | smoke on a real vault |
| REQ-002 | WHEN an import runs, pages, folders and attachments shall land in the brain and be indexed for full text, tags, links and the graph. | test on a fixture vault |
| REQ-003 | WHEN a page's slug collides with one already in the brain, the import shall neither overwrite nor silently rename, and shall report the collision for a decision. | test |
| REQ-004 | WHEN `.obsidian/bookmarks.json` is present, its bookmarks shall come across as Rusty bookmarks. | test |
| REQ-005 | WHEN an import finishes, it shall write a report naming what came in, what was skipped and why. | test; reading |
| REQ-006 | WHEN an import fails part way, the vault it read from shall be unchanged, and the brain shall be left consistent. | test |

## Scope

- In: a vault picker with a dry run, the importer, attachment handling, bookmark import, the report, `rusty-cli` and MCP entry points.
- Out: two-way sync with Obsidian; live watching of a foreign vault; plugin settings and community-plugin data; canvas files.

## Notes

- The source vault is read-only throughout. Nothing about this ticket writes to it.
- Obsidian's own vault is the best fixture; a small synthetic one belongs in the tests.
- Adjacent: `rusty-cli notes adopt` (TICKET-014) already does a one-time folder move and is the tone to match — refuse clashes, delete nothing, leave a README.
- Pipeline spec: TBC.
