---
title: OpenWiki in the workflow: notes
pipeline_id: 3f1c7d2a-9b64-4e0f-8a5d-2c7e6b1f4d90
---

# OpenWiki in the workflow: running notes

## Phase 1: Plan

- Recall: register `AD-rusty-brain-is-the-project-memory-001` (the brain plays the
  OpenWiki role for memory; this pipeline adds OpenWiki for documentation and leaves that
  decision for memory), `AD-rusty-local-work-record-001`, `PR-rusty-glob-loops-not-ls-pipes-001`.
  Completed notes: AAR-001 (the workflow), AAR-002 (the workspace). OmarchyGS read on
  2026-09-03: its setup script (pins, patches, provenance), check script, MCP wrapper,
  skill, PostToolUse receipt hook and Stop-hook claim check, and its generated
  `openwiki/` (brief, quickstart, seven pages, `.claims/`, `.last-update.json`).
  OpenWiki source read: `HostSessionManager.begin` (repo setup writes the agent-guide
  section, `init` backs up the old wiki, prepares the skeleton), `finish` (removes the
  temporary files, finalises index and page sources from claim evidence, writes
  `.last-update.json` with `status: complete`), the four tools.
- Decisions: the six in the spec.
- Seal: Chad's request, on the spec.

## Phase 2: Design

- Architecture and data flow: the host agent drives four MCP tools (`openwiki_begin`,
  `openwiki_inspect_claims`, `openwiki_resolve_claims`, `openwiki_finish`) served by
  `node .dev/pipeline-tools/openwiki/dist/cli/cli.js mcp --host <claude|codex>` through
  `scripts/mcp-openwiki.sh`. Pages are plain markdown under `openwiki/` that the agent
  writes with its own tools; claims with `repo://path#Lx-Ly` evidence are recorded
  through `openwiki_resolve_claims` and become each page's `sources` at finish. The
  finish result (`status: complete`) reaches the PostToolUse hook, which writes
  `.git/rusty-openwiki-receipt` (`version`, `fingerprint`, `pipeline`, `at`) with the
  same fingerprint as the gate receipt. The commit gate hook and the pre-commit shim
  require that receipt to match whenever a completed spec is staged.
- File manifest:

| File | Purpose |
|---|---|
| `scripts/setup-pipeline-tools.sh` | OpenWiki section: pinned clone, patches, verified pnpm, frozen install, tsc build, assets, provenance; idempotent |
| `scripts/check-pipeline-tools.sh` | OpenWiki checks: version, commit, patches, build, provenance |
| `scripts/mcp-openwiki.sh` | the MCP server wrapper; refuses to start when unprepared |
| `scripts/check-pipeline.sh` | wiring checks: wrapper, skill, both MCP configs, brief; `AGENTS.md` equals `CLAUDE.md` |
| `.mcp.json`, `.codex/config.toml` | the `openwiki` server |
| `.claude/skills/openwiki/SKILL.md` | the lifecycle as the host runs it |
| `.claude/hooks/record-pipeline-tool-use.sh`, `.claude/settings.json` | the completion receipt on a real finish |
| `bin/lib-gate.sh`, `bin/gate.sh` | receipt path, verify, report |
| `.claude/hooks/enforce-commit-gate.sh` | the completion rule |
| `CONSTITUTION.md`, `.claude/skills/rusty-workflow/{SKILL.md,references/phases.md}`, `AGENTS.md`, `CLAUDE.md`, `README.md`, `ROADMAP.md`, `docs/planning/README.md` | the rule, the phase step, the recall step |
| `openwiki/INSTRUCTIONS.md`, `.openwikiignore`, `.gitignore` | the brief, what the wiki ignores, the temporary files |
| `openwiki/*.md`, `openwiki/.claims/`, `openwiki/.last-update.json` | the first wiki |

- Store consequences: none in Rusty's store. The wiki is repository files.
- Tool contract: no Rusty tool changes.
- Regression plan: REQ-001 setup and check runs plus a provenance tamper; REQ-002 a
  `tools/list` over stdio and the unprepared exit; REQ-003 and REQ-004 hook self-tests
  with synthetic inputs (as TICKET-001 did); REQ-005 doc review and `check-pipeline`;
  REQ-006 the run itself.
- Risks: network for the clone and the packages (the pins fail closed on a mismatch);
  the agent-guide section is written into gated files at begin, so the gate reruns
  after the lifecycle; this session cannot load the new MCP server, so the first run
  drives the server over stdio from a scratch client and feeds the real finish result
  to the same hook.
- CodeGraph evidence: not applicable (no Rust changes).

## Phase 3: Implement

- Built: the manifest. `scripts/lib-openwiki.sh` holds the pins, the two patches, the
  verified pnpm bootstrap, the frozen install, the build and the provenance
  (`openwiki_build`, `openwiki_verify`); `setup-pipeline-tools.sh` and
  `check-pipeline-tools.sh` source it (`check-pipeline-tools.sh openwiki` checks the
  wiki alone, which the wrapper asks before starting). `scripts/mcp-openwiki.sh` serves
  the lifecycle with `OPENWIKI_HOST` (claude by default, codex from the Codex config).
  The receipt: `rusty_openwiki_receipt_path` and `rusty_verify_openwiki_receipt` in
  `bin/lib-gate.sh`, reported by `bin/gate.sh --verify`, written by
  `.claude/hooks/record-pipeline-tool-use.sh` (PostToolUse on `openwiki_finish`, only
  on `complete`), required by `enforce-commit-gate.sh` and the pre-commit shim when a
  completed spec is staged and no waiver is in force. The rules: constitution §15 and
  §18 with an amendment log entry, the phases (recall reads the wiki; Phase 5 runs the
  skill; delivery verifies both receipts), the workflow skill, both agent guides, the
  README, the roadmap, the planning README. The `openwiki` skill. The brief in
  `openwiki/INSTRUCTIONS.md`, `.openwikiignore`, the temporary files ignored.
