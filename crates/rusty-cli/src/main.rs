//! Rusty CLI — terminal access to Rusty's brain (the knowledge vault).
//!
//! A thin wrapper over `rusty_core`'s [`BrainManager`] so the terminal assistant
//! can search, read, capture to, and reindex the brain while keeping the FTS
//! index, link graph, and git auto-commit consistent — the same code path the
//! GUI uses. Tasks, notes, and memories are reached directly (sqlite3 + files);
//! this CLI deliberately covers only the brain, where the SQLite side is derived
//! state that must stay in sync with the markdown vault.

use rusty_core::brain::semantic;
use rusty_core::brain::{BrainManager, CaptureTarget};
use rusty_core::engine::conversation_archive::ConversationArchive;
use rusty_core::engine::db::Database;
use rusty_core::engine::secrets_manager::SecretsManager;
use rusty_core::engine::settings_manager::SettingsManager;
use rusty_core::skills::{self, SkillsManager};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;

const USAGE: &str = "rusty-cli — terminal access to Rusty's brain\n\
\n\
USAGE:\n\
  rusty-cli brain search <query...> [--limit N] [--type TYPE]\n\
  rusty-cli brain context <prompt...>\n\
  rusty-cli brain read <slug>\n\
  rusty-cli brain new <type> <title...> [--content <text>]\n\
  rusty-cli brain append <slug> <summary...> [--detail <text>]\n\
  rusty-cli brain set <slug> <content...>   (replace the page body / compiled truth)\n\
  rusty-cli brain capture <text...> [--to daily|inbox] [--date YYYY-MM-DD]\n\
  rusty-cli brain daily [--date YYYY-MM-DD]   (open or create the daily page)\n\
  rusty-cli brain types                       (page types, folders, counts)\n\
  rusty-cli brain migrate [--dry-run]         (timeline sections + vault-path links)\n\
  rusty-cli brain reindex\n\
  rusty-cli brain embed [--all]               (vectors for stale pages, or every page)\n\
  rusty-cli brain semantic                    (embedding provider and index state)\n\
  rusty-cli brain stats\n\
  rusty-cli skills list [--all]\n\
  rusty-cli skills view <name>\n\
  rusty-cli skills new <name> [--desc <text>] [--body <text>] [--force]\n\
  rusty-cli skills rm <name>\n\
  rusty-cli skills path\n\
  rusty-cli skills review                 (list pending skills + safety-scan findings)\n\
  rusty-cli skills approve <name> [--force]\n\
  rusty-cli skills reject <name>\n\
  rusty-cli ingest-conversation <path|session-id>   (archive a Claude Code transcript + brain node)\n\
  rusty-cli ingest-conversation --all [--dir <path>] [--limit N]   (backfill a project's transcripts)\n\
  rusty-cli conversations search <query...> [--limit N]\n\
  rusty-cli refresh   (signal the GUI to reload after a data change)";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match (
        args.get(1).map(String::as_str),
        args.get(2).map(String::as_str),
    ) {
        (Some("brain"), Some(sub)) => run_brain(sub, &args[3..]),
        (Some("skills"), _) => {
            let sub = args.get(2).map(String::as_str).unwrap_or("list");
            run_skills(sub, args.get(3..).unwrap_or_default());
        }
        (Some("ingest-conversation"), _) => {
            run_ingest_conversation(args.get(2..).unwrap_or_default())
        }
        (Some("conversations"), _) => {
            let sub = args.get(2).map(String::as_str).unwrap_or("search");
            run_conversations(sub, args.get(3..).unwrap_or_default());
        }
        (Some("refresh"), _) => refresh_signal(),
        (Some("--help") | Some("-h"), _) | (None, _) => {
            println!("{USAGE}");
        }
        _ => {
            eprintln!("{USAGE}");
            exit(2);
        }
    }
}

/// The brain vault path the GUI and MCP server use: the `brain_vault_path` setting,
/// else `~/.rusty/brain`.
fn configured_brain_path(db: &Arc<Database>) -> PathBuf {
    let settings = SettingsManager::new(Arc::clone(db));
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let default_brain = home
        .join(".rusty")
        .join("brain")
        .to_string_lossy()
        .to_string();
    PathBuf::from(
        settings
            .get_or_default("brain_vault_path", &default_brain)
            .unwrap_or(default_brain),
    )
}

