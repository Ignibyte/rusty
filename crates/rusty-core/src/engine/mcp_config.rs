//! MCP server configuration generation for Claude CLI.
//!
//! Generates per-task temporary JSON config files that are passed to Claude CLI
//! via `--mcp-config`. This gives Claude access to external tool servers
//! (filesystem, fetch) without modifying global Claude config.

use std::path::PathBuf;

/// Temp file prefix for MCP configs.
const TEMP_PREFIX: &str = "/tmp/rusty-mcp-";

/// Generate an MCP config JSON file for a task.
///
/// Creates a temp file at `/tmp/rusty-mcp-{task_id}.json` with default
/// MCP server definitions. Returns the file path.
pub fn generate_for_task(task_id: &str) -> Result<String, String> {
    let config = serde_json::json!({
        "mcpServers": default_servers()
    });

    let json =
        serde_json::to_string_pretty(&config).map_err(|e| format!("JSON serialize error: {e}"))?;

    let path = format!("{TEMP_PREFIX}{task_id}.json");
    std::fs::write(&path, &json)
        .map_err(|e| format!("Failed to write MCP config to {path}: {e}"))?;

    Ok(path)
}

/// Clean up the MCP config file for a task.
///
/// Best-effort: silently ignores if file doesn't exist.
pub fn cleanup(task_id: &str) {
    let path = format!("{TEMP_PREFIX}{task_id}.json");
    let _ = std::fs::remove_file(&path);
}

/// Generate an MCP config JSON file for an agent, scoped to its target directory.
///
/// The filesystem server is restricted to `scoped_dir` instead of the home directory.
pub fn generate_for_agent(agent_id: &str, scoped_dir: &str) -> Result<String, String> {
    let config = serde_json::json!({
        "mcpServers": agent_servers(scoped_dir)
    });

    let json =
        serde_json::to_string_pretty(&config).map_err(|e| format!("JSON serialize error: {e}"))?;

    let path = format!("{TEMP_PREFIX}agent-{agent_id}.json");
    std::fs::write(&path, &json)
        .map_err(|e| format!("Failed to write MCP config to {path}: {e}"))?;

    Ok(path)
}

/// Clean up the MCP config file for an agent.
pub fn cleanup_agent(agent_id: &str) {
    let path = format!("{TEMP_PREFIX}agent-{agent_id}.json");
    let _ = std::fs::remove_file(&path);
}

/// MCP server definitions scoped to an agent's target directory.
fn agent_servers(scoped_dir: &str) -> serde_json::Value {
    serde_json::json!({
        "filesystem": {
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-filesystem", scoped_dir]
        },
        "fetch": {
            "command": "npx",
            "args": ["-y", "mcp-server-fetch"]
        }
    })
}

/// Default MCP server definitions.
///
/// - **filesystem**: Scoped to user's home directory. Read/write files.
/// - **fetch**: Retrieve URL content as text/markdown.
fn default_servers() -> serde_json::Value {
    let home = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .to_string_lossy()
        .to_string();

    serde_json::json!({
        "filesystem": {
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-filesystem", &home]
        },
        "fetch": {
            "command": "npx",
            "args": ["-y", "mcp-server-fetch"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_servers_has_filesystem_and_fetch() {
        let servers = default_servers();
        assert!(servers["filesystem"].is_object());
        assert!(servers["fetch"].is_object());
        assert_eq!(servers["filesystem"]["command"], "npx");
        assert_eq!(servers["fetch"]["command"], "npx");
    }

    #[test]
    fn generate_creates_file() {
        let task_id = "test-mcp-gen-001";
        let path = generate_for_task(task_id).unwrap();
        assert!(std::path::Path::new(&path).exists());

        // Verify it's valid JSON
        let content = std::fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed["mcpServers"]["filesystem"].is_object());

        // Cleanup
        cleanup(task_id);
        assert!(!std::path::Path::new(&path).exists());
    }

    #[test]
    fn cleanup_removes_file() {
        let task_id = "test-mcp-cleanup-001";
        let path = generate_for_task(task_id).unwrap();
        assert!(std::path::Path::new(&path).exists());

        cleanup(task_id);
        assert!(!std::path::Path::new(&path).exists());
    }

    #[test]
    fn cleanup_nonexistent_is_silent() {
        // Should not panic or error
        cleanup("nonexistent-task-id");
    }
}
