---
title: TICKET-006-retire-obsidian-bridge
status: done
ticket_number: 006
type: chore
created: 2026-09-03
closed: 2026-09-03
intake: docs/planning/intake/INTAKE-knowledge-workspace.md
pipeline_spec: docs/planning/pipeline/completed/retire-obsidian-bridge.spec.md
---

# TICKET-006-retire-obsidian-bridge

## Summary

Remove the Obsidian bridge: the six `obsidian_*` tools, the `rusty_core::obsidian`
module, the CLI's `obsidian` commands, the installer's registration of the vault with
Obsidian, and the app's call that kept Obsidian's vault settings in step with the theme;
update every document that named them. The vault stays an Obsidian vault by format.

## Why

The workspace tiers (TICKET-002 to TICKET-005) cover what the bridge did through
Obsidian's CLI: links and backlinks, unresolved targets, renames that rewrite links,
opening a page. The intake's REQ-012 says the bridge goes when the must-have tier is
delivered and Chad says so; his goal of 2026-09-02 approves tickets 2 through 6. The
roadmap adds "once the tiers above are in daily use": that day has not come yet, so the
removal is one commit that git history keeps whole, and the ticket names that.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | The MCP server shall serve no `obsidian_*` tool; the router test and the smoke test shall pass without them, and the tool count in the docs shall match. | router test; smoke; `grep` |
| REQ-002 | `rusty_core` shall carry no Obsidian module and no dependency that only it needed. | `cargo build --workspace`; `grep` |
| REQ-003 | The CLI shall carry no `obsidian` command and its help shall not name one. | `cargo build`; help text |
| REQ-004 | The installer shall not register the vault with Obsidian or look for its CLI; the app shall not shell out to configure Obsidian's vault settings. | reading; `cargo build` |
| REQ-005 | README, `docs/architecture.md`, ROADMAP and the wiki shall describe the vault as an Obsidian vault by format and name no bridge, tool or command that is gone. | `grep`; wiki lifecycle |

## Scope

- In: the removals above, the tests that named the bridge, the docs and the wiki.
- Out: the vault's `.obsidian/` folder and Obsidian itself on Chad's machines (the
  private handbook owns those), the theme tokens read from Omarchy's `obsidian.css`
  (they are Omarchy's file, not the bridge).

## Notes

- Pipeline spec: `docs/planning/pipeline/completed/retire-obsidian-bridge.spec.md`
- Related docs: `openwiki/mcp-back-end.md`, `openwiki/vault-and-brain.md`,
  `openwiki/quickstart.md`, `openwiki/development-and-validation.md`
- Promoted from intake: `INTAKE-knowledge-workspace` (REQ-012 there)
- Follow-ups opened: none
