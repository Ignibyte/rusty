---
title: TICKET-015-secrets-behind-a-pin
status: open
ticket_number: 015
type: feature
created: 2026-09-03
intake:
pipeline_spec: docs/planning/pipeline/active/secrets-behind-a-pin.spec.md
---

# TICKET-015-secrets-behind-a-pin

## Summary

Secrets can be seen and edited in the app behind a PIN, while the server keeps never
returning a value.

## Why

The Secrets tab is write-only by design: a value is typed once, the file at `~/.rusty/.secret`
is owner-readable plaintext, and `secret_list` returns names, so no agent can pull a value
through the MCP tools. Chad wants to see and change values in the app. The PIN protects the
screen, not the disk: an agent with a shell reads the file regardless, and the back end must
read it headless for the embeddings key, so the PIN cannot be an encryption key. A reveal
tool on the server would hand values to every MCP client, so the reveal belongs in the app
alone.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the Secrets tab opens, values shall be masked and an Unlock control shall ask for the PIN. | screenshot |
| REQ-002 | WHEN no PIN is set, the tab shall offer to set one (six or more digits, or a passphrase), stored as an argon2id hash in `~/.config/rusty/pin` with mode 0600. | unit on the hash file |
| REQ-003 | WHEN the correct PIN is entered, the tab shall offer a per-row reveal showing one value at a time, inline edit saved on Enter, and a copy button, and the unlock shall end after five minutes (a setting), on window focus loss, and on Lock. | QML test of the timer; smoke |
| REQ-004 | WHEN five wrong PINs are entered in a row, the tab shall refuse further tries for one minute. | unit |
| REQ-005 | WHEN the app reads or writes a value, it shall do so through its own Rust type (cxx-qt) on the secrets file, and the server shall keep `secret_list` name-only with no reveal tool. | tool-list test; review |
| REQ-006 | WHEN the Secrets tab is shown, it shall state what the PIN protects and that the file stays owner-readable for the back end and agents. | doc review |

## Scope

- In: the PIN store, the app-side reader and writer, the reveal, edit and copy affordances, the lock timers, the page text.
- Out: encryption at rest (the system keyring, a separate ticket if wanted); a PIN on other views.

## Notes

- Pipeline spec: docs/planning/pipeline/active/secrets-behind-a-pin.spec.md
- Related docs: `crates/rusty-core/src/engine/secrets_manager.rs`, `crates/rusty-app/qml/SecretsPage.qml`, the `secret_*` tools in `crates/rusty-mcp/src/main.rs`.
- Promoted from intake: none; drafted by the rustal session on 2026-09-03 from Chad's words at 15:40: "secrets should be viewable and editable with a pin".
- Decision for the seal: REQ-005 has the app read and write the secrets file itself, an
  exception to `AD-rusty-mcp-only-back-end-001` (the app reaches the store only through the
  back end); the spec must amend that decision or find a shape that keeps it.
- Follow-ups opened: none.
