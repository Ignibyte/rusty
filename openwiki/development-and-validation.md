---
type: "Reference"
title: "Development and validation"
openwiki_generated: true
sources:
  - id: openwiki-source-164e2da859b5277df81c7d94
    resource: repo://.github/workflows/ci.yml
  - id: openwiki-source-4d8ab597958b0e5c2507d7fd
    resource: repo://omarchy/install.sh
  - id: openwiki-source-d4dc2c7ea0d931bfc9466b41
    resource: repo://scripts/screenshot.sh
generated: {by: "claude-code", at: "2026-09-03T04:50:24.252Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-09-03T04:50:24.252Z
---

# Development and validation

## Purpose

The commands that build, test and prove a change, the narrowest one for each kind of
change, and the ways the machine's state is kept out of the record.

## Building

- A Cargo workspace of four crates. `cargo build` builds all; `cargo build -p rusty-app`
  builds the app (cxx-qt generates the bridges and runs moc on the two C++ classes in
  `crates/rusty-app/cpp/`; Qt 6 and `qmltermwidget` come from the system).
- One cargo command at a time, never killed: a killed or concurrent cargo corrupts the
  incremental cache.
- `omarchy/install.sh`: release builds of the three binaries into `~/.local/bin`, the
  desktop entry and icon, the `rusty-mcp` user service (installed, enabled, restarted and
  probed over HTTP), the MCP config snippets. Idempotent.
- `packaging/PKGBUILD` (`rusty-git`) builds the same for the AUR, `!lto`.

## Testing

- `cargo test -p rusty-core`: the managers, the vault, the renderer, the scanner, the
  semantic index; scratch directories under the system temp dir, an in-memory database.
- `cargo test -p rusty-mcp`: the router tests and `tests/smoke.rs`, which spawns the
  built binary over stdio in a scratch `HOME` with `RUSTY_OBSIDIAN_CLI` pointed at a
  missing program, and walks tasks, the Obsidian status, resources and the workspace tools.
- `cargo test -p rusty-app`: the highlighter's tokenizer, the tabs JSON, the theme
  tokens and colour math.
- UI probes never drive the app with synthetic input on the user's desktop and never
  touch real data; screenshots and log checks stand in.

## The gate and CI

- `bin/gate.sh --fast` while working; `bin/gate.sh --diff` before delivery, which
  writes the receipt the commit hooks check; `--verify` to confirm it.
- `.github/workflows/ci.yml` runs the same set on every push (Qt installed with apt,
  `QMAKE` pointed at Qt 6) as a second witness; it never replaces the local receipt.

## Screenshots and visual evidence

`scripts/screenshot.sh <dir> [scene ...]` builds a scratch vault of invented pages,
starts a scratch `rusty-mcp` on its own port, and runs the app offscreen
(`QT_QPA_PLATFORM=offscreen`) with `RUSTY_SHOT`, `RUSTY_SHOT_SCENE`, `RUSTY_TABS`,
`RUSTY_STATE` and `RUSTY_OMARCHY_THEME_DIR` so that no real data, no workspace switch
and no user setting is involved; `SHOT_THEME`, `SHOT_SIZE`, `SHOT_ENV` and `SHOT_KEEP`
adjust it. The app grabs itself through `Tools.grabWindow`, because an item grab cannot
start on the offscreen platform. The images in `docs/screenshots/` come from it.

## Logs

- The back end logs to stderr (`journalctl --user -u rusty-mcp` for the service).
- The app's Qt messages go to journald when it is not started from a terminal
  (`journalctl --user -t rusty`); `QT_FORCE_STDERR_LOGGING=1` keeps them on stderr;
  `RUSTY_DEBUG=1` adds a line per tab, title, shot and settings event.

## Data on this machine

`~/.rusty/`: `rusty.db`, `brain/` (a git repository the managers commit to),
`notes/`, `skills/`, `.secret`. `~/.config/rusty/`: `tabs.json`, `workspace.json`,
`color-schemes/`. Tests and the screenshot script never use these paths.

## Primary sources

- `bin/gate.sh`, `.github/workflows/ci.yml`, `omarchy/install.sh`, `packaging/PKGBUILD`
- `crates/rusty-mcp/tests/smoke.rs`, `scripts/screenshot.sh`
