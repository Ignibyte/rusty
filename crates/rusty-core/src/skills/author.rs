//! Self-authoring loop — propose a reusable skill from a substantial completed task.
//!
//! A fire-and-forget hook (mirroring brain enrichment) runs after a successful main
//! task. When `skills_auto_author` is `suggest` or `auto` and the task was substantial enough, it
//! asks Claude whether the task embodies a reusable procedure and, if so, writes a
//! proposal to the **staging** dir. In `suggest` mode it stops there (a human approves via
//! `rusty-cli skills approve`); in `auto` mode it promotes the proposal to active only if
//! the safety scan ([`crate::skills::scan_skill_md`]) is clean, else it stays staged.
//! Default is `off`, so this loop incurs no cost unless explicitly enabled.

use crate::engine::output_parser;
use crate::engine::process_manager;
use crate::engine::settings_manager::SettingsManager;
use crate::skills::{self, SkillsAccess, SkillsManager};
use std::sync::Arc;

/// Default Claude CLI path (matches enrichment).
const DEFAULT_CLI_PATH: &str = "claude";
/// Settings key: self-authoring mode — `off` (default), `suggest`, or `auto`.
const SETTING_AUTHOR_MODE: &str = "skills_auto_author";
/// Settings key: minimum CLI turns for a task to be considered skill-worthy.
const SETTING_MIN_TURNS: &str = "skills_author_min_turns";
/// Default turn threshold below which a task is too trivial to propose a skill from.
const DEFAULT_MIN_TURNS: i64 = 5;
/// Budget cap for the lightweight review call.
const REVIEW_BUDGET_USD: f64 = 0.03;

/// A proposed skill extracted from the review model's JSON output.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Proposal {
    name: String,
    description: String,
    body: String,
}

/// Review a completed task and, if warranted, stage a proposed skill.
///
/// Gated by the `skills_auto_author` setting (`off`/`suggest`), the master skills
/// switch, and a minimum-turns heuristic. Errors are returned for the caller to log;
/// they never affect the originating task.
pub async fn review_task_for_skill(
    settings: Arc<SettingsManager>,
    prompt: &str,
    result: &str,
    num_turns: i64,
) -> Result<(), String> {
    let mode = settings
        .get_or_default(SETTING_AUTHOR_MODE, "off")
        .unwrap_or_else(|_| "off".to_string());
    // `suggest` stages a proposal; `auto` promotes it when the safety scan is clean.
    if mode != "suggest" && mode != "auto" {
        return Ok(());
    }
    if !skills::is_enabled(&settings) {
        return Ok(());
    }
    let min_turns = settings
        .get_or_default(SETTING_MIN_TURNS, &DEFAULT_MIN_TURNS.to_string())
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .unwrap_or(DEFAULT_MIN_TURNS);
    if num_turns < min_turns {
        return Ok(());
    }

    let proposal = match propose_skill(prompt, result).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let mgr = SkillsManager::new(skills::resolve_root(&settings));
    mgr.ensure_dirs()?;
    // Don't shadow or overwrite an existing skill (active or already-staged).
    if mgr.get(&proposal.name).is_some() {
        return Ok(());
    }
    mgr.create_pending_skill(&proposal.name, &proposal.description, &proposal.body)?;
    if mode == "auto" {
        // Auto mode: promote immediately if the safety scan is clean, else leave it staged.
        match mgr.approve(&proposal.name, false) {
            Ok(()) => {
                mgr.git_commit_blocking(&format!(
                    "skills: auto-add {} (scan clean)",
                    proposal.name
                ));
                eprintln!("[skills] auto-approved '{}' (scan clean)", proposal.name);
            }
            Err(e) => {
                mgr.git_commit_blocking(&format!(
                    "skills: propose {} (auto; staged — {e})",
                    proposal.name
                ));
                eprintln!("[skills] staged '{}' for approval — {e}", proposal.name);
            }
        }
    } else {
        mgr.git_commit_blocking(&format!(
            "skills: propose {} (pending approval)",
            proposal.name
        ));
        eprintln!(
            "[skills] proposed '{}' — pending approval in staging",
            proposal.name
        );
    }
    Ok(())
}

