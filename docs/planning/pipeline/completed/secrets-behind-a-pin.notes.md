---
title: Secrets behind a PIN: notes
pipeline_id: 3b0bcb0e-430a-4733-ac06-1f7ff3b35104
---

# Secrets behind a PIN: running notes

Chronological evidence and decisions. If a command did not run, these notes do not say it
passed.

## Phase 1: Plan

- Recall: bulletins (three notices). Register: `AD-rusty-mcp-only-back-end-001` (the app
  reaches the store only through the back end), §10 (nothing personal ships; nothing
  leaves the machine). Code read: `SecretsManager` (`list`, `get`, `set`, `delete`; the
  file written with mode 0600), the three `secret_*` tools, `SecretsPage.qml` (names,
  set, delete; "a value is typed once and never shown again"); the app never touches the
  file today; `sha2` is a dependency of `rusty-core`, `argon2` is not.
- Decisions: two locked in the spec; three open for the seal, with recommendations.
  Sealed at 17:12 (relayed by the rustal session): the server-owned shape, argon2 as a
  new dependency, the timeout as a setting; the safety rules Chad delegated are locked
  decisions 3 to 6.

## Phase 2: Design

- Architecture and data flow: a `PinLock` in `rusty-core` (`engine/pin_lock.rs`) owns
  the PIN: an argon2id hash (PHC string) at `~/.rusty/.pin` written with mode 0600, an
  in-memory state (failures, a lockout instant, one live token with its expiry). `set`
  takes the new PIN and, when one exists, the live token; `unlock` verifies, counts
  failures (five in a row lock for a minute, server-side), and hands back a random
  32-byte token good for the `pin_timeout_minutes` setting (default five); `check`
  accepts the live token until its expiry, clearing it after; `lock` clears it; `status`
  reports set, unlocked and any lockout. The server exposes six tools over both
  transports: `secret_pin_status`, `secret_pin_set`, `secret_unlock`, `secret_lock`,
  `secret_reveal` (a value against a live token) and `secret_update` (a value replaced
  against a live token, a `DataChanged`); `secret_list` stays name-only; nothing logs a
  PIN, a token or a value. The app keeps the token in the page's memory alone, masks by
  default, reveals one row at a time, relocks on a timer sized by the unlock's answer, on
  window focus loss and on Lock, and never touches `~/.rusty`. A server restart drops
  every token; the page relocks when the status says so.
