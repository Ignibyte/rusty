---
title: Folders, part three — notes
pipeline: 901896cf-c96e-410e-850b-2bf7ab150220
ticket: TICKET-020
---

# Folders, part three: git decorations: notes

## Recall (2026-09-05)

- Bulletins: none touch a spawned process or the explorer; bulletin 2 (no synthetic input
  on Chad's desktop) shapes validation as before.
- Register: `AD-rusty-disk-is-not-the-store-001` and `AD-rusty-disk-writes-refuse-and-trash-001`
  — the disk is the app's, read and written by `Folders`, never the back end. A git status
  is another read of the disk; the same owner.
- 016's locked decisions still stand: the Rust type, disk rows in the one list, the
  listing cached until Refresh. 019 added: every write ends in `refreshDisk()`, which
  drops the listing — the status cache takes the same lifetime.
- Wiki: `workspace-app.md` describes the roots and the file operations and says "Git
  decorations are TICKET-020".
- Brain: `brain_search "folders git status"` ranked no page of this project (learning
  notes and a migration plan); nothing to carry.
- Code read:
  - `folders.rs`: eleven invokables over pure functions; `list_json` gives
    `{name, path, kind, size}` per entry, the explorer maps `folder`/`file` to `dir`/`disk`
    in `walkDisk`. `rusty-core`'s `vault.rs` and `skills/mod.rs` already spawn `git`
    through `std::process::Command` (the auto-commit), so a spawned `git` is not new to
    the workspace.
  - `Explorer.qml`: `listing` is a map keyed by directory, filled lazily by `entriesOf`
    and cleared by `refreshDisk()`; `rebuild()` pushes one `root` row per root and walks
    an expanded root. The delegate draws the name in `muted` (`bright` when active) and,
    for `disk` rows, the extension in `faint` at the right; folders show `pages` counts
    for vault folders only. `rootOf(path)` (019) finds a row's root.
  - `Main.qml`: `addRoot(chosen)` strips `file://`, refuses a relative path, and writes
    `ui.roots`; scenes are one `if` chain in the shot timer; `explorer` is the id.
  - `scripts/screenshot.sh`: seeds a vault, a skill and a workspace state under a scratch
    `HOME`; no repository and no `roots`.
  - `git status --porcelain=v2 --branch -z`: `# branch.head <name>` (`(detached)` when
    detached), `1 <XY> …<path>`, `2 <XY> …<path>\0<orig>`, `u <XY> …<path>`, `? <path>`;
    with `-z` the records end in NUL and paths are verbatim; paths are relative to the
    repository's top level.

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-app/src/folders.rs` | `git_status(root)` invokable; `GitStatus { branch, files: Vec<(String, char)> }`; `parse_status(bytes)`, `fold_dirs(files)`, `git_status_json(root)`; tests: the parser on a fixture, the fold, a temporary repository, a plain folder |
| `crates/rusty-app/qml/Explorer.qml` | `git: ({})` beside `listing`, `gitOf(root)` lazy, cleared in `refreshDisk()`; `gitState(row)` and `branchOf(root)`; the delegate's name colour, the letter, the dot, the branch |
| `crates/rusty-app/qml/Main.qml` | scenes `root:<path>` and `expand:<path>` |
| `scripts/screenshot.sh` | a seeded repository under the scratch `HOME` (one committed tree, a modified file, a staged new file, an untracked file) when `git` is on the box |

### The read

`git_status_json(root)`:

1. `git --no-optional-locks -C <root> rev-parse --show-toplevel` — a failure (not a
   repository, no `git`) answers `{"repo": false}`.
2. `git --no-optional-locks -C <root> status --porcelain=v2 --branch -z --untracked-files=all -- .`
3. `parse_status`: the branch from `# branch.head`; for each `1`, `2` and `u` record the
   `XY` pair and the path (a `2` record's second path is skipped); `?` records are
   untracked. The state: `?` for untracked; `A` when either column is `A`; `M` when either
   column is `M`, `T`, `R`, `C`, `D` or `U`. Paths are made relative to the root by
   stripping the root's prefix under the toplevel; anything outside is dropped.
4. `fold_dirs`: every ancestor folder of a marked path gets the strongest state below it
   (`M` over `A` over `?`).
5. JSON: `{"repo": true, "branch": "main", "files": {"src/a.rs": "M"}, "dirs": {"src": "M"}}`.

The process runs on the GUI thread once per root per cache lifetime; `git status` on a
working tree of a few thousand files is tens of milliseconds, and Refresh is the user's
own act.

### The marks

`gitState(row)`: `root = rootOf(row.path)`, `s = gitOf(root)`; `""` unless `s.repo`;
`rel = row.path.slice(root.length + 1)`; `s.files[rel] || s.dirs[rel] || ""`.
`branchOf(root)`: `gitOf(root).branch || ""`.

Delegate: `readonly property string git: explorer.gitState(modelData)` for `disk` and
`dir` rows; the name's colour `gitColor` (`M` gold, `A` alive, `?` accentSoft, else the
colour it had); a `Text` after the extension with the letter in the same colour for
`disk` rows; a `•` for `dir` rows; on a `root` row the branch in `faint` after the name.

### Scenes

`root:<path>` calls `win.addRoot` with the path resolved as `file:` resolves it, then
`explorer.expandPath` on it after the roots change; `expand:<path>` expands any row.
The screenshot script seeds `$scratch/repo` (committed `README.md` and `src/lib.rs`,
then `src/lib.rs` modified, `src/new.rs` staged, `notes.txt` untracked) with the identity
given on the command line and `GIT_CONFIG_GLOBAL=/dev/null`, so nothing of Chad's git
configuration (signing included) reaches the scratch. The scene:
`root:repo,expand:repo/src`.

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | `git_status_marks_a_temporary_repository` (modified, added, untracked, the folder), `fold_dirs_takes_the_strongest_state`; the scene |
| REQ-002 | the same repository test asserts the branch; the scene |
| REQ-003 | `parse_status_reads_porcelain_v2` on a fixture; reading of `gitOf` and `refreshDisk` |
| REQ-004 | `git_status_answers_no_repo_for_a_plain_folder`; reading |
| REQ-005 | reading: `--no-optional-locks`, `rev-parse` and `status` only |

### Risks

- **Data safety.** Read-only; the repository test builds its own repository under a
  temporary directory with `HOME` and `GIT_CONFIG_GLOBAL` pointed away from Chad's.
- **A root that is a subfolder.** Paths from `status` are relative to the toplevel; the
  prefix is stripped and anything outside the root dropped; a test covers it.
- **Large untracked trees.** `--untracked-files=all` lists every untracked file; ignored
  files are not listed, so a `target/` or `node_modules/` costs nothing; an unignored
  large tree costs one listing, on Refresh only.
- **Theme.** Three existing tokens, the letters legible without colour.
- **Keyboard.** Nothing new to reach; the marks are read-only.
- **No git.** `rev-parse` fails, the JSON says `repo: false`, the tree is part two's.

### CodeGraph

`codegraph_explore` over `Folders`, `list_json`, `list_dir`: the type's callers are the
QML files alone; `rusty-core` spawns `git` in `vault.rs` (the auto-commit) and
`skills/mod.rs` — the precedent for decision 1. A second pass after implementation goes
in the ledger.

## Phase 3: Implement

Four files, as the manifest said. `folders.rs`: `git_status` in the bridge over
`git_status_json`, `git_status_of`, `parse_status`, `state_of`, `fold_dirs` and
`git_output` (a `git --no-optional-locks` runner); four tests. `Explorer.qml`: the `git`
cache beside `listing`, `gitOf` (the one writer, called from `rebuild` for every root),
`gitState`, `branchOf`, `gitColor`, `refreshDisk` dropping both caches, the delegate's
name colour, the letter or the dot, the branch. `Main.qml`: `shot.resolve` and the
`root:` and `expand:` scenes (`file:` now resolves through the same function).
`scripts/screenshot.sh`: the seeded repository. `qmllint` exit 0 on both QML files,
`bash -n` on the script, `cargo fmt --all` ran.

Deviations: none from the manifest. The eager `gitOf` in `rebuild` was the design's
"lazy" fetch moved one step earlier, so a row's binding only reads (F1 below).

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | correctness | a row binding that fetched the status would assign `git` inside a binding evaluation (a loop, or a warning at best) | **confirmed at design**; `rebuild` fetches for every root before the rows exist, `gitState` and `branchOf` only read; the scene log carries no `binding loop` |
| F2 | correctness | a `2` (rename) record's original path is its own NUL-terminated record | rejected: `parse_status` skips it; the fixture holds one |
| F3 | data safety | the repository test reaches the machine's git configuration (signing, hooks) | rejected: `HOME`, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM`, an identity in the environment, `commit.gpgsign=false`; the plain-folder test assumes `temp_dir()` is outside a repository — true of `/tmp` and of this box's `TMPDIR` |
| F4 | correctness | a staged deletion (`D.`) marks a file that is not in the tree | accepted: the file's folder carries the dot, which is right — the folder changed |
| F5 | performance | one `git status` per root on the GUI thread at every `refreshDisk` | accepted: cached until Refresh, a root change or a write, each the user's act; on this repository (see Phase 4) the read is well under a frame |
| F6 | theme | `gold`, `alive`, `accentSoft`, `faint`, `termFont` | tokens only; no new colour |
| F7 | keyboard first | nothing new to reach | the marks are read-only |
| F8 | correctness | `canonicalize()` against git's toplevel when a root is reached through a symlink or a bind mount | accepted: git prints the real path, `canonicalize` matches it; when they differ `strip_prefix` fails and the root shows no mark rather than wrong marks |
| F9 | correctness | `-- .` with `--untracked-files=all` limits the read to the root's subtree | rejected: the subfolder assertion in `git_status_marks_a_temporary_repository` |
| F10 | correctness | two roots where one lies under the other: `rootOf` picks the first listed | rejected: either root's status holds the row's path, so the mark is the same |
| F11 | correctness | `git_output` on a root that vanished | rejected: `current_dir` fails, `output()` errors, `None`, `{"repo":false}` |
| F12 | correctness | CodeGraph reports no covering tests for `parse_status` and `git_status_of` | rejected: three tests call them; the graph does not see test callers |
| F13 | correctness | `file:` moved onto `shot.resolve` | rejected: the same expression, read back |
| F14 | correctness | the script's seed chain `g init && g add && g commit` under `set -Eeuo pipefail` would abort every scene if the commit failed | **confirmed**; the chain is an `if` condition, and a failure prints a note and leaves a plain folder |
| F15 | prose | comments, the notes, the spec | read against `no-ai-slop` |
| F16 | correctness | part one already had a `root:` scene (an absolute path, expanded at once); the new branch sat above it and shadowed it — 019's F16 again, its lesson ("grep for the prefix") applied to `file:` and not to the whole chain | **confirmed** (found at complete); one branch: absolute or under `HOME`, expanded after the roots change |

CodeGraph after the change: `git_status_json → git_status_of → parse_status → state_of`
with `fold_dirs` beside; callers of the bridge's `git_status` are `Explorer.qml` alone;
nothing outside `folders.rs` depends on the new symbols. The blast radius matches the
manifest.

## Phase 4: Validate

- `bin/gate.sh --fast` after implement: `GATE GREEN [fast]` on the first run — the
  `rusty-app` binary's tests 33 passed, including `parse_status_reads_porcelain_v2`,
  `fold_dirs_takes_the_strongest_state`, `git_status_marks_a_temporary_repository`
  (a repository built with `git init -b main` under a temporary tree) and
  `git_status_answers_no_repo_for_a_plain_folder`.
- `cargo build -p rusty-app -p rusty-mcp` (23:09:10), then `SHOT_KEEP=1 scripts/screenshot.sh
  <scratch> "root:repo,expand:repo/src"`, offscreen against the scratch vault and the
  seeded repository. The log clean of `error|warning|TypeError|ReferenceError|Cannot
  assign|is not a type|Detected anchors|binding loop`.
  - `root-repo-expand-repo-src.png`: a Folders section with `repo` and `main` at its
    right; `src` open with a gold dot; `dock.rs` with **A** in the alive colour, `lib.rs`
    with **M** in gold, `notes.txt` with **?** in accentSoft, `README.md` unmarked —
    REQ-001 and REQ-002's scene.
  - The kept scratch repository's own `git status --short` agreed: `A  src/dock.rs`,
    ` M src/lib.rs`, `?? notes.txt`, branch `main`; the kept `workspace.json` carried the
    root and both expanded paths.
- F5's measure: `time git --no-optional-locks status --porcelain=v2 --branch -z
  --untracked-files=all -- .` on this repository (296 tracked files), read-only: 0.002 s
  wall (bash's `time`; `/usr/bin/time` is not on the box).
- REQ-003 rests on the fixture test and the reading of `gitOf`/`refreshDisk`; REQ-004 on
  the plain-folder test; REQ-005 on the reading of `git_output` (`--no-optional-locks`,
  `rev-parse` and `status` only).
- `bin/gate.sh --diff` after the last gated edit (F14): fmt, clippy, test, doc,
  shell-syntax, secrets, whitespace all ok, `receipt written: .git/rusty-gate-receipt`,
  `GATE GREEN [diff]`.
- F16 was a gated edit after that run; the gate ran again (below).
- `bin/gate.sh --diff` after F16: every step ok, `receipt written: .git/rusty-gate-receipt`,
  `GATE GREEN [diff]`.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-005 satisfied — REQ-001 and REQ-002 by the repository
  test and the scene, REQ-003 by the fixture test and the reading of `gitOf` and
  `refreshDisk`, REQ-004 by the plain-folder test, REQ-005 by the reading of
  `git_output`. None split, none waived.
- Wiki: run `07c17b0f-c1cd-4cfd-a364-84754f630be7`, `openwiki_finish` → `complete`; a
  part-three paragraph, the invariant, a failure mode and the tests; four claims
  re-anchored (part two's had gone stale as the bridge grew), one added. The PostToolUse
  hook did not fire (seventh sighting); bulletin 3's recovery with the pair under
  `active/`, then `bin/gate.sh --verify`.
- ROADMAP ticked under M8. `AD-rusty-git-status-read-only-001` in the AAR and the
  register. Brain: timeline entry on `projects/rusty-v3`.
