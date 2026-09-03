---
title: Session resilience: notes
pipeline_id: f2387293-078b-4927-80dd-90301bdd90dd
---

# Session resilience: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins read (two notices, nothing critical). Register: no rule touches units
  or the installer; `AD-rusty-mcp-only-back-end-001` and `AD-rusty-agents-are-terminals-001`
  hold (nothing here changes how the app reaches the store or runs agents). Nearest
  completed notes: TICKET-008 (the installer last changed there for the skin work) and
  TICKET-001 (the installer's shape). Wiki: `development-and-validation.md` describes
  `omarchy/install.sh`; `mcp-back-end.md` names the unit; `quickstart.md` says how to run.
  Brain: `projects/dev-box-hub` carries the 2026-09-03 incident; nothing on units before
  it. Measurements on the box: the user manager runs at `oom_score_adj` 100 with
  `DefaultOOMScoreAdjust=200`; a transient unit asked for `-100` came out at 100 and one
  asked for 900 at 900, so 100 is the floor. The manager's PATH holds `~/.local/bin`
  (uwsm imports the login environment), where `claude` and `codex` live. uwsm's README:
  autostarted units are wanted by and ordered after `graphical-session.target`;
  `app-graphical.slice` is stopped before the compositor. `/etc/default/earlyoom` belongs to
  the earlyoom package, not to Omarchy's scripts. `scripts/check-pipeline.sh` and
  `scripts/check-pipeline-tools.sh` passed.
- Decisions: the five locked decisions in the spec. The compositor drop-in and the earlyoom
  line are applied on this box by hand as part of the same request, outside the repo's
  installer.

## Phase 2: Design

- Architecture and data flow: two `systemd --user` units and one script, no Rust.
  `rusty-mcp.service` stays wanted by `default.target` (it serves agents' HTTP clients
  whether or not a session is up) and gains `Restart=always` and `OOMScoreAdjust=100`.
  `rusty-app.service` is wanted by and ordered after `graphical-session.target` (uwsm's
  own guidance), ordered after and wanting `rusty-mcp.service`, placed in
  `app-graphical.slice` (stopped before the compositor), `Restart=always` with
  `RestartPreventExitStatus=0`, `SyslogIdentifier=rusty` so `journalctl -t rusty` keeps
  working (PR-rusty-qt-logs-in-journal-001), `ExecStart=%h/.local/bin/rusty-session run`.
  systemd's default start limit (five starts in ten seconds) ends a crash loop.
  `rusty-session` owns the order and the idempotence: `up` starts the back end, imports
  the display variables into the user manager when it lacks them (a compositor started
  outside uwsm), refuses to start a second window when an unmanaged `rusty` runs, then
  starts the app unit; `down` stops the app unit only; `status` reads both units, posts
  an `initialize` to the port, and lists the app's processes; `run` completes PATH with
  `~/.local/bin` and `~/.cargo/bin` and execs `rusty`, exiting 0 with a journal line when
  the binary is missing so the unit stays stopped instead of looping. The desktop entry,
  the key snippet and the installer all call `rusty-session up`.
- File manifest:
  - `omarchy/rusty-mcp.service`: `Restart=always`, `OOMScoreAdjust=100`.
  - `omarchy/rusty-app.service` (new): the app unit above.
  - `omarchy/rusty-session.sh` (new; installed as `rusty-session`, the `.sh` keeps it under
    the gate's shell syntax check): `up`, `down`, `status`, `run`.
  - `omarchy/wayland-wm-oom.conf` (new): the drop-in for
    `wayland-wm@hyprland.desktop.service` with `OOMScoreAdjust=100`, shipped and pointed at.
  - `omarchy/install.sh`: installs the script and both units, enables both, restarts the
    back end, runs `rusty-session up` when `graphical-session.target` is active and the app
    was built, rewrites the desktop entry's `Exec` to `rusty-session up`, prints the key
    binding, the drop-in and the earlyoom pointers.
  - `omarchy/com.ignibyte.rusty.desktop`: `Exec=rusty-session up`.
  - `omarchy/hyprland-bindings.conf`: the anchored class pattern (a bare `rusty` matched a
    Chrome tab on the box on 2026-09-03) and `rusty-session up` as the launch command.
  - `packaging/PKGBUILD`: installs `rusty-session`, `rusty-app.service` (paths rewritten to
    `/usr/bin`), the drop-in under `/usr/share/rusty/`; the stale Obsidian `optdepends`
    line goes (the bridge was retired in TICKET-006).
  - `omarchy/README.md`: brought current (the bridge section goes); the session section.
  - `README.md`, `docs/architecture.md`, `ROADMAP.md`: the session paragraph, the dated
    bullet, the M7 line.
  - Wiki at Phase 5: `development-and-validation.md`, `mcp-back-end.md`, `quickstart.md`.
- Store consequences: none. No schema, no vault format, no state file changes.
- Tool contract: none. No tool is added, renamed or removed.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001 | `systemctl --user kill -s TERM rusty-mcp`, then `is-active` inside 5 s; `oom_score_adj` of the new main PID |
  | REQ-002 | `systemctl --user show rusty-app -p WantedBy,PartOf,After,Wants,Slice`; `list-dependencies graphical-session.target`; the window in `hyprctl clients` after `rusty-session up` |
  | REQ-003 | `systemctl --user kill -s TERM rusty-app` then a new `MainPID`; a transient unit with the same `Restart` directives running `sh -c 'exit 0'` shows `NRestarts=0` and one running `sh -c 'kill -TERM $$'` shows a restart |
  | REQ-004 | `bash -n`; each subcommand run with output pasted |
  | REQ-005 | `omarchy/install.sh` run on the box; `is-enabled` for both units; the desktop entry's `Exec`; `makepkg --printsrcinfo` |
  | REQ-006 | the files; the installer's printed pointers |
  | REQ-007 | the docs read back; `openwiki_finish` returns complete |
- Risks: data safety, none (no store touched). A second window: `up` checks for an
  unmanaged `rusty` first. A crash loop: the start limit. A missing binary: `run` exits 0.
  Theme and keyboard: unchanged. No back end: the app reconnects every 3 s as before, and
  `Wants=` starts the service anyway. The unmanaged app running on the box now has to be
  stopped once to move under the unit; its tabs and tmux sessions survive that.
- CodeGraph evidence: `codegraph_explore` over the launch path: `AGENT_CANDIDATES`
  (`terminals.rs:133`, two callers, no test) is resolved against PATH, so `run` completes
  PATH; `Desk` reads `HYPRLAND_INSTANCE_SIGNATURE` at construction and shells out to
  `hyprctl`, so the unit needs the session environment uwsm imports (measured present in
  the user manager); `Backend` dials `127.0.0.1:4174` and reconnects on its own. No Rust
  symbol changes, so the blast radius is the installed files only.

## Phase 3: Implement

- Built: the manifest as designed. `omarchy/rusty-mcp.service` (`Restart=always`,
  `OOMScoreAdjust=100`), `omarchy/rusty-app.service`, `omarchy/rusty-session.sh`,
  `omarchy/wayland-wm-oom.conf`, `omarchy/install.sh` (script and units installed, both
  enabled, `rusty-session up` under an active session, the desktop entry's `Exec`, the
  three pointers), `omarchy/com.ignibyte.rusty.desktop`, `omarchy/hyprland-bindings.conf`,
  `packaging/PKGBUILD`, `omarchy/README.md` rewritten, `README.md`, `docs/architecture.md`,
  `ROADMAP.md`.
- Deviations: the package edit first dropped the whole `optdepends` array with the stale
  Obsidian line; the ollama line was put back from `HEAD`. `omarchy/README.md` still
  carried the retired Obsidian bridge section and "the installer lands with M6"; both went,
  which is the "brought current" the ticket names.
- Fast gate: `bin/gate.sh --fast` on 2026-09-03: `GATE GREEN [fast]` (fmt, clippy, every
  test, the smoke test included).

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | correctness | `status` under `pipefail`: `pgrep` exits 1 with no match and would fail the substitution | low | fixed while writing: `|| true` on the pipeline |
| 2 | correctness | `run` with no `rusty` on PATH would exit non-zero and the unit would restart every two seconds until the start limit | medium | fixed while writing: exit 0 with a journal line, which `RestartPreventExitStatus=0` honours |
| 3 | correctness | a second window when `up` runs beside an app started from a terminal | medium | fixed while writing: `unmanaged_app` compares every `rusty` pid with the unit's `MainPID` |
| 4 | portability | `OOMScoreAdjust=100` on a system whose user manager defaults its services to 0 would raise the back end's score | low | accepted: Omarchy's systemd defaults user services to 200 (measured: `DefaultOOMScoreAdjust=200`, the manager at 100); the comment in the unit says what the number is |
| 5 | portability | `Slice=app-graphical.slice` off uwsm | ok | systemd creates an implicit slice from the name; uwsm ships it |
| 6 | correctness | the installer enables `rusty-app` when the app was not built | ok | guarded by `app_built` |
| 7 | correctness | `SyslogIdentifier` would become `rusty-session` after the exec | ok | set to `rusty`, so `journalctl -t rusty` still works (PR-rusty-qt-logs-in-journal-001) |
| 8 | data safety | nothing under `~/.rusty` or the vault is touched | ok | by reading |
| 9 | secrets, paths | no host names, no accounts; the earlyoom example in the README omits the box's own patterns | ok | by reading |
| 10 | prose | the README, the unit comments and the script's usage against `no-ai-slop`; zero em dashes in every new file | ok | `grep -o '—'` per file |

- Post-implementation CodeGraph: a fresh `codegraph_explore` over `AGENT_CANDIDATES`,
  `Desk` and `Backend` returns the Phase 2 topology unchanged; no Rust symbol was edited,
  so the blast radius is the installed files and the units.

## Phase 4: Validate

- Tests run (commands and output), all on 2026-09-03 against the live units on the box:
  - `omarchy/install.sh`: rc 0; both units installed and enabled (`Created symlink
    .../graphical-session.target.wants/rusty-app.service`), the back end answering on
    the port, `rusty-session up` printing `rusty is running outside rusty-app.service;
    quit it, then run: rusty-session up` beside the app Chad had started from a
    terminal, the three pointers printed (launch line, drop-in, earlyoom).
  - The app moved under the unit: the terminal-started `rusty` (pid 1908583) sent SIGTERM,
    then `rusty-session up`: `rusty-app.service active`, `MainPID=1923367`, comm `rusty`,
    `Slice=app-graphical.slice`, `WantedBy=graphical-session.target`,
    `PartOf=graphical-session.target`, `After=... rusty-mcp.service ...
    graphical-session.target`, `Wants=rusty-mcp.service`; the process environment holds
    `WAYLAND_DISPLAY=wayland-1` and the live `HYPRLAND_INSTANCE_SIGNATURE`, and its PATH
    holds `~/.local/bin`; `hyprctl clients` shows one `com.ignibyte.rusty` window;
    `systemctl --user list-dependencies graphical-session.target` lists
    `rusty-app.service`; `journalctl -t rusty` carries the app's lines. The tmux sessions
    behind the tabs, this session's included, came back attached.
  - REQ-001: `systemctl --user kill -s TERM rusty-mcp`: `before 1920615 after 1923527 state
    active oom_adj 100 restarts 1`.
  - REQ-003 on the real unit: `systemctl --user kill -s TERM rusty-app`: `before 1923367
    after 1923636 state active restarts 1`; the window was back by the next check.
  - REQ-003 on transient units carrying the same directives (`Restart=always`,
    `RestartPreventExitStatus=0`, `RestartSec=1`): `sh -c 'exit 0'`: 1 started, 0 restarts
    scheduled, unit unloaded as inactive/success; `sh -c 'exit 1'`: 5 started, 5 restarts
    scheduled, `start-limit-hit`; `sleep 300` killed from outside with `systemctl --user
    kill -s TERM`: pid 1927925 to 1927958, `NRestarts=1`, journal `Scheduled restart job,
    restart counter is at 1`. A first probe written as `sh -c 'kill -TERM $$'` was invalid:
    systemd expands `$$` to `$` in a transient command line (`kill: $: not a pid`), so it
    exited 0 and proved nothing; replaced by the external kill.
  - REQ-004: `rusty-session status` (both units active, `back end answering on
    http://127.0.0.1:4174/mcp`, `app process 1923636`); `rusty-session down`
    (`rusty-app.service stopped; rusty-mcp.service keeps serving`, then inactive/active);
    `rusty-session up` (both active, `app process 1923806`); `rusty-session bogus` exits 2
    with the usage; `rusty-session help` prints it; `bash -n omarchy/rusty-session.sh` ok.
  - REQ-005 and REQ-006: `is-enabled rusty-mcp rusty-app` both `enabled`; the desktop
    entry's `Exec=/home/cpeppers/.local/bin/rusty-session up`; the installed units diff
    clean against the repo's; `omarchy/wayland-wm-oom.conf`, `rusty-app.service` and
    `rusty-session.sh` present; the installer's log carries the two pointer lines;
    `makepkg --printsrcinfo` parses with `optdepends = ollama: local embeddings for semantic
    search` alone.
- Gate run: `bin/gate.sh --diff` on 2026-09-03: `GATE GREEN [diff]`, `receipt written:
  .git/rusty-gate-receipt` (fmt, clippy, tests, doc, shell-syntax, secrets over 142 gated
  files, whitespace).
- Smoke evidence: the app's window blinked twice on Chad's desktop (the kill test and
  `down`/`up`) and came back with its tabs; no synthetic input was sent.
- Skips or pre-existing failures: the quit path (exit 0) was not driven on the real app,
  no synthetic input; the transient probe with the same directives stands for it. The
  compositor drop-in reads `OOMScoreAdjust=100` on the unit now and applies at the next
  login. The gold-linker deprecation warning from the app's release build predates this
  ticket.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (the kill test, `oom_adj 100`); REQ-002 PASS (the unit's
  properties, the dependency listing, the window); REQ-003 PASS (the real kill, the three
  probes); REQ-004 PASS (every subcommand run); REQ-005 PASS (the installer run, the
  installed files, `--printsrcinfo`); REQ-006 PASS (the files, the pointer lines);
  REQ-007 PASS (README, `omarchy/README.md`, architecture, roadmap read back; the wiki
  line below).
- Docs: `README.md` (run it, the install paragraph), `omarchy/README.md` (rewritten: the
  file table, the session, memory pressure), `docs/architecture.md` (the `omarchy/` bullet
  and the dated decision), `ROADMAP.md` (M7 line).
- Wiki: `update` run `e34f6983` through the lifecycle: claims added on
  `development-and-validation.md` (five), `mcp-back-end.md` (one) and `quickstart.md`
  (one), the installer claim and the stale screenshot claim updated with fresh evidence;
  `openwiki_finish` returned `status: complete`; the PostToolUse hook did not fire, so the
  genuine result was fed to `record-pipeline-tool-use.sh` by hand (the TICKET-007 path) and
  it wrote `.git/rusty-openwiki-receipt`.
- AAR: `docs/planning/knowledge/aar/AAR-009-session-resilience.md`; register IDs
  PR-rusty-restart-always-001, PR-rusty-user-oom-floor-001,
  PR-rusty-probe-kills-from-outside-001, AD-rusty-app-as-session-service-001.
- Brain capture: timeline entry on `projects/rusty-v3` at delivery.
- Archive: this pair lives in `completed/`; the ticket in `closed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
| 2026-09-03 | the back end stayed down after earlyoom's SIGTERM | PR-rusty-restart-always-001 |
| 2026-09-03 | `-900` for the compositor was impossible from a user unit | PR-rusty-user-oom-floor-001 |
| 2026-09-03 | a self-kill probe exited 0 because systemd turned `$$` into `$` | PR-rusty-probe-kills-from-outside-001 |
| 2026-09-03 | the app as a session-bound user service, one entry script | AD-rusty-app-as-session-service-001 |
