---
title: AAR-020-folder-git-decorations
ticket: TICKET-020
pipeline: 901896cf-c96e-410e-850b-2bf7ab150220
status: open
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

_pending_

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 10m |
| 2 Design | 20m | 15m |
