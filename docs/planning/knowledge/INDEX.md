# Knowledge register

Search this file before planning or implementing, then read the linked AAR, pipeline
notes or architecture document. New IDs belong both in the run's AAR and here. The brain
carries the same lessons as project pages and memories for recall from any tool.

IDs: `PR-rusty-<slug>-NNN` prevention rules, `BF-rusty-<slug>-NNN` bug families,
`AD-rusty-<slug>-NNN` architecture decisions.

## Standing rules

| ID | Rule | Source |
|---|---|---|
| `PR-rusty-one-cargo-at-a-time-001` | Run cargo commands one at a time and never kill a running one; a killed or concurrent cargo corrupts the incremental cache and forces a full rebuild. | `CONSTITUTION.md` §0 |
| `PR-rusty-probes-use-throwaway-rows-001` | UI and service probes create their own rows and delete them by id; a probe once completed, archived and reordered Chad's real Personal list. Query SQLite read-only with `.timeout`, not `PRAGMA busy_timeout` (its output corrupts captured ids). | session 2026-09-02 |
| `PR-rusty-scope-the-sqlite-guard-001` | Never hold the `Database::conn()` guard across a call that takes it again; `sync_all` deadlocked on its first orphaned index row. | commit 22c6f8d |
| `PR-rusty-first-launch-into-a-registered-vault-001` | Obsidian is told about a vault by writing its config while the app is closed; opening an unregistered folder through `obsidian://open?path=` left the app in its picker for five minutes. | commit b10c382 |
| `PR-rusty-glob-loops-not-ls-pipes-001` | Count or iterate files with a glob loop guarded by `[[ -f ]]`, never `ls \| wc -l`, in a script under `set -o pipefail`; the pipe exits the script silently on an empty folder. | `aar/AAR-001-agent-workflow-bootstrap.md` |
| `PR-rusty-signals-through-connections-001` | Attach QML signal handlers to third-party types (qmltermwidget) through `Connections { ignoreUnknownSignals: true }`; an `onFoo` property for a signal the type lacks fails the whole component load silently. | commit 7b688c8 |

## Bug families

| ID | Family | Source |
|---|---|---|
| `BF-rusty-tool-name-drift-001` | A QML page called a tool by a name the server did not have (`settings_set` for `setting_set`); the router test lists tool names but nothing checked the pages. | commit b5681fe |

## Architecture decisions

| ID | Decision | Source |
|---|---|---|
| `AD-rusty-files-are-the-truth-001` | The brain is a markdown folder; SQLite holds rebuildable indexes (FTS, links, vectors). | `docs/architecture.md` |
| `AD-rusty-mcp-only-back-end-001` | App, CLI and agents reach the store only through `rusty-mcp`. | `docs/architecture.md` |
| `AD-rusty-vault-rules-001` | A page's timeline is its `## Timeline` section; wikilinks are vault paths `[[folder/slug]]`; Obsidian is set to write the same. | commit 22c6f8d |
| `AD-rusty-agents-are-terminals-001` | Claude Code and Codex run as tmux-backed terminals inside the app; there is no in-process chat with a model. | `docs/architecture.md` |
| `AD-rusty-brain-is-the-project-memory-001` | The brain plays the OpenWiki and OpenViking role for this project: lessons are captured at complete and recalled at the start of a pipeline; tiered context and session-to-memory extraction are roadmap items for the brain itself. | `aar/AAR-001-agent-workflow-bootstrap.md` |
| `AD-rusty-local-work-record-001` | The workflow record is repo files (tickets, spec/notes pairs, AARs, this register) with a worktree-bound gate receipt; no work-state server. | `CONSTITUTION.md` §19, 2026-09-02 |
