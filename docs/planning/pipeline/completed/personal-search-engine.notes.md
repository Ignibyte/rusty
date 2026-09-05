---
title: Personal search engine — notes
pipeline: 920db1c8-b417-413c-86d6-1c246551ff7d
ticket: TICKET-027
---

# Personal search engine: notes

## Recall (2026-09-05)

- The intake is binding: clean-room from the idea (hister is AGPL-3.0, Rusty MIT); no
  web UI, REST, OAuth or multi-user; no crawler; SQLite. None of hister's source was
  opened for this pipeline.
- Bulletins: bulletin 2 shapes validation; no test fetches the network — the capture is
  split into `fetch` (network) and `capture_fetched` (pure from a body).
- Register: `AD-rusty-vault-rules-001`, `AD-rusty-mcp-only-back-end-001`,
  `AD-rusty-search-operators-in-core-001` (`type:source` is already a search operator).
- Wiki: `vault-and-brain.md` (type folders, lenient pages, `sync_page`, the semantic index
  embedding stale pages when a provider is set); `mcp-back-end.md` (`json_result`,
  `mutate`, `brain_search` routing to hybrid or text search).
- Code read:
  - `vault.rs`: `TYPE_DIRS` maps a type to its folder and `ensure_dirs` creates them;
    `title_to_slug` lowers and strips; `unique_slug` suffixes.
  - `frontmatter.rs`: `BrainFrontmatter::new(type, title)` with `created`/`updated`
    today and `extra` flattened into the YAML; `render_page(fm, body, timeline)`;
    `set_property(raw, key, value)`.
  - `mod.rs`: `create_page` writes then indexes; `sync_page(slug)` indexes one page from
    disk (text, tags, links); `search_with` takes the operators; `read_page` gives a
    `BrainPage`; `BrainSearchResult` is built in five places, so the mark goes on the
    JSON in the tool layer rather than on the struct.
  - `semantic.rs`: `ureq` 3.4 with `Agent::config_builder().timeout_global(…)`; the
    embedder loop in the server takes stale pages.
  - `rusty-mcp`: `brain_search` runs on `spawn_blocking` and answers `json_result(results)`;
    `brain_read_page` answers the page; `SlugParams`.
  - `rusty-cli`: `run_brain(sub, rest)` dispatched from `(Some("brain"), Some(sub))`.
  - `Explorer.qml`: a page row's glyph is `◇`/`◆`; the row knows its path.
  - `pdftotext` is at `/usr/bin/pdftotext` on this box.

## Phase 2: Design

### File manifest

| File | Change |
|---|---|
| `crates/rusty-core/src/brain/sources.rs` | new: `Fetched`, `Extracted`, `fetch`, `kind_of`, `extract`, `extract_html`, `extract_pdf`, `site_of`, `slug_base`, `normalise`, `mark`, `mark_hits`, `UNTRUSTED_NOTE`; tests |
| `crates/rusty-core/src/brain/mod.rs` | `pub mod sources`; `source_for_url`, `capture_url`, `capture_fetched`; a test |
| `crates/rusty-core/src/brain/vault.rs` | `("source", "sources")` in `TYPE_DIRS` |
| `crates/rusty-mcp/src/main.rs` | `SourceCaptureParams`, `SourceSearchParams`; `source_capture`, `source_search`, `source_preview`; the mark on `brain_search` and `brain_read_page`; `EXPECTED` |
| `crates/rusty-cli/src/main.rs` | `source capture <url>`, `source search <query>` and their help |
| `crates/rusty-app/qml/Main.qml`, `Explorer.qml` | "Sources: Capture a URL…" with `captureDialog`, "Sources: Search sources"; the `⌁` glyph for a `sources/` page |
| `scripts/screenshot.sh` | a seeded source page |
| `CLAUDE.md`, `AGENTS.md`, `README.md`, `docs/architecture.md` | 85 tools |

### The capture

`capture_url(url)` = `capture_fetched(url, fetch(url))`. `fetch`: the scheme must be
http or https; `ureq` with a twenty-second timeout, five redirects, the body read under
an eight-megabyte cap; the content type header kept. `kind_of(content_type, url, bytes)`:
`pdf` from the type, the `.pdf` suffix or `%PDF` at the start; `markdown` from the type
or `.md`; `html` from the type or `<html`/`<!doctype` in the first kilobyte; else
`text`. `extract`: html through the state machine (title from `<title>`, else the first
`<h1>`, else the URL; the text of `<main>` or `<article>` when the document has one,
else `<body>`; `script`, `style`, `noscript`, `svg`, `template`, `iframe` dropped;
headings as `##` lines, list items as `- `, block tags as breaks, entities decoded,
whitespace collapsed, blank runs to one); pdf through `pdftotext -layout -enc UTF-8`
on a temporary file (missing `pdftotext` is a recorded failure); markdown and text as
they are, cut at a megabyte. `capture_fetched`: `source_for_url` finds the page whose
`url` property is the URL (a walk of `sources/`); the slug is that or
`unique_slug("sources", "<site>-<title slug>")`; the frontmatter `type: source` with
`url`, `site`, `captured` (local time, ISO), `kind`, `created` kept from an existing
page; the body the text; `vault.write_page`, `sync_page`, one commit. A failure on a new
URL writes the page with `status: failed`, `error`, and a body that says so; a failure
on a captured URL sets those two keys on the page as it is.

