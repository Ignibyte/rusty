---
title: AAR-014-notes-into-the-vault
pipeline_id: 01151524-3b0d-40a5-abd2-8f53a3bff0e9
ticket: TICKET-014
submitted: 2026-09-03
---

# AAR-014-notes-into-the-vault

## Recall log

- Register: files are the truth, lenient pages, the back end decision, throwaway rows.
  Code: the notes manager and its tests, the path default in `core.rs`, the six notes
  tools, the store's `/note` skill. The box: five notes and no `notes/` folder in the
  vault.

## 1. Outcomes

- REQ-001 to REQ-006 PASS. Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- The vault already typed a `notes/` page as `note`, so the store change was a default
  path and a one-shot; the four adoption tests wrote the refusal rules down before the
  command touched a real folder, and the dry run on the box proved the plumbing without
  moving a file.

## 3. What went poorly

- A docs edit ran from the skills store's directory after a `cd` in the same script and
  failed on a relative path; scripts that change directory should use absolute paths for
  everything after.

## 4. Surprises

- None.

## 5. Lessons

- No new register entries; `AD-rusty-files-are-the-truth-001` and
  `AD-rusty-lenient-pages-001` carried the design.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.3 h | 0.3 h |
| 2 Design | 0.3 h | 0.3 h |
| 3 Implement | 1 h | 0.6 h |
| 3.5 Inspect | 0.3 h | 0.2 h |
| 4 Validate | 0.3 h | 0.3 h |
| 5 Complete | 0.3 h | 0.3 h |
