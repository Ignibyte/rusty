---
title: OpenWiki in the workflow
pipeline_id: 3f1c7d2a-9b64-4e0f-8a5d-2c7e6b1f4d90
status: Phase 5 — Complete PASS
ticket: TICKET-007
ticket_doc: docs/planning/tickets/closed/TICKET-007-openwiki-in-the-workflow.md
aar: docs/planning/knowledge/aar/AAR-007-openwiki-in-the-workflow.md
sealed: 2026-09-03, Chad: "set up open code wiki on here regardless and force it to be used for documentation ... find time to add it + force it in the work flow"
created: 2026-09-03
---

# OpenWiki in the workflow: spec

## Intent

Generated engineering documentation that a pipeline cannot skip. OpenWiki (langchain-ai,
0.3.3) runs project-local as an MCP server; the agent that finishes a pipeline authors
the wiki pages with its own tools, grounded in claims with source evidence, and the
lifecycle's `complete` result becomes a receipt the delivery gate checks.

## Scope

- In: `scripts/setup-pipeline-tools.sh` (pinned clone, two patches, verified pnpm,
  frozen install, build, provenance), `scripts/check-pipeline-tools.sh` (the checks),
  `scripts/mcp-openwiki.sh`, `.mcp.json` and `.codex/config.toml`, the `openwiki` skill,
  the PostToolUse receipt hook, the receipt in `bin/lib-gate.sh` and `bin/gate.sh
  --verify`, the completion rule in the commit gate hook and the pre-commit shim, the
  constitution and phases, the agent guides, `scripts/check-pipeline.sh`, the first wiki.
- Out (named seams): the Stop-hook claim check (M9); OpenWiki's model-driven modes and
  cron; OpenViking.

## Acceptance criteria (EARS)

REQ-001 to REQ-006 as in the ticket.

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | OpenWiki is used only through its MCP lifecycle, with the host agent (Claude Code, or Codex) authoring pages; its model-driven `init`/`update` and the scheduled workflow are patched out. | Nothing leaves the machine without a setting (§10); the host already has the code in view. | letting OpenWiki call OpenAI itself |
| 2 | Same pins as OmarchyGS (commit `a525ed88`, pnpm 10.33.2 by SHA-512), built from the frozen lockfile, with a provenance file the check script verifies. | Known-good on this box; a changed dependency fails closed. | latest release; a global install |
| 3 | The completion receipt is the same shape as the gate receipt and lives next to it (`.git/rusty-openwiki-receipt`), written by a PostToolUse hook from a real `openwiki_finish` result, never by hand. | One fingerprint, one verify path. | a status flag in the notes |
| 4 | Enforcement is at delivery: a commit that carries a completed spec needs a matching completion receipt (hook and shim), unless a waiver is in force. | A pipeline's last act is the commit; that is where the gate already stands. | a Stop hook (later, M9) |
| 5 | Both `AGENTS.md` and `CLAUDE.md` carry OpenWiki's managed block, because both are hosts here (the lifecycle writes the section into `AGENTS.md` and a pointer into `CLAUDE.md`); `scripts/check-pipeline.sh` keeps the two files identical outside that block. | Claude Code is the first host, Codex the second. | the Codex-only patch OmarchyGS applies |
| 6 | The brain stays the project's memory; the wiki documents the code. Recall reads both. | TICKET-001's decision stands for memory; Chad's request adds documentation. | replacing the brain's role |

## Linked artifacts

- Ticket: TICKET-007
- Design references: OmarchyGS `scripts/setup-pipeline-tools.sh`,
  `scripts/check-pipeline-tools.sh`, `.agents/skills/openwiki/SKILL.md`,
  `.codex/hooks/record-pipeline-tool-use.sh`; OpenWiki's `session-manager.ts`
- Architecture: `CONSTITUTION.md` §15, §18

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | this spec, the ticket, the AAR opened | Chad's words on the seal line |
| 2 Design | the file manifest and the receipt contract in the notes | design actionable |
| 3 Implement | the manifest; the tools prepared on this box | `scripts/check-pipeline-tools.sh` green |
| 3.5 Inspect | ledger; hook self-tests | confirmed findings resolved |
| 4 Validate | the first `init` run finished complete; the receipt; `bin/gate.sh --diff` | receipt matches |
| 5 Complete | audit, docs, AAR, register, brain capture, archive | pair archived |
