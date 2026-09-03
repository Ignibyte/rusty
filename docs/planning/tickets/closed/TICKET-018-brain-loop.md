---
title: TICKET-018-brain-loop
status: done
ticket_number: 018
type: workflow
created: 2026-09-03
closed: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/active/brain-loop.spec.md
---

# TICKET-018-brain-loop

## Summary

The brain loop, Ask, Decide, Follow up: a `decision` page type with typed edges, three MCP
tools that write it, receipts the hooks can check, a skill, and a Decisions view. An agent
consults the brain before it decides, records the decision with what it was based on, and
comes back to say how it went.

## Why

Today the brain is consulted when a skill says so (`/recall`, `/brief`) and written when a
session ends (the transcript archive). Nothing makes a consultation happen before a decision,
and nothing records the decision as a node linked to the pages it rested on, so the graph
holds facts and no reasoning. Rustal's pipeline proves the shape on code: a knowledge recall
before the first write, receipts a hook can check, and a capture rule that puts what was
learned where the next session finds it. This is the same loop for the brain.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `brain_ask` is called with a question, the server shall return a consultation: ranked pages from keyword and semantic search, the decisions touching the topic with their status, the follow-ups due, and a consultation id recorded in `rusty.db` with the question and the hits. | tool test |
| REQ-002 | WHEN `brain_decide` is called with a consultation id, the server shall write a `decision` page under `decisions/` carrying the question, the choice, the rationale, the alternatives, links to every consulted page, `follow_up_by` and `status: decided`, and add a timeline entry to each linked page. | tool test on a vault fixture |
| REQ-003 | WHEN `brain_follow_up` is called on a decision, the server shall append the outcome and set the status to kept, revised or superseded (with a link to the successor), clearing or rescheduling the date. | tool test |
| REQ-004 | WHEN `brain_graph` is called, decision pages shall carry typed edges (consulted, supersedes, follows up), and the app's graph view shall draw them behind a filter. | graph test; screenshot |
| REQ-005 | WHEN a session in a repository wired to Rusty makes its first file write with no `brain_ask` in its transcript, the PreToolUse hook shall block once and name the tool. | hook corpus test |
| REQ-006 | WHEN such a session stops after writing files with neither a `brain_decide` nor a `brain_no_decision` record, the Stop hook shall refuse once and name the missing record. | hook corpus test |
| REQ-007 | WHEN the daily brief runs (`/brief` and the daily view), it shall list follow-ups due or overdue, and `rusty-cli brain ask`, `decide`, `follow-up` and `due` shall mirror the tools. | CLI tests; skill review |
| REQ-008 | WHEN the loop ships, a store skill `ask-decide-follow-up` shall carry it for agents, and an ADR shall record the decisions listed under Notes. | doc review |

## Scope

- In: the page type, folder and frontmatter schema; the three tools and `brain_no_decision`; the receipts table; edges in `brain_graph`; the app's Decisions view and graph filter; the hooks, shipped in this repo's `.claude/hooks` for consumers and wired box-wide by omarchy-ops; the skill; the CLI.
- Out: mining decisions out of archived transcripts (later); enforcement on sessions not wired to Rusty; any cloud service.

## Notes

- Pipeline spec: docs/planning/pipeline/active/brain-loop.spec.md
- Decisions for the seal: where the hooks live (this repo for consumers, omarchy-ops for the box, or both); receipts by transcript scan, which rustal proves and which the server cannot do since it never sees the session id; how strict the Stop rule is (refuse once, with an explicit `brain_no_decision` as the honest way out); one decision page per question, or one per topic with a log; whether only `brain_ask` counts as consultation, since it is the call that records the receipt.
- Related docs: rustal's `enforce-docs-before-code.sh` and its capture rule (`/srv/stacks/rustal/CLAUDE.md`, Local knowledge base), omarchy-ops `auto-capture-conversation.sh`, `docs/architecture.md`, the brain tools in `crates/rusty-mcp/src/main.rs`, `brain_page_types`.
- Promoted from intake: none; drafted by the rustal session on 2026-09-03 from Chad's words at 15:45: "we need to look at defining a system in which you should always use the brain for consulting and also ensure we are interacting with the brain (updates and such). This will be a workflow similar to Plan->code->test->document building but its the brain with decision graphs from the MCP layer. Ask->Decide->Follow up".
- Delivered on 2026-09-03 with the rustal session's four answers as the seal (Chad at
  17:20: "lets run them all including 010"). The write hook is scoped to files under
  the working directory as well (the rustal session's review). The ADR:
  `docs/architecture/brain-loop.md`.
- Follow-ups opened: none. Seams: mining decisions out of archived transcripts; an
  indexed property when decisions count in the thousands.
