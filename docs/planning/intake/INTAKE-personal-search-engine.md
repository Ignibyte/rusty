---
title: INTAKE-personal-search-engine
status: promoted
created: 2026-09-04
ticket: TICKET-027
---

# INTAKE-personal-search-engine

## Problem or opportunity

Chad, 2026-09-04, after reading [asciimoo/hister](https://github.com/asciimoo/hister): *"see if it
would be useful on our roadmap to rebuild as part of rusty. its very interesting"* — and, on being
offered three ways in, *"3 would be the idea."* Option 3 was the full rebuild in Rust, not the small
capture ticket and not running hister alongside.

The gap it answers is real and Rusty is already half of the way there. **The brain holds what Chad
deliberately writes; nothing holds what he merely passed through.** The SessionEnd hook has already
archived 139 `conversations` pages, so one stream of "what went by" is captured automatically. Web
reading and the files on disk are the other streams, and there is no `source` page type, no capture
path, and no prior art anywhere in `docs/planning/`.

Hister is the proof the shape works: full-text over page *contents* rather than titles and URLs, a
real query language, optional semantic search, and an MCP surface so an agent can search a person's
own reading. 3.6k stars in eight months, by the author of searx.

## Proposed outcome

Rusty can answer "where did I read that" over everything Chad has seen — pages, local documents, and
the brain — from the app, the CLI and an agent, with no second service and nothing leaving the box.

## Scope notes

**In**

- A `source` page type and an ingest path: URL in, readable text out, written as a page with `url`,
  `title`, `site` and `captured` frontmatter. Files are the truth, so a captured page is a file.
- Content extraction for the formats worth having: HTML (readability), PDF, plain text, markdown.
- Full-text and semantic search over sources, reusing the index Rusty already has rather than adding
  a second engine.
- MCP tools for search and preview, and a Sources view in the app.
- **Untrusted-content handling.** Hister wraps every MCP result as an untrusted record and normalises
  it before a model sees it. Anything that ingests arbitrary web pages and hands them to an agent
  needs this from the first commit, not as a hardening pass.

**Out, and why**

- **No web UI, no REST layer, no browser extension talking HTTP, no OAuth, no multi-user.** All four
  are explicit Rusty non-goals and principles; hister has all four. This is the line that keeps a
  rebuild from turning Rusty into a different program. Capture arrives through the existing back end
  or a `rusty <name>` script, not a new protocol.
- **No headless-Chrome crawler** in a first pass. Site crawling is where hister's weight is; a
  fetch-and-extract of a page Chad is looking at delivers most of the value.
- **No Postgres.** SQLite holds the index, as it does now.

## Licence boundary — read before writing code

Hister is **AGPL-3.0**. Rusty is **MIT** and public. Not a line of hister may be copied, and linking
it would force Rusty to AGPL. The rebuild has to be clean-room: take the *idea* from the README, the
feature list and the docs, and do not read `asciimoo/hister`'s source while writing Rusty's. Whoever
picks this up should say so in the ticket, because "explore the repo" and "reimplement it" pull in
opposite directions and the boundary is easy to cross by accident.

## Sizing, honestly

Hister is 1.9 MB of Go, 560 KB of Svelte and 115 KB of TypeScript, plus two browser extensions. The
index core is the tractable part — bleve maps onto tantivy, and Rusty's semantic index already
exists. The cost is the periphery: extractors per format, the crawler, the query language, the
capture path. Cutting the Out list above is what makes this a Rusty feature rather than a second
product; without those cuts it is a months-long project of its own.

## Candidate requirements (EARS)

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN given a URL, the system shall fetch it, extract its readable text, and write a `source` page carrying `url`, `title`, `site` and `captured`. | test + a real capture |
| REQ-002 | WHEN a `source` page is written, the system shall index its full text and, where a provider is set, its embedding, without a separate index engine. | test |
| REQ-003 | WHEN an agent searches sources over MCP, the system shall return every result marked as untrusted content and normalised before it reaches a model. | test + review |
| REQ-004 | WHEN a captured URL already exists as a source, the system shall update that page rather than create a duplicate. | test |
| REQ-005 | WHEN the extractor cannot read a page, the system shall record the failure on the page rather than write an empty source. | test |

## References

- [asciimoo/hister](https://github.com/asciimoo/hister) — the prior art. README and docs only; see the
  licence boundary above.
- `ROADMAP.md` — Principles ("One back end process… No web UI, no REST layer, no second protocol";
  "Files are the truth") and Non-goals ("No browser UI… no multi-user").
- TICKET-010 (scripts as commands) — `rusty capture <url>` needs no new dispatch machinery.
- TICKET-016 (folders from the machine) — local files are already partly in scope.
- The 139 `conversations` pages from the SessionEnd hook — the existing precedent for archiving a
  stream automatically.

## Promotion

- [x] Ticket created under `docs/planning/tickets/open/` and rowed into `INDEX.md`.
- [ ] Spec/notes pair created under `docs/planning/pipeline/active/`.
- [x] `ticket:` set; `status:` set to `promoted`.
