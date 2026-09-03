---
title: AAR-007-openwiki-in-the-workflow
pipeline_id: 3f1c7d2a-9b64-4e0f-8a5d-2c7e6b1f4d90
ticket: TICKET-007
submitted: 2026-09-03
---

# AAR-007-openwiki-in-the-workflow

## Recall log

- Register: `AD-rusty-brain-is-the-project-memory-001` (memory stays with the brain),
  `AD-rusty-local-work-record-001`, `PR-rusty-glob-loops-not-ls-pipes-001`.
- Completed notes: AAR-001 (the workflow), AAR-002 (the workspace).
- OmarchyGS: the setup, check and wrapper scripts, the skill, the receipt hooks, the
  generated wiki. OpenWiki 0.3.3 source: the host session lifecycle.

## 1. Outcomes

- REQ-001 PASS, REQ-002 PASS, REQ-003 PASS, REQ-004 PASS, REQ-005 PASS, REQ-006 PASS.
  Evidence in the pipeline notes, Phases 4 and 5.

## 2. What went well

- OmarchyGS's pins, patches and provenance carried over unchanged; the build came up on
  the first run from the local mirror.
- The receipt shape from TICKET-001 fit the completion receipt without change, so the
  hook, the verify path and the commit gate were small additions with self-tests.
- Driving the MCP lifecycle from a scratch stdio client kept the first wiki honest: the
  receipt came from the real `openwiki_finish` result through the real hook.

## 3. What went poorly

- The identity check between the two agent guides failed after the first run, because
  OpenWiki writes a full section into `AGENTS.md` and a pointer into `CLAUDE.md`; the
  check now strips the managed block.
- An evidence range past a file's end failed the whole claims batch; ranges are read
  from the file first now.

## 4. Surprises

- OpenWiki's `init` backs the previous wiki up and rolls it back when the run fails.
- The finish step removes `_plan.md`, fills each page's `sources` from its claims'
  evidence and writes `generated` and `verified` stamps into the frontmatter.

## 5. Lessons

- PR-rusty-openwiki-managed-block-001: the two agent guides are identical outside
  OpenWiki's managed block; never hand-edit the block, and compare the files with it
  stripped.
- PR-rusty-openwiki-evidence-ranges-001: `resolve_claims` rejects the whole batch when
  one `repo://path#Lx-Ly` range does not exist; count the file's lines before citing.
- AD-rusty-openwiki-for-documentation-001: OpenWiki, pinned and project-local, is the
  generated engineering wiki; the host agent authors it through the MCP lifecycle at
  Phase 5 and the completion receipt gates delivery. The brain stays the memory.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 0.5 h | 0.5 h |
| 2 Design | 0.5 h | 0.5 h (reading OmarchyGS and OpenWiki's source) |
| 3 Implement | 2 h | 2 h |
| 3.5 Inspect | 0.5 h | 0.5 h |
| 4 Validate | 1 h | 1 h (the first wiki, two runs) |
| 5 Complete | 0.5 h | 0.5 h |
