//! Task tools: create groups, add tasks, toggle completion, and list tasks.

use crate::engine::tool_registry::{ToolDef, ToolRegistry};
use crate::engine::user_tasks::UserTaskManager;
use std::sync::Arc;

/// Register task-related tools with the registry.
pub fn register(registry: &ToolRegistry, tasks: Arc<UserTaskManager>) {
    // create_group
    let tm = Arc::clone(&tasks);
    registry.register(ToolDef {
        name: "create_task_group".into(),
        description: "Create a new task group/category".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Group name (e.g. 'Work', 'Shopping')"}
            },
            "required": ["name"]
        }),
        handler: Box::new(move |params| {
            let name = params["name"]
                .as_str()
                .ok_or_else(|| "name is required".to_string())?;
            let id = tm.create_header(name)?;
            Ok(format!("Task group '{name}' created (id: {id})"))
        }),
    });

    // add_task
    let tm = Arc::clone(&tasks);
    registry.register(ToolDef {
        name: "add_task".into(),
        description: "Add a task to a group. Use list_tasks first to find the group name.".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "group_name": {"type": "string", "description": "Name of the task group"},
                "title": {"type": "string", "description": "Task title"}
            },
            "required": ["group_name", "title"]
        }),
        handler: Box::new(move |params| {
            let group_name = params["group_name"]
                .as_str()
                .ok_or_else(|| "group_name is required".to_string())?;
            let title = params["title"]
                .as_str()
                .ok_or_else(|| "title is required".to_string())?;

            // Find the header by name
            let headers = tm.list_headers()?;
            let header = headers
                .iter()
                .find(|h| h.name.eq_ignore_ascii_case(group_name))
                .ok_or_else(|| {
                    format!("Task group '{group_name}' not found. Use create_task_group first.")
                })?;

            let id = tm.create_task(header.id, title)?;
            Ok(format!("Task '{title}' added to '{group_name}' (id: {id})"))
        }),
    });

    // toggle_task
    let tm = Arc::clone(&tasks);
    registry.register(ToolDef {
        name: "toggle_task".into(),
        description: "Toggle a task's completion status by ID".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Task ID"}
            },
            "required": ["id"]
        }),
        handler: Box::new(move |params| {
            let id = params["id"]
                .as_i64()
                .ok_or_else(|| "id is required".to_string())?;
            let completed = tm.toggle_complete(id)?;
            let status = if completed { "completed" } else { "incomplete" };
            Ok(format!("Task {id} marked as {status}"))
        }),
    });

    // list_tasks
    let tm = Arc::clone(&tasks);
    registry.register(ToolDef {
        name: "list_tasks".into(),
        description: "List all task groups and their tasks".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        handler: Box::new(move |_| {
            let headers = tm.list_headers()?;
            if headers.is_empty() {
                return Ok("No task groups found.".to_string());
            }
            let mut lines = Vec::new();
            for header in &headers {
                lines.push(format!("## {} (id: {})", header.name, header.id));
                let tasks = tm.list_tasks(header.id, false)?;
                if tasks.is_empty() {
                    lines.push("  (no tasks)".to_string());
                } else {
                    for task in &tasks {
                        let check = if task.completed { "x" } else { " " };
                        lines.push(format!("  [{}] {} (id: {})", check, task.title, task.id));
                    }
                }
            }
            Ok(lines.join("\n"))
        }),
    });
}
