---
title: AAR-026-obsidian-vault-migration
ticket: TICKET-026
pipeline: bd3dd522-2b7b-4d64-ab22-2c7148c62a14
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-026: Obsidian vault migration

## 0. Recall log

- The format compatibility was settled in TICKET-006; the migration's `LinkIndex` and
  `notes::adopt`'s refusals are the two pieces the front door is built from.
- Bookmarks are app state, so core reports them and the app writes them.

## 1. Outcome

The front door: a plan that names everything before a byte moves, an import at the
vault's own paths with collisions skipped and named, links rewritten, bookmarks carried
into the app, a report page, and a rollback that leaves the brain as it was. A new
module and two methods, two tools (82), a CLI subcommand, a dialog, six tests, one scene.
`GATE GREEN [diff]`.

## 2. What went well

- Two existing pieces made the feature: the migration's `LinkIndex` for the links and
  `notes::adopt`'s refusals for the tone. The design was mostly choosing where each
  piece sits.
- The fixture vault carries every case in one place — a collision, a nested page, an
  attachment, a bookmarks file with a group and a dead entry, the folders the walk
  skips — and the three tests read it three ways, the rollback included.
- The scene's kept scratch showed the plan writing nothing: the seeded vault's six
  files as seeded and no `inbox/` page.

## 3. What went poorly

- A patch (the report slug's suffix) missed its anchor because `cargo fmt` had
  rewrapped the line; the assertion said so, but the gate had already run green on code
  without the fix, and the notes' ledger already called it confirmed. The fix landed on
  the formatted text and the gate ran again. Patch the text as fmt left it, and write
  the ledger row after the patch's own check prints.
- A Rust raw string `r#"…"#` closed on the fixture's `"#Heading"`; `r##` fixed it.
- The wiki claim for the vault page was anchored before the last code edit; OpenWiki's
  finish left that page's sidecar unchanged with a warning, and a second run re-added
  the claim. Anchor claims after the last code edit.

## 4. Surprises

- `[[Note A]]` at the vault's root already is its vault path, so the rewrite counts one
  link where two were expected; the test's number was mine to correct.
- OpenWiki verifies evidence by content at finish and prefers to leave a sidecar
  unchanged over failing the run; the warning is the only sign.

## 5. Lessons

- `AD-rusty-import-keeps-paths-and-refuses-001`: an import keeps the vault's paths,
  refuses collisions by name, rewrites links with the migration's index, reports
  bookmarks for the app to write, and rolls back whole.
- Patch the text as `cargo fmt` left it; a ledger row is written after the patch's
  check prints.
- Resolve wiki claims after the last code edit of the pipeline.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 10m |
| 2 Design | 30m | 25m |
