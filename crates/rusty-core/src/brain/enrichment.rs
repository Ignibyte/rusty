//! Post-conversation entity extraction and brain enrichment.
//!
//! After every conversation completes, this module extracts entity names
//! (people, companies, projects) from the exchange and updates their brain
//! pages. This is the compounding loop — every conversation makes the brain
//! smarter.
//!
//! Two extraction modes:
//! - **Passive**: Regex-free heuristic that finds capitalized multi-word names.
//!   Zero cost, instant, catches common entities.
//! - **Active**: Lightweight Claude CLI call to extract structured entities.
//!   ~$0.02 per conversation, catches ~95% of entities.

use crate::brain::BrainManager;
use crate::engine::output_parser;
use crate::engine::process_manager;
use crate::engine::settings_manager::SettingsManager;
use std::sync::Arc;

/// Default enrichment mode.
const DEFAULT_ENRICHMENT_MODE: &str = "passive";

/// Default CLI path for active enrichment.
const DEFAULT_CLI_PATH: &str = "claude";

/// An entity detected from conversation text.
#[derive(Debug, Clone)]
pub struct DetectedEntity {
    /// Entity name as it appeared in text.
    pub name: String,
    /// Inferred type: "person", "company", or "project".
    pub entity_type: String,
}

/// Run the enrichment pipeline on a completed conversation.
///
/// Checks the `brain_auto_enrich` setting, extracts entities from the
/// conversation text, and creates/updates brain pages accordingly.
/// Errors are logged, never propagated to the user.
pub async fn enrich_from_conversation(
    brain: Arc<BrainManager>,
    settings: Arc<SettingsManager>,
    prompt: &str,
    result: &str,
) -> Result<(), String> {
    // Check enrichment mode
    let mode = settings
        .get_or_default("brain_auto_enrich", DEFAULT_ENRICHMENT_MODE)
        .unwrap_or_else(|_| DEFAULT_ENRICHMENT_MODE.to_string());

    if mode == "off" {
        return Ok(());
    }

    // Combine prompt + result for entity extraction
    let text = format!("{prompt}\n\n{result}");

    // Extract entities based on mode
    let entities = if mode == "active" {
        match extract_entities_active(&text).await {
            Ok(active_entities) if !active_entities.is_empty() => active_entities,
            _ => extract_entities_passive(&text), // Fall back to passive on error
        }
    } else {
        extract_entities_passive(&text)
    };

    if entities.is_empty() {
        return Ok(());
    }

    let today = today_iso();

    // Process each detected entity
    for entity in &entities {
        // Try to resolve to an existing page
        let existing = brain.resolve_slug(&entity.name).unwrap_or_default();

        if let Some(slug) = existing.first() {
            // Entity exists — add a timeline entry
            if let Err(e) = brain.add_timeline(
                slug,
                &today,
                "conversation",
                "Mentioned in conversation",
                None,
            ) {
                eprintln!("Enrichment: failed to add timeline for {slug}: {e}");
            }
        } else {
            // New entity — create a brain page
            match brain.create_page(
                &entity.entity_type,
                &entity.name,
                "Mentioned in conversation.",
            ) {
                Ok(page) => {
                    // Add initial timeline entry
                    let _ = brain.add_timeline(
                        &page.slug,
                        &today,
                        "conversation",
                        "First mentioned in conversation",
                        None,
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Enrichment: failed to create page for '{}': {e}",
                        entity.name
                    );
                }
            }
        }
    }

    Ok(())
}

