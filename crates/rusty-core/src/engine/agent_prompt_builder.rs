//! Agent prompt composition: builds layered system prompts from templates + memory + context.

use std::path::Path;

/// Default fallback prompt if the template file can't be loaded.
const FALLBACK_PROMPT: &str = "You are Rusty, a helpful AI personal assistant running as a desktop app. You have full tool access and persistent memory. Be direct and concise.";

/// Build a complete system prompt by composing layers.
///
/// Layers:
/// 1. Agent base prompt (from helper.md template or fallback)
/// 2. Current date/time
/// 3. Memory context (`<user_memory>` block, may be empty)
/// 4. Brain context (`<brain_context>` block, may be empty)
/// 5. Tool catalog (available app functions, may be empty)
pub fn build_system_prompt(
    memory_context: &str,
    brain_context: &str,
    tool_catalog: &str,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Layer 1: Agent base prompt
    let base = load_prompt_template();
    parts.push(base);

    // Layer 2: Current date/time
    let now = chrono_now_formatted();
    parts.push(format!("\n## Current Date\n{now}"));

    // Layer 3: Memory context (if any)
    if !memory_context.is_empty() {
        parts.push(format!("\n{memory_context}"));
    }

    // Layer 4: Brain context (if any)
    if !brain_context.is_empty() {
        parts.push(format!("\n{brain_context}"));
    }

    // Layer 5: Tool catalog (if any)
    if !tool_catalog.is_empty() {
        parts.push(format!("\n{tool_catalog}"));
    }

    parts.join("\n")
}

/// Load the agent prompt template from the prompts directory.
///
/// Tries to read `prompts/helper.md` relative to the executable, then
/// falls back to a bundled directory, then to a hardcoded fallback.
fn load_prompt_template() -> String {
    // Try relative to executable (for development: backend/prompts/)
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    let candidate_paths = [
        // Development: running from backend/
        exe_dir
            .as_ref()
            .map(|d| d.join("../prompts/helper.md"))
            .unwrap_or_default(),
        // Production: bundled in app resources
        exe_dir
            .as_ref()
            .map(|d| d.join("prompts/helper.md"))
            .unwrap_or_default(),
        // Fallback: relative to CWD
        Path::new("prompts/helper.md").to_path_buf(),
        // Workspace development path
        Path::new("backend/prompts/helper.md").to_path_buf(),
    ];

    for path in &candidate_paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if !content.trim().is_empty() {
                    return content;
                }
            }
        }
    }

    // Hardcoded fallback
    FALLBACK_PROMPT.to_string()
}

/// Get current date/time formatted for prompt injection.
fn chrono_now_formatted() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Simple date formatting without chrono crate
    // Returns Unix timestamp as a readable approximation
    // For production, consider adding the chrono crate
    let secs_per_day = 86400u64;
    let days_since_epoch = now / secs_per_day;

    // Zeller-like calculation for year/month/day
    let mut remaining = days_since_epoch;
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u64;
    for &days in &days_in_months {
        if remaining < days {
            break;
        }
        remaining -= days;
        month += 1;
    }
    let day = remaining + 1;

    let hour = (now % secs_per_day) / 3600;
    let minute = (now % 3600) / 60;

    format!("{year}-{month:02}-{day:02} {hour:02}:{minute:02} UTC")
}

/// Check if a year is a leap year.
fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_system_prompt_without_memory() {
        let prompt = build_system_prompt("", "", "");
        // Should contain base prompt (either loaded or fallback)
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Current Date"));
        // Should NOT contain actual memory content block (## Core) when empty
        // Note: helper.md mentions <user_memory> in instructions — that's expected
        assert!(!prompt.contains("## Core\n-"));
    }

    #[test]
    fn build_system_prompt_with_memory() {
        let memory = "<user_memory>\n## Core\n- Prefers Rust\n</user_memory>";
        let prompt = build_system_prompt(memory, "", "");
        assert!(prompt.contains("<user_memory>"));
        assert!(prompt.contains("Prefers Rust"));
        assert!(prompt.contains("Current Date"));
    }

    #[test]
    fn build_system_prompt_with_tool_catalog() {
        let catalog = "## Available App Functions\n\n### create_note\nCreate a note.";
        let prompt = build_system_prompt("", "", catalog);
        assert!(prompt.contains("Available App Functions"));
        assert!(prompt.contains("create_note"));
    }

    #[test]
    fn build_system_prompt_with_brain_context() {
        let brain = "<brain_context>\n## Relevant Brain Pages\n\n### Sarah Chen (person)\nCTO of Acme Corp.\n</brain_context>";
        let prompt = build_system_prompt("", brain, "");
        assert!(prompt.contains("<brain_context>"));
        assert!(prompt.contains("Sarah Chen"));
        assert!(prompt.contains("CTO of Acme"));
    }

    #[test]
    fn build_system_prompt_empty_brain_context_excluded() {
        let prompt = build_system_prompt("", "", "");
        assert!(!prompt.contains("<brain_context>"));
    }

    #[test]
    fn chrono_now_formatted_produces_valid_date() {
        let date = chrono_now_formatted();
        // Should look like "2026-04-05 02:30 UTC"
        assert!(date.contains("UTC"));
        assert!(date.contains("-"));
        assert!(date.contains(":"));
    }

    #[test]
    fn fallback_prompt_is_valid() {
        assert!(FALLBACK_PROMPT.contains("Rusty"));
    }

    #[test]
    fn is_leap_year_works() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
    }
}
