//! Dispatched agent lifecycle management.
//!
//! Manages Claude agents dispatched to work in local project directories.
//! Each agent runs as an independent Claude CLI subprocess with its own working
//! directory, MCP config, and budget. Results are stored in SQLite.

use crate::engine::db::Database;
use crate::engine::mcp_config;
use crate::engine::output_parser;
use crate::engine::process_manager;
use crate::events::{AppEvent, EventBus};
use crate::skills::SkillsAccess;
use std::sync::Arc;

/// Default Claude CLI path.
const DEFAULT_CLI_PATH: &str = "claude";

/// Agent system prompt — minimal, relies on project's CLAUDE.md.
const AGENT_SYSTEM_PROMPT: &str = "You are an AI agent dispatched by Rusty to work in this project directory. Complete the task below. You have full tool access to read, write, and execute in this directory.";

/// Maximum characters returned in tool call results.
const MAX_RESULT_CHARS: usize = 2000;

/// Agent execution status.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent created, not yet started.
    Pending,
    /// Agent CLI subprocess is running.
    Running,
    /// Agent completed successfully.
    Completed,
    /// Agent failed with an error.
    Failed,
}

impl AgentStatus {
    fn from_str(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }

    /// Get the string representation of the status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// A dispatched agent record.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    /// Unique agent ID.
    pub id: String,
    /// Parent conversation that dispatched this agent.
    pub conversation_id: String,
    /// Absolute path to the project directory.
    pub directory: String,
    /// The task prompt given to the agent.
    pub prompt: String,
    /// Current execution status.
    pub status: AgentStatus,
    /// Result text from the agent (may be truncated in tool responses).
    pub result: String,
    /// Error message if the agent failed.
    pub error: String,
    /// Total cost in USD.
    pub cost_usd: f64,
    /// Number of CLI turns used.
    pub num_turns: i64,
    /// Execution duration in milliseconds.
    pub duration_ms: i64,
    /// Claude CLI session ID.
    pub session_id: String,
    /// Unix timestamp when agent was created.
    pub created_at: i64,
    /// Unix timestamp when agent started running.
    pub started_at: i64,
    /// Unix timestamp when agent completed or failed.
    pub completed_at: i64,
}

/// Event payload emitted when an agent completes.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentCompletedEvent {
    /// Agent ID.
    pub agent_id: String,
    /// Whether the agent succeeded.
    pub success: bool,
    /// Target directory.
    pub directory: String,
}

/// Manages dispatched agent lifecycle in SQLite.
pub struct AgentManager {
    db: Arc<Database>,
}

impl AgentManager {
    /// Create a new AgentManager backed by the given database.
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// Create a new agent record in PENDING state. Returns the agent ID.
    pub fn create_agent(
        &self,
        conversation_id: &str,
        directory: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono_now();

        let conn = self.db.conn()?;
        conn.execute(
            "INSERT INTO agents (id, conversation_id, directory, prompt, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, conversation_id, directory, prompt, "pending", now],
        )
        .map_err(|e| format!("Failed to create agent: {e}"))?;

        Ok(id)
    }

