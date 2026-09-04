---
title: Scripts as commands: notes
pipeline_id: 0cbefd28-f640-49c3-a2cf-78bc996d66c2
---

# Scripts as commands: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: the ticket's seal note (four questions, answered at 17:20); register
  (`AD-rusty-mcp-only-back-end-001`); code: `SkillsManager` (active and staging
  directories, `list`, `get`, `create_skill`, `scan`, `approve`, `reject`, the two
  commit paths), the app's `main` (Qt starts at once), `command_for` in `terminals.rs`,
  the CLI's `run_skills`, `SkillsPage.qml` (list, editor, scan, approve).

## Phase 2: Design

- Architecture and data flow: `skills/scripts.rs` adds to `SkillsManager`: `scripts`
  (every executable-or-not `*.sh` under the active skill directories, and under staging
  when asked, as `Script { name, skill, path, status }`), `resolve_script` (`name` or
  `skill/name`; a clash between skills errors and names both), `script_text`,
  `create_script` (a missing skill is created with a one-line description; the script
  gets a header and mode 0755), `update_script`, `delete_script`, `scan_script` (the
  skill scan over the script's text), `run_script` (active only; a child process with
  the arguments, stdout and stderr captured, sixty seconds), and `exec_script` (unix
  `exec`, for the CLI and the app). The app binary, before Qt: when the first argument
  does not start with `-` and resolves to an active script, `exec_script`; otherwise the
  GUI as today. The CLI mirrors `skills` under `scripts`. The tools list, view, update and
  run (approved only). The Skills tab gains a Scripts section under the skill list; a
  selected script shows its text in the body editor (Save writes it, Scan reads it) and a
  Run button raises `runScript(path)`, which the window turns into a terminal tab with the
  program `run:<path>` (`command_for` runs the script and keeps a shell).
- File manifest:
  - `crates/rusty-core/src/skills/scripts.rs` (new), `skills/mod.rs` (`pub mod scripts`).
  - `crates/rusty-mcp/src/main.rs`: four tools, parameters, router names; the smoke test.
  - `crates/rusty-cli/src/main.rs`: `scripts ...`, usage.
  - `crates/rusty-app/src/main.rs` (the dispatch), `src/terminals.rs` (`run:`),
    `qml/SkillsPage.qml` (the section), `qml/Main.qml` (the signal, the label).
  - `scripts/screenshot.sh`: a script in the scratch store.
  - Phase 5: README, `docs/architecture.md`, ROADMAP, the counts (80), the wiki.
- Store consequences: none in the database; the store gains scripts beside skills.
- Tool contract: four tools added; 80 tools.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001, REQ-002 | `resolve_script` tests; the built `rusty` run against a scratch store (`RUSTY_SKILLS`) with a script that prints and exits 3, and with an unknown name under `QT_QPA_PLATFORM=offscreen` and a shot delay so it exits |
  | REQ-003 | `create_script` and `delete_script` tests; the CLI by reading |
  | REQ-004 | the `view:skills` scene with a seeded script selected |
  | REQ-005 | `a_pending_script_does_not_run` |
  | REQ-006 | `scripts_resolve_by_basename_and_disambiguate` |
  | REQ-007 | the router test's names; the smoke flow (list, view, update, run) |
- Risks: `rusty <name>` reads the default store, not the `skills_path` setting (the app
  has no database at that point; documented); a script that never exits holds
  `script_run` for sixty seconds (the cap); the exec replaces the app process, which is
  the point.
- CodeGraph evidence: `scan_skill_md` gains a caller; `command_for` has one caller in
  QML; `main` in the app crate has no callers.

## Phase 3.5: Inspect

The Fable session that wrote Phase 3 stopped mid-file on a usage limit at 18:52; an Opus
session reviewed the whole diff on 2026-09-03 21:00 and finished the run. Ledger:

