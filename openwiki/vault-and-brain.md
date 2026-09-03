---
type: "Reference"
title: "Vault and brain: files as the truth, SQLite as the index"
openwiki_generated: true
sources:
  - id: openwiki-source-9c78ae52164f32a938f17cce
    resource: repo://crates/rusty-core/src/brain/frontmatter.rs
  - id: openwiki-source-84bd94f4d6c1ff8ab953d365
    resource: repo://crates/rusty-core/src/brain/links.rs
  - id: openwiki-source-c7501cab00d475ec77094adb
    resource: repo://crates/rusty-core/src/brain/mod.rs
  - id: openwiki-source-061d1620011caae21b2a0d24
    resource: repo://crates/rusty-core/src/brain/semantic.rs
  - id: openwiki-source-79e92c26a49d3b5ce7f4c00a
    resource: repo://crates/rusty-core/src/brain/vault.rs
generated: {by: "claude-code", at: "2026-09-03T05:07:54.592Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-09-03T05:46:21.914Z
---

# Vault and brain: files as the truth, SQLite as the index

## Purpose

The brain is a folder of markdown (`~/.rusty/brain`, a git repository) that Obsidian or
any editor can open. SQLite (`~/.rusty/rusty.db`) holds derived indexes: page metadata,
full text (FTS5), links, tags, aliases, timeline rows, versions, and vectors. Everything
in the database can be rebuilt from the folder.

## Ownership

`crates/rusty-core/src/brain/`:

- `vault.rs`: `VaultManager`, the filesystem side: type folders, the real nested tree,
  read and write by slug, rename and move, soft deletes into `archive/`, git auto-commits.
- `frontmatter.rs`: the page format: YAML frontmatter, the compiled truth, the
  `## Timeline` section; lenient parsing; ordered properties.
- `links.rs`: the one wikilink scanner (`scan`, `targets`, `rewrite_targets`) shared by
  the indexer, the renderer, the migration and the move rewrite, and the inline tag
  scanner (`tags`).
- `mod.rs`: `BrainManager`, the manager over both: create, read, update, whole-file
  write, rename, folders, timeline, capture, daily pages, sync, search, migration.
- `semantic.rs`: chunking, the `Embedder` trait and providers, the `sqlite-vec` index,
  rank fusion.
- `engine/db.rs`: the schema and `migrate()`; `sqlite-vec` is registered as an auto
  extension.

## The page rules

- A page is `<folder>/<name>.md`; its slug is the path without `.md`. Type folders
  (`people`, `companies`, `projects`, `concepts`, `meetings`, `ideas`, `daily`, `inbox`,
  `conversations`) imply a type; any other folder, and the root, imply `note`.
- Frontmatter carries `title`, `type`, `aliases`, `tags`, `created`, `updated` and any
  extra keys. A file without frontmatter is still a page: the title is the file name and
  the type comes from the top folder (`BrainFrontmatter::fill_defaults`). Unreadable YAML
  degrades the same way (`parse_lenient`); only strict callers see the error.
- The timeline is the `## Timeline` section that runs to the end of the file; the body
  above it is the compiled truth. The bare `---` rule of older pages is read, never
  written; `rusty-cli brain migrate` rewrites it.
- Wikilinks are vault paths: `[[projects/orbit]]`, `[[projects/orbit|alias]]`,
  `[[projects/orbit#Heading]]`, `![[embed]]`. The scanner skips fenced and inline code.
- Deletes are soft: a page or folder moves to `archive/<name>_<timestamp>`.
- Rusty's writers always write frontmatter; a whole-file write (`write_raw`) writes
  exactly the text it was given after snapshotting the previous version.
- Tags come from two places and land in one index: the frontmatter `tags` list and
  inline `#tags` in the body (a `#` at a boundary, then letters, digits, `_`, `-` and
  `/`, at least one letter, never inside code), compared without case and stored as
  first written. A nested tag `a/b` counts under `a` as well.
- A property edit (`set_property`, `remove_property`) rewrites only the frontmatter
  mapping, keys in their order, and writes the body back byte for byte; values are text,
  numbers, booleans, dates as `YYYY-MM-DD` text, or lists of strings. A page without
  frontmatter gains it; removing the last key drops it.

## Runtime flow

- Read: `read_page` parses leniently and fills defaults; `render_page` renders the body
  (see [Markdown rendering](markdown-rendering.md)) and returns the ordered properties
  and the raw file.
- Write: `create_page`, `update_page` (body only, frontmatter and timeline kept),
  `write_raw` (the whole file), `add_timeline`, `capture`. Each writes the file,
  re-indexes the page and fires a background git commit.
- Rename or move (`rename`): the file or folder moves; `links::rewrite_targets` rewrites
  every spelling of the old target in every page (exact slug, `.md`, leading `/`, and the
  bare file name when it was unique; markdown links too; fenced code untouched); the
  index rows follow by slug or by folder prefix (`move_index_rows`); one commit records
  it. A title that was the old file name follows the new one.
- Index: `sync_page` re-indexes when the content hash changed and refreshes the link
  rows either way; `sync_all` walks every folder, removes orphans, then resolves link
  rows whose targets arrived later. The back end runs `sync_all` after every burst of
  file changes, so edits made by Obsidian or an editor are indexed within seconds.
- Links: each row in `brain_links` holds the resolved slug (exact, case-insensitive;
  else a unique file name anywhere; else a unique title or alias) or the raw target, plus
  the line it sits on as context. `unresolved()` lists rows whose target is no page.
- Search: `parse_query` takes a query apart (`tag:`, `path:`, `file:` and `type:`
  terms, a value in quotes keeping its spaces, a leading `-` excluding; the words keep
  their quotes for FTS5 phrases). `search_with` narrows the pages by the operators
  (`tag:` through `brain_tags`, the rest over the indexed page rows), then matches the
  words through FTS5 (`porter unicode61`), or as typed when `case_sensitive`, or as a
  pattern over the indexed title and text when `regex` (ranked by match count, a
  snippet around the first match); operator terms alone list the admitted pages newest
  first. `search_hybrid_with` fuses FTS5 with vector hits through reciprocal rank
  fusion, applies the same operators to both halves, and hands the two text modes to
  `search_with`. `search` and `search_hybrid` are the same with the default options.
- Tags: `tags()` groups the index by tag with a page count, parents counting their
  nested tags too, for the app's Tags pane and for agents.

## Semantic search and what leaves the machine

`resolve_embedder` reads `embedding_provider` (`auto`, `ollama`, `openai`, `off`),
`embedding_model` and `ollama_url` from settings and `openai_api_key` (or
`OPENAI_API_KEY`) from the secrets vault. `auto` picks Ollama only when it answers
locally; OpenAI is used only when the setting names it and a key exists, because it
sends page text off the machine. Vectors live in the `vec0` table `brain_vec`, created at
the model's width; changing the model rebuilds them. With no provider, search stays
full text and nothing else changes.

## Invariants

- Files are the truth; never write the database without the file.
- One `Mutex<Connection>`: a manager method never holds the guard across a call that
  takes it again (`sync_all` once deadlocked on that).
- Slugs never contain `..`; every path stays inside the vault root.
- Rewrites never touch fenced code, and never change a file where nothing resolved.

## Failure modes

- A page with broken YAML opens with empty frontmatter; a strict operation on it errors.
- A rename to an existing target is refused; a folder cannot move into itself.
- `sync_all` on a vault with pages that link by bare name resolves them in a second pass.

## Extension points

- New page types: `TYPE_DIRS` in `vault.rs` and the templates under `.templates/`.
- New providers: implement `Embedder` and add it to `resolve_embedder`.
- New index tables: additive `CREATE TABLE IF NOT EXISTS` in `engine/db.rs`.

## Tests

- `cargo test -p rusty-core brain::` covers parsing, the tree, folders, renames with
  every link spelling, the renderer, the scanner, the semantic index and migration.
- `crates/rusty-core/tests/integration.rs` exercises the managers together.

## Primary sources

- `crates/rusty-core/src/brain/mod.rs`, `vault.rs`, `frontmatter.rs`, `links.rs`, `semantic.rs`
- `crates/rusty-core/src/engine/db.rs`, `crates/rusty-core/src/core.rs`
