---
title: Scripts as commands
pipeline_id: 03f21d1e-614d-4710-9298-72b4dd6f8851
status: Phase 1 — Plan PASS; awaiting Chad's seal before Phase 2 — Design
ticket: TICKET-010
ticket_doc: docs/planning/tickets/open/TICKET-010-scripts-as-commands.md
aar: docs/planning/knowledge/aar/AAR-010-scripts-as-commands.md
sealed:
created: 2026-09-03
---

# Scripts as commands: spec

## Intent

Scripts in Rusty's store become commands a person runs by name from any terminal
(`rusty usb`), lists, views, creates and removes with the CLI the way skills are, and
reads and edits in the app beside the skills, with a Run action. The first script exists
already: `usb-reset.sh` beside the `dev-box-usb` skill, linked by hand into
`~/.local/bin` after the third xHCI reset of the week. Why now: Chad, 2026-09-03 14:40, to
the rustal session: "we should probably create a script i can just run. that makes me
think we could create a command line rusty for things like rusty <command> that are
basically shells. so i could do rusty usb etc. we could make the scripts editable and
viewable here as well."

## Scope

- In: the resolver and its entry points, the store layout for scripts, `rusty-cli
  scripts`, the app view with Run, the MCP tools, the review and approve gate for
  scripts.
- Out (named seams, not forgotten): a script language or templating; anything that runs
  a script as root without `sudo -n`; a scheduler; the repo's own `rusty-session`,
  which stays an installed file and not a store script.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `rusty <name> [args...]` is invoked and `<name>` is a script in the store, the app binary shall exec that script with the arguments, without starting the GUI, and exit with the script's status. | test on the resolver; smoke: `rusty usb check` |
| REQ-002 | WHEN `<name>` is not a store script, `rusty` shall behave exactly as today (the app's own flags and the window). | test: existing flags unchanged |
| REQ-003 | WHEN `rusty-cli scripts list\|view\|new\|rm\|path\|edit <name>` is invoked, the CLI shall act on the store's scripts with the same store commit behaviour as `skills`. | tests per subcommand |
| REQ-004 | WHEN the store holds scripts, the app shall show them in the Skills tab (or a Scripts section of it) with the same view and edit affordances as a skill, and a Run action that opens a terminal tab running the script. | screenshot; smoke |
| REQ-005 | WHEN a script is created or edited through Rusty, the store shall commit it, and the safety scan that gates a pending skill shall gate a pending script (scripts run with the user's `sudo -n`). | test: a pending script is not runnable until approved |
| REQ-006 | WHEN a skill directory contains an executable `*.sh` file, the resolver shall offer it under the file's basename without the suffix, so `dev-box-usb/usb-reset.sh` answers to `rusty usb-reset`. | test on the resolver |
| REQ-007 | The MCP server shall expose `script_list`, `script_view` and `script_run` alongside the skill tools, with `script_run` refused for pending scripts. | tool tests |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | Scripts live in the store (`~/.rusty/skills`), committed like skills, gated by the same review and approve step. | Chad: editable and viewable in Rusty; the store is already a git repository with a scan and an approve gate. | A separate scripts directory outside the store (no history, no gate). |
| 2 | A script runs as the user; root only through `sudo -n`, never through Rusty's own privileges. | The back end and the app run as the user; nothing in Rusty escalates. | A helper with elevated rights. |

## Open before the seal (Chad decides)

1. The entry point for `rusty usb`: the app binary dispatching subcommands before Qt starts
   (the draft), `rusty-cli <name>`, or a small `rusty` shim that execs the app when no
   subcommand matches. The app binary is the shortest word but the heaviest program to
   start for a script.
2. Whether `script_run` belongs on the MCP surface at all: it lets any connected agent
   execute a store script, gated by the approve step. `script_list` and `script_view` are
   read-only and uncontroversial.
3. The store layout: a `scripts/` directory in the store, scripts inside skill
   directories (the `usb-reset.sh` form), or both.
4. The app surface: a Scripts section inside the Skills tab, or a tab of its own.

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-010-scripts-as-commands.md`
- Intake: none (Chad's words above; the draft came from the rustal session)
- Design references: `~/.rusty/skills/.claude/skills/dev-box-usb/usb-reset.sh` (the first
  script and the smoke target); `crates/rusty-core/src/skills/mod.rs` (the store, the
  scan, the approve gate); `crates/rusty-cli/src/main.rs` (the `skills` subcommands);
  `omarchy/README.md` (why `rusty-session` is not a store script)
- Architecture: `docs/architecture.md`; `AD-rusty-mcp-only-back-end-001` (the CLI and the
  app reach the store through the managers and the server, never the database)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled; **sealed by Chad** (new tools, a new surface) |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
