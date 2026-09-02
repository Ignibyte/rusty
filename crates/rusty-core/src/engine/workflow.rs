//! Workflow template resolution for controlling task turn limits and budgets.

/// A workflow template defining turn and budget limits for a task.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkflowTemplate {
    /// Template name (quick, standard, deep).
    pub name: String,
    /// Maximum number of Claude CLI turns.
    pub max_turns: u32,
    /// Maximum budget in USD.
    pub max_budget_usd: f64,
}

/// Resolve which workflow template to use for a given prompt.
///
/// If `explicit` is provided and matches a known template, use it.
/// Otherwise, scan the prompt for keywords to auto-detect.
/// Falls back to "standard" if no match.
pub fn resolve(prompt: &str, explicit: Option<&str>) -> WorkflowTemplate {
    // Explicit template override
    if let Some(name) = explicit {
        if let Some(t) = find_template(name) {
            return t;
        }
    }

    // Keyword-based auto-detection
    let lower = prompt.to_lowercase();

    // Deep work keywords
    let deep_keywords = [
        "build",
        "implement",
        "refactor",
        "create",
        "architect",
        "redesign",
    ];
    for keyword in &deep_keywords {
        if lower.contains(keyword) {
            return find_template("deep").unwrap();
        }
    }

    // Quick answer keywords
    let quick_keywords = ["what is", "explain", "how do", "define", "quick question"];
    for keyword in &quick_keywords {
        if lower.contains(keyword) {
            return find_template("quick").unwrap();
        }
    }

    // Default: standard
    find_template("standard").unwrap()
}

/// Get all available workflow templates.
pub fn all_templates() -> Vec<WorkflowTemplate> {
    vec![
        WorkflowTemplate {
            name: "quick".to_string(),
            max_turns: 5,
            max_budget_usd: 0.50,
        },
        WorkflowTemplate {
            name: "standard".to_string(),
            max_turns: 25,
            max_budget_usd: 5.00,
        },
        WorkflowTemplate {
            name: "deep".to_string(),
            max_turns: 75,
            max_budget_usd: 10.00,
        },
    ]
}

/// Find a template by name.
fn find_template(name: &str) -> Option<WorkflowTemplate> {
    all_templates().into_iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_default_returns_standard() {
        let t = resolve("hello there", None);
        assert_eq!(t.name, "standard");
        assert_eq!(t.max_turns, 25);
    }

    #[test]
    fn resolve_explicit_template() {
        let t = resolve("anything", Some("deep"));
        assert_eq!(t.name, "deep");
        assert_eq!(t.max_turns, 75);
    }

    #[test]
    fn resolve_explicit_unknown_falls_back() {
        let t = resolve("anything", Some("nonexistent"));
        assert_eq!(t.name, "standard");
    }

    #[test]
    fn resolve_keyword_deep() {
        let t = resolve("build a REST API for the app", None);
        assert_eq!(t.name, "deep");
    }

    #[test]
    fn resolve_keyword_quick() {
        let t = resolve("what is the capital of France?", None);
        assert_eq!(t.name, "quick");
    }

    #[test]
    fn resolve_deep_takes_priority_over_quick() {
        // "implement" (deep) should win even if "explain" (quick) is present
        let t = resolve("explain how to implement this", None);
        assert_eq!(t.name, "deep");
    }

    #[test]
    fn all_templates_returns_three() {
        assert_eq!(all_templates().len(), 3);
    }
}