### The mark

`normalise(text)`: control characters out (tab and newline kept, carriage returns
dropped), runs of blank lines to one, the text cut at four thousand characters with an
ellipsis. `mark(value)`: a JSON object whose `page_type` is `source` gains
`untrusted: true` and `note: UNTRUSTED_NOTE`, and its `snippet`, `compiled_truth` and
`timeline` strings are normalised; `mark_hits` does it to every element of an array.
`source_search` runs `search_with` on the query with ` type:source` appended and marks
the hits; `source_preview` reads the page, refuses one that is not a source, marks it;
`brain_search` and `brain_read_page` mark what they answer. The app parses the same JSON
and ignores the mark.

### Regression table

| Requirement | Evidence |
|---|---|
| REQ-001 | `capture_writes_updates_and_records_failures` (the keys, the body); `extract_html_keeps_the_article_and_drops_the_chrome`; `kind_of_reads_the_type_then_the_bytes`; `fetch_refuses_other_schemes` |
| REQ-002 | the capture test's `search("… type:source")` and `read_page().page_type`; reading of `sync_page` and the embedder loop |
| REQ-003 | `normalise_strips_controls_and_caps`; `mark_flags_a_source_and_leaves_a_page`; reading of the tools |
| REQ-004 | the capture test: the second capture keeps the slug, changes the title, one page |
| REQ-005 | the capture test: a failure on a new URL says why; on a captured URL the text stays and `status`/`error` appear |
| REQ-006 | the `open:sources/example-com-reading-list` scene: the glyph, the page's type and `url`; reading of the search command |

### Risks

- **What leaves the box.** One GET of the URL the user chose, with a `rusty` user agent;
  no key, no cookie. The embedding of the text follows the provider setting as every
  page does.
- **Injection.** A captured page can say anything; every tool answer about a source is
  marked and normalised, and the system prompt of the pane's agent already names Rusty's
  tools as the way — the mark is what the agent sees.
- **Data safety.** Tests write under a temporary vault; a recapture keeps `created` and
  a failed recapture keeps the text.
- **Big pages.** Eight megabytes fetched, a megabyte kept.
- **Keyboard.** The palette commands; the dialog's field and buttons.
- **Theme.** The glyph in `faint`.

### CodeGraph

`codegraph_explore` over `create_page`, `sync_page`, `search_with`, `Embedder`: the index
path is one (`sync_page`), the embedder loop takes stale pages, `search_with` carries the
operators. A second pass after implementation goes in the ledger.

## Phase 3: Implement

As the manifest said. `brain/sources.rs` (new): the constants, `Fetched`, `Extracted`,
`fetch`, `kind_of`, `extract`, `extract_pdf`, `extract_html` (a state machine over the
tags), `decode_entities`, `site_of`, `slug_base`, `normalise`, `mark`, `mark_hits`,
`is_source_slug`; six tests. `brain/mod.rs`: `pub mod sources`, `source_for_url`,
`capture_url`, `capture_fetched`; one test on a fixture body. `vault.rs`: the `source`
type and its folder. `rusty-mcp`: `SourceCaptureParams`, `SourceSearchParams`,
`source_capture`, `source_search`, `source_preview`; the mark on `brain_search`,
`brain_read_page` and (F8) `brain_render`; `EXPECTED`. `rusty-cli`: `source capture`,
`source search`. `Main.qml`: two commands and `captureDialog`; `Explorer.qml`: the
`⌁` glyph for a `sources/` page. `scripts/screenshot.sh`: a seeded source page, and the
`reading` scene mapped to the empty scene by an exact match (F1). The counts: 85.
`qmllint` exit 0, `bash -n` on the script, `cargo fmt --all` ran.