    /// Transition an agent to RUNNING state.
    pub fn set_running(&self, agent_id: &str) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE agents SET status = 'running', started_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono_now(), agent_id],
        )
        .map_err(|e| format!("Failed to update agent: {e}"))?;
        Ok(())
    }

    /// Mark an agent as COMPLETED with result data.
    pub fn set_completed(
        &self,
        agent_id: &str,
        result: &str,
        session_id: &str,
        cost_usd: f64,
        num_turns: i64,
        duration_ms: i64,
    ) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE agents SET status = 'completed', result = ?1, session_id = ?2, cost_usd = ?3, num_turns = ?4, duration_ms = ?5, completed_at = ?6 WHERE id = ?7",
            rusqlite::params![result, session_id, cost_usd, num_turns, duration_ms, chrono_now(), agent_id],
        )
        .map_err(|e| format!("Failed to complete agent: {e}"))?;
        Ok(())
    }

    /// Mark an agent as FAILED with an error message.
    pub fn set_failed(&self, agent_id: &str, error: &str) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE agents SET status = 'failed', error = ?1, completed_at = ?2 WHERE id = ?3",
            rusqlite::params![error, chrono_now(), agent_id],
        )
        .map_err(|e| format!("Failed to fail agent: {e}"))?;
        Ok(())
    }

    /// Get an agent by ID.
    pub fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>, String> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, conversation_id, directory, prompt, status, result, error, cost_usd, num_turns, duration_ms, session_id, created_at, started_at, completed_at FROM agents WHERE id = ?1",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let agent = stmt
            .query_row(rusqlite::params![agent_id], row_to_agent)
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;

        Ok(agent)
    }

    /// List agents ordered by creation time (most recent first).
    pub fn list_agents(&self, limit: usize) -> Result<Vec<Agent>, String> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, conversation_id, directory, prompt, status, result, error, cost_usd, num_turns, duration_ms, session_id, created_at, started_at, completed_at FROM agents ORDER BY created_at DESC LIMIT ?1",
            )
            .map_err(|e| format!("Query error: {e}"))?;

        let agents = stmt
            .query_map(rusqlite::params![limit as i64], row_to_agent)
            .map_err(|e| format!("Query error: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {e}"))?;

        Ok(agents)
    }

    /// Get a truncated result suitable for tool call responses.
    pub fn get_result_truncated(&self, agent_id: &str) -> Result<String, String> {
        let agent = self
            .get_agent(agent_id)?
            .ok_or_else(|| format!("Agent '{agent_id}' not found"))?;

        match agent.status {
            AgentStatus::Pending => Ok(format!("Agent {} is pending (not yet started).", agent.id)),
            AgentStatus::Running => Ok(format!(
                "Agent {} is still running in {}.",
                agent.id, agent.directory
            )),
            AgentStatus::Failed => Ok(format!("Agent {} failed: {}", agent.id, agent.error)),
            AgentStatus::Completed => {
                if agent.result.len() <= MAX_RESULT_CHARS {
                    Ok(agent.result)
                } else {
                    let truncated =
                        &agent.result[..agent.result.floor_char_boundary(MAX_RESULT_CHARS)];
                    Ok(format!(
                        "{truncated}\n\n[Result truncated — full result available in the Agents view]"
                    ))
                }
            }
        }
    }
}

/// Execute a dispatched agent asynchronously.
///
/// Spawns Claude CLI in the target directory, parses output,
/// updates agent status in SQLite, and emits `agent-completed` event.
#[allow(clippy::too_many_arguments)] // Agent execution requires all context params
pub async fn execute_agent(
    events: EventBus,
    agent_manager: Arc<AgentManager>,
    brain_manager: Arc<crate::brain::BrainManager>,
    agent_id: String,
    directory: String,
    prompt: String,
    skills: SkillsAccess,
    max_turns: u32,
    max_budget_usd: f64,
) {
    // Transition to running
    if let Err(e) = agent_manager.set_running(&agent_id) {
        eprintln!("Failed to set agent running: {e}");
        return;
    }

    let cli_path = std::env::var("RUSTY_CLI_PATH").unwrap_or_else(|_| DEFAULT_CLI_PATH.to_string());

    // Build system prompt with brain context
    let brain_context = brain_manager.build_context_for_prompt(&prompt);
    let system_prompt = if brain_context.is_empty() {
        AGENT_SYSTEM_PROMPT.to_string()
    } else {
        format!("{AGENT_SYSTEM_PROMPT}\n\n{brain_context}")
    };

    // Detect remote SSH paths (user@host:/path)
    let is_remote = directory.contains(':') && directory.contains('@');

    let output = if is_remote {
        // Remote execution via SSH
        run_agent_remote(
            &directory,
            &cli_path,
            &prompt,
            &system_prompt,
            &skills,
            max_turns,
            max_budget_usd,
        )
        .await
    } else {
        // Local execution
        let mcp_config_path = mcp_config::generate_for_agent(&agent_id, &directory).ok();
        let result = process_manager::run_cli(
            &cli_path,
            &prompt,
            &system_prompt,
            mcp_config_path.as_deref(),
            None,
            Some(&directory),
            &skills,
            max_turns,
            max_budget_usd,
        )
        .await;
        mcp_config::cleanup_agent(&agent_id);
        result
    };

    let (success, dir) = match output {
        Ok(stdout) => {
            let parsed = output_parser::parse(&stdout);
            if parsed.success {
                let _ = agent_manager.set_completed(
                    &agent_id,
                    &parsed.result,
                    &parsed.session_id,
                    parsed.cost_usd,
                    parsed.num_turns,
                    parsed.duration_ms,
                );
                (true, directory.clone())
            } else {
                let _ = agent_manager.set_failed(&agent_id, &parsed.error);
                (false, directory.clone())
            }
        }
        Err(error) => {
            let _ = agent_manager.set_failed(&agent_id, &error);
            (false, directory.clone())
        }
    };

    // Emit event for any connected frontend.
    events.emit(AppEvent::AgentCompleted(AgentCompletedEvent {
        agent_id,
        success,
        directory: dir,
    }));
}

