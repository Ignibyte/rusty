---
title: Agent workflow bootstrap
pipeline_id: 0f74e76f-64ac-4bf3-b728-50ba62bdbd97
status: Phase 5 — Complete PASS
ticket: TICKET-001
ticket_doc: docs/planning/tickets/closed/TICKET-001-agent-workflow-bootstrap.md
aar: docs/planning/knowledge/aar/AAR-001-agent-workflow-bootstrap.md
sealed: Chad, 2026-09-02: "make this workflow a bit more formal … merge a good workflow system"
created: 2026-09-02
---

# Agent workflow bootstrap: spec

## Intent

Merge the workflow shape from OmarchyGS, rustal-workflow and aic into this repo, with
file-based state and the tools that fit a public Rust and QML project, so every later
change runs through recall, plan, design, implement, inspect, validate, complete and
delivery with evidence in the repo.

## Scope

- In: `CONSTITUTION.md`; `AGENTS.md` and `CLAUDE.md`; `.claude/skills/rusty-workflow/`;
  `docs/planning/` (README, templates, indexes, register seeded with this session's
  lessons); `bin/gate.sh` and `bin/lib-gate.sh` with the receipt; three PreToolUse hooks in
  `.claude/settings.json`; `scripts/setup-pipeline-tools.sh`, `codegraph.sh`,
  `mcp-codegraph.sh`, `check-pipeline.sh`, `check-pipeline-tools.sh`; `.mcp.json` and
  `.codex/config.toml`; the git pre-commit shim.
- Out (seams): Stop-hook claim checking; CodeGraph and completion receipts under `.git`;
  `rw` enrollment; OpenWiki as the repo wiki; OpenViking as the context store.

## Acceptance criteria (EARS)

See the ticket's table (REQ-001 to REQ-004).

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | Work state lives in repo files; no work-state server. | The three ancestors that run this way (aic, OmarchyGS, rustal's own repo) work; `rw` needs rustal-brain, which is not on this box. Enrolling later is additive. | `rw` now |
| 2 | GitHub Actions stays as a second witness; the local receipt is the delivery proof. | Public repo with contributors; CI is normal there. OmarchyGS forbids hosted CI for a private product, a different situation. | local-only evidence |
| 3 | CodeGraph is the structural-evidence tool at design and inspect, project-local and pinned. | Reads Rust; already proven in OmarchyGS; MCP and CLI. QML and shell stay hand-inspected. | none |
| 4 | Rusty's brain is the project's recall and memory surface (lessons captured at complete, searched at recall). | OpenWiki generates a wiki from the repo with an LLM and provenance; OpenViking is a context database for agents. Both duplicate what the brain does for Chad, and the brain is the product. Their ideas (tiered L0/L1/L2 context, session-to-memory extraction) go on the roadmap for the brain itself. | OpenWiki lifecycle at complete; OpenViking server |
| 5 | Hooks refuse three things (phase, secrets, receipt) and nothing else in v1. | Small, testable, no transcript parsing. | porting OmarchyGS's Stop-claim hook |

## Linked artifacts

- Ticket: TICKET-001
- Architecture: `CONSTITUTION.md`, `docs/planning/README.md`
- References: `/srv/stacks/omarchy_gaming_system` (AGENTS.md, CONSTITUTION.md, `.agents/skills/omarchy-workflow`, ADR-0001), `/srv/stacks/rustal-workflow/docs/planning/specs/0001-birth.md` and `0002-workflow-process.md`, `/srv/stacks/aic/docs/pipeline/`, `~/git/codegraph`, `~/git/OpenViking`, `~/git/openwiki`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | this spec, the ticket | Chad's words above |
| 2 Design | the file list in scope | reviewed against the three ancestors |
| 3 Implement | the files | shell syntax clean |
| 3.5 Inspect | hook self-tests, structure check | all pass |
| 4 Validate | `bin/gate.sh --diff` green, receipt written and verified | receipt matches |
| 5 Complete | AAR, register seeded, roadmap and README updated | pair archived |