| # | Finding | Verdict | Disposition |
|---|---|---|---|
| 1 | `terminals.rs`: the `run:` arm's `format!` carried an unescaped `"` — the crate did not compile. | Confirmed (the build says so) | Fixed. |
| 2 | The same arm wrapped the command in `sh -c '…'` around an already single-quoted path, so tmux's own `sh -c` split a path containing a space into two words. | Confirmed by reading the tmux call in `AgentTerminal.qml`: the string is one shell command, not a word list. | Fixed: the arm returns `bash '<path>'; exec "${SHELL:-/bin/bash}"`, and `a_script_program_runs_then_leaves_a_shell` pins a plain path, a spaced one and a quoted one. |
| 3 | `skills/mod.rs`: `pub mod scripts;` took `author`'s doc comment, leaving `author` undocumented and `scripts` described as the self-authoring loop. | Confirmed by reading. | Fixed: each module carries its own line. |
| 4 | The app's dispatch reads `RUSTY_SKILLS`; `resolve_root`, which the `rusty-cli scripts run` it execs uses, read only the `skills_path` setting and the default. A script found in one store would run from another, and the design's own regression plan (a scratch store through `RUSTY_SKILLS`) could not pass. | Confirmed: `resolve_root` had no env branch. | Fixed at the source: `resolve_root` takes `RUSTY_SKILLS` first, then the setting, then the default. New rule `PR-rusty-one-store-one-resolver-001`. |
| 5 | `store_script_exists` duplicates a slice of the resolver in the app. | Rejected as a defect: the app has no database open before Qt starts, so it cannot call the resolver; it decides only "is there a file", and the CLI it hands over to decides everything else (pending, clash). The seam is named in the spec's risks and in the README. | None. |
| 6 | A pending skill's script is invisible to the dispatch, so `rusty <name>` opens the window instead of saying "pending". | Confirmed as behaviour, rejected as a defect: the seal says a pending script never runs, and REQ-002 says a name that is not an active script behaves as today. `rusty-cli scripts run` names the reason. | Recorded here and in the AAR. |

## Phase 4: Validate

Commands and their real output, run 2026-09-03 21:0x–21:2x with
`CARGO_TARGET_DIR=/mnt/fast/target` (the workspace's own target; this session pins another):

- `cargo build --workspace --all-targets` — `Finished dev profile in 14.77s`.
- `cargo test --workspace` — 289 passed, 0 failed across the crates: rusty-app 24 (with
  `a_script_program_runs_then_leaves_a_shell`), rusty-cli 6, rusty-core 248 (with the four
  `skills::scripts::tests`), rusty-mcp 3 + the smoke test, and the rest.
- REQ-001 and REQ-003 end to end against a scratch store: `RUSTY_SKILLS=<scratch>
  rusty-cli scripts list` prints `usb-reset [active] dev-box-usb/usb-reset.sh`, `scripts
  path` prints the file, and `RUSTY_SKILLS=<scratch> rusty usb-reset alpha beta` prints
  `usb-reset ran with: alpha beta` and exits 3. This is the run that proves finding 4's fix:
  before it, the CLI would have looked in the real store.
- REQ-002: `HOME=<scratch> RUSTY_SKILLS=<scratch> QT_QPA_PLATFORM=offscreen rusty
  not-a-script` ran until the 20 s timeout killed it (exit 124) and printed no script
  output, so an unknown name took the window path.
- REQ-005: a pending skill seeded in `staging/`; `scripts list --all` shows
  `reset [pending] net-fix/reset.sh`, and `scripts run reset` exits 1 with `reset is
  pending; approve the skill "net-fix" first`.
- REQ-004: `scripts/screenshot.sh <out> view:skills` rendered the Skills tab with a
  Scripts section listing `$ usb-reset  dev-box-usb` under the skill list.
- REQ-006 and REQ-007: `skills::scripts::tests::scripts_resolve_by_basename_and_disambiguate`;
  `router_advertises_every_tool_once` and `every_tool_has_a_description` over 80 tools, with
  `script_list`, `script_view`, `script_update` and `script_run` registered.

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-007 satisfied, each with the evidence above.
- Docs: README (the section and the store-resolution paragraph), `docs/architecture.md`,
  ROADMAP, `AGENTS.md` and `CLAUDE.md` (80 tools), the wiki's quickstart count.
- Outstanding, and the reason this pair is not archived: the OpenWiki reconciliation of
  Phase 5 step 2 needs the `openwiki` MCP server, which only a session inside this
  repository has. `openwiki/.last-update.json` records the interrupted run honestly. The
  next session here runs `openwiki update` to `openwiki_finish`, takes the receipt, then
  archives this pair and closes the ticket.

