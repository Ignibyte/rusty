---
title: TICKET-027-personal-search-engine
status: done
ticket_number: 027
type: feature
created: 2026-09-04
intake: docs/planning/intake/INTAKE-personal-search-engine.md
pipeline_spec: docs/planning/pipeline/active/personal-search-engine.spec.md
---

# TICKET-027-personal-search-engine

## Summary

Capture what Chad reads and keeps, index its contents, and search it beside the brain — a personal search engine, built rather than borrowed.

## Why

The brain holds what he deliberately writes. Nothing holds what he merely passed through. The SessionEnd hook already archives one such stream — 139 `conversations` pages — so the precedent exists; web reading and the files on disk are the streams with nothing behind them.

Promoted from `INTAKE-personal-search-engine.md` after Chad read `asciimoo/hister` and chose the full rebuild over a small capture ticket or running hister alongside.

**Read the intake before starting.** Two things in it are binding: hister is AGPL-3.0 against Rusty's MIT, so this is clean-room from the idea and not from that source; and hister's shape is four of Rusty's own non-goals, so the Out list below is what keeps this a Rusty feature rather than a second product.

## EARS requirements

| ID | Requirement | Verification |
|---|---|---|
| REQ-001 | WHEN given a URL, the system shall fetch it, extract its readable text, and write a `source` page carrying `url`, `title`, `site` and `captured`. | test; a real capture |
| REQ-002 | WHEN a `source` page is written, the system shall index its full text and, where a provider is set, its embedding, through the index the brain already uses. | test |
| REQ-003 | WHEN an agent searches sources over MCP, every result shall be marked as untrusted content and normalised before it can reach a model. | test; review |
| REQ-004 | WHEN a URL already captured is captured again, the system shall update that page rather than create a second one. | test |
| REQ-005 | WHEN extraction fails, the system shall record the failure on the page rather than write an empty source. | test |
| REQ-006 | WHEN sources exist, they shall be searchable and browsable from the app alongside brain pages, and distinguishable from them. | smoke; screenshot |

## Scope

- In: a `source` page type; capture by URL through the back end and as `rusty capture <url>`; extraction for HTML, PDF, plain text and markdown; reuse of the existing full-text and semantic index; MCP search and preview; a Sources view.
- Out: a web UI, a REST layer, OAuth, multi-user — all four are stated Rusty non-goals and all four are why this is a rebuild and not an adoption. Also out for a first pass: a headless-browser crawler, and Postgres.

## Notes

- REQ-003 is a first-commit requirement, not later hardening. Anything that ingests arbitrary web pages and hands them to an agent needs it from the start; hister marks every MCP result untrusted and normalises it, and that part is worth taking as a principle.
- TICKET-010 means `rusty capture <url>` needs no new dispatch machinery.
- TICKET-016 already mounts machine folders, so local files are partly in scope already.
- Promoted from intake: `docs/planning/intake/INTAKE-personal-search-engine.md`.
- Pipeline spec: TBC.