Deviations: `slug_base` takes the path after the host and turns dots into hyphens (two
expectations of mine said so; the first draft took the host as the name for a URL with
no path). The 025 spawn tests take a lock and land their fake by rename (an `ETXTBSY`
between test threads, seen once under this pipeline's load). The capture test builds a
vault of its own name (F2).

## Phase 3.5: Inspect — finding ledger

| # | Lens | Finding | Disposition |
|---|---|---|---|
| F1 | correctness | `scripts/screenshot.sh` mapped the default scene by `${scene/reading/}`, which strips the word from any scene: `open:sources/example-com-reading-list` reached the app as `…example-com--list` (seen in the first scene) | **confirmed**; an exact match maps `reading` alone to the empty scene, and the seed's name lost the word |
| F2 | data safety | the capture test shared its temporary vault name with an older test (`test_brain("capture")`), and the two, in parallel, wiped each other: the recapture found no page and wrote a second one (the delivery gate's first run) | **confirmed**; the test has a vault of its own name |
| F3 | secrets | what the fetch sends: one GET with a `rusty (local capture)` user agent, no cookie, no key; the embedding follows the provider setting as for every page | no finding |
| F4 | data safety | a failed recapture of a page that has text | rejected (tested): the text stays, `status` and `error` land in the frontmatter |
| F5 | correctness | `kind_of` takes a `.pdf` URL served as HTML for a PDF | accepted: `pdftotext` fails and the failure is recorded; the next capture reads the page again |
| F6 | correctness | `extract_html`'s scope: a document with `<main>` whose text lies elsewhere | accepted: "no readable text" is recorded, the page says so |
| F7 | correctness | the app reads `snippet` and `compiled_truth` from the same JSON the mark normalises | rejected: the app's page views come through `brain_render`, whose `raw` and `rendered` the mark leaves alone; a search hit's snippet, normalised, is what the pane wants anyway |
| F8 | secrets | `brain_render` handed a source's page to an agent unmarked (the pane pre-allows it) | **confirmed**; the answer carries `untrusted` and the note, its `raw` and `rendered` untouched (they are for display and editing) |
| F9 | performance | `source_for_url` reads every `sources/` page's frontmatter on each capture | accepted: hundreds of files in milliseconds; a column is a seam |
| F10 | correctness | the `sources/` folder appears in every vault on the next `ensure_dirs` | accepted: it is the type's folder, as `decisions/` and `conversations/` were; in the wiki |
| F11 | keyboard first | the two palette commands; the dialog's field takes Enter | no finding |
| F12 | theme | the glyph in `faint`, the accent when active | tokens only |
| F13 | prose | the tool descriptions, the dialog, the CLI lines, the note | read against `no-ai-slop` |
| F14 | correctness | CodeGraph: `capture_url → capture_fetched → extract → extract_html`; the callers of `capture_url` are the tool and the CLI; `sync_page` gains one caller; nothing else moved | the blast radius matches the manifest |

## Phase 4: Validate

- `bin/gate.sh --fast`: red twice on tests of mine (`ETXTBSY` in a 025 spawn test under
  load; two slug expectations), then `GATE GREEN [fast]` with the seven new tests
  passing: `extract_html_keeps_the_article_and_drops_the_chrome`,
  `kind_of_reads_the_type_then_the_bytes`, `fetch_refuses_other_schemes`,
  `site_and_slug_follow_the_url`, `normalise_strips_controls_and_caps`,
  `mark_flags_a_source_and_leaves_a_page`, `capture_writes_updates_and_records_failures`.
- `cargo build -p rusty-app -p rusty-mcp` (00:12:56), then `scripts/screenshot.sh
  <scratch> "open:sources/example-com-launchers" "reading"` with `RUSTY_SHOT_DELAY=4500`
  after F1: logs clean of `error|warning|TypeError|ReferenceError|Cannot assign|is not a
  type|Detected anchors|binding loop`. `open-sources-example-com-launchers.png`: the
  page under `sources/` open — `type: source`, `url`, `site`, `captured`, `kind` in the
  properties, the body rendered, `sources` in the tree with its count — REQ-006's scene;
  `reading.png` the default scene, unchanged.
- REQ-001 to REQ-005 rest on the tests the regression table names; REQ-003 also on the
  reading of the five marked answers (F8 added the fifth).
- `bin/gate.sh --diff` after F8 and F2: GATE GREEN [diff]

## Phase 5: Complete

- Requirement audit: REQ-001 to REQ-006 satisfied — REQ-001 by the capture and
  extractor tests (a real capture is Chad's), REQ-002 by the capture test's search and
  the reading of `sync_page` and the embedder loop, REQ-003 by the normaliser and mark
  tests and the reading of the five marked answers, REQ-004 and REQ-005 by the capture
  test, REQ-006 by the scene and the reading of the search command. None split, none
  waived.
- Wiki: two runs. `929cf1a7-145d-4500-b7d4-f1274c404f7f` → `complete` with two
  warnings (the back end's `brain_render` claim cited a body F8 changed after the
  inspect; the app's scenes claim cited script lines the seeded source moved), so those
  two sidecars were left unchanged; `a99c93ca-c285-4281-8a13-d1a60287f494` re-anchored
  every stale claim on both pages and re-added the two new ones → `complete`. Prose:
  the vault page (a bullet, a paragraph on what leaves the machine, a failure mode, the
  tests), the back end (a bullet, the tests, 85), the app (a bullet, the tests).
  `docs/architecture.md`, `CLAUDE.md`, `AGENTS.md`, `README.md` say 85 tools. The
  PostToolUse hook did not fire (eleventh sighting); bulletin 3's recovery with the pair
  under `active/`, then `bin/gate.sh --verify`.
- ROADMAP ticked under M8. `AD-rusty-sources-are-pages-marked-untrusted-001` in the AAR
  and the register. Brain: timeline entry on `projects/rusty-v3`.
