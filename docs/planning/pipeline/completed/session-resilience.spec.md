---
title: Session resilience
pipeline_id: f2387293-078b-4927-80dd-90301bdd90dd
status: Phase 5 — Complete PASS
ticket: TICKET-009
ticket_doc: docs/planning/tickets/open/TICKET-009-session-resilience.md
aar: docs/planning/knowledge/aar/AAR-009-session-resilience.md
sealed: Chad, 2026-09-03: "yep lets do all three and lets come up with a standard way in the repo that users should do this. this should be a script also probably that brings back up the MCP server as well on load"
created: 2026-09-03
---

# Session resilience: spec

## Intent

Rusty comes back on its own after the things that take a desktop down. The back end
restarts after any exit but an operator's stop. The app runs as a user service bound to
the graphical session, so a login starts it, a kill restarts it, and a quit leaves it
stopped. One script, `rusty-session`, is the entry point the installer, the desktop
entry and the launch key share, and it brings the back end up before the app. The
installer wires it on Omarchy and points at the two protections a user cannot apply:
the compositor's OOM score and earlyoom's avoid list. Why now: on 2026-09-03 earlyoom
killed Hyprland under a mutation audit, and the back end stayed dead because
`Restart=on-failure` ignores a SIGTERM exit.

## Scope

- In: `omarchy/rusty-mcp.service` (restart policy, OOM score), `omarchy/rusty-app.service`
  (new), `omarchy/rusty-session` (new), `omarchy/wayland-wm-oom.conf` (new, shipped and
  pointed at), `omarchy/install.sh`, `packaging/PKGBUILD`, the desktop entry, the key
  snippet, `omarchy/README.md` (brought current), `README.md`, `docs/architecture.md`,
  `ROADMAP.md`, the wiki.
- Out (named seams, not forgotten): any Rust change; applying the compositor drop-in or the
  earlyoom line from the installer (root, or another program's unit); surviving a
  compositor death inside the app (no Wayland client can); a Restart for the app on a
  clean quit.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The back end unit shall restart after any exit except an operator's stop, and shall carry the lowest OOM score a user unit can (100, the user manager's own). | `systemctl --user kill -s TERM rusty-mcp` then `is-active` within 5 s; `/proc/<pid>/oom_score_adj` |
| REQ-002 | WHEN a graphical session starts under uwsm, the app shall start as the user service `rusty-app.service`, wanted by and ordered after `graphical-session.target` and after the back end, in `app-graphical.slice`, and shall stop with the session. | `systemctl --user show` properties; `list-dependencies graphical-session.target`; the window in `hyprctl clients` |
| REQ-003 | WHEN the app exits by a signal or a crash, the service shall start it again within a few seconds; WHEN it exits with status 0, it shall stay stopped. | `systemctl --user kill -s TERM rusty-app` then a new main PID; a transient unit with the same directives exiting 0 and not restarting |
| REQ-004 | `rusty-session` shall provide `up` (the back end, then the app unit; the display variables imported into the user manager when it lacks them; nothing started when an unmanaged app already runs), `down` (the app unit stopped, the back end kept), `status` (both units, the port, the app process), and `run` (the unit's foreground command: PATH completed, the app exec'd). | each subcommand run and its output recorded; `bash -n` |
| REQ-005 | `omarchy/install.sh` shall install the script and both units, enable them, start the back end, start the app through `rusty-session up` when a graphical session is active, and write the desktop entry and the key snippet to launch through `rusty-session up`; `packaging/PKGBUILD` shall install the same files. | the installer run on the box; the installed files; `makepkg --printsrcinfo` unchanged in shape |
| REQ-006 | The repo shall ship the compositor drop-in (`OOMScoreAdjust=100` on the uwsm compositor unit) and document the earlyoom avoid line; the installer shall point at both and apply neither. | files present; installer output |
| REQ-007 | The docs shall say how the session comes back: `README.md`, `omarchy/README.md` (brought current), `docs/architecture.md`, `ROADMAP.md`, and the wiki pages that describe the installer and the back end. | doc review; `openwiki_finish` complete |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The app is a user service wanted by `graphical-session.target`, not an `exec-once` line. | uwsm's own guidance for autostarted apps; a unit can restart, stop with the session, and refuse a second instance. | `exec-once = uwsm-app -- rusty` in `autostart.conf` (no restart, a second window on every re-run). |
| 2 | `Restart=always` on both units, with `RestartPreventExitStatus=0` on the app. | earlyoom and the session teardown send SIGTERM, which `on-failure` treats as clean; a quit exits 0 and must stay quit. | `Restart=on-failure` (what let the back end stay dead). |
| 3 | `OOMScoreAdjust=100` on the back end and in the shipped compositor drop-in. | Measured on 2026-09-03: a user unit cannot go below the user manager's own score, 100; every session app sits at 200, so 100 is the last of the session to go. | `-900` (rejected by the kernel for an unprivileged manager); leaving the compositor at 200 (what earlyoom killed). |
| 4 | `rusty-session` is the single entry point, and `run` is the unit's `ExecStart`. | One place knows the order (back end, then app), the display import, the idempotence, and the PATH the app needs to find the agents. | Bare `ExecStart=%h/.local/bin/rusty` with `Wants=` only (no PATH guard, no by-hand recovery command). |
| 5 | Root-level protection is shipped and pointed at, never applied by the installer. | The installer runs as the user and does not edit another program's configuration; the earlyoom file is the package's. | The installer editing `/etc/default/earlyoom` with sudo. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-009-session-resilience.md`
- Intake: none
- Design references: uwsm README (`/usr/share/doc/uwsm/README.md`, applications and slices); the incident record in the ops handbook (`docs/ops/dev-box.md`, earlyoom section)
- Architecture: `docs/architecture.md` (the `omarchy/` bullet)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled; sealed by Chad's words above |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