- Deviations: the Codex-only agent-guide patch OmarchyGS applies was not taken; both
  guides carry the managed block, and the lifecycle writes the full section into
  `AGENTS.md` and a pointer into `CLAUDE.md`, so `scripts/check-pipeline.sh` compares
  the two files with that block stripped. The clone can come from a local mirror
  (`OPENWIKI_MIRROR`) for an offline setup. This session could not load the new MCP
  server as a tool, so the first runs were driven by a scratch stdio client
  (`scratchpad/wiki/mcp_run.py`) that fed the real finish result to the same hook.
- Fast gate: not applicable (no Rust changed); `bash -n` on every script, the checks.

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | `AGENTS.md` and `CLAUDE.md` differ inside OpenWiki's managed block by design; the identity check failed after the first run | medium | fixed: the check strips the block (`strip_managed`) |
| 2 | correctness | an evidence range past a file's end fails the whole `resolve_claims` batch (`invalid_input: Evidence does not resolve`) | low | ranges clamped; the skill says to read the file first |
| 3 | data safety | the lifecycle writes only under `openwiki/` and the managed blocks; `init` backs the old wiki up and rolls back on failure; no provider is ever called | ok | verified by reading `session-manager.ts` and the run |
| 4 | secrets | the pnpm tarball is verified by SHA-512, the checkout by commit, the build by provenance; telemetry off through `DO_NOT_TRACK` and `OPENWIKI_TELEMETRY_DISABLED` | ok | verified by the check script and a provenance tamper test |
| 5 | evidence | a synthetic finish response must never leave a receipt behind | ok | the self-test's receipt was removed before the real run; the real receipt names the pipeline |
| 6 | keyboard, theme | not applicable | | |
| 7 | prose | scripts, skill, wiki pages and rules read against `no-ai-slop` | ok | clean |

Hook self-tests (all as expected): finish not complete (no receipt), another tool (no
receipt), finish complete (receipt written, `--verify` reports it), commit gate with a
completed spec staged and a matching receipt (allow), with the receipt removed (block),
the pre-commit shim in the same state (block), under a waiver (allow).

## Phase 4: Validate

- Tests run (commands and output): `scripts/setup-pipeline-tools.sh` built OpenWiki
  0.3.3 at `a525ed88` from the local mirror ("copy-visualize-assets: copied 1 asset",
  "built"); `scripts/check-pipeline-tools.sh` printed "pipeline tools ready";
  `scripts/check-pipeline-tools.sh openwiki` printed the version, commit and "provenance
  matches"; with the provenance file moved aside, `scripts/mcp-openwiki.sh` exited 1
  with "OpenWiki's build provenance does not match its pins"; the scratch client's
  `tools/list` returned `openwiki_begin`, `openwiki_inspect_claims`,
  `openwiki_resolve_claims`, `openwiki_finish` from `openwiki 0.3.3`;
  `scripts/check-pipeline.sh` passed.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: GATE GREEN [diff]; fmt, clippy (-D warnings), tests, doc, shell-syntax, secrets (134 gated files), whitespace all ok; receipt at 04:56:44Z.
- Smoke evidence: the `init` run (run `79f033ad`) recorded 29 claims over seven pages
  with `repo://path#Lx-Ly` evidence and finished `status: complete`; finish wrote
  `openwiki/index.md`, the page `sources`, `openwiki/.claims/` and
  `openwiki/.last-update.json` (`"command": "init"`, `"status": "complete"`,
  `"model": "host-agent/claude"`), removed `_plan.md`, and the hook wrote
  `.git/rusty-openwiki-receipt` naming pipeline `3f1c7d2a`. An `update` run after the
  check fix finished the same way: run 8ff86ac9 finished status complete at 04:58:47Z and the hook wrote the receipt. `bin/gate.sh --verify` reports both
  receipts matching.
- Skips or pre-existing failures: none.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (setup, check, tamper test); REQ-002 PASS
  (`tools/list`, the unprepared exit); REQ-003 PASS (hook self-tests, `--verify`);
  REQ-004 PASS (commit gate, shim and waiver self-tests); REQ-005 PASS (constitution,
  phases, skill, guides, README, roadmap; `check-pipeline`); REQ-006 PASS (the runs, the
  receipt, the pages read back).
- Docs: README, ROADMAP (M9), `docs/planning/README.md`, the constitution's amendment
  log, this pair.
- AAR: `docs/planning/knowledge/aar/AAR-007-openwiki-in-the-workflow.md`.
- Brain capture: timeline entry on `projects/rusty-v3`.
- Archive: this pair lives in `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-03 | the guides differ inside the managed block | PR-rusty-openwiki-managed-block-001 |
| 2026-09-03 | evidence ranges must exist | PR-rusty-openwiki-evidence-ranges-001 |
| 2026-09-03 | OpenWiki at complete, host-authored | AD-rusty-openwiki-for-documentation-001 |
