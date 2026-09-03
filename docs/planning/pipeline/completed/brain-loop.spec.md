---
title: The brain loop
pipeline_id: 3d2d1091-fee1-4e82-9f3e-1d944b2803c3
status: Phase 5 — Complete PASS
ticket: TICKET-018
ticket_doc: docs/planning/tickets/closed/TICKET-018-brain-loop.md
aar: docs/planning/knowledge/aar/AAR-018-brain-loop.md
sealed: Chad, 2026-09-03 17:20, in the rustal session (relayed): "lets run them all including 010", with the rustal session's four answers: the hook scripts ship in this repo and `rusty-cli hooks install` wires `~/.claude/settings.json` idempotently, omarchy-ops calling it; receipts by transcript scan (a non-error `brain_ask` tool use before the first write; a `brain_decide` or `brain_no_decision` after writes for the Stop rule), scoped to sessions whose cwd holds a `.mcp.json` with a rusty server, failing open on an unreadable transcript; the Stop rule refuses once through `stop_hook_active`; one decision page per question, a topic's history being the decisions linked to its page; the tools `brain_ask`, `brain_decide`, `brain_follow_up`, `brain_no_decision`, `brain_due`; the edges consulted, supersedes, follows_up
created: 2026-09-03
---

# The brain loop: spec

## Intent

Ask, Decide, Follow up. An agent consults the brain before it decides, records the
decision as a page linked to what it rested on, and comes back to say how it went. Chad,
2026-09-03 15:45: "we need to look at defining a system in which you should always use
the brain for consulting and also ensure we are interacting with the brain (updates and
such) ... Ask->Decide->Follow up". The hooks make the first two steps happen in a session
wired to Rusty; the follow-up date makes the third one due.

## Scope

- In: the `decision` page type under `decisions/`; the consultation record in
  `rusty.db`; the five tools; typed edges in `brain_graph` and the app's graph view; a
  Decisions view; the CLI mirror (`brain ask|decide|follow-up|no-decision|due`); the two
  hooks shipped in `crates/rusty-cli/hooks/` with `rusty-cli hooks install|uninstall|status`;
  the seed skill `ask-decide-follow-up`; the ADR; the due list in `/brief`.
- Out (named seams): mining decisions out of archived transcripts; enforcement on
  sessions not wired to Rusty; a daily view of its own (the Decisions view carries the due
  list); any cloud service.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `brain_ask` is called with a question, the server shall return a consultation: ranked pages from keyword and semantic search, the decisions touching the topic with their status, the follow-ups due, and a consultation id recorded in `rusty.db` with the question and the hits. | core test; smoke flow |
| REQ-002 | WHEN `brain_decide` is called with a consultation id, the server shall write a `decision` page under `decisions/` carrying the question, the choice, the rationale, the alternatives, links to every consulted page, `follow_up_by` and `status: decided`, and add a timeline entry to each linked page. | core test on a vault fixture |
| REQ-003 | WHEN `brain_follow_up` is called on a decision, the server shall append the outcome and set the status to kept, revised or superseded (with a link to the successor), clearing or rescheduling the date. | core test |
| REQ-004 | WHEN `brain_graph` is called, decision pages shall carry typed edges (consulted, supersedes, follows_up), and the app's graph view shall draw them behind a filter. | core test; the graph scene |
| REQ-005 | WHEN a session in a repository wired to Rusty makes its first file write with no non-error `brain_ask` in its transcript, the PreToolUse hook shall block once and name the tool. | hook corpus test |
| REQ-006 | WHEN such a session stops after writing files with neither a `brain_decide` nor a `brain_no_decision` record, the Stop hook shall refuse once and name the missing record. | hook corpus test |
| REQ-007 | WHEN the brief runs (`/brief`) and the Decisions view opens, follow-ups due or overdue shall be listed, and `rusty-cli brain ask`, `decide`, `follow-up`, `no-decision` and `due` shall mirror the tools. | CLI by reading; the skill edit; the view scene |
| REQ-008 | WHEN the loop ships, the seed skill `ask-decide-follow-up` shall carry it for agents, and an ADR shall record the decisions. | doc review |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The hooks ship in this repo (`crates/rusty-cli/hooks/`, embedded in the binary) and `rusty-cli hooks install` writes them to `~/.rusty/hooks/` and wires `~/.claude/settings.json` idempotently; omarchy-ops calls it. | The seal. A consumer of the public repo gets the loop with the binary. | Hooks in omarchy-ops alone. |
| 2 | Receipts by transcript scan: the needle for a write is a `mcp__rusty__brain_ask` tool use whose result was not an error; for a stop, a `mcp__rusty__brain_decide` or `mcp__rusty__brain_no_decision` after any write. Scoped to a cwd with a `.mcp.json` naming a rusty server. Unreadable transcript, no jq: fail open. | The seal; the server never sees a session id. | Server-side receipts keyed by session. |
| 3 | The Stop rule refuses once (`stop_hook_active` passes the second attempt); `brain_no_decision` with a reason is the honest exit. | The seal. | A hard refusal. |
| 4 | One decision page per question; a topic's history is the decisions linked to its page (each consulted page gets a timeline entry). | The seal. | One page per topic with a log. |
| 5 | Only `brain_ask` counts as consultation, since it is the call that records the receipt. | The seal. | Counting any brain read. |
| 6 | Typed edges come from the decision page's frontmatter (`consulted`, `supersedes`, `follows_up`) at graph time; the same targets are wikilinks in the body, so the vault's link rules and backlinks see them too. | The vault stays the truth; the index is rebuildable. | A typed link table. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-018-brain-loop.md`
- Intake: none
- Design references: rustal's `enforce-docs-before-code.sh` and `lib-hook-helpers.sh`
  (the transcript predicates), `crates/rusty-core/src/brain/mod.rs` (`create_page`,
  `set_property`, `add_timeline`, `search_hybrid`, `graph`), `vault.rs` (`TYPE_DIRS`),
  `engine/db.rs` (`migrate`), `crates/rusty-cli/src/main.rs` (`run_brain`), the app's
  `GraphView.qml` and the view tabs
- Architecture: `AD-rusty-brain-is-the-project-memory-001`, `AD-rusty-files-are-the-truth-001`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled; sealed |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