/// Extract entities using passive heuristic (capitalized multi-word names).
///
/// Finds sequences of 2+ consecutive capitalized words, filters common
/// false positives, and infers entity type from suffixes.
/// Extract entities using Claude CLI for structured analysis.
///
/// Sends a lightweight prompt asking Claude to extract people, companies,
/// and projects as JSON. Budget: $0.02, max 3 turns. Calls run_cli directly
/// (not execute_task) to prevent recursive enrichment.
async fn extract_entities_active(text: &str) -> Result<Vec<DetectedEntity>, String> {
    let cli_path = std::env::var("RUSTY_CLI_PATH").unwrap_or_else(|_| DEFAULT_CLI_PATH.to_string());

    // Truncate text to avoid massive prompts
    let truncated = if text.len() > 4000 {
        &text[..text.floor_char_boundary(4000)]
    } else {
        text
    };

    let prompt = format!(
        "Extract all people, companies, and projects mentioned in this conversation. \
         Return ONLY a JSON array, no other text. Each item should have \"name\" and \"type\" \
         (person, company, or project). Example: [{{\"name\":\"Sarah Chen\",\"type\":\"person\"}}]\n\n\
         Conversation:\n{truncated}"
    );

    let output = process_manager::run_cli(
        &cli_path,
        &prompt,
        "You extract entities from text. Return only valid JSON arrays. No explanation.",
        None,
        None,
        None,
        &crate::skills::SkillsAccess::Disabled,
        3,
        0.02,
    )
    .await
    .map_err(|e| format!("CLI error: {e}"))?;

    let parsed = output_parser::parse(&output);
    if !parsed.success {
        return Err(format!("CLI failed: {}", parsed.error));
    }

    // Parse the JSON array from the result
    // Try to find a JSON array in the response (Claude may add surrounding text)
    let result = parsed.result.trim();
    let json_start = result.find('[').ok_or("No JSON array in response")?;
    let json_end = result.rfind(']').ok_or("No closing bracket in response")?;
    let json_str = &result[json_start..=json_end];

    let arr: Vec<serde_json::Value> =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;

    let mut entities = Vec::new();
    for item in &arr {
        let name = item["name"].as_str().unwrap_or("").trim();
        let entity_type = item["type"].as_str().unwrap_or("person").trim();
        if !name.is_empty() && name.len() >= 2 {
            entities.push(DetectedEntity {
                name: name.to_string(),
                entity_type: entity_type.to_string(),
            });
        }
    }

    Ok(entities)
}

/// Extract entities using passive heuristic (capitalized multi-word names).
///
/// Finds sequences of 2+ consecutive capitalized words, filters common
/// false positives, and infers entity type from suffixes.
pub fn extract_entities_passive(text: &str) -> Vec<DetectedEntity> {
    let mut entities: Vec<DetectedEntity> = Vec::new();
    let mut seen: Vec<String> = Vec::new();

    for line in text.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;

        while i < words.len() {
            // Look for sequences of capitalized words
            if is_capitalized_word(words[i]) && !is_stop_word(words[i]) {
                let start = i;
                i += 1;
                while i < words.len() && is_capitalized_word(words[i]) {
                    i += 1;
                }

                // Only keep sequences of 2+ words (single caps words are too noisy)
                if i - start >= 2 {
                    let name_words: Vec<&str> = words[start..i]
                        .iter()
                        .map(|w| strip_punctuation(w))
                        .collect();
                    let name = name_words.join(" ");

                    // Skip if too short or a known false positive phrase
                    if name.len() >= 4 && !is_false_positive_phrase(&name) {
                        let lower = name.to_lowercase();
                        if !seen.contains(&lower) {
                            seen.push(lower);
                            let entity_type = infer_entity_type(&name);
                            entities.push(DetectedEntity { name, entity_type });
                        }
                    }
                }
            } else {
                i += 1;
            }
        }
    }

    entities
}

