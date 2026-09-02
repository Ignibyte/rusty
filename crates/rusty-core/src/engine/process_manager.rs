//! Spawn and manage Claude Code CLI subprocesses.

use crate::brain::enrichment;
use crate::brain::BrainManager;
use crate::engine::agent_prompt_builder;
use crate::engine::mcp_config;
use crate::engine::memory_manager::{self, MemoryManager};
use crate::engine::output_parser;
use crate::engine::settings_manager::SettingsManager;
use crate::engine::task_manager::TaskManager;
use crate::engine::tool_registry::{self, ToolRegistry};
use crate::events::{AppEvent, EventBus};
use crate::skills::SkillsAccess;
use std::sync::Arc;
use tokio::process::Command;

/// Maximum number of tool-call round-trips per task.
const MAX_TOOL_ROUNDS: usize = 5;

/// Default Claude CLI path.
const DEFAULT_CLI_PATH: &str = "claude";

/// Task completion event payload sent to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskCompletedEvent {
    pub task_id: String,
    pub success: bool,
    pub result: String,
    pub error: String,
    pub session_id: String,
    pub cost_usd: f64,
}

/// Execute a Claude CLI task asynchronously.
///
/// Spawns the CLI as a subprocess, captures stdout (JSON result),
/// updates the task in SQLite, and emits a `task-completed` Tauri event.
#[allow(clippy::too_many_arguments)] // Central execution function — all params are required
pub async fn execute_task(
    events: EventBus,
    task_manager: Arc<TaskManager>,
    memory_manager: Arc<MemoryManager>,
    tool_registry: Arc<ToolRegistry>,
    brain_manager: Arc<BrainManager>,
    settings_manager: Arc<SettingsManager>,
    task_id: String,
    prompt: String,
    resume_session_id: Option<String>,
    max_turns: u32,
    max_budget_usd: f64,
) {
    // Transition to running
    if let Err(e) = task_manager.set_running(&task_id) {
        eprintln!("Failed to set task running: {e}");
        return;
    }

    let cli_path = std::env::var("RUSTY_CLI_PATH").unwrap_or_else(|_| DEFAULT_CLI_PATH.to_string());

    // Build system prompt (shared across initial and resume calls)
    let memory_context = memory_manager
        .build_system_prompt_context()
        .unwrap_or_default();
    let brain_context = brain_manager.build_context_for_prompt(&prompt);
    let tool_catalog = tool_registry.generate_catalog();
    let system_prompt =
        agent_prompt_builder::build_system_prompt(&memory_context, &brain_context, &tool_catalog);

    // Generate MCP config for tool access (filesystem, fetch)
    let mcp_config_path = mcp_config::generate_for_task(&task_id).ok();

    // Decide how this task's subprocess calls should treat Agent Skills.
    let skills_access = crate::skills::access_from_settings(&settings_manager);

    // --- Initial CLI call ---
    let initial_output = run_cli(
        &cli_path,
        &prompt,
        &system_prompt,
        mcp_config_path.as_deref(),
        resume_session_id.as_deref(),
        None,
        &skills_access,
        max_turns,
        max_budget_usd,
    )
    .await;

    let initial_output = match initial_output {
        Ok(out) => out,
        Err(error) => {
            let _ = task_manager.set_failed(&task_id, &error, "");
            events.emit(AppEvent::TaskCompleted(TaskCompletedEvent {
                task_id,
                success: false,
                result: String::new(),
                error,
                session_id: String::new(),
                cost_usd: 0.0,
            }));
            return;
        }
    };

    let mut parsed = output_parser::parse(&initial_output);
    let mut total_cost = parsed.cost_usd;

    // --- Tool call loop ---
    if parsed.success && !parsed.session_id.is_empty() {
        for round in 0..MAX_TOOL_ROUNDS {
            let (tool_calls, cleaned) = tool_registry::extract_tool_calls(&parsed.result);
            if tool_calls.is_empty() {
                parsed.result = cleaned;
                break;
            }

            // Execute tool calls
            let mut tool_results = Vec::new();
            for call in &tool_calls {
                let result = tool_registry.execute(&call.name, call.arguments.clone());
                match result {
                    Ok(res) => tool_results.push(format!("{}: {res}", call.name)),
                    Err(err) => tool_results.push(format!("{}: ERROR: {err}", call.name)),
                }
            }

            let follow_up = format!("Tool results:\n{}", tool_results.join("\n"));

            // Resume with tool results
            let resume_output = run_cli(
                &cli_path,
                &follow_up,
                &system_prompt,
                mcp_config_path.as_deref(),
                Some(&parsed.session_id),
                None,
                &skills_access,
                max_turns,
                max_budget_usd,
            )
            .await;

            match resume_output {
                Ok(stdout) => {
                    parsed = output_parser::parse(&stdout);
                    total_cost += parsed.cost_usd;
                }
                Err(e) => {
                    eprintln!("Tool resume round {round} failed: {e}");
                    parsed.result = cleaned;
                    break;
                }
            }

            if !parsed.success {
                break;
            }
        }
    }

    parsed.cost_usd = total_cost;
    let task_num_turns = parsed.num_turns;

    // Final cleanup: strip any remaining tool_call tags from the result
    // (e.g., if MAX_TOOL_ROUNDS was exhausted)
    if parsed.success {
        let (_, cleaned) = tool_registry::extract_tool_calls(&parsed.result);
        parsed.result = cleaned;
    }

    let event = if parsed.success {
        // Extract and store inline <memory> tags
        let (extracted_memories, cleaned_result) =
            memory_manager::extract_memory_tags(&parsed.result);
        for mem in &extracted_memories {
            let _ = memory_manager.store(&mem.category, &mem.importance, &mem.content, "inline");
        }

        let _ = task_manager.set_completed(
            &task_id,
            &cleaned_result,
            &parsed.session_id,
            parsed.cost_usd,
            parsed.num_turns,
            parsed.duration_ms,
        );

        TaskCompletedEvent {
            task_id: task_id.clone(),
            success: true,
            result: cleaned_result,
            error: String::new(),
            session_id: parsed.session_id,
            cost_usd: parsed.cost_usd,
        }
    } else {
        let _ = task_manager.set_failed(&task_id, &parsed.error, &parsed.session_id);

        TaskCompletedEvent {
            task_id: task_id.clone(),
            success: false,
            result: String::new(),
            error: parsed.error,
            session_id: parsed.session_id,
            cost_usd: parsed.cost_usd,
        }
    };

    // Cleanup MCP config temp file
    if mcp_config_path.is_some() {
        mcp_config::cleanup(&task_id);
    }

    // Brain enrichment — fire-and-forget, never blocks the event
    if event.success {
        let bm = Arc::clone(&brain_manager);
        let sm = Arc::clone(&settings_manager);
        let prompt_clone = prompt.clone();
        let result_clone = event.result.clone();
        tokio::spawn(async move {
            if let Err(e) =
                enrichment::enrich_from_conversation(bm, sm, &prompt_clone, &result_clone).await
            {
                eprintln!("Brain enrichment error: {e}");
            }
        });

        // Skill self-authoring — fire-and-forget; proposes a staged skill for substantial tasks.
        let sm_author = Arc::clone(&settings_manager);
        let prompt_author = prompt.clone();
        let result_author = event.result.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::skills::author::review_task_for_skill(
                sm_author,
                &prompt_author,
                &result_author,
                task_num_turns,
            )
            .await
            {
                eprintln!("Skill authoring error: {e}");
            }
        });
    }

    events.emit(AppEvent::TaskCompleted(event));
}

