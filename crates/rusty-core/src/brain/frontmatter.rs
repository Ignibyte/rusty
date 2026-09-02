//! YAML frontmatter parsing and serialization for brain pages.
//!
//! Brain pages use YAML frontmatter delimited by `---` markers at the top of the file.
//! The page body is split into compiled truth (above a `---` separator) and timeline
//! (below it). This module handles parsing raw markdown into structured parts and
//! rendering structured parts back to markdown.

use std::collections::HashMap;

/// Structured YAML frontmatter for a brain page.
///
/// Known fields are typed explicitly. Arbitrary extra fields (e.g., `company`, `role`,
/// `relationship`) are captured in `extra` via `serde(flatten)` — this preserves
/// Obsidian-compatible frontmatter with user-defined properties.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrainFrontmatter {
    /// Page title.
    pub title: String,
    /// Entity type (person, company, project, concept, meeting, idea, daily, inbox).
    #[serde(rename = "type")]
    pub page_type: String,
    /// Alternative names for fuzzy resolution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Tags for categorization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// ISO date when the page was created.
    #[serde(default)]
    pub created: String,
    /// ISO date when the page was last updated.
    #[serde(default)]
    pub updated: String,
    /// Arbitrary extra fields (company, role, relationship, etc.).
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl BrainFrontmatter {
    /// Create new frontmatter with defaults for a given type and title.
    pub fn new(page_type: &str, title: &str) -> Self {
        let today = today_iso();
        Self {
            title: title.to_string(),
            page_type: page_type.to_string(),
            aliases: Vec::new(),
            tags: Vec::new(),
            created: today.clone(),
            updated: today,
            extra: HashMap::new(),
        }
    }
}

/// Parsed sections of a brain page.
pub struct ParsedPage {
    /// YAML frontmatter.
    pub frontmatter: BrainFrontmatter,
    /// Content above the body separator (synthesized knowledge).
    pub compiled_truth: String,
    /// Content below the body separator (append-only evidence).
    pub timeline: String,
}

/// Parse raw markdown content into frontmatter, compiled truth, and timeline.
///
/// Expects the file to start with `---\n`, followed by YAML, then `---\n`.
/// The remaining body is split at a standalone `\n---\n` separator into
/// compiled truth and timeline sections.
pub fn parse_page(raw: &str) -> Result<ParsedPage, String> {
    // Find frontmatter delimiters
    let trimmed = raw.trim_start();
    if !trimmed.starts_with("---") {
        return Err("Missing frontmatter: file must start with ---".to_string());
    }

    // Find the closing --- (skip the opening one)
    let after_open = &trimmed[3..].trim_start_matches(['\r', '\n']);
    let close_pos = after_open
        .find("\n---")
        .ok_or_else(|| "Missing closing frontmatter delimiter (---)".to_string())?;

    let yaml_str = &after_open[..close_pos];
    let after_frontmatter = &after_open[close_pos + 4..]; // skip \n---

    // Skip any trailing newlines after the closing ---
    let body = after_frontmatter.trim_start_matches(['\r', '\n']);

    // Parse YAML
    let frontmatter: BrainFrontmatter =
        serde_yaml::from_str(yaml_str).map_err(|e| format!("Failed to parse frontmatter: {e}"))?;

    // Split body at standalone --- separator into compiled_truth and timeline
    let (compiled_truth, timeline) = if let Some(sep_pos) = body.find("\n---\n") {
        let truth = body[..sep_pos].trim().to_string();
        let tl = body[sep_pos + 5..].trim().to_string();
        (truth, tl)
    } else {
        (body.trim().to_string(), String::new())
    };

    Ok(ParsedPage {
        frontmatter,
        compiled_truth,
        timeline,
    })
}

