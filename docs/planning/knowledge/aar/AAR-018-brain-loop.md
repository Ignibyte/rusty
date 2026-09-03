---
title: AAR-018-brain-loop
pipeline_id: 3d2d1091-fee1-4e82-9f3e-1d944b2803c3
ticket: TICKET-018
submitted: 2026-09-03
---

# AAR-018-brain-loop

## Recall log

- As in the notes; the seal relayed at 17:20 with the rustal session's four answers.

## 1. Outcomes

- REQ-001 to REQ-008 PASS. Evidence in the pipeline notes, Phases 3 to 5.

## 2. What went well

- The frontmatter's `extra` map made a decision page one write with its properties, and
  the typed edges read from the same keys, so the vault stayed the truth without a table.
- The corpus tests run the shipped scripts under bash, so the hook and its test cannot
  drift apart; rustal's transcript predicates were the model.
- The rustal session's review caught the scope hole (a scratch script tripping the gate)
  before the commit.

## 3. What went poorly

- The formatter rewrapped a test between two edits, and an anchor-based insert had to
  replace an exact-string one.

## 4. Surprises

- `deploy_seeds` writes seeds only when the active store has no skill, so the box's
  store needs the skill created from the seed text by hand; a fresh install gets it.

## 5. Lessons

- Gate on the repository, not on the tool: a write hook scoped to the working directory
  leaves scratch work alone.
- `due` and `decision_edges` read the decision files; index a property when decisions
  count in the thousands.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 20 min | 15 min |
| 2 Design | 30 min | 30 min |
| 3 Implement | 120 min | 90 min |
| 3.5 Inspect | 15 min | 10 min |
| 4 Validate | 15 min | 15 min |
| 5 Complete | 40 min | 40 min |
