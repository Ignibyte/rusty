# Rusty: agent guide

Read [CONSTITUTION.md](CONSTITUTION.md) before changing this repository. It is binding.

## Product

Rusty is a local-first AI personal assistant for Omarchy:

- `crates/rusty-core`: the managers (tasks, notes, memories, brain vault and index,
  semantic index, skills, secrets, settings, events, watcher).
- `crates/rusty-mcp`: the back end, an MCP server (59 tools, resources, notifications)
  over stdio for agents and Streamable HTTP for the app.
- `crates/rusty-app`: the desktop app, cxx-qt on Qt 6, QML pages with native agent
  terminals.
- `crates/rusty-cli`: terminal access to the same store.
- `docs/architecture.md`: the standing shape. `ROADMAP.md`: the product list.

## Work routing

- Non-trivial feature, fix, migration or workflow changes use the repository skill
  `rusty-workflow` (`.claude/skills/rusty-workflow/SKILL.md`). Read it and its
  `references/phases.md` before touching files.
- Product exploration that should not produce code goes into
  `docs/planning/intake/` from `docs/planning/_templates/intake.md`.
- Read-only questions and diagnosis are answered directly.
- A user may waive the ceremony for a small change: write the reason to
  `docs/planning/pipeline/WAIVER.md`, report it at handoff, delete it after. Quality,
  tests, secrets and receipt rules still apply.
- One active spec/notes pair at a time.
- Commit and push only when the user has authorized delivery. On this box Chad has
  standing authorization for `main` of this repo.

The pipeline:

```
recall → plan → design → implement → inspect → validate → complete → delivery
```

## Quality commands

```bash
bin/gate.sh --fast     # fmt, clippy, test; no receipt
bin/gate.sh --diff     # the delivery gate; green writes .git/rusty-gate-receipt
bin/gate.sh --verify   # does the receipt match this worktree
omarchy/install.sh     # rebuild and reinstall the binaries and the service on Omarchy
```

Run cargo commands one at a time. Never kill a running cargo.

## Local knowledge

Before designing or implementing:

1. `docs/planning/bulletins/INDEX.md`, then `docs/planning/knowledge/INDEX.md` for `PR-`,
   `BF-` and `AD-` entries that touch the work.
2. The nearest notes under `docs/planning/pipeline/completed/`.
3. `docs/architecture.md` and `docs/architecture/*.md`.
4. The brain: `brain_search` and `brain_context` through the `rusty` MCP server, which
   holds this project's pages and lessons.
5. CodeGraph for the Rust symbols, callers and blast radius; QML and shell by reading.

At complete, record lessons in the AAR, the knowledge register, and the brain.

## Tools

- `.mcp.json` wires the `rusty` server and CodeGraph for Claude Code; `.codex/config.toml`
  does the same for Codex. `scripts/setup-pipeline-tools.sh` installs CodeGraph pinned
  and project-local under `.dev/` (ignored); `scripts/codegraph.sh` is the CLI wrapper.
- Hooks in `.claude/settings.json` refuse: edits to gated paths outside an
  implementing pipeline (or a waiver), writes that contain something that looks like a
  secret, and `git commit` without a matching gate receipt.
