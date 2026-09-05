---
title: Personal search engine
pipeline_id: 920db1c8-b417-413c-86d6-1c246551ff7d
status: Phase 5 — Complete PASS
ticket: TICKET-027
ticket_doc: docs/planning/tickets/open/TICKET-027-personal-search-engine.md
aar: docs/planning/knowledge/aar/AAR-027-personal-search-engine.md
sealed: three tools (85), one page type, one CLI family, one dialog; no new crate (the fetch is `ureq`, already a dependency; PDF text is `pdftotext` when the box has it); clean-room from hister's idea, none of its source read
created: 2026-09-05
---

# Personal search engine: spec

## Intent

The brain holds what Chad writes; nothing holds what he merely read. A `source` page
type and a capture path give "where did I read that" an answer from the app, the CLI and
an agent, through the index Rusty already has, with nothing leaving the box but the
fetch of the URL he chose. Everything an agent gets back about a source is marked
untrusted and normalised, from the first commit.

## Scope

- In: the `source` page type under `sources/`; capture by URL (`source_capture`,
  `rusty-cli source capture <url>`, a dialog in the app): fetch, extract (HTML, PDF,
  markdown, plain text), write the page with `url`, `site`, `captured` and `kind`, index
  it as any page; a URL captured again updates its page; a failure recorded on the page;
  `source_search` and `source_preview` with every answer marked untrusted and normalised,
  and the same marking on `brain_search` hits and `brain_read_page` answers for a source;
  "Sources: Search sources" and a glyph in the explorer; a seeded source page for a scene.
- Out (named seams, not forgotten): a web UI, a REST layer, OAuth, multi-user (Rusty's
  non-goals); a headless-browser crawler; Postgres; a `rusty capture` script in the store
  (one `capture.sh` beside a skill calls `rusty-cli source capture "$1"` — Chad's store,
  not the repo); readability-grade extraction (the extractor is a small state machine);
  a Sources tab of its own (the folder, the search operator and the glyph serve).

## Acceptance criteria (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN given a URL, the system shall fetch it, extract its readable text, and write a `source` page carrying `url`, `title`, `site` and `captured`. | `capture_writes_updates_and_records_failures` (the page and its keys from a fetched body); `extract_html_keeps_the_article_and_drops_the_chrome`; a real capture by Chad |
| REQ-002 | WHEN a `source` page is written, the system shall index its full text and, where a provider is set, its embedding, through the index the brain already uses. | the capture test's `search("… type:source")`; reading: `sync_page` is the one indexer and the embedder loop takes stale pages |
| REQ-003 | WHEN an agent searches sources over MCP, every result shall be marked as untrusted content and normalised before it can reach a model. | `mark_flags_a_source_and_leaves_a_page`, `normalise_strips_controls_and_caps`; reading of the three tools and the two marked ones |
| REQ-004 | WHEN a URL already captured is captured again, the system shall update that page rather than create a second one. | the capture test (the same slug, the new title, one page) |
| REQ-005 | WHEN extraction fails, the system shall record the failure on the page rather than write an empty source. | the capture test (a new page says why; a page that had text keeps it and gains `status: failed` and `error`) |
| REQ-006 | WHEN sources exist, they shall be searchable and browsable from the app alongside brain pages, and distinguishable from them. | the `open:sources/…` scene (the glyph in the tree, the type and the `url` in the page); "Sources: Search sources" is `type:source` |

## Locked decisions

| # | Decision | Why | Alternatives set aside |
|---|---|---|---|
| 1 | A source is a page: type `source`, folder `sources/`, frontmatter `url`, `site`, `captured`, `kind`, the extracted text as the body; indexed by `sync_page` like every page, embedded by the same loop | Files are the truth; one index (`AD-rusty-tags-one-index-001`'s spirit); the explorer, search, links and the graph get sources for free | a second table or engine |
| 2 | The fetch is `ureq` (a dependency already), http and https only, twenty seconds, five redirects, eight megabytes, a `rusty` user agent; the extractor is a small HTML state machine in Rust (title from `<title>`, the text of `<main>` or `<article>` when present, else the body, scripts, styles and the like dropped, block tags as line breaks, entities decoded); PDF text is `pdftotext` as a process when the box has it, markdown and plain text as they are | No new crate, nothing of hister's read; readability-grade extraction is a seam | the `readability` crate; a PDF crate |
| 3 | A URL's page is found by its `url` property among `sources/`; a recapture rewrites the page keeping `created`; a failure on a new URL writes a page that says why (`status: failed`, `error`), a failure on a captured URL keeps the text and records the failure in the frontmatter | REQ-004 and REQ-005 without a table; the vault is the truth | a URL table in SQLite; overwriting good text with an error |
| 4 | Every MCP answer that carries a source's text is marked and normalised in the tool layer: `source_search`, `source_preview`, and the hits and pages of type `source` in `brain_search` and `brain_read_page`; `untrusted: true`, a `note`, control characters out, the text capped | hister's principle, taken as a principle: a page from the web is data, never instructions; the app ignores the mark | marking in core (the app reads the same structs) |
| 5 | The app adds a capture dialog and a search command, and marks a `sources/` page with its own glyph in the tree; no Sources tab | The folder, the `type:source` operator and the page header distinguish a source; a tab is a later ticket if the folder is not enough | a Sources view |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-027-personal-search-engine.md`; intake:
  `docs/planning/intake/INTAKE-personal-search-engine.md` (the licence boundary)
- Register: `AD-rusty-vault-rules-001`, `AD-rusty-mcp-only-back-end-001`, `AD-rusty-search-operators-in-core-001`
- Architecture: `openwiki/vault-and-brain.md`, `openwiki/mcp-back-end.md`, `openwiki/workspace-app.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spec, notes, open AAR | scope settled |
| 2 Design | Manifest, the page, the fetch and extractors, the marking, regression table | design actionable |
| 3 Implement | `brain/sources.rs`, `brain/mod.rs`, `vault.rs`, `rusty-mcp`, `rusty-cli`, `Main.qml`, `Explorer.qml`, the script, the counts | `bin/gate.sh --fast` green |
| 3.5 Inspect | Finding ledger; CodeGraph over the capture path | confirmed findings resolved |
| 4 Validate | The tests, the scene, `--diff` green | receipt matches worktree |
| 5 Complete | Audit, wiki, AAR, register, brain, archive | pair archived |