/// Ask Claude whether the task is worth saving as a skill; parse the JSON proposal.
async fn propose_skill(prompt: &str, result: &str) -> Result<Option<Proposal>, String> {
    let cli_path = std::env::var("RUSTY_CLI_PATH").unwrap_or_else(|_| DEFAULT_CLI_PATH.to_string());
    let p = truncate(prompt, 2000);
    let r = truncate(result, 3000);
    let review_prompt = format!(
        "A task just completed. Decide whether it embodies a REUSABLE, GENERALIZABLE procedure \
         worth saving as an Agent Skill for next time. Most tasks do NOT — only propose one for a \
         non-trivial, multi-step workflow likely to recur.\n\n\
         Return ONLY JSON, no prose:\n\
         - worth saving: {{\"action\":\"create\",\"name\":\"kebab-case-slug\",\
         \"description\":\"one sentence with trigger keywords\",\"body\":\"## Procedure\\n...\"}}\n\
         - otherwise: {{\"action\":\"none\"}}\n\n\
         Rules: name is lowercase letters/digits/hyphens only. Do NOT put frontmatter or an \
         allowed-tools key in the body. Do NOT use backtick-bang command injection.\n\n\
         --- USER PROMPT ---\n{p}\n\n--- ASSISTANT RESULT ---\n{r}"
    );
    let output = process_manager::run_cli(
        &cli_path,
        &review_prompt,
        "You decide whether a completed task should become a reusable skill. Return only valid JSON.",
        None,
        None,
        None,
        &SkillsAccess::Disabled,
        3,
        REVIEW_BUDGET_USD,
    )
    .await
    .map_err(|e| format!("author CLI error: {e}"))?;

    let parsed = output_parser::parse(&output);
    if !parsed.success {
        return Err(format!("author CLI failed: {}", parsed.error));
    }
    Ok(parse_proposal(&parsed.result))
}

/// Extract a `{action:create,...}` proposal from the model's text. Returns `None` for
/// `action:none`, missing fields, an invalid name, or unparseable output.
fn parse_proposal(text: &str) -> Option<Proposal> {
    let text = text.trim();
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end < start {
        return None;
    }
    let json: serde_json::Value = serde_json::from_str(&text[start..=end]).ok()?;
    if json["action"].as_str() != Some("create") {
        return None;
    }
    let name = json["name"].as_str().unwrap_or("").trim().to_string();
    let description = json["description"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_string();
    let body = json["body"].as_str().unwrap_or("").trim().to_string();
    if name.is_empty() || description.is_empty() || body.is_empty() {
        return None;
    }
    // Reject names the skills store would refuse anyway, so we don't log a write error.
    if !skills::is_valid_skill_name(&name) {
        return None;
    }
    Some(Proposal {
        name,
        description,
        body,
    })
}

/// Truncate `s` to at most `max` bytes, on a char boundary.
fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        s[..s.floor_char_boundary(max)].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_create_proposal_with_surrounding_prose() {
        let text = "Sure:\n{\"action\":\"create\",\"name\":\"deploy-web\",\"description\":\"Deploy the web app\",\"body\":\"## Procedure\\nrun it\"}";
        let p = parse_proposal(text).unwrap();
        assert_eq!(p.name, "deploy-web");
        assert_eq!(p.description, "Deploy the web app");
        assert!(p.body.contains("## Procedure"));
    }

    #[test]
    fn parse_none_action() {
        assert!(parse_proposal(r#"{"action":"none"}"#).is_none());
    }

    #[test]
    fn parse_missing_or_empty_fields() {
        assert!(parse_proposal(r#"{"action":"create","name":"x"}"#).is_none());
        assert!(
            parse_proposal(r#"{"action":"create","name":"","description":"d","body":"b"}"#)
                .is_none()
        );
    }

    #[test]
    fn parse_rejects_invalid_name() {
        assert!(parse_proposal(
            r#"{"action":"create","name":"Bad Name","description":"d","body":"b"}"#
        )
        .is_none());
    }

    #[test]
    fn parse_garbage() {
        assert!(parse_proposal("no json here").is_none());
        assert!(parse_proposal("").is_none());
    }

    fn test_settings() -> Arc<SettingsManager> {
        use crate::engine::db::Database;
        use rusqlite::Connection;
        let conn = Connection::open_in_memory().unwrap();
        let db = Database::from_conn(conn);
        db.migrate().unwrap();
        Arc::new(SettingsManager::new(Arc::new(db)))
    }

    struct TempDir(std::path::PathBuf);
    impl TempDir {
        fn new() -> Self {
            let p =
                std::env::temp_dir().join(format!("rusty-author-test-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(&p).unwrap();
            TempDir(p)
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    // The default (`off`) returns before any subprocess or write — even on a big task.
    #[tokio::test]
    async fn off_mode_is_a_noop() {
        let tmp = TempDir::new();
        let settings = test_settings();
        settings
            .set(skills::SETTING_PATH, tmp.0.to_str().unwrap())
            .unwrap();
        review_task_for_skill(settings, "prompt", "result", 100)
            .await
            .unwrap();
        assert!(SkillsManager::new(tmp.0.clone()).list(true).is_empty());
    }

    // `suggest` mode still returns before any subprocess when the task is below the
    // turn threshold (so the gate is verified without invoking the model).
    #[tokio::test]
    async fn below_min_turns_is_a_noop() {
        let tmp = TempDir::new();
        let settings = test_settings();
        settings
            .set(skills::SETTING_PATH, tmp.0.to_str().unwrap())
            .unwrap();
        settings.set(SETTING_AUTHOR_MODE, "suggest").unwrap();
        review_task_for_skill(settings, "prompt", "result", 1)
            .await
            .unwrap();
        assert!(SkillsManager::new(tmp.0.clone()).list(true).is_empty());
    }
}
