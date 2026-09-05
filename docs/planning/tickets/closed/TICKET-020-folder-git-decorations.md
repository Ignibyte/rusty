---
title: TICKET-020-folder-git-decorations
status: done
ticket_number: 020
type: feature
created: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/active/folder-git-decorations.spec.md
---

# TICKET-020-folder-git-decorations

## Summary

A folder root that is a git repository decorates modified, added and untracked files in the tree, and its branch shows on the root row.

## Why

Agents work in repositories, and the state of a tree at a glance (what changed since the last commit) is what a developer looks at before opening a terminal there.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN a folder root is a git repository, the tree shall decorate modified, added and untracked files, and folders that contain them. | test on a temporary repository |
| REQ-002 | WHEN the root row is shown, it shall carry the current branch name. | test on a temporary repository |
| REQ-003 | WHEN the status is read, it shall come from `git status --porcelain` in the root, cached until Refresh. | reading |

## Scope

- In: REQ-001 to REQ-003; the porcelain reader; the decorations.
- Out: commits, diffs, blame, a git client; anything that writes to the repository.

## Notes

- Pipeline spec: docs/planning/pipeline/active/folder-git-decorations.spec.md
- Related docs: `crates/rusty-app/qml/Explorer.qml`, `crates/rusty-app/src/folders.rs`,
  `docs/planning/tickets/closed/TICKET-016-folders.md`.
- Promoted from intake: none; minted at TICKET-016's design on 2026-09-03 as the seal
  said ("file operations and git decorations become parts two and three, minted at
  design").
- Follow-ups opened: none.
