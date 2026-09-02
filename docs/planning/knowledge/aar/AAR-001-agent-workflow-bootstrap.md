---
title: AAR-001-agent-workflow-bootstrap
pipeline_id: 0f74e76f-64ac-4bf3-b728-50ba62bdbd97
ticket: TICKET-001
submitted: 2026-09-02
---

# AAR-001-agent-workflow-bootstrap

## Recall log

- Four generations of the same workflow exist on this box (aic, rustal, rustal-workflow,
  OmarchyGS); rustal-workflow's spec 0001 names what each contributed. OmarchyGS is the
  closest fit for a Qt/QML product with a Rust core and file-based state.
- The brain already held nothing on "omarchy bbs", "codegraph" or "openviking" beyond a
  conversation page; the repos themselves were the sources.

## 1. Outcomes

- `REQ-001` PASS: `scripts/check-pipeline.sh` passes.
- `REQ-002` PASS: `bin/gate.sh --diff` green, receipt written and verified.
- `REQ-003` PASS: nine hook self-tests with the expected exit codes.
- `REQ-004` PASS: CodeGraph 1.5.0 installed under `.dev/`, index built, explore answers.

## 2. What went well

- Reading the three ancestors before writing produced a merge, not a fourth invention: the
  phases, the ledger shape and the receipt idea are theirs verbatim.
- The hooks came with their own tests in the same hour; the `--no-verify` refusal and the
  docs-only allowance were exercised before the first real commit.

## 3. What went poorly

- Two silent failures in shell (`pipefail` on an empty glob; index before init) were only
  found by running the scripts, not by reading them.

## 4. Surprises

- The public repo's `.gitignore` still ignored `.mcp.json`, a v2 habit that would have
  hidden the project's MCP wiring from every clone.

## 5. Lessons

- `PR-rusty-glob-loops-not-ls-pipes-001`: count or iterate files with a glob loop guarded
  by `[[ -f ]]`, never `ls | wc -l`, in a script under `set -o pipefail`.
- `AD-rusty-local-work-record-001` (recorded in the register): repo files are the work
  state; the receipt is the proof; CI is a second witness.
- `AD-rusty-brain-is-the-project-memory-001`: the brain plays the OpenWiki and OpenViking
  role for this project; their tiered-context and session-to-memory ideas go on the
  brain's roadmap.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| recall | 30m | 40m |
| plan and design | 30m | 20m |
| implement | 60m | 50m |
| inspect and validate | 20m | 15m |
| complete | 15m | 15m |
