---
title: AAR-027-personal-search-engine
ticket: TICKET-027
pipeline: 920db1c8-b417-413c-86d6-1c246551ff7d
status: closed
created: 2026-09-05
submitted: 2026-09-05
---

# AAR-027: Personal search engine

## 0. Recall log

- The intake set the boundaries before the code: clean-room, no second protocol, no
  crawler; untrusted marking from the first commit.
- A source is a page; the index, the search operators and the embedder loop were all
  already there. The work is the fetch, the extractors, the mark and the front doors.

## 1. Outcome

The first pass of the personal search engine: a `source` page type, capture by URL
through three tools, the CLI and a dialog, the brain's own index reused, and every
answer an agent gets about a source marked untrusted and normalised. A new module, two
methods, one type folder, seven tests, one scene, 85 tools. `GATE GREEN [diff]`.

## 2. What went well

- Splitting the capture into `fetch` (the network) and `capture_fetched` (the page)
  made the whole feature testable without a byte leaving the box; the fixture bodies
  are the tests.
- The mark lives in the tool layer on the JSON, so the app's structs and its reads are
  untouched, and five answers were covered by one function.
- `ureq` was already a dependency and `pdftotext` is on the box: no crate for the fetch
  or the PDF, and nothing of hister's read.

## 3. What went poorly

- A fixture name collision: `test_brain("capture")` was also an older test's vault, and
  the two ran in parallel on one temporary directory. The fast gate passed by ordering
  luck; the delivery gate caught it. A fixture name is a namespace: the test's own name,
  never a word two tests share.
- The wiki needed a second pass: F8 changed `brain_render` after its claim had been
  inspected, and the seeded source moved the script lines the scenes claim cited.
  Inspect claims after the last code edit.
- Two slug expectations of mine, and the screenshot script's `${scene/reading/}` —
  a latent bug since the script began, found by a scene path that held the word.

## 4. Surprises

- OpenWiki's finish verifies every cited range on a page, not only the claims a run
  touched; one edit after the inspect leaves the whole sidecar unchanged, with a warning
  as the only sign.
- `ETXTBSY` between test threads: one writes its fake script while another forks to
  exec its own, and the child inherits the writer's descriptor until the exec. A lock and
  a rename settle it.

## 5. Lessons

- `AD-rusty-sources-are-pages-marked-untrusted-001`: a source is a page; the capture
  is one GET and an in-house read; every agent answer about it is marked in the tool
  layer.
- A test's temporary vault is named after the test.
- Resolve wiki claims after the last code edit of the pipeline — F8 was the third
  time this cost a pass.

## 6. Time spent

| Phase | Estimated | Actual |
|---|---|---|
| 1 Plan | 15m | 10m |
| 2 Design | 30m | 25m |
