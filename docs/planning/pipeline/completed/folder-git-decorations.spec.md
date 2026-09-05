---
title: Folders, part three: git decorations
pipeline_id: 901896cf-c96e-410e-850b-2bf7ab150220
status: Phase 5 — Complete PASS
ticket: TICKET-020
ticket_doc: docs/planning/tickets/open/TICKET-020-folder-git-decorations.md
aar: docs/planning/knowledge/aar/AAR-020-folder-git-decorations.md
sealed: minted at TICKET-016's design on 2026-09-03 under its seal ("file operations and git decorations become parts two and three"); no new tab, store or dependency here — git is a process the app already spawns for the vault
created: 2026-09-05
---

# Folders, part three: git decorations: spec

## Intent

Agents work in repositories, and what changed since the last commit is what a developer
looks at before opening a terminal there. A folder root that lies in a git repository
marks its modified, added and untracked files and the folders above them, and its root row
carries the branch. Nothing is written; the repository is read the way `git status` reads
it.

## Scope

- In: one read of `git status` per root, parsed in Rust with tests on a temporary
  repository; the marks in the tree (a colour and a letter on files, a dot on folders,
  the branch on the root); the cache beside the listing, dropped with it; two screenshot
  scenes and a seeded repository in the screenshot script.
- Out (named seams, not forgotten): commits, diffs, blame, stash, a git client; ignored
  files as a mark; a watcher on `.git`; submodule and worktree detail beyond what the
  porcelain says; anything that writes to the repository.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN a folder root lies inside a git repository, the tree shall mark files that are modified, added or untracked and every folder above them; a root outside a repository shows no mark. | Rust tests on a temporary repository and on a plain folder; offscreen scene |
| REQ-002 | WHEN the root row of a repository is shown, it shall carry the current branch name, or `detached` when HEAD is detached. | Rust test on a temporary repository; the scene |
| REQ-003 | WHEN the status is read, it shall come from one `git status --porcelain=v2 --branch -z` limited to the root's subtree and made relative to the root, cached until Refresh, a root change or a disk write. | parser test on a fixture; reading of the cache |
| REQ-004 | WHEN `git` is missing or the command fails, the tree shall render as it did in part two, with no mark and no notice. | Rust test on a plain folder; reading |
| REQ-005 | WHEN the status is read, nothing shall be written to the repository. | reading: read-only commands under `--no-optional-locks` |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The status is read by the app's `Folders` type spawning `git` and parsing porcelain v2; no `git2` or `gix` crate | The seal (no new dependency); the vault's auto-commit already spawns `git` from Rust; porcelain v2 is a documented, stable contract | `git2` (a dependency and a libgit2 build), a hand-written index reader |
| 2 | One JSON per root, `{repo, branch, files, dirs}`, paths relative to the root, folders folded to the strongest state below them (M over A over ?), cached in the explorer beside the listing and dropped with it | The explorer already keys a cache by path and drops it on Refresh, a root change and every write (part two); the same lifetime is right for the status | A watcher on `.git`; a timer |
| 3 | Marks are tokens: modified `gold`, added `alive`, untracked `accentSoft`; a letter beside the extension on a file, a dot on a folder, the branch in `faint` on the root | No new colour (the wiki's invariant); the letter is what `git status --short` prints, so it needs no legend | Icons; a colour alone (unreadable to some eyes) |
| 4 | The read is `git --no-optional-locks -C <root>`: `rev-parse --show-toplevel` then `status --porcelain=v2 --branch -z --untracked-files=all -- .` | `--no-optional-locks` keeps `status` from refreshing the index on disk; the toplevel makes paths relative to the root when the root is a subfolder of a repository; `-z` makes paths verbatim | A single `status` call (paths would be relative to the toplevel, not the root) |
| 5 | Scenes `root:<path>` and `expand:<path>` in `Main.qml`; `scripts/screenshot.sh` seeds a small repository under its scratch `HOME` when `git` is on the box | The marks can be photographed offscreen against scratch data, as every scene is | A scene against a real repository |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-020-folder-git-decorations.md`
- Parts one and two: `docs/planning/pipeline/completed/folders.spec.md`,
  `docs/planning/pipeline/completed/folder-file-operations.spec.md`
- Architecture: `openwiki/workspace-app.md` (the explorer), `AD-rusty-disk-is-not-the-store-001`,
  `AD-rusty-disk-writes-refuse-and-trash-001`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled; covered by 016's seal |
| 2 Design | Manifest, the parser and its JSON, the marks, the scenes, regression table | design actionable |
| 3 Implement | `folders.rs`, `Explorer.qml`, `Main.qml`, `scripts/screenshot.sh` | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger; CodeGraph over `folders.rs` | confirmed findings resolved |
| 4 Validate | The tests, an offscreen scene, `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, AAR, register, brain, archive | pair archived |
