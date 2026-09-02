---
title: <TITLE>
pipeline_id: <uuid4>
status: Phase 1 — Plan: in progress
ticket: TICKET-<number>
ticket_doc: docs/planning/tickets/open/TICKET-<number>-<slug>.md
aar: docs/planning/knowledge/aar/AAR-<number>-<slug>.md
sealed:
created: YYYY-MM-DD
---

# <TITLE>: spec

## Intent

<What this pipeline ships and why now.>

## Scope

- In:
- Out (named seams, not forgotten):

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `<trigger>`, the system shall `<observable behaviour>`. | `<test, command, smoke, screenshot, review>` |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | | | |

## Linked artifacts

- Ticket:
- Intake:
- Design references:
- Architecture:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled; seal when the spec changes what Rusty is |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
