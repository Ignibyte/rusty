//! Memory tools: store and list memories via the AI function registry.

use crate::engine::memory_manager::MemoryManager;
use crate::engine::tool_registry::{ToolDef, ToolRegistry};
use std::sync::Arc;

/// Register memory-related tools with the registry.
pub fn register(registry: &ToolRegistry, memory: Arc<MemoryManager>) {
    // store_memory
    let mm = Arc::clone(&memory);
    registry.register(ToolDef {
        name: "store_memory".into(),
        description: "Store a new memory about the user".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "content": {"type": "string", "description": "What to remember"},
                "category": {"type": "string", "description": "Category: preference, fact, project, or context"},
                "importance": {"type": "string", "description": "Importance: high, normal, or low"}
            },
            "required": ["content"]
        }),
        handler: Box::new(move |params| {
            let content = params["content"]
                .as_str()
                .ok_or_else(|| "content is required".to_string())?;
            let category = params["category"].as_str().unwrap_or("context");
            let importance = params["importance"].as_str().unwrap_or("normal");
            let id = mm.store(category, importance, content, "ai-tool")?;
            Ok(format!("Memory stored (id: {id})"))
        }),
    });

    // list_memories
    let mm = Arc::clone(&memory);
    registry.register(ToolDef {
        name: "list_memories".into(),
        description: "List stored memories, optionally filtered by category".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "category": {"type": "string", "description": "Filter by category: preference, fact, project, or context"}
            }
        }),
        handler: Box::new(move |params| {
            let category = params["category"].as_str();
            let memories = mm.list(category)?;
            if memories.is_empty() {
                return Ok("No memories found.".to_string());
            }
            let lines: Vec<String> = memories
                .iter()
                .map(|m| format!("- [{}] [{}] {}", m.category, m.importance, m.content))
                .collect();
            Ok(lines.join("\n"))
        }),
    });
}