- File manifest:
  - `crates/rusty-core/Cargo.toml`: `argon2 = "0.5"` (the seal's new dependency).
  - `crates/rusty-core/src/engine/pin_lock.rs` (new) and `engine/mod.rs`: `PinLock`, its
    constants, `Unlock`, `PinStatus`, six tests.
  - `crates/rusty-core/src/core.rs`: `pin_lock` on `Core` at `~/.rusty/.pin`.
  - `crates/rusty-mcp/src/main.rs`: four parameter structs, six tools, six names in the
    router test.
  - `crates/rusty-app/qml/SecretsPage.qml`: the PIN block (set, unlock, lock, change),
    the per-row reveal, edit and copy, the timers, the page text.
  - `crates/rusty-app/qml/SettingsPage.qml`: `pin_timeout_minutes` among the known keys.
  - Phase 5: `README.md` (a Secrets section, the tool count), `openwiki/quickstart.md` and
    `docs/architecture.md` (the count), `ROADMAP.md`, the wiki pages `mcp-back-end.md`
    and `workspace-app.md`.
- Store consequences: one new file, `~/.rusty/.pin` (0600), absent until a PIN is set;
  the secrets file's format and permissions unchanged; one new setting key.
- Tool contract: six tools added, none changed; the count goes from 65 to 71.
- Regression plan:
  | REQ | Evidence |
  |---|---|
  | REQ-001 | the `view:secrets` scene (no PIN in the scratch state: masked rows and the set-PIN offer) |
  | REQ-002 | `set_unlock_check_lock`, `a_short_pin_and_a_wrong_pin_are_refused`, `the_pin_file_is_private` |
  | REQ-003 | `an_expired_unlock_is_no_unlock`, `changing_the_pin_needs_the_live_unlock`; the page's timer, focus and Lock paths by reading |
  | REQ-004 | `five_wrong_pins_lock_out_for_a_minute` |
  | REQ-005 | `router_advertises_every_tool_once` with the six names; `secret_reveal` and `secret_update` call `check` first, by reading; `secret_list` untouched |
  | REQ-006 | the page text in the scene |
- Risks: data safety, the secrets file is written only through the existing `set`; a
  wrong token never reaches it. Secrets in memory: the app holds one revealed value at a
  time and drops it on lock. Logging: no `eprintln!` in the new code carries a PIN, a
  token or a value. Concurrency: one `Mutex` on the lock state, held across the argon2
  verify (tens of milliseconds) so two guesses cannot race the failure count. Keyboard:
  Enter unlocks, Enter saves an edit. No back end: the page shows the status text as
  before.
- CodeGraph evidence: `SecretsManager::get` has one caller today (the embedder's key
  lookup in `semantic.rs`); `secret_reveal` becomes the second and goes through `check`.
  `Core` is constructed in `core.rs` alone.

## Phase 3: Implement

- Built: the manifest as designed. `argon2 = "0.5"` and `rand_core = { version =
  "0.6", features = ["getrandom"] }` in `rusty-core` (the second names a crate argon2
  already pulls, for `OsRng`; argon2 re-exports `rand_core` without the `getrandom`
  feature, which was the first compile error). `PinLock` with six tests, `pin_lock` on
  `Core`, six tools and four parameter structs, six names in the router test. The Secrets
  page rewritten around the lock block (set, unlock, lock, change), the per-row reveal,
  edit and copy, the expiry timer, the focus relock and a status poll while a lockout
  runs; `pin_timeout_minutes` among the known settings.
- Deviations: none from the design. The reveal drops on any failed reveal or update (the
  token died server-side), and a failed unlock re-asks the status so a lockout shows.
- Fast gate: below.
- Evidence: `cargo test -p rusty-core pin_lock` → 6 passed; `cargo test -p rusty-mcp
  router_advertises` → 1 passed. The `view:secrets` scene at 17:23 on a fresh build shows
  the page text, the "No PIN yet" block with the set-PIN fields, the masked entry row and
  the empty vault (the scratch state has no PIN and no secret).

## Phase 3.5: Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Logging | Neither `rusty-mcp` nor the app's `backend.rs` logs a tool call, its parameters or its result (grep for `tracing::`, `eprintln!`, `println!` around the call paths: nothing); the new code logs nothing. | none | confirmed clean |
| 2 | Concurrency | `unlock` holds the state mutex across the argon2 verify, so two guesses cannot race the failure count; `status` waits those milliseconds. | low | accepted |
| 3 | UI state | A lockout showed a fixed number of seconds until the next status call. | medium | fixed: a five-second status poll runs while a lockout is on |
| 4 | Contract | `secret_pin_set` with no PIN on disk can be called over stdio, so an agent could set a PIN before Chad does. | low | documented: delete `~/.rusty/.pin` to start over; the file is his |
| 5 | Data safety | A revealed value lives in one page property and is dropped on lock, on a failed reveal or update, and on focus loss; the secrets file is written only through the existing `set`. | none | confirmed |
| 6 | Errors | A failed tool comes back as a JSON-RPC error (the server's shape for every failed tool), which the page shows as `tool: message`. | none | confirmed |

- Post-implementation CodeGraph: `SecretsManager::get` now has two callers (the embedder's
  key lookup and `secret_reveal`); `PinLock::check` is called by `secret_reveal`,
  `secret_update` and `PinLock::set`; nothing else reaches `~/.rusty/.pin`.

## Phase 4: Validate

- Tests run (commands and output): `cargo test -p rusty-core pin_lock` → 6 passed;
  `cargo test -p rusty-mcp router_advertises` → 1 passed; the workspace under the gate
  below.
- Gate run: below.
- Smoke evidence: a scratch `HOME` back end (`rusty-mcp --http 127.0.0.1:4199`) driven
  over Streamable HTTP at 17:24. In order: status set=false; `secret_set probe_key`;
  `secret_list` → `["probe_key"]`; `secret_reveal` with a made-up token → error;
  `secret_pin_set 12345` → error (too short); `secret_pin_set 123456` → set;
  `secret_unlock 000000` → error; `secret_unlock 123456` → a 64-character token,
  expires_in_seconds 300; status unlocked=true; `secret_reveal probe_key` → the value;
  `secret_reveal absent` → error; `secret_update` → updated; a reveal shows the new value;
  `secret_pin_set 654321` without the token → error; `secret_lock` → locked; a reveal with
  the old token → error; five wrong PINs → the fifth answers the lockout; the right PIN
  → error while locked out; status locked_out_seconds=59. Afterwards `.pin` and `.secret`
  both `-rw-------`, and the server log holds neither the PIN nor a value (grep count 0).
  The `view:secrets` scene at 17:23: the page text, the set-PIN block, the masked entry row.
- Skips or pre-existing failures: none.

## Phase 5: Complete

- Requirement audit: REQ-001 PASS (the scene: masked entry, the lock block; a set PIN
  shows the Unlock field by the page's `pinSet && !unlocked` branch). REQ-002 PASS
  (`a_short_pin_and_a_wrong_pin_are_refused`, `the_pin_file_is_private`, the probe's
  short-PIN refusal). REQ-003 PASS (the probe's reveal, update and lock; the expiry test;
  the page's timer, focus and Lock paths). REQ-004 PASS
  (`five_wrong_pins_lock_out_for_a_minute`, the probe's fifth wrong PIN and the 59-second
  status). REQ-005 PASS (the router test with the six names; `secret_list` untouched; the
  probe's reveal without a token refused). REQ-006 PASS (the page text in the scene).
- Docs: README (a Secrets section, the count), `docs/architecture.md`, ROADMAP, the wiki
  pages `mcp-back-end.md` and `workspace-app.md` through the OpenWiki run
  74d2683c (complete; the PostToolUse hook stayed silent again and the genuine result
  was fed to it by hand, as the bulletin describes), the quickstart's count.
- AAR: `AAR-015-secrets-behind-a-pin.md`. Register: `AD-rusty-secrets-pin-screen-lock-001`,
  `PR-rusty-argon2-brings-no-osrng-001`.
- Brain capture: the project page's timeline, after the commit.
- Archive: this pair to `completed/`.

## Defect and lesson ledger

| When | What | Lesson or rule ID |
|---|---|---|
