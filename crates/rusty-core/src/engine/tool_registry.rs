//! AI function registry: registration, catalog generation, dispatch, and response parsing.
//!
//! Provides a `ToolRegistry` that holds callable tool definitions. Each tool has a name,
//! description, JSON parameter schema, and a handler function. The registry generates a
//! tool catalog for system prompt injection and dispatches tool calls parsed from Claude's
//! response text.

use std::collections::HashMap;
use std::sync::RwLock;

/// Definition of a callable AI tool.
pub struct ToolDef {
    /// Tool name (e.g., "create_note").
    pub name: String,
    /// Human-readable description for the tool catalog.
    pub description: String,
    /// JSON Schema describing the tool's parameters.
    pub parameters_schema: serde_json::Value,
    /// Synchronous handler that executes the tool and returns a result string.
    pub handler: Box<dyn Fn(serde_json::Value) -> Result<String, String> + Send + Sync>,
}

/// Central registry of AI-callable tools.
///
/// Uses interior mutability (`RwLock`) so tools can be registered after construction
/// (e.g., during Tauri's `setup()` phase when `AppHandle` becomes available).
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, ToolDef>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
        }
    }

    /// Register a tool. Overwrites any existing tool with the same name.
    pub fn register(&self, tool: ToolDef) {
        let name = tool.name.clone();
        let mut tools = self.tools.write().expect("tool registry poisoned");
        tools.insert(name, tool);
    }

    /// Generate a formatted tool catalog for system prompt injection.
    ///
    /// Produces a markdown-formatted block listing each tool's name, description,
    /// and parameter schema.
    pub fn generate_catalog(&self) -> String {
        let tools = self.tools.read().expect("tool registry poisoned");
        if tools.is_empty() {
            return String::new();
        }

        let mut parts = vec![
            "## Available App Functions\n".to_string(),
            "You can execute app functions by including <tool_call> blocks in your response. Format:".to_string(),
            "<tool_call name=\"tool_name\">{\"param\": \"value\"}</tool_call>\n".to_string(),
            "Each tool call will be executed and results provided back to you.\n".to_string(),
        ];

        // Sort by name for deterministic output
        let mut names: Vec<&String> = tools.keys().collect();
        names.sort();

        for name in names {
            let tool = &tools[name];
            parts.push(format!("### {}", tool.name));
            parts.push(tool.description.clone());
            parts.push(format!("Parameters: {}", tool.parameters_schema));
            parts.push(String::new());
        }

        parts.join("\n")
    }

    /// Execute a tool by name with the given parameters.
    ///
    /// Returns `Err` if the tool is not found or the handler fails.
    pub fn execute(&self, name: &str, params: serde_json::Value) -> Result<String, String> {
        let tools = self.tools.read().expect("tool registry poisoned");
        let tool = tools
            .get(name)
            .ok_or_else(|| format!("Unknown tool: {name}"))?;
        (tool.handler)(params)
    }
}

/// A parsed tool call extracted from Claude's response text.
#[derive(Debug, Clone)]
pub struct ParsedToolCall {
    /// Tool name from the `name` attribute.
    pub name: String,
    /// Parsed JSON arguments from the tag body.
    pub arguments: serde_json::Value,
}

/// Extract `<tool_call>` tags from response text.
///
/// Returns the parsed tool calls and the cleaned text with tags removed.
///
/// Format: `<tool_call name="tool_name">{"param": "value"}</tool_call>`
pub fn extract_tool_calls(text: &str) -> (Vec<ParsedToolCall>, String) {
    let mut calls = Vec::new();
    let mut cleaned = text.to_string();

    while let Some(start) = cleaned.find("<tool_call") {
        let Some(tag_end) = cleaned[start..].find('>') else {
            break;
        };
        let tag_end = start + tag_end;

        let Some(close) = cleaned[tag_end..].find("</tool_call>") else {
            break;
        };
        let close = tag_end + close;

        // Extract name attribute
        let tag_str = &cleaned[start..=tag_end];
        let name = extract_attr(tag_str, "name").unwrap_or_default();

        // Extract JSON body
        let body = cleaned[tag_end + 1..close].trim();

        if !name.is_empty() {
            match serde_json::from_str(body) {
                Ok(args) => {
                    calls.push(ParsedToolCall {
                        name,
                        arguments: args,
                    });
                }
                Err(e) => {
                    calls.push(ParsedToolCall {
                        name: name.clone(),
                        arguments: serde_json::json!({"__parse_error": format!("Invalid JSON in tool_call '{name}': {e}")}),
                    });
                }
            }
        }

        // Remove the entire tag from cleaned text
        let close_tag_end = close + "</tool_call>".len();
        cleaned = format!("{}{}", &cleaned[..start], &cleaned[close_tag_end..]);
    }

    (calls, cleaned.trim().to_string())
}