/// Expand `~` prefix to the user's home directory.
/// Run an agent on a remote server via SSH.
///
/// Parses `user@host:/path` format, SSHs in, and runs Claude CLI remotely.
/// Requires SSH key auth and Claude CLI installed on the remote machine.
async fn run_agent_remote(
    directory: &str,
    cli_path: &str,
    prompt: &str,
    system_prompt: &str,
    skills: &SkillsAccess,
    max_turns: u32,
    max_budget_usd: f64,
) -> Result<String, String> {
    // Parse user@host:/path
    let (ssh_target, remote_path) = directory
        .split_once(':')
        .ok_or_else(|| format!("Invalid remote path: {directory}"))?;

    // Build the remote claude command
    let remote_cli =
        std::env::var("RUSTY_REMOTE_CLI_PATH").unwrap_or_else(|_| cli_path.to_string());
    let escaped_prompt = prompt.replace('\'', "'\\''");
    let escaped_system = system_prompt.replace('\'', "'\\''");

    // Honor the master kill switch remotely; otherwise inherit the remote's own skills.
    let skills_flag = match skills {
        SkillsAccess::Disabled => " --disallowedTools 'Skill(*)'",
        _ => "",
    };

    let remote_cmd = format!(
        "cd '{}' && '{}' -p '{}' --output-format json --dangerously-skip-permissions \
         --max-turns {} --max-budget-usd {:.2} --append-system-prompt '{}'{}",
        remote_path,
        remote_cli,
        escaped_prompt,
        max_turns,
        max_budget_usd,
        escaped_system,
        skills_flag
    );

    let output = tokio::process::Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            ssh_target,
            &remote_cmd,
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("SSH failed: {e}. Is SSH key auth configured for {ssh_target}?"))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("Remote agent stderr: {}", &stderr[..stderr.len().min(500)]);
    }

    if !output.status.success() {
        return Err(format!("SSH command failed: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().to_string();
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home.to_string_lossy().to_string();
        }
    }
    path.to_string()
}

/// Row mapper for SQLite query results.
fn row_to_agent(row: &rusqlite::Row) -> rusqlite::Result<Agent> {
    Ok(Agent {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        directory: row.get(2)?,
        prompt: row.get(3)?,
        status: AgentStatus::from_str(&row.get::<_, String>(4)?),
        result: row.get(5)?,
        error: row.get(6)?,
        cost_usd: row.get(7)?,
        num_turns: row.get(8)?,
        duration_ms: row.get(9)?,
        session_id: row.get(10)?,
        created_at: row.get(11)?,
        started_at: row.get(12)?,
        completed_at: row.get(13)?,
    })
}

