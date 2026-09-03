---
title: TICKET-007-openwiki-in-the-workflow
status: done
ticket_number: 007
type: workflow
created: 2026-09-03
closed: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/completed/openwiki-in-the-workflow.spec.md
---

# TICKET-007-openwiki-in-the-workflow

## Summary

Adopt OpenWiki, project-local and pinned, as the generated engineering documentation of
this repository, and make the workflow require it: every pipeline reconciles the wiki at
Phase 5 through the `openwiki` skill and its MCP lifecycle, and a completed pipeline
cannot be delivered without a completion receipt bound to the worktree, the way the gate
receipt binds the tests.

## Why

Chad, 2026-09-03: "can we go ahead and set up open code wiki on here regardless and
force it to be used for documentation. i think it has great success in conjunction with
codegraph. find time to add it + force it in the work flow". OmarchyGS runs the same pair
(CodeGraph at design and inspect, OpenWiki at complete); TICKET-001 set OpenWiki aside
and this ticket reverses that call for documentation. The brain keeps the memory role.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | `scripts/setup-pipeline-tools.sh` shall install OpenWiki 0.3.3 at a pinned commit under `.dev/pipeline-tools/`, built from a frozen lockfile with a verified pnpm, with two local patches (no hosted refresh workflow, local-only guidance), and write a provenance file; `scripts/check-pipeline-tools.sh` shall fail when any of it is missing or changed. | setup run; check run; tamper test on the provenance |
| REQ-002 | The OpenWiki MCP server shall be wired for Claude Code (`.mcp.json`) and Codex (`.codex/config.toml`) through `scripts/mcp-openwiki.sh`, which refuses to start when the tools are not prepared. | `tools/list` over stdio returns the four lifecycle tools; the script exits 1 without the build |
| REQ-003 | WHEN `openwiki_finish` returns `complete`, a PostToolUse hook shall write `.git/rusty-openwiki-receipt` bound to the worktree fingerprint; `bin/gate.sh --verify` shall report it. | hook self-test with a synthetic finish response; verify output |
| REQ-004 | WHEN a commit carries a completed spec (`docs/planning/pipeline/completed/*.spec.md`) and no waiver is in force, the commit gate hook and the git pre-commit shim shall refuse it unless the OpenWiki receipt matches the worktree. | hook self-tests (block and allow); shim test |
| REQ-005 | The constitution, the workflow skill's phases, the agent guides and `scripts/check-pipeline.sh` shall require the lifecycle at Phase 5 and its recall at the start of a pipeline, and shall verify the wiring. | doc review; check run |
| REQ-006 | The repository shall carry a generated `openwiki/` for its current state (an `init` run finished as `complete`), with a hand-written brief in `openwiki/INSTRUCTIONS.md`. | the run's receipt; `openwiki/.last-update.json` status complete; pages read back |

## Scope

- In: the tool install and checks, the MCP wiring, the skill, the receipt and its
  enforcement, the rules, the first wiki.
- Out: a Stop-hook claim check (M9, later); OpenWiki's own model-driven modes (`init`,
  `update`, `ingest`, cron), which would send code to a provider; OpenViking.

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/openwiki-in-the-workflow.spec.md`
- Related docs: `CONSTITUTION.md`, `.claude/skills/rusty-workflow/references/phases.md`
- Reference: OmarchyGS's `scripts/setup-pipeline-tools.sh`, `.agents/skills/openwiki`
- Follow-ups opened: the Stop-hook claim check stays on the roadmap (M9)
