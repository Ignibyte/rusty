---
title: AAR-015-secrets-behind-a-pin
pipeline_id: 3b0bcb0e-430a-4733-ac06-1f7ff3b35104
ticket: TICKET-015
submitted: 2026-09-03
---

# AAR-015-secrets-behind-a-pin

## Recall log

- Register: the back end decision and the product boundaries. Code: the secrets manager,
  the three tools, the Secrets page; the app never touches the file; `argon2` would be
  new. Awaiting Chad's seal on the reveal's home and the hash.

## 1. Outcomes

- REQ-001 to REQ-006 PASS. Evidence in the pipeline notes, Phases 4 and 5: six unit
  tests on the lock, the router test with the six names, the scratch back end driven over
  HTTP through the whole contract, the `view:secrets` scene.

## 2. What went well

- The seal settled the shape before a line was written: the rustal session's
  server-owned counter-proposal replaced the draft's app-side file read, and the
  standing decision (`AD-rusty-mcp-only-back-end-001`) held without an exception.
- The scratch back end over Streamable HTTP proved the lockout, the token life and the
  file modes in one run, which no unit test covers end to end.

## 3. What went poorly

- The first build failed on `argon2::password_hash::rand_core::OsRng`; the crate
  re-exports `rand_core` without `getrandom`. Ten minutes and one more dependency line.
- The first `view:secrets` scene ran the debug binaries from 16:32 and showed the old
  page; `scripts/screenshot.sh` builds nothing, its header says so, and the eye did not
  read it. A fresh build fixed it.

## 4. Surprises

- The server's `instructions` text counts the tools from the router, so it said 71 on
  its own; three docs carried a hand-written 65.

## 5. Lessons

- `PR-rusty-argon2-brings-no-osrng-001`: name `rand_core` with `getrandom` next to
  `argon2`.
- Build before a scene: `scripts/screenshot.sh` runs whatever debug binaries exist.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 20 min | 15 min (plus the wait for the seal) |
| 2 Design | 20 min | 15 min |
| 3 Implement | 60 min | 45 min |
| 3.5 Inspect | 10 min | 10 min |
| 4 Validate | 20 min | 20 min |
| 5 Complete | 30 min | 30 min |
