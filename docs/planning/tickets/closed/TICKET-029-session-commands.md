---
title: TICKET-029-session-commands
status: done
ticket_number: 029
type: feature
created: 2026-09-05
closed: 2026-09-05
intake:
pipeline_spec: docs/planning/pipeline/completed/session-commands.spec.md
---

# TICKET-029-session-commands

## Summary

`rusty <noun> <verb>` becomes the command convention of the app binary. The first noun is
`session`: `rusty session start|stop|status|run` replaces the `rusty-session` script, which
goes. The same ticket points the app at the Omarchy 4 theme directory,
`~/.local/state/omarchy/current/theme`, so the terminal pane and the Follow Omarchy skin
take the desktop's colours again.

## Why

Chad, 2026-09-05: "can we redo where instead of rusty-session we do "rusty <command>" that
way going forward as we create commands we can have that convention. lets do rusty session
start or something." Two ways in (`rusty`, `rusty-session`) is one too many, and every
future command wants one home. The theme path moved with the Omarchy 4.0.2 upgrade on
2026-09-04; since the first app start after it the palette has fallen back to defaults, the
Konsole scheme has not been rewritten and the terminal pane has been black.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN `rusty session start` is invoked, the app binary shall start the back end unit, then the app unit unless it is active or a `rusty` outside the unit is running (a message says so), importing the display variables into the user manager when it holds none, and print the status, all without starting Qt. | unit tests on the decision logic; smoke on the box: with the unit active no second window opens and the status prints |
| REQ-002 | WHEN `rusty session stop` is invoked, the binary shall stop the app unit and leave the back end serving. | smoke: `is-active` of both units after |
| REQ-003 | WHEN `rusty session status` is invoked, the binary shall print both units' state, whether the back end answers an MCP `initialize` on its port, and the app's process ids. | test on the response parser; smoke output recorded |
| REQ-004 | WHEN `rusty session run` is invoked (the unit's command), the binary shall complete `PATH` with `~/.local/bin` and `~/.cargo/bin` and open the window in the same process. | test on the PATH function; the unit's `ExecStart` |
| REQ-005 | The dispatch shall hand dash-prefixed arguments to Qt untouched, match a built-in noun before a store script, run a store script as TICKET-010 defined, print usage for `help`, `--help` and `-h`, and answer any other bare word with the usage on stderr and exit status 2. | tests on the dispatcher |
| REQ-006 | The repo shall invoke or install no `rusty-session`: the script deleted, the unit's `ExecStart`, the desktop entry, the key snippet, the installer and the PKGBUILD saying `rusty session start` or `run`, and the installer removing a stale `~/.local/bin/rusty-session` (the one place the name survives, beside a line of history in the README). | a test that refuses the name under `omarchy/` and `packaging/`; the installer run on the box |
| REQ-007 | WHEN the desktop keeps the active theme at `~/.local/state/omarchy/current/theme` (Omarchy 4), the app shall read `colors.toml` and `alacritty.toml` there and watch its parent, falling back to `~/.config/omarchy/current/theme` (Omarchy 3), with `RUSTY_OMARCHY_THEME_DIR` winning over both. | test with a scratch home; smoke: the scheme file is rewritten and the pane takes the theme |
| REQ-008 | The docs shall say `rusty session …` and the theme path: `README.md`, `omarchy/README.md`, `docs/architecture.md`, `ROADMAP.md`, the wiki, and the knowledge register with the superseding decision. | doc review; `openwiki_finish` complete |

## Scope

- In: `crates/rusty-app/src/main.rs` (the dispatcher), a new `session.rs` in the app
  crate, `omarchy.rs` and `theme.rs` (the theme directory), `omarchy/` (script deleted,
  unit, desktop entry, key snippet, installer, README), `packaging/PKGBUILD`, the docs and
  the wiki.
- Out: any other noun (`rusty mcp …`, `rusty brain …` stay in `rusty-cli` until asked);
  a `rusty help` that lists store scripts (`rusty-cli scripts list` does); the units'
  semantics (restart policy, slices, OOM); the Omarchy 4 leftovers in the desktop's own
  config (`hyprlock.conf` still names the old background link; not ours).

## Notes

- Pipeline spec: docs/planning/pipeline/completed/session-commands.spec.md
- Related docs: `omarchy/README.md`, `docs/architecture.md` (the `omarchy/` bullet and the
  theme line), `AD-rusty-app-as-session-service-001` (the entry point it names moves),
  TICKET-010 (the store-script dispatch this ticket orders behind built-in nouns).
- Promoted from intake: none; drafted from Chad's words above.
- Follow-ups opened: TICKET-030 (the key snippet in Omarchy 4's Lua; found dead on the box at validation).