/// Check if a word starts with an uppercase letter.
fn is_capitalized_word(word: &str) -> bool {
    let clean = strip_punctuation(word);
    clean
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

/// Strip leading/trailing punctuation from a word.
fn strip_punctuation(word: &str) -> &str {
    word.trim_matches(|c: char| !c.is_alphanumeric())
}

/// Check if a word is a common stop word (frequent false positives).
fn is_stop_word(word: &str) -> bool {
    let clean = strip_punctuation(word).to_lowercase();
    matches!(
        clean.as_str(),
        "i" | "the"
            | "this"
            | "that"
            | "these"
            | "those"
            | "here"
            | "there"
            | "it"
            | "its"
            | "my"
            | "we"
            | "our"
            | "you"
            | "your"
            | "he"
            | "she"
            | "they"
            | "if"
            | "so"
            | "but"
            | "or"
            | "and"
            | "for"
            | "not"
            | "no"
            | "yes"
            | "ok"
            | "let"
            | "can"
            | "will"
            | "would"
            | "should"
            | "could"
            | "may"
            | "just"
            | "also"
            | "sure"
            | "thank"
            | "thanks"
            | "please"
            | "note"
            | "however"
            | "therefore"
            | "furthermore"
            | "additionally"
            | "later"
            | "after"
            | "before"
            | "then"
            | "next"
            | "first"
            | "finally"
            | "recently"
            | "today"
            | "yesterday"
            | "tomorrow"
            | "when"
            | "where"
            | "while"
            | "since"
            | "until"
            | "during"
            | "about"
            | "each"
            | "every"
            | "both"
            | "all"
            | "any"
            | "some"
            | "most"
            | "many"
            | "much"
            | "more"
            | "other"
            | "new"
            | "old"
    )
}

/// Check if a multi-word name is a common false positive phrase.
fn is_false_positive_phrase(name: &str) -> bool {
    let lower = name.to_lowercase();
    // Common sentence starters and filler phrases
    let phrases = [
        "let me",
        "here is",
        "here are",
        "thank you",
        "phase plan",
        "phase design",
        "phase implement",
        "for example",
        "in order",
        "on the",
        "at the",
        "key context",
        "compiled truth",
    ];
    for phrase in &phrases {
        if lower == *phrase {
            return true;
        }
    }

    // Month + day combinations
    let months = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];
    let first_word = lower.split_whitespace().next().unwrap_or("");
    if months.contains(&first_word) {
        return true;
    }

    false
}

/// Infer entity type from the name.
///
/// Company suffixes (Corp, Inc, LLC, Ltd) → "company".
/// Default → "person" (most entities in conversation are people).
fn infer_entity_type(name: &str) -> String {
    let lower = name.to_lowercase();
    let company_suffixes = [
        "corp", "inc", "llc", "ltd", "co", "company", "group", "labs",
    ];

    for suffix in &company_suffixes {
        if lower.ends_with(suffix) {
            return "company".to_string();
        }
    }

    "person".to_string()
}

/// Get today's date as an ISO 8601 string.
fn today_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = now / 86400;
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passive_extracts_names() {
        let text = "I talked to Sarah Chen about the project yesterday.";
        let entities = extract_entities_passive(text);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "Sarah Chen");
        assert_eq!(entities[0].entity_type, "person");
    }

    #[test]
    fn passive_extracts_companies() {
        let text = "Working with Acme Corp on the integration.";
        let entities = extract_entities_passive(text);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].name, "Acme Corp");
        assert_eq!(entities[0].entity_type, "company");
    }

    #[test]
    fn passive_filters_false_positives() {
        let text = "Thank You. Let Me check. Here Is the result.";
        let entities = extract_entities_passive(text);
        assert!(entities.is_empty());
    }

    #[test]
    fn passive_deduplicates() {
        let text = "Sarah Chen said hello. Later, Sarah Chen also mentioned it.";
        let entities = extract_entities_passive(text);
        assert_eq!(entities.len(), 1);
    }

    #[test]
    fn passive_mixed_entities() {
        let text = "Sarah Chen at Acme Corp discussed the integration with Bob Williams.";
        let entities = extract_entities_passive(text);
        let names: Vec<&str> = entities.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"Sarah Chen"));
        assert!(names.contains(&"Acme Corp"));
        assert!(names.contains(&"Bob Williams"));
        assert_eq!(entities.len(), 3);
    }

    #[test]
    fn passive_skips_single_capitalized_words() {
        let text = "Rust is great. Python too.";
        let entities = extract_entities_passive(text);
        assert!(entities.is_empty());
    }

    #[test]
    fn infer_type_company_suffixes() {
        assert_eq!(infer_entity_type("Acme Corp"), "company");
        assert_eq!(infer_entity_type("Tech Labs"), "company");
        assert_eq!(infer_entity_type("OpenAI Inc"), "company");
        assert_eq!(infer_entity_type("Sarah Chen"), "person");
    }
}
