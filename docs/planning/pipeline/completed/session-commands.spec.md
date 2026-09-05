---
title: Session commands
pipeline_id: c7987c16-45b5-40b6-b99d-46cb5f0d4915
status: Phase 5 — Complete PASS
ticket: TICKET-029
ticket_doc: docs/planning/tickets/closed/TICKET-029-session-commands.md
aar: docs/planning/knowledge/aar/AAR-029-session-commands.md
sealed: Chad, 2026-09-05: "can we redo where instead of rusty-session we do "rusty <command>" that way going forward as we create commands we can have that convention. lets do rusty session start or something."
created: 2026-09-05
---

# Session commands: spec

## Intent

One way in. The app binary owns the commands a person types, as `rusty <noun> <verb>`,
and the first noun is `session`: `rusty session start` brings the back end and the app
up under their user units, `stop` takes the app down, `status` says where both stand,
and `run` is what the app unit executes. The `rusty-session` script that did this since
TICKET-009 goes, together with every mention of it in the installer, the desktop entry,
the key snippet and the package. Built-in nouns come before store scripts, so the
convention holds as nouns are added. The same slice moves the app's idea of where the
Omarchy theme lives to where Omarchy 4 keeps it, `~/.local/state/omarchy/current/theme`,
with the Omarchy 3 path as the fallback; since the 2026-09-04 upgrade the app has read a
directory that no longer exists, and the terminal pane has been black.

## Scope

- In: `crates/rusty-app/src/main.rs` (dispatch before Qt), `crates/rusty-app/src/session.rs`
  (new: the four verbs, the systemctl calls, the port probe, the display import, the
  unmanaged-window check, PATH completion, usage), `crates/rusty-app/src/omarchy.rs` and
  `theme.rs` (the theme directory and its watcher), `omarchy/rusty-session.sh` (deleted),
  `omarchy/rusty-app.service`, `omarchy/install.sh`, `omarchy/com.ignibyte.rusty.desktop`,
  `omarchy/hyprland-bindings.conf`, `omarchy/README.md`, `packaging/PKGBUILD`, `README.md`,
  `docs/architecture.md`, `ROADMAP.md`, the knowledge register, the wiki.
- Out (named seams, not forgotten): other nouns (`mcp`, `brain`, `skills` stay with
  `rusty-cli`); a store-script listing under `rusty help`; the units' restart, slice and
  OOM semantics (TICKET-009 stands); Omarchy's own stale `hyprlock.conf` background link.

## Acceptance criteria (EARS)

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

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | `rusty <noun> <verb>` is the command convention of the app binary; built-in nouns are matched before store scripts; dash-prefixed arguments go to Qt as they always have. | Chad's words above: one home for every command as they are added, and one binary a person needs to know. A store script named like a noun is shadowed on purpose and the README says so. | A wrapper per feature (`rusty-session`, `rusty-next`); `rusty-cli` as the home (it is the agents' and the store's tool, and a desktop launcher should not depend on it). |
| 2 | The session logic is Rust inside the app binary (`session.rs`); the shell script is deleted; `run` completes PATH in-process and continues into the window, no exec. | One place, tested like the rest of the crate, and the unit runs the binary directly. | Keeping the script and having `rusty session` exec it (two homes, and the wrapper stays on PATH as a second way in). |
| 3 | The back end probe is a minimal HTTP/1.1 `POST` of the MCP `initialize` over `std::net::TcpStream`; a `200` status line means answering. | No new dependency and no runtime for one status line; the same request the script sent through curl. | The rmcp client on tokio (a runtime spun up for `status`); shelling out to curl (a second program the desktop entry would depend on). |
| 4 | The theme directory is `RUSTY_OMARCHY_THEME_DIR` when set, else `~/.local/state/omarchy/current/theme` when it exists, else `~/.config/omarchy/current/theme`; the watcher watches the chosen directory's parent. | Omarchy 4 generates the per-app files (`colors.toml`, `alacritty.toml`) under state and rewrites `theme.name`, `background` and `theme/` there on a switch; Omarchy 3 boxes keep working. | Reading `theme.name` and resolving into `/usr/share/omarchy/themes/<name>` (the source theme has no `alacritty.toml`; the generated one lives only under state). |
| 5 | An unknown bare first word is an error: usage on stderr, exit 2. | A typo in a command must not open a window; the convention is only legible when the binary says what it knows. Supersedes TICKET-010 REQ-002's "behave exactly as today" for bare words; flags are untouched. | Opening the window as TICKET-010 left it. |
| 6 | The installer removes a stale `~/.local/bin/rusty-session` left by its own earlier runs. | It is the installer's artifact; leaving it would keep a second, drifting way in. | Leaving it (harmless but misleading). |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-029-session-commands.md`
- Intake: none
- Design references: `omarchy/rusty-session.sh` at `9f1d0e0` (the behaviour being ported);
  `/usr/share/omarchy/bin/omarchy-theme-set` (what a switch rewrites under
  `~/.local/state/omarchy/current`); `pipeline/completed/session-resilience.spec.md`
  (TICKET-009, whose decision 4 this supersedes); `pipeline/completed/scripts-as-commands.*`
  (TICKET-010, the store-script dispatch)
- Architecture: `docs/architecture.md` (the `omarchy/` bullet, the theme line, the
  session-bound entry)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled; sealed by Chad's words above |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