/// The embedding provider the settings and the secrets vault point at, if any.
fn configured_embedder() -> Option<std::sync::Arc<dyn semantic::Embedder>> {
    let db = Arc::new(Database::open().unwrap_or_else(|e| fail(&format!("open database: {e}"))));
    let settings = SettingsManager::new(Arc::clone(&db));
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let secrets = SecretsManager::new(home.join(".rusty").join(".secret"));
    semantic::resolve_embedder(&settings, &secrets)
}

/// Open the database and a `BrainManager` rooted at the configured vault path.
///
/// Mirrors the GUI/MCP path resolution so all three operate on one vault.
fn brain() -> BrainManager {
    let db = Arc::new(Database::open().unwrap_or_else(|e| fail(&format!("open database: {e}"))));
    let brain_path = configured_brain_path(&db);
    let brain = BrainManager::new(Arc::clone(&db), PathBuf::from(&brain_path));
    brain
        .ensure_vault()
        .unwrap_or_else(|e| fail(&format!("init brain vault: {e}")));
    brain
}

/// Open a `ConversationArchive` over the shared db + brain vault.
fn archive() -> (Arc<Database>, ConversationArchive) {
    let db = Arc::new(Database::open().unwrap_or_else(|e| fail(&format!("open database: {e}"))));
    let brain_path = configured_brain_path(&db);
    let brain = Arc::new(BrainManager::new(
        Arc::clone(&db),
        PathBuf::from(&brain_path),
    ));
    brain
        .ensure_vault()
        .unwrap_or_else(|e| fail(&format!("init brain vault: {e}")));
    let arch = ConversationArchive::new(Arc::clone(&db), brain);
    (db, arch)
}

/// Claude Code's transcript directory for the current working directory.
fn claude_project_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // Claude Code slugifies the cwd by replacing every non-alphanumeric char with '-'.
    let slug: String = cwd
        .to_string_lossy()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    home.join(".claude").join("projects").join(slug)
}

/// Resolve a transcript argument: an existing path, or a session id looked up
/// under `~/.claude/projects/*/<sid>.jsonl`.
fn resolve_transcript(target: &str) -> Option<PathBuf> {
    let p = PathBuf::from(target);
    if p.is_file() {
        return Some(p);
    }
    let sid = target.trim_end_matches(".jsonl");
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let projects = home.join(".claude").join("projects");
    let entries = std::fs::read_dir(&projects).ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join(format!("{sid}.jsonl"));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// List `*.jsonl` transcript files in a directory, sorted.
fn list_jsonl(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().and_then(|x| x.to_str()) == Some("jsonl"))
            .collect(),
        Err(_) => Vec::new(),
    };
    files.sort();
    files
}

/// `rusty-cli ingest-conversation <path|session-id> | --all [--dir <path>] [--limit N]`.
fn run_ingest_conversation(args: &[String]) {
    let (pos, flags) = parse_with_bools(args, &["all"]);
    let (_db, arch) = archive();

    if flags.contains_key("all") {
        let dir = flags
            .get("dir")
            .map(PathBuf::from)
            .unwrap_or_else(claude_project_dir);
        let limit: usize = flags
            .get("limit")
            .and_then(|s| s.parse().ok())
            .unwrap_or(usize::MAX);
        let files = list_jsonl(&dir);
        if files.is_empty() {
            fail(&format!("no transcripts found in {}", dir.display()));
        }
        let mut seen = 0usize;
        let mut ok = 0usize;
        for f in files {
            if seen >= limit {
                break;
            }
            seen += 1;
            match arch.ingest(&f) {
                Ok(o) => {
                    ok += 1;
                    println!(
                        "  [{}] {} ({} msgs) -> {}",
                        if o.created { "new" } else { "upd" },
                        o.title,
                        o.message_count,
                        o.brain_slug
                    );
                }
                Err(e) => eprintln!("  skip {}: {e}", f.display()),
            }
        }
        println!("ingested {ok}/{seen} transcripts from {}", dir.display());
        return;
    }

    let target = pos.first().unwrap_or_else(|| {
        fail("ingest-conversation: usage: rusty-cli ingest-conversation <path|session-id> | --all [--dir <path>] [--limit N]")
    });
    let path = resolve_transcript(target)
        .unwrap_or_else(|| fail(&format!("transcript not found for '{target}'")));
    match arch.ingest(&path) {
        Ok(o) => {
            println!(
                "ingested \"{}\" ({} messages){}",
                o.title,
                o.message_count,
                if o.created { "" } else { " [updated]" }
            );
            println!("  brain node: {}", o.brain_slug);
            if !o.linked.is_empty() {
                println!("  linked: {}", o.linked.join(", "));
            }
        }
        Err(e) => fail(&e),
    }
}

