---
title: TICKET-009-session-resilience
status: done
ticket_number: 009
type: infrastructure
created: 2026-09-03
closed: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/completed/session-resilience.spec.md
---

# TICKET-009-session-resilience

## Summary

Rusty comes back on its own. The back end restarts after any exit but an operator's
stop; the app runs as a user service bound to the graphical session, restarted when it is
killed and left alone when the user quits; one script, `rusty-session`, is the entry point
the installer, the desktop entry and the launch key share; the installer wires all of it
and points at the two protections that need more than a user's rights: a lower OOM score
for the compositor and an earlyoom avoid list.

## Why

On 2026-09-03 a mutation audit pushed the dev box past its memory and earlyoom killed
Hyprland. Every window went, and the back end service, killed with SIGTERM in the same
sweep, stayed down because `Restart=on-failure` does not restart a clean signal exit. The
app itself had been started from a terminal, so it belonged to that terminal. Chad asked
for a standard way, shipped in the repo, that brings the back end and the app back on load.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The back end unit shall restart after any exit except an operator's stop, and shall carry the lowest OOM score a user unit can (100, the user manager's own). | `systemctl --user kill -s TERM rusty-mcp` then `is-active` within 5 s; `/proc/<pid>/oom_score_adj` |
| REQ-002 | WHEN a graphical session starts under uwsm, the app shall start as the user service `rusty-app.service`, wanted by and ordered after `graphical-session.target` and after the back end, in `app-graphical.slice`, and shall stop with the session. | `systemctl --user show` properties; `list-dependencies graphical-session.target`; the window in `hyprctl clients` |
| REQ-003 | WHEN the app exits by a signal or a crash, the service shall start it again within a few seconds; WHEN it exits with status 0, it shall stay stopped. | `systemctl --user kill -s TERM rusty-app` then a new main PID; a transient unit with the same directives exiting 0 and not restarting |
| REQ-004 | `rusty-session` shall provide `up` (the back end, then the app unit; the display variables imported into the user manager when it lacks them; nothing started when an unmanaged app already runs), `down` (the app unit stopped, the back end kept), `status` (both units, the port, the app process), and `run` (the unit's foreground command: PATH completed, the app exec'd). | each subcommand run and its output recorded; `bash -n` |
| REQ-005 | `omarchy/install.sh` shall install the script and both units, enable them, start the back end, start the app through `rusty-session up` when a graphical session is active, and write the desktop entry and the key snippet to launch through `rusty-session up`; `packaging/PKGBUILD` shall install the same files. | the installer run on the box; the installed files; `makepkg --printsrcinfo` unchanged in shape |
| REQ-006 | The repo shall ship the compositor drop-in (`OOMScoreAdjust=100` on the uwsm compositor unit) and document the earlyoom avoid line; the installer shall point at both and apply neither. | files present; installer output |
| REQ-007 | The docs shall say how the session comes back: `README.md`, `omarchy/README.md` (brought current), `docs/architecture.md`, `ROADMAP.md`, and the wiki pages that describe the installer and the back end. | doc review; `openwiki_finish` complete |

## Scope

- In: the two units, the script, the installer and the package, the desktop entry and
  the key snippet, the shipped drop-in, the docs and the wiki.
- Out: any Rust change; protecting the compositor from the installer (root, and another
  program's unit); a restart of the app on a compositor death (no Wayland client survives
  it; the session brings the unit back).

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/session-resilience.spec.md`
- Related docs: `openwiki/development-and-validation.md`, `openwiki/mcp-back-end.md`,
  the ops handbook's dev-box page (the incident), uwsm's README (units wanted by
  `graphical-session.target`, `app-graphical.slice`)
- Promoted from intake: none (Chad's request of 2026-09-03)
- Follow-ups opened: none yet
