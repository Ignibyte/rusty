---
title: Scripts as commands
pipeline_id: 0cbefd28-f640-49c3-a2cf-78bc996d66c2
status: Phase 4 — Validate PASS; Phase 5 — Complete all but the OpenWiki step, which needs the openwiki server a session inside this repository has. The pair stays here until that run leaves its receipt; then it archives and the ticket closes.
ticket: TICKET-010
ticket_doc: docs/planning/tickets/open/TICKET-010-scripts-as-commands.md
aar: docs/planning/knowledge/aar/AAR-010-scripts-as-commands.md
sealed: Chad, 2026-09-03 17:20, in the rustal session (relayed): "lets run them all including 010", with the four answers from the ticket's seal note: dispatch in the app binary (`rusty <name>` execs a store script before any GUI starts, `rusty-cli scripts run` sharing the same engine); `script_run` stays on the MCP surface for approved scripts only; scripts live inside skill directories (a script with no skill gets a skill), resolved by basename without the `.sh`; scripts are a section of the Skills tab
created: 2026-09-03
---

# Scripts as commands: spec

## Intent

A script in the store is a command: `rusty usb-reset` runs `dev-box-usb/usb-reset.sh`
from any terminal, the CLI lists, shows, creates, removes and runs scripts the way it does
skills, and the app shows, edits and runs them beside the skills. Chad, 2026-09-03 14:40:
"we could create a command line rusty for things like rusty <command> that are basically
shells. so i could do rusty usb etc. we could make the scripts editable".

## Scope

- In: the resolver in `rusty-core` (scripts inside skill directories, active and
  pending); `rusty <name> [args]` in the app binary before Qt starts; `rusty-cli scripts
  list|view|new|rm|path|edit|run`; the tools `script_list`, `script_view`,
  `script_update` and `script_run` (approved only); a Scripts section of the Skills tab
  with view, edit, scan and a Run that opens a terminal tab; the safety scan on a script's
  text; store commits.
- Out (named seams): a script language or templating; anything as root without
  `sudo -n`; a scheduler; a `scripts/` directory of its own (the seal put scripts beside
  their skills).

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `rusty <name> [args]` is invoked and `<name>` resolves to an active store script, the app binary shall exec that script with the arguments before any GUI starts, so the process ends with the script's status. | resolver test; a run of the built binary against a scratch store |
| REQ-002 | WHEN `<name>` is not a store script (or starts with `-`), `rusty` shall behave as today. | the same run: an unknown name opens no script (exit path unchanged by reading) |
| REQ-003 | WHEN `rusty-cli scripts list\|view\|new\|rm\|path\|edit\|run <name>` is invoked, the CLI shall act on the store's scripts with the same store commit behaviour as `skills`. | resolver and writer tests; the CLI by reading |
| REQ-004 | WHEN the store holds scripts, the Skills tab shall list them in a Scripts section with view, edit, scan and a Run that opens a terminal tab running the script. | the `view:skills` scene with a seeded script |
| REQ-005 | WHEN a script is created or edited through Rusty, the store shall commit it, and a pending script (in staging) shall not run until its skill is approved. | `a_pending_script_does_not_run` |
| REQ-006 | WHEN a skill directory contains an executable `*.sh` file, the resolver shall offer it under the file's basename without the suffix (`skill/name` disambiguates a clash). | `scripts_resolve_by_basename_and_disambiguate` |
| REQ-007 | The MCP server shall expose `script_list`, `script_view`, `script_update` and `script_run`, with `script_run` refused for pending scripts. | router names; smoke flow |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The app binary dispatches: `rusty <name>` resolves against the default store (`~/.rusty/skills`, or `RUSTY_SKILLS`) and execs before Qt loads; the CLI's `scripts run` and the app share the core resolver. | The seal. Nothing starts a GUI to run a shell script. | A shim; `rusty-cli <name>`. |
| 2 | Scripts live inside skill directories; a script created without a skill gets one named after it. | The seal; the skill carries the procedure and the script carries the steps. | A `scripts/` directory. |
| 3 | A pending script never runs (the resolver marks it; `run` and `script_run` refuse); approving the skill approves its scripts, and the safety scan that reads a skill reads a script's text too. | REQ-005; one gate for both. | A scan of scripts alone. |
| 4 | `script_run` on the MCP surface runs an approved script synchronously with a cap of sixty seconds and returns status, stdout and stderr. | The seal (approved only); an agent needs the output. | Fire and forget. |
| 5 | The app runs a script in a terminal tab whose program is `run:<path>`: the shell runs the script and stays open after it. | The output is visible and the tab is a shell afterwards. | Capturing output into a pane. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-010-scripts-as-commands.md`
- Intake: none
- Design references: `crates/rusty-core/src/skills/mod.rs` (`SkillsManager`, `scan_skill_md`,
  `create_skill`, `approve`, `git_commit_blocking`), `crates/rusty-app/src/main.rs`,
  `crates/rusty-app/src/terminals.rs` (`command_for`), `crates/rusty-cli/src/main.rs`
  (`run_skills`), `crates/rusty-app/qml/SkillsPage.qml`
- Architecture: `AD-rusty-mcp-only-back-end-001` (the app reaches the store's scripts
  through the tools; the dispatch reads the store's files to exec one, which is the same
  boundary the CLI has)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled; sealed |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
