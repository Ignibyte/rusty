---
title: The brain loop: notes
pipeline_id: 3d2d1091-fee1-4e82-9f3e-1d944b2803c3
---

# The brain loop: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins; register (`AD-rusty-brain-is-the-project-memory-001`,
  `AD-rusty-files-are-the-truth-001`, `AD-rusty-mcp-only-back-end-001`); rustal's
  `enforce-docs-before-code.sh` (block on the first gated write until a recall is in the
  transcript, fail open on an unreadable transcript) and its predicates; the brain
  manager's writers and search; `TYPE_DIRS`; the database's `migrate`; the CLI's
  `run_brain`; the app's `GraphView.qml` filters and the view tabs; the `/brief` store
  skill. Sealed at 17:20 (relayed) with the rustal session's four answers.

## Phase 2: Design

- Architecture and data flow: a `decision` page type (`decisions/`). `brain_ask`
  runs the hybrid search (text when no embedder), lists the decisions whose text
  matches (type `decision`) with their status and dates, lists the follow-ups due, and
  records a consultation row (`brain_consultations`: id, question, hits as JSON,
  created_at, outcome) whose id it returns. `brain_decide` writes
  `decisions/<slug>` through `create_page` with a body of Question, Choice, Rationale,
  Alternatives and Consulted (wikilinks to every hit of the consultation), sets the
  frontmatter (`question`, `status: decided`, `decided`, `follow_up_by`, `consulted`,
  `consultation`, `supersedes`), marks the old decision superseded when `supersedes`
  names one, adds a timeline entry to every consulted page, and marks the consultation's
  outcome. `brain_follow_up` appends a dated Follow-up section, sets the status (kept,
  revised, superseded with `successor`), clears or reschedules `follow_up_by`, and adds
  a timeline entry. `brain_no_decision` marks the consultation's outcome with the reason.
  `brain_due` lists decisions whose status is decided or revised and whose `follow_up_by`
  is on or before today plus `days`. `brain_graph` gains `kind` on every edge (`link`,
  `consulted`, `supersedes`, `follows_up`) from the decision pages' frontmatter. The app
  draws typed edges dashed in the accent colour behind a "Decisions" filter and gets a
  Decisions view (due first, then every decision with status and dates; a click opens the
  page). The CLI mirrors the five tools under `brain`, and `hooks install|uninstall|status`
  writes the two embedded scripts to `~/.rusty/hooks/` and wires `~/.claude/settings.json`
  (PreToolUse on Edit, Write, MultiEdit, NotebookEdit; Stop), idempotently, keeping every
  other entry.
- File manifest:
  - `crates/rusty-core/src/brain/vault.rs`: `("decision", "decisions")` in `TYPE_DIRS`.
  - `crates/rusty-core/src/engine/db.rs`: `brain_consultations`.
  - `crates/rusty-core/src/brain/decisions.rs` (new): the loop on `BrainManager`, tests.
  - `crates/rusty-core/src/brain/mod.rs`: `GraphEdge.kind`, typed edges in `graph`.
  - `crates/rusty-core/src/skills/mod.rs`: the seed skill `ask-decide-follow-up`.
  - `crates/rusty-mcp/src/main.rs`: five tools, their parameters, the router names; the
    smoke test's loop flow.
  - `crates/rusty-cli/hooks/*.sh` (new), `crates/rusty-cli/src/hooks.rs` (new, the
    installer and the corpus tests), `crates/rusty-cli/src/main.rs` (`brain ask|decide|
    follow-up|no-decision|due`, `hooks ...`, usage).
  - `crates/rusty-app/qml/DecisionsPage.qml` (new), `Main.qml` (the view, the palette,
    the ribbon), `GraphView.qml` (the edge kinds and the filter).
  - Phase 5: README, `docs/architecture.md`, `docs/architecture/brain-loop.md` (the ADR),
    ROADMAP, the wiki, the tool count (76) in `CLAUDE.md`, `AGENTS.md`, `.mcp.json`, the
    quickstart; the `/brief` store skill.
- Store consequences: one table (`brain_consultations`); one page type and folder; a
  `kind` on graph edges (additive).
- Tool contract: five tools added, `brain_graph` edges gain `kind`; 76 tools.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001 | `ask_records_a_consultation_and_ranks_pages` |
  | REQ-002 | `decide_writes_a_linked_decision_page_and_timeline_entries` |
  | REQ-003 | `follow_up_sets_the_status_and_the_date` |
  | REQ-004 | `graph_carries_typed_edges_for_decisions`; the graph scene |
  | REQ-005 | the corpus test in `hooks.rs`: no ask blocks, an errored ask blocks, an ask allows, out of scope allows, unreadable allows |
  | REQ-006 | the corpus test: writes without a record refuse, a record allows, `stop_hook_active` allows, no writes allow |
  | REQ-007 | the CLI by reading and one `due` run; the `/brief` edit; the Decisions scene |
  | REQ-008 | the seed skill in the seed table test; the ADR by review |
- Risks: a transcript scan reads the whole file on every write (rustal does the same;
  transcripts are megabytes at most); a session that writes through Bash is not seen by
  the PreToolUse hook (the Stop hook does not see it either: the honest exit stays
  `brain_no_decision`); `settings.json` is rewritten by the installer (keys sorted by
  serde, every entry kept).
- CodeGraph evidence: `create_page`, `set_property`, `add_timeline` and `search_hybrid`
  each gain one caller in `decisions.rs`; `graph` is called by `brain_graph` alone.

## Phase 3: Implement