/// Extract an attribute value from an XML-like tag string.
fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    let start = tag.find(&pattern)?;
    let value_start = start + pattern.len();
    let value_end = tag[value_start..].find('"')?;
    Some(tag[value_start..value_start + value_end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> ToolRegistry {
        let registry = ToolRegistry::new();
        registry.register(ToolDef {
            name: "greet".into(),
            description: "Say hello".into(),
            parameters_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"}
                },
                "required": ["name"]
            }),
            handler: Box::new(|params| {
                let name = params["name"].as_str().unwrap_or("world");
                Ok(format!("Hello, {name}!"))
            }),
        });
        registry
    }

    #[test]
    fn registry_register_and_execute() {
        let reg = test_registry();
        let result = reg
            .execute("greet", serde_json::json!({"name": "Chad"}))
            .unwrap();
        assert_eq!(result, "Hello, Chad!");
    }

    #[test]
    fn registry_missing_tool_returns_error() {
        let reg = test_registry();
        let result = reg.execute("nonexistent", serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown tool"));
    }

    #[test]
    fn generate_catalog_includes_all_tools() {
        let reg = test_registry();
        reg.register(ToolDef {
            name: "farewell".into(),
            description: "Say goodbye".into(),
            parameters_schema: serde_json::json!({}),
            handler: Box::new(|_| Ok("Bye!".into())),
        });

        let catalog = reg.generate_catalog();
        assert!(catalog.contains("greet"));
        assert!(catalog.contains("farewell"));
        assert!(catalog.contains("Say hello"));
        assert!(catalog.contains("Say goodbye"));
        assert!(catalog.contains("Available App Functions"));
    }

    #[test]
    fn generate_catalog_empty_registry() {
        let reg = ToolRegistry::new();
        assert!(reg.generate_catalog().is_empty());
    }

    #[test]
    fn extract_tool_calls_parses_tags() {
        let text = r#"I'll create that note. <tool_call name="create_note">{"name": "Shopping", "content": "milk"}</tool_call> Done!"#;
        let (calls, cleaned) = extract_tool_calls(text);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "create_note");
        assert_eq!(calls[0].arguments["name"], "Shopping");
        assert_eq!(calls[0].arguments["content"], "milk");
        assert_eq!(cleaned, "I'll create that note.  Done!");
    }

    #[test]
    fn extract_tool_calls_no_tags() {
        let text = "No tool calls here.";
        let (calls, cleaned) = extract_tool_calls(text);
        assert!(calls.is_empty());
        assert_eq!(cleaned, "No tool calls here.");
    }

    #[test]
    fn extract_tool_calls_strips_tags() {
        let text = "Before <tool_call name=\"test\">{}</tool_call> after";
        let (calls, cleaned) = extract_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(cleaned, "Before  after");
    }

    #[test]
    fn extract_tool_calls_malformed_json() {
        let text = r#"<tool_call name="bad">not valid json</tool_call>"#;
        let (calls, _) = extract_tool_calls(text);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "bad");
        assert!(calls[0].arguments["__parse_error"].as_str().is_some());
    }

    #[test]
    fn execute_known_tool_succeeds() {
        let reg = test_registry();
        assert!(reg
            .execute("greet", serde_json::json!({"name": "test"}))
            .is_ok());
        assert!(reg.execute("nonexistent", serde_json::json!({})).is_err());
    }

    #[test]
    fn register_overwrites_existing() {
        let reg = test_registry();
        reg.register(ToolDef {
            name: "greet".into(),
            description: "new".into(),
            parameters_schema: serde_json::json!({}),
            handler: Box::new(|_| Ok("replaced".into())),
        });
        let result = reg.execute("greet", serde_json::json!({})).unwrap();
        assert_eq!(result, "replaced");
    }
}