/// `rusty-cli conversations search <query...> [--limit N]`.
fn run_conversations(sub: &str, rest: &[String]) {
    match sub {
        "search" => {
            let (pos, flags) = parse(rest);
            let query = pos.join(" ");
            if query.is_empty() {
                fail("conversations search: missing <query>");
            }
            let limit = flags
                .get("limit")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10);
            let (_db, arch) = archive();
            match arch.search(&query, limit) {
                Ok(hits) if hits.is_empty() => {
                    println!("(no conversations match \"{query}\")")
                }
                Ok(hits) => {
                    for h in hits {
                        let snip = h.snippet.replace('\n', " ");
                        println!(
                            "{}  [{}]\n  {} · {}\n  {}",
                            h.title,
                            h.session_id,
                            h.started_at,
                            h.brain_slug,
                            snip.trim()
                        );
                    }
                }
                Err(e) => fail(&e),
            }
        }
        other => {
            eprintln!("unknown conversations subcommand: '{other}'\n\n{USAGE}");
            exit(2);
        }
    }
}

/// Dispatch a `brain` subcommand.
fn run_brain(sub: &str, rest: &[String]) {
    match sub {
        "search" => {
            let (pos, flags) = parse(rest);
            let query = pos.join(" ");
            if query.is_empty() {
                fail("search: missing <query>");
            }
            let limit = flags.get("limit").and_then(|s| s.parse::<usize>().ok());
            let ptype = flags.get("type").map(String::as_str);
            match brain().search(&query, limit, ptype) {
                Ok(results) if results.is_empty() => println!("(no brain matches for \"{query}\")"),
                Ok(results) => {
                    for r in results {
                        let snip = r.snippet.replace("<b>", "").replace("</b>", "");
                        println!(
                            "{}\t[{}] {}\n    {}",
                            r.slug,
                            r.page_type,
                            r.title,
                            snip.trim()
                        );
                    }
                }
                Err(e) => fail(&e),
            }
        }
        "context" => {
            let prompt = rest.join(" ");
            if prompt.is_empty() {
                fail("context: missing <prompt>");
            }
            let ctx = brain().build_context_for_prompt(&prompt);
            if ctx.is_empty() {
                println!("(no relevant brain context)");
            } else {
                println!("{ctx}");
            }
        }
        "read" => {
            let (pos, _) = parse(rest);
            let slug = pos.first().unwrap_or_else(|| fail("read: missing <slug>"));
            match brain().read_page(slug) {
                Ok(Some(p)) => {
                    println!(
                        "# {} ({})\nslug: {}\n\n{}",
                        p.title, p.page_type, p.slug, p.compiled_truth
                    );
                    if !p.timeline.trim().is_empty() {
                        println!("\n## Timeline\n{}", p.timeline);
                    }
                }
                Ok(None) => fail(&format!("no brain page: {slug}")),
                Err(e) => fail(&e),
            }
        }
        "new" => {
            let (pos, flags) = parse(rest);
            if pos.len() < 2 {
                fail("new: usage: brain new <type> <title...> [--content <text>]");
            }
            let content = flags.get("content").cloned().unwrap_or_default();
            let bm = brain();
            match bm.create_page(&pos[0], &pos[1..].join(" "), &content) {
                Ok(p) => println!("created {} ({})", p.slug, p.page_type),
                Err(e) => fail(&e),
            }
            bm.flush_commits();
        }
        "append" => {
            let (pos, flags) = parse(rest);
            if pos.len() < 2 {
                fail("append: usage: brain append <slug> <summary...> [--detail <text>]");
            }
            let detail = flags.get("detail").map(String::as_str);
            let date = flags.get("date").cloned().unwrap_or_else(today);
            let bm = brain();
            match bm.add_timeline(&pos[0], &date, "terminal", &pos[1..].join(" "), detail) {
                Ok(_) => println!("appended to {}", pos[0]),
                Err(e) => fail(&e),
            }
            bm.flush_commits();
        }
        "set" => {
            let (pos, _) = parse(rest);
            if pos.len() < 2 {
                fail("set: usage: brain set <slug> <content...>");
            }
            let bm = brain();
            match bm.update_page(&pos[0], &pos[1..].join(" ")) {
                Ok(p) => println!("updated {}", p.slug),
                Err(e) => fail(&e),
            }
            bm.flush_commits();
        }
        "capture" => {
            let (pos, flags) = parse(rest);
            let text = pos.join(" ");
            if text.is_empty() {
                fail("capture: missing <text>");
            }
            let target =
                CaptureTarget::parse(flags.get("to").map(String::as_str).unwrap_or("daily"))
                    .unwrap_or_else(|e| fail(&e));
            let brain = brain();
            match brain.capture(
                &text,
                target,
                flags.get("date").map(String::as_str),
                "terminal",
            ) {
                Ok(r) => println!("captured to {}", r.slug),
                Err(e) => fail(&e),
            }
            brain.flush_commits();
        }
        "daily" => {
            let (_, flags) = parse(rest);
            let brain = brain();
            match brain.daily_page(flags.get("date").map(String::as_str)) {
                Ok(page) => {
                    println!("{}\n", page.slug);
                    if !page.compiled_truth.is_empty() {
                        println!("{}\n", page.compiled_truth);
                    }
                    if !page.timeline.is_empty() {
                        println!("## Timeline\n\n{}", page.timeline);
                    }
                }
                Err(e) => fail(&e),
            }
            brain.flush_commits();
        }
        "types" => match brain().page_types() {
            Ok(types) => {
                for t in types {
                    println!("{:<13} {:<14} {}", t.page_type, t.dir, t.count);
                }
            }
            Err(e) => fail(&e),
        },
        "migrate" => {
            let (_, flags) = parse_with_bools(rest, &["dry-run"]);
            let dry = flags.contains_key("dry-run");
            match brain().migrate_vault(dry) {
                Ok(r) => {
                    println!(
                        "{}scanned {} pages: {} changed, {} timelines converted, {} links rewritten",
                        if dry { "dry run: " } else { "" },
                        r.pages_scanned,
                        r.pages_changed,
                        r.timelines_converted,
                        r.links_rewritten
                    );
                    for slug in &r.changed_slugs {
                        println!("  changed: {slug}");
                    }
                    for u in &r.unresolved_links {
                        println!("  unresolved: {u}");
                    }
                }
                Err(e) => fail(&e),
            }
        }
        "embed" => {
            let (_, flags) = parse_with_bools(rest, &["all"]);
            let Some(embedder) = configured_embedder() else {
                fail("no embedding provider: set embedding_provider to ollama, or to openai with openai_api_key in the vault")
            };
            match brain().index_stale(embedder.as_ref(), flags.contains_key("all")) {
                Ok(r) => {
                    println!(
                        "{}: embedded {} pages ({} chunks), removed {}, failed {}",
                        r.model,
                        r.pages_indexed,
                        r.chunks_written,
                        r.pages_removed,
                        r.pages_failed.len()
                    );
                    for f in &r.pages_failed {
                        println!("  failed: {f}");
                    }
                }
                Err(e) => fail(&e),
            }
        }
        "semantic" => {
            let provider = configured_embedder().map(|e| e.id());
            let index = brain().semantic();
            match index.stats() {
                Ok(s) => {
                    println!(
                        "provider: {}",
                        provider
                            .clone()
                            .unwrap_or_else(|| "none (full-text only)".to_string())
                    );
                    println!(
                        "indexed with: {}",
                        s.model.unwrap_or_else(|| "nothing yet".to_string())
                    );
                    println!("pages: {}  chunks: {}  dims: {}", s.pages, s.chunks, s.dims);
                    if let Some(model) = provider {
                        if let Ok((stale, _)) = index.stale_slugs(&model) {
                            println!("waiting: {}", stale.len());
                        }
                    }
                }
                Err(e) => fail(&e),
            }
        }
        "reindex" => match brain().sync_all() {
            Ok(n) => println!("reindexed {n} brain pages from the vault"),
            Err(e) => fail(&e),
        },
        "stats" => match brain().stats() {
            Ok(s) => {
                println!(
                    "pages: {}\nlinks: {}\ntags: {}\ntimeline entries: {}",
                    s.page_count, s.link_count, s.tag_count, s.timeline_count
                );
                let mut by_type: Vec<_> = s.pages_by_type.into_iter().collect();
                by_type.sort();
                for (t, c) in by_type {
                    println!("  {t}: {c}");
                }
            }
            Err(e) => fail(&e),
        },
        other => {
            eprintln!("unknown brain subcommand: '{other}'\n\n{USAGE}");
            exit(2);
        }
    }
}