- Built: the manifest as designed. `brain/decisions.rs` (ask, decide, follow_up,
  no_decision, due, decision_summary, decision_edges; five tests), `("decision",
  "decisions")` in the type table, `brain_consultations` in `migrate`, `GraphEdge.kind`
  with the typed edges pushed before the plain links so they win the dedup, the seed
  skill; five tools with their parameter structs and router names, the loop flow in the
  smoke test (ask, decide with a follow-up date in the past, due shows it overdue, follow
  up kept, the graph carries `"kind": "consulted"`, an unknown consultation is refused);
  `hooks.rs` (install, uninstall, status; the corpus tests run the shipped scripts under
  bash with jq), the two scripts, `brain ask|decide|follow-up|no-decision|due` and `hooks
  ...` in the CLI; `DecisionsPage.qml`, the view, the palette command, the ribbon button,
  the dashed typed edges behind "Decision edges" in the graph view; a decision page in the
  screenshot script's scratch vault.
- Deviations: the write hook is scoped to files under the working directory as well
  (the rustal session's check: a scratch script elsewhere must not trip the gate); the
  Decisions view is fed by one tool, `brain_due`, which returns the due list and every
  decision, so the app needs no per-page reads.
- Evidence: `cargo test -p rusty-core -- decisions deploy_seeds page_types` → 7 passed;
  `cargo test -p rusty-mcp` → router 3 passed, smoke 1 passed (the loop flow inside);
  `cargo test -p rusty-cli hooks` → 3 passed (nine corpus cases for the write hook, six
  for the stop hook, the installer's idempotence and the untouched entries); the scenes
  `view:decisions` and `graph` at 18:29: the seeded decision under Due (decided
  2026-09-01, follow up by 2026-09-02) and in the list; the graph with two dashed edges
  from the decision to `projects/orbit` and `concepts/compiled-truth` and the "Decision
  edges" filter checked.
- Fast gate: below.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Scope | The write hook gated every Write or Edit, a scratchpad script included, so the first thing a session writes would have tripped it before any question. | high | fixed: absolute paths outside the working directory pass; relative paths count as inside; two corpus cases |
| 2 | Data safety | `decide` writes through `create_page` and `write_raw` (a version is kept), adds timeline entries through `add_timeline`, and touches an older decision only through `set_property`; nothing deletes. | none | confirmed |
| 3 | Fail open | Both hooks exit 0 without jq, without a transcript path, on an unreadable transcript, and when jq fails on a line. | none | confirmed by the corpus |
| 4 | Contract | `no_decision` on an unknown consultation is an error, so a fabricated id cannot satisfy the loop server-side; the hook, by design, sees only the tool use and its non-error result. | low | accepted (the seal: receipts by transcript scan) |
| 5 | Performance | `due` and `decision_edges` read every decision page's file; hundreds are fine, thousands would want an indexed property. | low | accepted, noted on the AAR |
| 6 | Settings | `hooks install` rewrites `~/.claude/settings.json` with serde's key order. | low | accepted; every entry and value kept, tested |
| 7 | Nothing personal | The screenshot vault's decision page is fiction; no host, account or vault page enters the repo. | none | confirmed |

- Post-implementation CodeGraph: `ask` is called by `brain_ask` and the CLI; `decide`,
  `follow_up`, `no_decision` and `due` each by their tool and the CLI; `decision_edges`
  by `graph` alone; `hooks::install` by `run_hooks` and its test.

## Phase 4: Validate

- Tests run (commands and output): as under Phase 3, and the workspace under the gate.
- Gate run: `bin/gate.sh --fast` → GATE GREEN [fast] at 18:41 (after a clippy stop on a
  nested `format!` and on the CLI's `run_hooks` placed after the test module, both
  fixed); `--diff` below.
- Smoke evidence: the smoke test's loop flow over stdio; the two scenes.
- Skips or pre-existing failures: none.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (`ask_records_a_consultation_and_ranks_pages`, the
  smoke flow). REQ-002 PASS (`decide_writes_a_linked_decision_page_and_timeline_entries`).
  REQ-003 PASS (`follow_up_sets_the_status_and_the_date`). REQ-004 PASS
  (`graph_carries_typed_edges_for_decisions`, the smoke flow's `"kind": "consulted"`, the
  graph scene with the dashed edges and the filter). REQ-005 PASS (the write hook corpus:
  no ask blocks, an errored ask blocks, an ask allows, only `brain_ask` counts, out of scope
  allows, an unreadable transcript allows, a file outside the repository allows, a relative
  path counts as inside). REQ-006 PASS (the stop hook corpus: writes without a record
  refuse, the retry passes, a decision allows, a no-decision allows, an errored record does
  not count, no writes allow). REQ-007 PASS (the CLI's five `brain` commands by reading;
  the `/brief` store skill's due line; the Decisions scene). REQ-008 PASS (the seed skill in
  the seed table, its test; the ADR `docs/architecture/brain-loop.md`).
- Docs: README (a brain loop section, the count), `docs/architecture.md` (as built),
  `docs/architecture/brain-loop.md` (the ADR), ROADMAP, the tool count in `CLAUDE.md`,
  `AGENTS.md`, `.mcp.json` and the quickstart, the wiki pages `mcp-back-end.md`,
  `workflow-and-gates.md` and `workspace-app.md` through the OpenWiki run dcb639e9.
- AAR: `AAR-018-brain-loop.md`. Register: `AD-rusty-brain-loop-001`,
  `PR-rusty-gate-the-repository-not-the-tool-001`.
- Brain capture: the project page's timeline, after the commit.
- Store: the `/brief` skill gained a "follow-ups due" line (`rusty-cli brain due`), and
  `ask-decide-follow-up` was written into the store from the seed text, since seeds deploy
  only into an empty store; both committed there.
- Archive: this pair to `completed/`.
