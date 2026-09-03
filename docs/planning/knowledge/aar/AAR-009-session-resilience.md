---
title: AAR-009-session-resilience
pipeline_id: f2387293-078b-4927-80dd-90301bdd90dd
ticket: TICKET-009
submitted: 2026-09-03
---

# AAR-009-session-resilience

## Recall log

- Register: no rule on units or the installer; the two standing architecture decisions
  on the back end and the agents hold. Completed notes: TICKET-008 and TICKET-001 for the
  installer. Wiki: `development-and-validation.md`, `mcp-back-end.md`, `quickstart.md`.
  Brain: `projects/dev-box-hub`, the 2026-09-03 earlyoom incident. Measured before
  planning: a user unit's OOM floor is the user manager's own score, 100.

## 1. Outcomes

- REQ-001 to REQ-007 PASS. Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- Measuring before designing: one `systemd-run --user -p OOMScoreAdjust=-100` showed the
  floor is 100, so the drop-in shipped with a number that works instead of one that would
  have been silently rejected.
- Validating on the live units: the kill tests on `rusty-mcp` and `rusty-app` proved the
  restart semantics on the real thing, and the transient probes covered the one path
  (exit 0) that no test may drive on Chad's desktop.
- The installer's `rusty-session up` refused a second window beside the app Chad had
  started from a terminal, first time, which is the idempotence the design asked for.

## 3. What went poorly

- The package edit dropped the whole `optdepends` array with the stale Obsidian line and
  the ollama line had to come back from `HEAD`; a regex over a multi-line array needs its
  result printed before it is trusted.
- The first self-kill probe (`sh -c 'kill -TERM $$'`) proved nothing and looked like a
  pass with zeros: an unloaded transient unit answers `systemctl show` with defaults.
- The PostToolUse hook did not write the OpenWiki receipt when `openwiki_finish`
  returned `complete`; the genuine result was fed to the hook by hand, as the TICKET-007
  notes describe, and the commit gate then accepted it.

## 4. Surprises

- systemd expands `$$` to `$` in a transient unit's command line, so the probe killed
  nothing and exited 0; `systemctl --user kill -s TERM` from outside is the honest probe.
- The user manager's own OOM score (100) is the floor for its units; every uwsm session
  process sits at 200, the compositor included, which is why earlyoom reached Hyprland.
- `systemctl show` on a unit that has been unloaded returns `inactive`, `success` and
  `NRestarts=0`, indistinguishable from a healthy quit without the journal.

## 5. Lessons

- PR-rusty-restart-always-001: a user service that must outlive a session teardown or an
  earlyoom sweep carries `Restart=always`; `on-failure` treats SIGTERM as clean and leaves
  it down; an app unit adds `RestartPreventExitStatus=0` so a quit stays quit.
- PR-rusty-user-oom-floor-001: a user unit cannot set `OOMScoreAdjust` below the user
  manager's own score (100 on Omarchy); measure with `systemd-run --user -p
  OOMScoreAdjust=...` before designing around a number.
- PR-rusty-probe-kills-from-outside-001: systemd turns `$$` into `$` in a transient
  unit's command line, so a self-kill probe kills nothing; kill probes with `systemctl
  --user kill -s TERM <unit>` and read `NRestarts` and the journal, never `show` alone.
- AD-rusty-app-as-session-service-001: the app runs as `rusty-app.service`, wanted by
  uwsm's `graphical-session.target` in `app-graphical.slice`, restarted when killed and
  left alone when quit; `rusty-session` is the one entry point the installer, the desktop
  entry and the key share; root-level protections ship as pointers, never applied.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.5 h | 0.4 h |
| 2 Design | 0.5 h | 0.3 h |
| 3 Implement | 1 h | 0.7 h |
| 3.5 Inspect | 0.3 h | 0.2 h |
| 4 Validate | 0.5 h | 0.6 h |
| 5 Complete | 0.5 h | 0.5 h |