/// Open a `SkillsManager` rooted at the configured skills path.
fn skills() -> SkillsManager {
    let db = Arc::new(Database::open().unwrap_or_else(|e| fail(&format!("open database: {e}"))));
    let settings = SettingsManager::new(Arc::clone(&db));
    let mgr = SkillsManager::new(skills::resolve_root(&settings));
    mgr.ensure_dirs()
        .unwrap_or_else(|e| fail(&format!("init skills: {e}")));
    mgr
}

/// Dispatch a `skills` subcommand.
fn run_skills(sub: &str, rest: &[String]) {
    match sub {
        "list" => {
            let (_, flags) = parse_with_bools(rest, &["all"]);
            let all = flags.contains_key("all");
            let list = skills().list(all);
            if list.is_empty() {
                println!("(no skills)");
            } else {
                for s in list {
                    println!(
                        "{}\t[{}/{}] {}",
                        s.name,
                        s.status.as_str(),
                        s.origin.as_str(),
                        s.description
                    );
                }
            }
        }
        "view" => {
            let (pos, _) = parse(rest);
            let name = pos.first().unwrap_or_else(|| fail("view: missing <name>"));
            match skills().get(name) {
                Some(s) => println!(
                    "# {} ({}/{})\npath: {}\n\n{}",
                    s.display_name,
                    s.status.as_str(),
                    s.origin.as_str(),
                    s.path,
                    s.body
                ),
                None => fail(&format!("no skill: {name}")),
            }
        }
        "new" => {
            let (pos, flags) = parse_with_bools(rest, &["force"]);
            let name = pos.first().unwrap_or_else(|| {
                fail("new: usage: skills new <name> [--desc <text>] [--body <text>] [--force]")
            });
            let desc = flags.get("desc").cloned().unwrap_or_default();
            let body = flags
                .get("body")
                .cloned()
                .unwrap_or_else(|| "## Procedure\n\n## Pitfalls\n\n## Verification\n".to_string());
            let force = flags.contains_key("force");
            let mgr = skills();
            match mgr.create_skill(name, &desc, &body, force) {
                Ok(s) => {
                    mgr.git_commit_blocking(&format!("skills: add {}", s.name));
                    println!("created {}", s.name);
                }
                Err(e) => fail(&e),
            }
        }
        "rm" => {
            let (pos, _) = parse(rest);
            let name = pos.first().unwrap_or_else(|| fail("rm: missing <name>"));
            let mgr = skills();
            match mgr.delete_skill(name) {
                Ok(()) => {
                    mgr.git_commit_blocking(&format!("skills: remove {name}"));
                    println!("removed {name}");
                }
                Err(e) => fail(&e),
            }
        }
        "path" => println!("{}", skills().active_dir().display()),
        "review" => {
            let mgr = skills();
            let pending: Vec<_> = mgr
                .list(true)
                .into_iter()
                .filter(|s| s.status.as_str() == "pending")
                .collect();
            if pending.is_empty() {
                println!("(no pending skills)");
            } else {
                for s in pending {
                    let flag = match mgr.scan(&s.name) {
                        Ok(f) if f.is_empty() => "clean".to_string(),
                        Ok(f) => format!("FLAGGED: {}", f.join("; ")),
                        Err(e) => format!("scan error: {e}"),
                    };
                    println!("{}\t{}\t[{}]", s.name, s.description, flag);
                }
            }
        }
        "approve" => {
            let (pos, flags) = parse_with_bools(rest, &["force"]);
            let name = pos
                .first()
                .unwrap_or_else(|| fail("approve: missing <name>"));
            let force = flags.contains_key("force");
            let mgr = skills();
            match mgr.approve(name, force) {
                Ok(()) => {
                    mgr.git_commit_blocking(&format!("skills: approve {name}"));
                    println!("approved {name}");
                }
                Err(e) => fail(&e),
            }
        }
        "reject" => {
            let (pos, _) = parse(rest);
            let name = pos
                .first()
                .unwrap_or_else(|| fail("reject: missing <name>"));
            let mgr = skills();
            match mgr.reject(name) {
                Ok(()) => {
                    mgr.git_commit_blocking(&format!("skills: reject {name}"));
                    println!("rejected {name}");
                }
                Err(e) => fail(&e),
            }
        }
        other => {
            eprintln!("unknown skills subcommand: '{other}'\n\n{USAGE}");
            exit(2);
        }
    }
}

