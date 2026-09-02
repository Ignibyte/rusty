//! Agent dispatch tools: dispatch, list, and get results of Claude agents.

use crate::brain::BrainManager;
use crate::engine::agent_manager::{self, AgentManager};
use crate::engine::settings_manager::SettingsManager;
use crate::engine::tool_registry::{ToolDef, ToolRegistry};
use crate::engine::workflow;
use crate::events::EventBus;
use std::sync::Arc;

/// Register agent-related tools with the registry.
pub fn register(
    registry: &ToolRegistry,
    agent_manager: Arc<AgentManager>,
    brain_manager: Arc<BrainManager>,
    settings_manager: Arc<SettingsManager>,
    events: EventBus,
) {
    // dispatch_agent
    let am = Arc::clone(&agent_manager);
    let bm = Arc::clone(&brain_manager);
    let sm = Arc::clone(&settings_manager);
    let bus = events.clone();
    registry.register(ToolDef {
        name: "dispatch_agent".into(),
        description: "Dispatch a Claude agent to work in a project directory (local or remote via SSH). Returns immediately with an agent ID — the agent runs in the background."
            .into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "directory": {"type": "string", "description": "Path to the project directory. Local: absolute or ~/relative. Remote: user@host:/path"},
                "prompt": {"type": "string", "description": "What the agent should do in that directory"},
                "workflow": {"type": "string", "description": "Budget: quick ($0.50), standard ($5), or deep ($10). Default: deep"}
            },
            "required": ["directory", "prompt"]
        }),
        handler: Box::new(move |params| {
            let directory = params["directory"]
                .as_str()
                .ok_or_else(|| "directory is required".to_string())?;
            let prompt = params["prompt"]
                .as_str()
                .ok_or_else(|| "prompt is required".to_string())?;
            let wf_name = params["workflow"].as_str().unwrap_or("deep");

            // Expand ~ and validate (skip validation for SSH remote paths)
            let expanded = agent_manager::expand_tilde(directory);
            let is_remote = expanded.contains(':') && expanded.contains('@');
            if !is_remote && !std::path::Path::new(&expanded).is_dir() {
                return Err(format!("Directory not found: {expanded}"));
            }

            // Resolve workflow for budget/turns
            let wf = workflow::resolve(prompt, Some(wf_name));

            // Create agent record (synchronous SQLite)
            let agent_id = am.create_agent("", &expanded, prompt)?;

            // Fire-and-forget async execution
            let am2 = Arc::clone(&am);
            let bm2 = Arc::clone(&bm);
            let sm2 = Arc::clone(&sm);
            let bus2 = bus.clone();
            let id = agent_id.clone();
            let dir = expanded.clone();
            let p = prompt.to_string();
            tokio::spawn(async move {
                // Dispatched agents use Claude Code's default skill discovery when
                // enabled, but honor the master kill switch when skills are off.
                let skills = if crate::skills::is_enabled(&sm2) {
                    crate::skills::SkillsAccess::Inherit
                } else {
                    crate::skills::SkillsAccess::Disabled
                };
                agent_manager::execute_agent(
                    bus2,
                    am2,
                    bm2,
                    id,
                    dir,
                    p,
                    skills,
                    wf.max_turns,
                    wf.max_budget_usd,
                )
                .await;
            });

            Ok(format!(
                "Agent dispatched (id: {agent_id}). Running in {expanded}. Use get_agent_result to check when done."
            ))
        }),
    });

    // list_agents
    let am = Arc::clone(&agent_manager);
    registry.register(ToolDef {
        name: "list_agents".into(),
        description: "List dispatched agents with their status".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        handler: Box::new(move |_| {
            let agents = am.list_agents(20)?;
            if agents.is_empty() {
                return Ok("No agents have been dispatched yet.".to_string());
            }
            let lines: Vec<String> = agents
                .iter()
                .map(|a| {
                    let status = a.status.as_str();
                    let dir = &a.directory;
                    let prompt_preview = if a.prompt.len() > 60 {
                        format!("{}...", &a.prompt[..a.prompt.floor_char_boundary(57)])
                    } else {
                        a.prompt.clone()
                    };
                    format!("[{status}] {} — {dir} — \"{prompt_preview}\"", a.id)
                })
                .collect();
            Ok(lines.join("\n"))
        }),
    });

    // get_agent_result
    let am = Arc::clone(&agent_manager);
    registry.register(ToolDef {
        name: "get_agent_result".into(),
        description: "Get the result of a dispatched agent by ID".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "The agent ID returned by dispatch_agent"}
            },
            "required": ["agent_id"]
        }),
        handler: Box::new(move |params| {
            let agent_id = params["agent_id"]
                .as_str()
                .ok_or_else(|| "agent_id is required".to_string())?;
            am.get_result_truncated(agent_id)
        }),
    });
}
