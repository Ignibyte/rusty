---
title: Retire the Obsidian bridge
pipeline_id: 6bba8f0f-df4a-4d8c-ad83-22c33f553f1e
status: Phase 5 — Complete PASS
ticket: TICKET-006
ticket_doc: docs/planning/tickets/closed/TICKET-006-retire-obsidian-bridge.md
aar: docs/planning/knowledge/aar/AAR-006-retire-obsidian-bridge.md
sealed: 2026-09-02, Chad: "lets work ticket 2 through 6 auto approved until finished ... I want near identical to obsidian except that we have the shell built in and an MCP in which the agent can interact with"
created: 2026-09-03
---

# Retire the Obsidian bridge: spec

## Intent

Rusty is the workspace; the code that drove Obsidian's CLI from the tools, the CLI and
the installer goes, and every document says so.

## Scope

- In: `crates/rusty-core/src/obsidian.rs` and its `pub mod`; the six tools, their
  parameter types, the `Obsidian` handle in the server and the names in `EXPECTED`;
  the smoke test's Obsidian status call; the CLI's `obsidian` commands and help; the
  installer's registration and CLI check; the app's `obsidian configure` call; the
  screenshot script's `RUSTY_OBSIDIAN_CLI`; README, `docs/architecture.md`, ROADMAP,
  the wiki.
- Out (named seams): the vault's `.obsidian/` folder; the Omarchy theme file
  `obsidian.css` the app reads its tokens from.

## Acceptance criteria (EARS)

REQ-001 to REQ-005 as in the ticket.

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The whole module goes, `register` and `configure_vault` included: they wrote Obsidian's own config files, which is the bridge's business, not the vault's. | Half a bridge is a bridge to maintain; git keeps the whole. | keep the config writers |
| 2 | A removed tool is a versioned break; the README's tool count and the wiki say the six are gone and what replaces each. | The tool surface is a contract. | silent removal |
| 3 | The vault stays "an Obsidian vault" in every document: the format is the point, the app was the bridge. | Obsidian still opens the vault; the docs must not read as if the format changed. | renaming the vault format |

## Linked artifacts

- Ticket: TICKET-006
- Intake: `docs/planning/intake/INTAKE-knowledge-workspace.md` (REQ-012)
- Architecture: `openwiki/mcp-back-end.md`, `openwiki/vault-and-brain.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | this spec, the ticket, the AAR opened | sealed by Chad's goal of 2026-09-02 |
| 2 Design | manifest, regression table in the notes | design actionable |
| 3 Implement | removals, docs | `bin/gate.sh --fast` green |
| 3.5 Inspect | ledger | confirmed findings resolved |
| 4 Validate | tests, smoke, `bin/gate.sh --diff` | receipt matches |
| 5 Complete | audit, wiki update, docs, AAR, register, brain capture, archive | pair archived |
