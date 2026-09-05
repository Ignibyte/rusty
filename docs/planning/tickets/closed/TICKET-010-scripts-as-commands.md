---
title: TICKET-010-scripts-as-commands
status: done
ticket_number: 010
type: feature
created: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/active/scripts-as-commands.spec.md
---

# TICKET-010-scripts-as-commands

## Summary

Scripts in Rusty's store become commands: `rusty usb` runs the store's `usb` script, the
CLI lists, shows, creates and removes them the way it does skills, and the app shows and
edits them beside the skills.

## Why

The machine procedures already live in the store as skills, and the ones an agent runs
by hand keep turning into scripts (the first: `usb-reset.sh` beside `dev-box-usb`, added
2026-09-03 after the xHCI reset was needed three times in two days). Chad wants to run
them himself, from any terminal, by a name he remembers, and to read and change them in
the app rather than hunt for the file. Today a script is reachable only by its path or a
hand-made link in `~/.local/bin`.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `rusty <name> [args…]` is invoked and `<name>` is a script in the store, the app binary shall exec that script with the arguments, without starting the GUI, and exit with the script's status. | test on the resolver; smoke: `rusty usb check` |
| REQ-002 | WHEN `<name>` is not a store script, `rusty` shall behave exactly as today (the app's own flags and the window). | test: existing flags unchanged |
| REQ-003 | WHEN `rusty-cli scripts list|view|new|rm|path|edit <name>` is invoked, the CLI shall act on the store's scripts with the same store commit behaviour as `skills`. | tests per subcommand |
| REQ-004 | WHEN the store holds scripts, the app shall show them in the Skills tab (or a Scripts section of it) with the same view and edit affordances as a skill, and a Run action that opens a terminal tab running the script. | screenshot; smoke |
| REQ-005 | WHEN a script is created or edited through Rusty, the store shall commit it, and the safety scan that gates a pending skill shall gate a pending script (scripts run with the user's `sudo -n`). | test: a pending script is not runnable until approved |
| REQ-006 | WHEN a skill directory contains an executable `*.sh` file, the resolver shall offer it under the file's basename without the suffix, so `dev-box-usb/usb-reset.sh` answers to `rusty usb-reset`. | test on the resolver |
| REQ-007 | The MCP server shall expose `script_list`, `script_view` and `script_run` alongside the skill tools, with `script_run` refused for pending scripts. | tool tests |

## Scope

- In: the resolver and its two entry points (`rusty <name>`, `rusty-cli scripts run`), the store layout for scripts (design decides between a `scripts/` directory in the store and scripts inside skill directories; REQ-006 keeps the skill-side form working either way), the CLI subcommands, the app view, the MCP tools, the review/approve gate.
- Out: a script language or templating; anything that runs a script as root without `sudo -n`; a scheduler (that is the reminders side).

## Notes

- Pipeline spec: docs/planning/pipeline/active/scripts-as-commands.spec.md
- Related docs: `omarchy/README.md` (the session script is not a store script; it stays in the repo), `~/.rusty/skills/.claude/skills/dev-box-usb/usb-reset.sh` (the first script, and the smoke target), `CLAUDE.md` in omarchy-ops (the skills-as-commands convention `bin/install.sh` links into `~/.claude/skills/`; scripts want the same link into `~/.local/bin` or the resolver makes it unnecessary).
- Promoted from intake: none; drafted by the rustal session on 2026-09-03 from Chad's words
  at 14:40: "we should probably create a script i can just run. that makes me think we could create a command line rusty for things like rusty <command> that are basically shells. so i could do rusty usb etc. we could make the scripts editable and viewable here as well."
- Decisions for the seal (from the Phase 1 pass of 2026-09-03, spec to be recreated when
  the ticket is picked up): the entry point for `rusty <name>` (the app binary dispatching
  before Qt starts, `rusty-cli <name>`, or a small `rusty` shim that execs the app when no
  subcommand matches; the recommendation is the CLI plus the shim, the app binary being the
  heaviest program to start for a script); whether `script_run` belongs on the MCP surface
  at all (recommendation: not yet; list and view only); the store layout (recommendation:
  scripts beside their skills, the `usb-reset.sh` form); the app surface (recommendation: a
  Scripts section of the Skills tab).
- Locked already: scripts live in the store, committed and gated like skills; a script runs
  as the user, root only through `sudo -n`.
- Follow-ups opened: none.
