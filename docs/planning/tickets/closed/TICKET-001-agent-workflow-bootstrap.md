---
title: TICKET-001-agent-workflow-bootstrap
status: done
ticket_number: 001
type: workflow
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/agent-workflow-bootstrap.spec.md
---

# TICKET-001-agent-workflow-bootstrap

## Summary

Give the repo a spec-driven, phase-gated, evidence-recorded workflow: constitution, agent
guide, driving skill, planning record, gate with a worktree-bound receipt, agent hooks,
CodeGraph project-local.

## Why

Chad, 2026-09-02: "we probably need to make this workflow a bit more formal. Look at rustals
and aic workflow. its spec driven … look over at the omarchy_bbs system as well. it uses a
bit easier tools like codegraph and we should look at openviking. then sort of merge a good
workflow system."

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The repo shall carry a constitution, an agent guide (AGENTS.md and CLAUDE.md), a workflow skill with a per-phase contract, and templates for intake, ticket, spec, notes and AAR. | `scripts/check-pipeline.sh` |
| REQ-002 | `bin/gate.sh --diff` shall run fmt, clippy, tests, docs, shell syntax, secrets and whitespace checks and on green write a receipt bound to HEAD and every gated file; `--verify` shall fail when any gated file changes. | run recorded in notes |
| REQ-003 | WHEN an agent edits a gated path outside an implementing pipeline and without a waiver, writes credential-looking bytes, or commits gated files without a matching receipt, the hooks shall refuse with the reason. | hook self-tests recorded in notes |
| REQ-004 | The repo shall install CodeGraph pinned and project-local with an MCP entry for Claude Code and Codex and a CLI wrapper. | `scripts/check-pipeline-tools.sh`; an explore query |

## Scope

- In: the record, the gate, the hooks, the tools, the register seeded with this session's lessons.
- Out: Stop-hook claim checking, tool receipts bound to the worktree (OmarchyGS has both; here they are a named seam), rw enrollment (needs rustal-brain on the box), OpenWiki and OpenViking as dependencies.

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/agent-workflow-bootstrap.spec.md`
- Related docs: `CONSTITUTION.md`, `docs/planning/README.md`
- Follow-ups opened: none