/// Spawn a single Claude CLI call and return stdout.
///
/// If `working_dir` is provided, the CLI subprocess runs in that directory.
#[allow(clippy::too_many_arguments)] // CLI invocation requires all these params
pub(crate) async fn run_cli(
    cli_path: &str,
    prompt: &str,
    system_prompt: &str,
    mcp_config_path: Option<&str>,
    resume_session_id: Option<&str>,
    working_dir: Option<&str>,
    skills: &SkillsAccess,
    max_turns: u32,
    max_budget_usd: f64,
) -> Result<String, String> {
    let mut cmd = Command::new(cli_path);
    cmd.arg("-p")
        .arg(prompt)
        .arg("--output-format")
        .arg("json")
        .arg("--dangerously-skip-permissions")
        .arg("--max-turns")
        .arg(max_turns.to_string())
        .arg("--max-budget-usd")
        .arg(format!("{max_budget_usd:.2}"));

    cmd.arg("--append-system-prompt").arg(system_prompt);

    if let Some(mcp_path) = mcp_config_path {
        cmd.arg("--mcp-config").arg(mcp_path);
    }

    if let Some(session_id) = resume_session_id {
        if !session_id.is_empty() {
            cmd.arg("--resume").arg(session_id);
        }
    }

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Expose (or suppress) Agent Skills for this call.
    match skills {
        SkillsAccess::Enabled(dir) => {
            cmd.arg("--add-dir").arg(dir);
        }
        SkillsAccess::Disabled => {
            cmd.arg("--disallowedTools").arg("Skill(*)");
        }
        SkillsAccess::Inherit => {}
    }

    // Critical: unset CLAUDECODE to prevent nested session detection
    cmd.env_remove("CLAUDECODE");
    cmd.env_remove("CLAUDE_CODE_ENTRY_POINT");

    // Ensure Claude CLI is in PATH
    if let Ok(home) = std::env::var("HOME") {
        let path = std::env::var("PATH").unwrap_or_default();
        if !path.contains(&format!("{home}/.local/bin")) {
            cmd.env("PATH", format!("{home}/.local/bin:{path}"));
        }
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to spawn Claude CLI: {e}. Is it installed at {cli_path}?"))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("Claude CLI stderr: {}", &stderr[..stderr.len().min(500)]);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
