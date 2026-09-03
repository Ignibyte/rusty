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
| `PR-rusty-qml-component-scope-001` | Inside an inline `Component`, an unqualified name finds the component's own property before an id of the enclosing document, so `theme: theme` binds to itself; bind shared objects through the window (`theme: win.theme`). | `aar/AAR-002-knowledge-workspace-shell.md` |
| `PR-rusty-qt-logs-in-journal-001` | Qt routes messages to journald when stderr is not a tty; when the app prints nothing, read `journalctl -t rusty` or run with `QT_FORCE_STDERR_LOGGING=1`. A component that fails to load leaves the app running with no window. | `aar/AAR-002-knowledge-workspace-shell.md` |
| `PR-rusty-workspace-state-in-json-001` | QtCore `Settings` rewrote string properties with their declared defaults during a run (ints survived); app state beyond the window geometry lives in the JSON files the Rust side owns (`tabs.json`, `workspace.json`). | `aar/AAR-002-knowledge-workspace-shell.md` |
| `PR-rusty-offscreen-shots-grab-window-001` | On the offscreen platform a window is never exposed and `grabToImage` cannot start; `QQuickWindow::grabWindow` renders on demand. `scripts/screenshot.sh` runs the app against a scratch vault this way. | `aar/AAR-002-knowledge-workspace-shell.md` |
| `PR-rusty-lazy-pane-terminals-001` | A terminal component starts its tmux session when first shown, never at load; a hidden pane once launched an agent during a screenshot run. | `aar/AAR-002-knowledge-workspace-shell.md` |
| `PR-rusty-openwiki-managed-block-001` | `AGENTS.md` and `CLAUDE.md` are identical outside OpenWiki's managed block (a full section in one, a pointer in the other); never hand-edit the block, compare with it stripped. | `aar/AAR-007-openwiki-in-the-workflow.md` |
| `PR-rusty-openwiki-evidence-ranges-001` | `openwiki_resolve_claims` rejects a whole batch when one `repo://path#Lx-Ly` range does not exist; read the file's length before citing. | `aar/AAR-007-openwiki-in-the-workflow.md` |
| `PR-rusty-yaml-mapping-shift-remove-001` | Removing a key from a `serde_yaml::Mapping` is `shift_remove`; `remove` swaps the last key into the hole and reorders the frontmatter. | `aar/AAR-003-tags-and-properties.md` |
| `PR-rusty-qml-signal-names-001` | Never declare a signal `<name>Changed` beside `property <name>`; the property owns that signal and the component fails to load with "Duplicate signal name". | `aar/AAR-004-graph-views.md` |
| `PR-rusty-never-kill-zero-001` | A cleanup trap kills a pid only when it holds one; `kill "${var:-0}"` is `kill 0` and takes the calling shell and its pipeline with it. | `aar/AAR-004-graph-views.md` |

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
| `AD-rusty-renderer-in-core-001` | Obsidian-flavoured markdown is rendered to Qt's rich-text HTML in `rusty-core` (`brain::render`) and served by `brain_render`; colours are inlined from a style the caller sends, because rich text has no stylesheet. The source editor's highlighter is a C++ `QSyntaxHighlighter` over spans from a Rust tokenizer. | `pipeline/completed/knowledge-workspace-shell.spec.md` |
| `AD-rusty-workspace-is-obsidian-001` | The app is laid out as Obsidian is: ribbon, explorer and search, document tabs, backlinks, outgoing links, outline, status bar; pages, agent terminals and built-in views are all tabs, the right sidebar holds an agent pane, keys are Obsidian's and stand down while a terminal has focus. Colours come from the Omarchy theme's `obsidian.css` tokens and Alacritty palette. | `pipeline/completed/knowledge-workspace-shell.spec.md` |
| `AD-rusty-lenient-pages-001` | A vault file without frontmatter is a page: title from the file name, type from its top folder or `note`; pages may live in any folder; renames rewrite links vault-wide outside fenced code and move the index rows; deletes are soft. | `pipeline/completed/knowledge-workspace-shell.spec.md` |
| `AD-rusty-openwiki-for-documentation-001` | OpenWiki 0.3.3, pinned and project-local, is the generated engineering wiki (`openwiki/`); the host agent authors it through the MCP lifecycle at Phase 5 and the completion receipt gates delivery of a completed pipeline. The brain stays the project's memory. | `pipeline/completed/openwiki-in-the-workflow.spec.md` |
| `AD-rusty-tags-one-index-001` | Frontmatter and inline `#tags` share `brain_tags`, stored as first written and compared without case, nested tags counting under their parents; `tag:<name>` terms are part of `brain_search`, not a separate tool; a property edit touches only the frontmatter mapping. | `pipeline/completed/tags-and-properties.spec.md` |
| `AD-rusty-graph-in-the-app-001` | The graph's data is one tool (`brain_graph`: nodes, edges, tags and unresolved on request, a neighbourhood by depth); the force layout, interaction and settings live in the app on a canvas, settings in the workspace state. | `pipeline/completed/graph-views.spec.md` |
