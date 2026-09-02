# Planning: the local work record

Spec-driven, phase-gated, evidence-recorded, entirely in this repo. The shape was merged on
2026-09-02 from three of Chad's projects: OmarchyGS (recall-first pipeline, local record,
gate receipt, CodeGraph at design and inspect), rustal-workflow (sealed specs, EARS
requirements, evidence policy) and aic (the AAR, one active pipeline). The rules are in
`CONSTITUTION.md`; the driving manual is `.claude/skills/rusty-workflow/`.

| Directory | Holds |
|---|---|
| `intake/` | Ideas not yet approved as work (`_templates/intake.md`). Promotion creates a ticket. |
| `tickets/` | `INDEX.md` (next number, open queue, closed list), `open/`, `closed/` (`_templates/ticket.md`). |
| `pipeline/` | The active spec/notes pair (`active/`, at most one), archived pairs (`completed/`), templates. `WAIVER.md` here means a small change is running without ceremony. |
| `knowledge/` | `INDEX.md`, the register of `PR-` prevention rules, `BF-` bug families and `AD-` architecture decisions; `aar/` holds each pipeline's after-action review. |
| `bulletins/` | Standing notices; a critical one blocks work until read. |

The product list stays in `ROADMAP.md`; a roadmap item is ticked by the pipeline that
delivers it. Design references live in `docs/design/`; standing architecture in
`docs/architecture.md` and `docs/architecture/`.

## Tools around the record

- `bin/gate.sh` runs the gate; `--diff` green writes `.git/rusty-gate-receipt`, bound to
  the worktree; `--verify` checks it. The pre-commit hook and the agent hooks refuse a
  commit of gated files without a matching receipt.
- CodeGraph (`scripts/setup-pipeline-tools.sh`, `scripts/codegraph.sh`, MCP server
  `codegraph`) supplies structural evidence at design and inspect.
- The `rusty` MCP server is the project's memory: lessons are captured into the brain at
  complete and recalled at the start of the next pipeline. OpenWiki and OpenViking were
  reviewed for this role and set aside; the brain already does the job locally.
