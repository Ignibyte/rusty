//! Conversation tools: search, delete, and summarize conversations.

use crate::engine::task_manager::TaskManager;
use crate::engine::tool_registry::{ToolDef, ToolRegistry};
use std::sync::Arc;

/// Register conversation-related tools with the registry.
pub fn register(registry: &ToolRegistry, task_manager: Arc<TaskManager>) {
    // search_conversations
    let tm = Arc::clone(&task_manager);
    registry.register(ToolDef {
        name: "search_conversations".into(),
        description: "Search past conversations by keyword across prompts and responses".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Keyword to search for"}
            },
            "required": ["query"]
        }),
        handler: Box::new(move |params| {
            let query = params["query"]
                .as_str()
                .ok_or_else(|| "query is required".to_string())?;

            let results = tm.search_conversations(query, 10)?;
            if results.is_empty() {
                return Ok(format!("No conversations found matching '{query}'."));
            }

            let lines: Vec<String> = results
                .iter()
                .map(|c| {
                    format!(
                        "- [{}] \"{}\" ({} messages)",
                        c.id, c.title, c.message_count
                    )
                })
                .collect();
            Ok(format!(
                "Found {} conversations matching '{query}':\n{}",
                results.len(),
                lines.join("\n")
            ))
        }),
    });

    // delete_conversation
    let tm = Arc::clone(&task_manager);
    registry.register(ToolDef {
        name: "delete_conversation".into(),
        description: "Delete a conversation and all its messages by conversation ID".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "conversation_id": {"type": "string", "description": "The conversation ID to delete"}
            },
            "required": ["conversation_id"]
        }),
        handler: Box::new(move |params| {
            let conv_id = params["conversation_id"]
                .as_str()
                .ok_or_else(|| "conversation_id is required".to_string())?;

            tm.delete_conversation(conv_id)?;
            Ok(format!("Conversation '{conv_id}' deleted."))
        }),
    });

    // get_conversation_summary
    let tm = Arc::clone(&task_manager);
    registry.register(ToolDef {
        name: "get_conversation_summary".into(),
        description: "Get a summary of a conversation's exchanges (prompt/response pairs)".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "conversation_id": {"type": "string", "description": "The conversation ID to summarize"}
            },
            "required": ["conversation_id"]
        }),
        handler: Box::new(move |params| {
            let conv_id = params["conversation_id"]
                .as_str()
                .ok_or_else(|| "conversation_id is required".to_string())?;

            let tasks = tm.get_conversation_tasks(conv_id)?;
            if tasks.is_empty() {
                return Ok(format!("No messages found in conversation '{conv_id}'."));
            }

            let mut lines = Vec::new();
            for (i, task) in tasks.iter().enumerate() {
                let prompt_preview = truncate(&task.prompt, 200);
                let result_preview = truncate(&task.result, 300);
                lines.push(format!("--- Exchange {} ---", i + 1));
                lines.push(format!("User: {prompt_preview}"));
                if !task.result.is_empty() {
                    lines.push(format!("Assistant: {result_preview}"));
                }
            }
            Ok(format!(
                "Conversation has {} exchanges:\n{}",
                tasks.len(),
                lines.join("\n")
            ))
        }),
    });
}

/// Truncate a string to a maximum length, adding ellipsis if needed.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..s.floor_char_boundary(max.saturating_sub(3))])
    }
}