/// Extension trait for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Current Unix timestamp in seconds.
fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::db::Database;
    use rusqlite::Connection;

    fn test_db() -> Arc<Database> {
        let conn = Connection::open_in_memory().unwrap();
        let db = Database::from_conn(conn);
        db.migrate().unwrap();
        Arc::new(db)
    }

    #[test]
    fn create_and_get_agent() {
        let db = test_db();
        let am = AgentManager::new(db);

        let id = am
            .create_agent("conv-1", "/tmp/test-project", "Fix the bug")
            .unwrap();
        let agent = am.get_agent(&id).unwrap().unwrap();

        assert_eq!(agent.directory, "/tmp/test-project");
        assert_eq!(agent.prompt, "Fix the bug");
        assert_eq!(agent.status, AgentStatus::Pending);
        assert_eq!(agent.conversation_id, "conv-1");
    }

    #[test]
    fn agent_lifecycle_completed() {
        let db = test_db();
        let am = AgentManager::new(db);

        let id = am.create_agent("", "/tmp/proj", "do stuff").unwrap();
        am.set_running(&id).unwrap();

        let agent = am.get_agent(&id).unwrap().unwrap();
        assert_eq!(agent.status, AgentStatus::Running);
        assert!(agent.started_at > 0);

        am.set_completed(&id, "All done", "sess-1", 0.05, 3, 2000)
            .unwrap();

        let agent = am.get_agent(&id).unwrap().unwrap();
        assert_eq!(agent.status, AgentStatus::Completed);
        assert_eq!(agent.result, "All done");
        assert_eq!(agent.session_id, "sess-1");
        assert!(agent.completed_at > 0);
    }

    #[test]
    fn agent_lifecycle_failed() {
        let db = test_db();
        let am = AgentManager::new(db);

        let id = am.create_agent("", "/tmp/proj", "do stuff").unwrap();
        am.set_running(&id).unwrap();
        am.set_failed(&id, "CLI not found").unwrap();

        let agent = am.get_agent(&id).unwrap().unwrap();
        assert_eq!(agent.status, AgentStatus::Failed);
        assert_eq!(agent.error, "CLI not found");
    }

    #[test]
    fn list_agents_ordered() {
        let db = test_db();
        let am = AgentManager::new(db);

        am.create_agent("", "/tmp/a", "first").unwrap();
        am.create_agent("", "/tmp/b", "second").unwrap();

        let agents = am.list_agents(10).unwrap();
        assert_eq!(agents.len(), 2);
        // Most recent first
        assert_eq!(agents[0].prompt, "second");
        assert_eq!(agents[1].prompt, "first");
    }

    #[test]
    fn list_agents_empty() {
        let db = test_db();
        let am = AgentManager::new(db);
        let agents = am.list_agents(10).unwrap();
        assert!(agents.is_empty());
    }

    #[test]
    fn get_result_truncated_short() {
        let db = test_db();
        let am = AgentManager::new(db);
        let id = am.create_agent("", "/tmp", "test").unwrap();
        am.set_completed(&id, "Short result", "s1", 0.01, 1, 100)
            .unwrap();

        let result = am.get_result_truncated(&id).unwrap();
        assert_eq!(result, "Short result");
    }

    #[test]
    fn get_result_truncated_long() {
        let db = test_db();
        let am = AgentManager::new(db);
        let id = am.create_agent("", "/tmp", "test").unwrap();
        let long_result = "x".repeat(3000);
        am.set_completed(&id, &long_result, "s1", 0.01, 1, 100)
            .unwrap();

        let result = am.get_result_truncated(&id).unwrap();
        assert!(result.contains("[Result truncated"));
        assert!(result.len() < 3000);
    }

    #[test]
    fn get_result_running_agent() {
        let db = test_db();
        let am = AgentManager::new(db);
        let id = am.create_agent("", "/tmp", "test").unwrap();
        am.set_running(&id).unwrap();

        let result = am.get_result_truncated(&id).unwrap();
        assert!(result.contains("still running"));
    }

    #[test]
    fn expand_tilde_works() {
        let expanded = expand_tilde("~/Projects/test");
        assert!(!expanded.starts_with('~'));
        assert!(expanded.contains("Projects/test"));

        // Non-tilde paths pass through
        assert_eq!(expand_tilde("/usr/local"), "/usr/local");
    }

    #[test]
    fn agent_status_round_trip() {
        assert_eq!(AgentStatus::from_str("pending"), AgentStatus::Pending);
        assert_eq!(AgentStatus::from_str("running"), AgentStatus::Running);
        assert_eq!(AgentStatus::from_str("completed"), AgentStatus::Completed);
        assert_eq!(AgentStatus::from_str("failed"), AgentStatus::Failed);
        assert_eq!(AgentStatus::Pending.as_str(), "pending");
        assert_eq!(AgentStatus::Completed.as_str(), "completed");
    }
}