/// Render structured page parts back to a full markdown string.
pub fn render_page(
    frontmatter: &BrainFrontmatter,
    compiled_truth: &str,
    timeline: &str,
) -> Result<String, String> {
    let yaml =
        serde_yaml::to_string(frontmatter).map_err(|e| format!("Failed to serialize YAML: {e}"))?;

    // Wrap YAML in --- delimiters (serde_yaml does not add them)
    let yaml_clean = yaml.trim_end().trim_start_matches("---").trim();
    let mut page = format!("---\n{yaml_clean}\n---\n");

    if !compiled_truth.is_empty() {
        page.push('\n');
        page.push_str(compiled_truth);
        page.push('\n');
    }

    if !timeline.is_empty() {
        page.push_str("\n---\n\n");
        page.push_str(timeline);
        page.push('\n');
    }

    Ok(page)
}

/// Get today's date as an ISO 8601 string (YYYY-MM-DD).
fn today_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple conversion: seconds since epoch → days → date
    let days = now / 86400;
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
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
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_complete_page() {
        let raw = "---\ntitle: Sarah Chen\ntype: person\ntags:\n  - engineering\n---\n\nShe is a CTO.\n\n---\n\n## Timeline\n\n### 2026-04-12\nMet at conference.\n";
        let parsed = parse_page(raw).unwrap();
        assert_eq!(parsed.frontmatter.title, "Sarah Chen");
        assert_eq!(parsed.frontmatter.page_type, "person");
        assert_eq!(parsed.frontmatter.tags, vec!["engineering"]);
        assert_eq!(parsed.compiled_truth, "She is a CTO.");
        assert!(parsed.timeline.contains("## Timeline"));
        assert!(parsed.timeline.contains("Met at conference."));
    }

    #[test]
    fn parse_no_timeline() {
        let raw = "---\ntitle: Rust\ntype: concept\n---\n\nA systems programming language.\n";
        let parsed = parse_page(raw).unwrap();
        assert_eq!(parsed.compiled_truth, "A systems programming language.");
        assert!(parsed.timeline.is_empty());
    }

    #[test]
    fn parse_empty_body() {
        let raw = "---\ntitle: Empty\ntype: inbox\n---\n";
        let parsed = parse_page(raw).unwrap();
        assert!(parsed.compiled_truth.is_empty());
        assert!(parsed.timeline.is_empty());
    }

    #[test]
    fn parse_extra_fields_preserved() {
        let raw =
            "---\ntitle: Sarah Chen\ntype: person\ncompany: Acme\nrole: CTO\n---\n\nContent.\n";
        let parsed = parse_page(raw).unwrap();
        assert_eq!(
            parsed
                .frontmatter
                .extra
                .get("company")
                .and_then(|v| v.as_str()),
            Some("Acme")
        );
        assert_eq!(
            parsed
                .frontmatter
                .extra
                .get("role")
                .and_then(|v| v.as_str()),
            Some("CTO")
        );
    }

    #[test]
    fn render_round_trip() {
        let fm = BrainFrontmatter {
            title: "Test Page".to_string(),
            page_type: "concept".to_string(),
            aliases: vec!["test".to_string()],
            tags: vec!["testing".to_string()],
            created: "2026-04-12".to_string(),
            updated: "2026-04-12".to_string(),
            extra: HashMap::new(),
        };
        let truth = "This is compiled truth.";
        let timeline = "## Timeline\n\n### 2026-04-12\nCreated.";

        let rendered = render_page(&fm, truth, timeline).unwrap();
        let parsed = parse_page(&rendered).unwrap();

        assert_eq!(parsed.frontmatter.title, "Test Page");
        assert_eq!(parsed.frontmatter.page_type, "concept");
        assert_eq!(parsed.frontmatter.aliases, vec!["test"]);
        assert_eq!(parsed.compiled_truth, truth);
        assert_eq!(parsed.timeline, timeline);
    }

    #[test]
    fn parse_missing_frontmatter() {
        let raw = "No frontmatter here.";
        assert!(parse_page(raw).is_err());
    }
}
