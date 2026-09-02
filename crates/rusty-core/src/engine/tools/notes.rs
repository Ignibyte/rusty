//! Note tools: create, read, list, and delete notes via the AI function registry.

use crate::engine::tool_registry::{ToolDef, ToolRegistry};
use crate::notes::NotesManager;
use std::sync::Arc;

/// Register note-related tools with the registry.
pub fn register(registry: &ToolRegistry, notes: Arc<NotesManager>) {
    // create_note
    let nm = Arc::clone(&notes);
    registry.register(ToolDef {
        name: "create_note".into(),
        description: "Create a new markdown note".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Note title"},
                "content": {"type": "string", "description": "Note content in markdown"}
            },
            "required": ["name"]
        }),
        handler: Box::new(move |params| {
            let name = params["name"]
                .as_str()
                .ok_or_else(|| "name is required".to_string())?;
            let content = params["content"].as_str().unwrap_or("");
            let path = nm.create_note("", name, false)?;
            if !content.is_empty() {
                nm.save_note(&path, content)?;
            }
            Ok(format!("Note '{name}' created at {path}"))
        }),
    });

    // list_notes
    let nm = Arc::clone(&notes);
    registry.register(ToolDef {
        name: "list_notes".into(),
        description: "List all notes in the tree".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        handler: Box::new(move |_| {
            let tree = nm.list_tree()?;
            if tree.is_empty() {
                return Ok("No notes found.".to_string());
            }
            let mut lines = Vec::new();
            format_tree(&tree, &mut lines, 0);
            Ok(lines.join("\n"))
        }),
    });

    // read_note
    let nm = Arc::clone(&notes);
    registry.register(ToolDef {
        name: "read_note".into(),
        description: "Read the content of a note by its path".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Relative path to the note (e.g. 'Shopping List.md')"}
            },
            "required": ["path"]
        }),
        handler: Box::new(move |params| {
            let path = params["path"]
                .as_str()
                .ok_or_else(|| "path is required".to_string())?;
            nm.read_note(path)
        }),
    });

    // delete_note
    let nm = Arc::clone(&notes);
    registry.register(ToolDef {
        name: "delete_note".into(),
        description: "Delete a note (moves to trash)".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Relative path to the note"}
            },
            "required": ["path"]
        }),
        handler: Box::new(move |params| {
            let path = params["path"]
                .as_str()
                .ok_or_else(|| "path is required".to_string())?;
            nm.delete_note(path)?;
            Ok(format!("Note '{path}' deleted."))
        }),
    });
}

/// Format a note tree into indented lines for display.
fn format_tree(entries: &[crate::notes::NoteTreeEntry], lines: &mut Vec<String>, depth: usize) {
    let indent = "  ".repeat(depth);
    for entry in entries {
        if entry.entry_type == "folder" {
            lines.push(format!("{indent}📁 {}/", entry.name));
            format_tree(&entry.children, lines, depth + 1);
        } else {
            lines.push(format!("{indent}📄 {}", entry.name));
        }
    }
}