/// Split raw args into positionals and `--flag value` pairs (no boolean flags).
fn parse(args: &[String]) -> (Vec<String>, HashMap<String, String>) {
    parse_with_bools(args, &[])
}

/// Split raw args into positionals and `--flag value` pairs. Flags named in
/// `bool_flags` are valueless switches (recorded with an empty value) and do NOT
/// consume the following token — so `--force <name>` keeps `<name>` as a positional.
fn parse_with_bools(
    args: &[String],
    bool_flags: &[&str],
) -> (Vec<String>, HashMap<String, String>) {
    let mut pos = Vec::new();
    let mut flags = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        if let Some(name) = args[i].strip_prefix("--") {
            if bool_flags.contains(&name) {
                flags.insert(name.to_string(), String::new());
                i += 1;
            } else {
                let val = args.get(i + 1).cloned().unwrap_or_default();
                flags.insert(name.to_string(), val);
                i += 2;
            }
        } else {
            pos.push(args[i].clone());
            i += 1;
        }
    }
    (pos, flags)
}

/// Today's date as `YYYY-MM-DD` in local time.
fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

/// Touch the sentinel file the GUI watches, so it reloads after a terminal data change.
fn refresh_signal() {
    let sentinel = dirs::home_dir()
        .unwrap_or_default()
        .join(".rusty")
        .join(".changed");
    match std::fs::write(&sentinel, chrono::Local::now().to_rfc3339()) {
        Ok(()) => println!("refresh signal sent"),
        Err(e) => fail(&format!("refresh: {e}")),
    }
}

/// Print an error to stderr and exit with status 1.
fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    exit(1);
}

#[cfg(test)]
mod tests {
    use super::{parse, parse_with_bools};

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_pairs_value_flags() {
        let (pos, flags) = parse(&argv(&["a", "--k", "v", "b"]));
        assert_eq!(pos, vec!["a", "b"]);
        assert_eq!(flags.get("k").map(String::as_str), Some("v"));
    }

    #[test]
    fn bool_flag_does_not_eat_next_token() {
        let (pos, flags) = parse_with_bools(&argv(&["--force", "my-skill"]), &["force"]);
        assert_eq!(pos, vec!["my-skill"]);
        assert_eq!(flags.get("force").map(String::as_str), Some(""));
    }

    #[test]
    fn bool_flag_mixed_with_value_flag() {
        let (pos, flags) =
            parse_with_bools(&argv(&["--force", "--desc", "hello", "name"]), &["force"]);
        assert_eq!(pos, vec!["name"]);
        assert!(flags.contains_key("force"));
        assert_eq!(flags.get("desc").map(String::as_str), Some("hello"));
    }
}
