//! Parse JSON output from Claude Code CLI (`--output-format json`).

/// Parsed result from Claude CLI JSON output.
#[derive(Debug, Clone)]
pub struct ParsedOutput {
    pub success: bool,
    pub result: String,
    pub error: String,
    pub session_id: String,
    pub cost_usd: f64,
    pub num_turns: i64,
    pub duration_ms: i64,
}

/// Parse the JSON stdout from Claude CLI.
///
/// Expected format:
/// ```json
/// {
///   "type": "result",
///   "subtype": "success",
///   "is_error": false,
///   "result": "response text",
///   "session_id": "uuid",
///   "total_cost_usd": 0.05,
///   "num_turns": 3,
///   "duration_ms": 2000
/// }
/// ```
pub fn parse(stdout: &str) -> ParsedOutput {
    let empty = ParsedOutput {
        success: false,
        result: String::new(),
        error: "Empty output from Claude CLI".to_string(),
        session_id: String::new(),
        cost_usd: 0.0,
        num_turns: 0,
        duration_ms: 0,
    };

    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return empty;
    }

    let json: serde_json::Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => {
            return ParsedOutput {
                success: false,
                result: String::new(),
                error: format!(
                    "Failed to parse CLI output as JSON: {}",
                    &trimmed[..trimmed.len().min(200)]
                ),
                session_id: String::new(),
                cost_usd: 0.0,
                num_turns: 0,
                duration_ms: 0,
            };
        }
    };

    let session_id = json["session_id"].as_str().unwrap_or("").to_string();
    let cost_usd = json["total_cost_usd"].as_f64().unwrap_or(0.0);
    let num_turns = json["num_turns"].as_i64().unwrap_or(0);
    let duration_ms = json["duration_ms"].as_i64().unwrap_or(0);
    let is_error = json["is_error"].as_bool().unwrap_or(false);
    let subtype = json["subtype"].as_str().unwrap_or("");

    if is_error || subtype != "success" {
        let error = match subtype {
            "error_max_turns" => format!("Agent hit max turns limit ({num_turns} turns)"),
            "error_max_budget_usd" => format!("Budget exhausted (${cost_usd:.2})"),
            _ => json["result"]
                .as_str()
                .or_else(|| json["error"].as_str())
                .unwrap_or("Unknown CLI error")
                .to_string(),
        };

        return ParsedOutput {
            success: false,
            result: String::new(),
            error,
            session_id,
            cost_usd,
            num_turns,
            duration_ms,
        };
    }

    let result = json["result"].as_str().unwrap_or("").to_string();

    ParsedOutput {
        success: true,
        result,
        error: String::new(),
        session_id,
        cost_usd,
        num_turns,
        duration_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_success() {
        let json = r#"{"type":"result","subtype":"success","is_error":false,"result":"Hello!","session_id":"abc-123","total_cost_usd":0.05,"num_turns":1,"duration_ms":2000}"#;
        let parsed = parse(json);
        assert!(parsed.success);
        assert_eq!(parsed.result, "Hello!");
        assert_eq!(parsed.session_id, "abc-123");
        assert!((parsed.cost_usd - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_error_max_turns() {
        let json = r#"{"type":"result","subtype":"error_max_turns","is_error":true,"num_turns":25,"session_id":"def-456","total_cost_usd":5.0,"duration_ms":30000}"#;
        let parsed = parse(json);
        assert!(!parsed.success);
        assert!(parsed.error.contains("max turns"));
    }

    #[test]
    fn parse_error_budget() {
        let json = r#"{"type":"result","subtype":"error_max_budget_usd","is_error":true,"session_id":"ghi-789","total_cost_usd":5.0,"duration_ms":1000,"num_turns":1}"#;
        let parsed = parse(json);
        assert!(!parsed.success);
        assert!(parsed.error.contains("Budget"));
    }

    #[test]
    fn parse_empty() {
        let parsed = parse("");
        assert!(!parsed.success);
        assert!(parsed.error.contains("Empty"));
    }

    #[test]
    fn parse_invalid_json() {
        let parsed = parse("not json at all");
        assert!(!parsed.success);
        assert!(parsed.error.contains("Failed to parse"));
    }
}
