//! Navigation tools: switch views in the app via the AI function registry.

use crate::engine::tool_registry::{ToolDef, ToolRegistry};
use crate::events::{AppEvent, EventBus};

/// Register navigation-related tools with the registry.
pub fn register(registry: &ToolRegistry, events: EventBus) {
    // switch_view
    registry.register(ToolDef {
        name: "switch_view".into(),
        description: "Switch the app to a different view (chat, notes, tasks, memory, agents, settings)".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "view": {"type": "string", "description": "View name: chat, notes, tasks, memory, agents, or settings"}
            },
            "required": ["view"]
        }),
        handler: Box::new(move |params| {
            let view = params["view"]
                .as_str()
                .ok_or_else(|| "view is required".to_string())?;

            let valid_views = ["chat", "notes", "tasks", "memory", "agents", "settings"];
            if !valid_views.contains(&view) {
                return Err(format!(
                    "Invalid view '{view}'. Valid views: {}",
                    valid_views.join(", ")
                ));
            }

            events.emit(AppEvent::NavigateView(view.to_string()));

            Ok(format!("Switched to {view} view"))
        }),
    });
}
