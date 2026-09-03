---
title: Secrets behind a PIN
pipeline_id: 3b0bcb0e-430a-4733-ac06-1f7ff3b35104
status: Phase 5 — Complete: PASS (delivered 2026-09-03)
ticket: TICKET-015
ticket_doc: docs/planning/tickets/open/TICKET-015-secrets-behind-a-pin.md
aar: docs/planning/knowledge/aar/AAR-015-secrets-behind-a-pin.md
sealed: Chad, 2026-09-03 17:12, in the rustal session (relayed): "that sounds good to me ill defer you on safety", said to the server-owned PIN shape with argon2 accepted as a new dependency; the safety rules delegated to the rustal session are recorded under Locked decisions
created: 2026-09-03
---

# Secrets behind a PIN: spec

## Intent

Secrets can be seen and changed in the app behind a PIN, while no agent gains a way to
read a value it could not read before. Chad, 2026-09-03 15:40: "secrets should be
viewable and editable with a pin". The PIN protects the screen, not the disk: the file at
`~/.rusty/.secret` stays owner-readable plaintext, because the back end reads it headless
for the embeddings key.

## Scope

- In: a PIN (set once, six or more digits or a passphrase, stored as a salted hash with
  mode 0600), the unlock with a five-minute expiry and a lockout after five wrong tries,
  per-row reveal, inline edit, copy, the Lock control, the page text that says what the PIN
  protects, docs and wiki.
- Out (named seams, not forgotten): encryption at rest (the system keyring, its own
  ticket if wanted); a PIN on other views; any change to the file's format.

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN the Secrets tab opens, values shall be masked and an Unlock control shall ask for the PIN. | screenshot |
| REQ-002 | WHEN no PIN is set, the tab shall offer to set one (six or more digits, or a passphrase), stored as a salted hash in a file with mode 0600. | unit on the hash file |
| REQ-003 | WHEN the correct PIN is entered, the tab shall offer a per-row reveal showing one value at a time, inline edit saved on Enter, and a copy button, and the unlock shall end after five minutes (a setting), on window focus loss, and on Lock. | test of the expiry; smoke |
| REQ-004 | WHEN five wrong PINs are entered in a row, further tries shall be refused for one minute. | unit |
| REQ-005 | The server shall keep `secret_list` name-only, and no tool shall return a value without a live unlock that only the PIN grants. | tool-list test; tool tests |
| REQ-006 | WHEN the Secrets tab is shown, it shall state what the PIN protects and that the file stays owner-readable for the back end and agents. | doc review |

## Open before the seal (settled by the seal of 17:12)

1. **Where the reveal lives.** The ticket's draft has the app read and write the secrets
   file itself through its own Rust type, an exception to
   `AD-rusty-mcp-only-back-end-001`. The rustal session's counter-proposal, which this spec
   recommends: the server owns the PIN (hash in `~/.rusty/.pin`, 0600, the lockout) and
   gains `secret_unlock(pin)` returning a token that expires in five minutes, plus
   `secret_reveal(key, token)` and `secret_update(key, value, token)`; the app never
   touches the file; the tools exist on both transports and return nothing without the
   token, which only the PIN grants, and the PIN is typed in the app alone. An agent with
   a shell can read the file regardless, so this adds no exposure the file does not have.
2. **The hash.** argon2id is the right primitive and `argon2` would be a new dependency
   (a seal item under §3). The alternative with no new dependency is a salted, iterated
   SHA-256 through the `sha2` crate already in `rusty-core`, weaker against a stolen hash
   file but adequate for a six-digit screen lock. The recommendation is `argon2`.
3. **The Lock timer's home.** Five minutes as a setting (`pin_timeout_minutes`), or
   fixed. The recommendation is the setting, since the ticket names it.

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | The file format and permissions do not change; the PIN is a screen lock. | The back end reads the file headless; a PIN as a key would break the embeddings at every start. | Encrypting the file with the PIN. |
| 2 | `secret_list` stays name-only. | Nothing an agent could not already read appears in a tool answer by default. | A list with values behind the token (one call reveals everything). |
| 3 | The back end owns the PIN: an argon2id hash at `~/.rusty/.pin` (0600), set through the app when none exists, six or more characters. | The seal; the app reads and writes nothing under `~/.rusty` itself (`AD-rusty-mcp-only-back-end-001` holds). | The app-side file read in the draft. |
| 4 | `secret_unlock(pin)` returns a random token good for `pin_timeout_minutes` (default five), invalidated by `secret_lock`, by a server restart and by any new unlock; five wrong PINs in a row refuse further tries for one minute, counted server-side. | The safety rules delegated by Chad. | A per-session lockout (an agent could retry from a new session). |
| 5 | `secret_reveal(key, token)` and `secret_update(key, value, token)` need a live token; `secret_list` stays name-only; no reveal exists without a token on either transport; tokens and values never reach a log. | The same. | A reveal tool without a token. |
| 6 | The app masks by default, reveals one value at a time per row, relocks on the timer, on window focus loss and on Lock; its text says the PIN protects the screen while the file stays owner-readable, and that the PIN is never typed to an agent. | The same. | A page that stays open once unlocked. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-015-secrets-behind-a-pin.md` (the seal note
  added on 2026-09-03 at review)
- Intake: none
- Design references: `crates/rusty-core/src/engine/secrets_manager.rs`, the three
  `secret_*` tools in `crates/rusty-mcp/src/main.rs`, `crates/rusty-app/qml/SecretsPage.qml`;
  the rustal session's message of 2026-09-03 on the server-owned shape
- Architecture: `AD-rusty-mcp-only-back-end-001`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope settled; **sealed by Chad** (a new dependency, the reveal's home) |
| 2 Design | Architecture, file manifest, regression plan, CodeGraph evidence | design actionable |
| 3 Implement | The manifest, built | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger, post-implementation CodeGraph | confirmed findings resolved |
| 4 Validate | Regression tests run, `bin/gate.sh --diff` green, receipt | receipt matches worktree |
| 5 Complete | Requirement audit, docs, AAR, register, brain capture, archive | pair archived |
