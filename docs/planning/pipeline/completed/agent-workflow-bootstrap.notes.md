---
title: Agent workflow bootstrap: notes
pipeline_id: 0f74e76f-64ac-4bf3-b728-50ba62bdbd97
---

# Agent workflow bootstrap: running notes

## Phase 1: Plan

- Recall: read rustal-workflow's sealed specs 0001 and 0002 (evidence policy, receipt,
  EARS), rustal's planning templates, aic's phase-by-phase contract and ticket contract,
  OmarchyGS's AGENTS.md, CONSTITUTION.md, ADR-0001, the omarchy-workflow skill and its
  phases.md, its pipeline templates, knowledge register, gate modes and hooks; the
  codegraph README (languages, CLI); OpenViking's README (viking:// filesystem, L0/L1/L2
  tiers, session-to-memory); openwiki's README.
- Decisions: the five in the spec.

## Phase 2: Design

- File manifest: as listed in the spec's scope. The receipt fingerprint covers
  `crates`, `Cargo.*`, `bin`, `scripts`, `omarchy`, `packaging`, `.claude`, `.codex`,
  `.mcp.json`, `.github`, and the three law files; `docs/**` and the roadmap are exempt.
- CodeGraph evidence: not applicable (no Rust changed in this pipeline).

## Phase 3: Implement

- Built: everything in the manifest. `.mcp.json` un-ignored (it was in `.gitignore` from
  the v2 days) so the project wiring is tracked.
- Deviations: `scripts/check-pipeline.sh` first exited silently on an empty active folder
  (`ls | wc -l` under `pipefail`); replaced with a glob loop. `codegraph index` needs
  `codegraph init` first; the setup script now runs it.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | structure check silent exit | medium | fixed (glob loop) |
| 2 | correctness | setup script indexed before init | low | fixed |
| 3 | secrets | the secrets self-test needs a key-shaped string; storing one would trip the gate | note | assembled at runtime in the test (`PR-omarchy-bbs-secret-fixtures-must-not-match-source-001` applied) |

Hook self-tests (all as expected): phase gate blocks a gated path with no pipeline (2),
allows docs (0), allows under a waiver (0); secrets blocks a runtime-assembled key (2),
allows plain text (0); commit gate blocks `--no-verify` (2), ignores non-commits (0),
blocks gated files staged without a receipt (2), allows docs-only (0).

## Phase 4: Validate

- Gate run: `bin/gate.sh --diff` on 2026-09-02 23:42Z: fmt, clippy, test (211 + 7 + 3 + 1
  + 7 passed), doc, shell-syntax, secrets (117 gated files scanned), whitespace all ok;
  `GATE GREEN [diff]`; receipt written; `bin/gate.sh --verify` reports a match.
- `scripts/check-pipeline.sh`: passed.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (structure check), REQ-002 PASS (gate run and verify),
  REQ-003 PASS (nine self-tests), REQ-004 PASS (setup, index, explore recorded below).
- Docs: README (workflow section), ROADMAP (workflow line), this pair.
- AAR: `docs/planning/knowledge/aar/AAR-001-agent-workflow-bootstrap.md`.
- Brain capture: timeline entry on the project page.
- Archive: this pair lives in `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-02 | `ls … \| wc -l` under `set -o pipefail` exits a script silently when the glob is empty | `PR-rusty-glob-loops-not-ls-pipes-001` |
