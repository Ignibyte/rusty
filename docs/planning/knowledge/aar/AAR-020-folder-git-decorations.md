---
title: AAR-020-folder-git-decorations
ticket: TICKET-020
pipeline: 901896cf-c96e-410e-850b-2bf7ab150220
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-020: Folders, part three — git decorations

## 0. Recall log

- Parts one and two settled the owner: the disk is the app's `Folders` type. A git status
  is another read of the disk, cached with the listing and dropped with it.
- The workspace already spawns `git` from Rust (the vault's auto-commit), so porcelain
  through a process needs no crate and stays inside 016's seal.

## 1. Outcome

The marks and the branch on a folder root that lies in a repository, read through `git`
and never written. Four files, four tests. `GATE GREEN [diff]` twice (F16 came after the
first), one scene photographed against a repository the screenshot script now seeds.

## 2. What went well

- Two layers of test: the porcelain parser on a fixture that needs no git, and a
  repository built under a temporary tree with `HOME` and `GIT_CONFIG_GLOBAL` pointed
  away from the machine's, so signing and hooks never enter. Green on the first run.
- F1 was settled at design: the status is fetched in `rebuild` before any row exists, so
  a row's binding only reads and the scene log has no binding loop.
- The read costs nothing a user would notice: 3 ms on this repository.

## 3. What went poorly

- F16 again. 019's AAR said "grep for the prefix before adding a scene branch"; the grep
  was run for `file:` and not for `root:`, and part one's `root:` scene was shadowed the
  same way. The lesson was too narrow: list the chain's prefixes and look for a duplicate
  (`grep -oE 'startsWith\("[a-z:]+"\)' | sort | uniq -d`), whatever the new prefix is.
- A measurement line was written into the notes before the command ran; `/usr/bin/time`
  is not on the box, the command failed, and the line was rewritten with bash's `time`
  and the real number. The rule already says it: paste what printed, after it printed.

## 4. Surprises

- Porcelain paths are relative to the repository's top level, not to the directory the
  command ran in; the `rev-parse --show-toplevel` step exists for that, and the subfolder
  assertion in the test is what proves it.
- The pipeline check wants a `submitted:` date in every completed AAR; none of the
  earlier ones carried it. Dated in one commit before this pipeline opened.

## 5. Lessons

- `AD-rusty-git-status-read-only-001`: a root's git state is one read of the porcelain by
  the app, cached with the listing, tokens and a letter for the marks, never a write, no
  crate.
- Before adding a branch to a chain of prefixes, list the chain and look for the
  duplicate; a lesson phrased for one prefix does not cover the next one.
- A measurement goes into the record after it prints, never as a placeholder.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 10m |
| 2 Design | 20m | 15m |
