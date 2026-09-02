//! Desktop automation tools: clipboard, open URL, and shell commands.

use crate::engine::tool_registry::{ToolDef, ToolRegistry};
use std::process::Command;
use std::time::Duration;

/// Maximum output length for shell commands (chars).
const MAX_OUTPUT_CHARS: usize = 4000;

/// Default timeout for shell commands.
const COMMAND_TIMEOUT_SECS: u64 = 10;

/// Register desktop automation tools with the registry.
pub fn register(registry: &ToolRegistry) {
    // clipboard_read
    registry.register(ToolDef {
        name: "clipboard_read".into(),
        description: "Read the current clipboard contents".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        handler: Box::new(|_| {
            let output = Command::new("pbpaste")
                .output()
                .map_err(|e| format!("Failed to read clipboard: {e}"))?;

            if !output.status.success() {
                return Err("Failed to read clipboard (pbpaste error)".to_string());
            }

            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if text.is_empty() {
                Ok("Clipboard is empty.".to_string())
            } else {
                Ok(truncate_output(&text))
            }
        }),
    });

    // clipboard_write
    registry.register(ToolDef {
        name: "clipboard_write".into(),
        description: "Write text to the clipboard".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "Text to copy to clipboard"}
            },
            "required": ["text"]
        }),
        handler: Box::new(|params| {
            let text = params["text"]
                .as_str()
                .ok_or_else(|| "text is required".to_string())?;

            let mut child = Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to write clipboard: {e}"))?;

            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin
                    .write_all(text.as_bytes())
                    .map_err(|e| format!("Failed to write to pbcopy stdin: {e}"))?;
            }

            child.wait().map_err(|e| format!("pbcopy failed: {e}"))?;

            Ok(format!("Copied {} characters to clipboard.", text.len()))
        }),
    });

    // open_url
    registry.register(ToolDef {
        name: "open_url".into(),
        description: "Open a URL in the default browser".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "URL to open (must start with http:// or https://)"}
            },
            "required": ["url"]
        }),
        handler: Box::new(|params| {
            let url = params["url"]
                .as_str()
                .ok_or_else(|| "url is required".to_string())?;

            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err(format!(
                    "Invalid URL: must start with http:// or https://. Got: {url}"
                ));
            }

            // Use macOS `open` command — works without Tauri AppHandle
            Command::new("open")
                .arg(url)
                .output()
                .map_err(|e| format!("Failed to open URL: {e}"))?;

            Ok(format!("Opened {url} in default browser."))
        }),
    });

    // run_command
    registry.register(ToolDef {
        name: "run_command".into(),
        description: "Run a shell command and return its output. Use for quick system queries (e.g., date, IP address, disk space). Has a 10-second timeout.".into(),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "command": {"type": "string", "description": "Shell command to execute (e.g., 'date', 'whoami', 'df -h')"}
            },
            "required": ["command"]
        }),
        handler: Box::new(|params| {
            let command = params["command"]
                .as_str()
                .ok_or_else(|| "command is required".to_string())?;

            let output = run_with_timeout(command, COMMAND_TIMEOUT_SECS)?;
            Ok(truncate_output(&output))
        }),
    });
}

/// Run a shell command with a timeout. Returns stdout + stderr combined.
fn run_with_timeout(command: &str, timeout_secs: u64) -> Result<String, String> {
    let cmd = command.to_string();
    let (tx, rx) = std::sync::mpsc::channel();

    // Run in a thread so we can enforce a timeout
    std::thread::spawn(move || {
        let result = Command::new("sh").arg("-c").arg(&cmd).output();
        let _ = tx.send(result);
    });

    let timeout = Duration::from_secs(timeout_secs);
    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let mut result = stdout;
            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&stderr);
            }

            if !output.status.success() {
                result = format!("Command exited with code {}.\n{}", output.status, result);
            }

            Ok(result)
        }
        Ok(Err(e)) => Err(format!("Failed to run command: {e}")),
        Err(_) => Err(format!(
            "Command timed out after {timeout_secs} seconds: {command}"
        )),
    }
}

/// Truncate output to a maximum length.
fn truncate_output(s: &str) -> String {
    if s.len() <= MAX_OUTPUT_CHARS {
        s.to_string()
    } else {
        let truncated = &s[..s.floor_char_boundary(MAX_OUTPUT_CHARS.saturating_sub(30))];
        format!("{truncated}\n\n[Output truncated at {MAX_OUTPUT_CHARS} chars]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_output_short() {
        let short = "hello world";
        assert_eq!(truncate_output(short), "hello world");
    }

    #[test]
    fn truncate_output_long() {
        let long = "x".repeat(5000);
        let result = truncate_output(&long);
        assert!(result.len() < 5000);
        assert!(result.contains("[Output truncated"));
    }

    #[test]
    fn run_with_timeout_succeeds() {
        let result = run_with_timeout("echo hello", 5).unwrap();
        assert_eq!(result.trim(), "hello");
    }

    #[test]
    fn run_with_timeout_captures_stderr() {
        let result = run_with_timeout("echo err >&2", 5).unwrap();
        assert!(result.contains("err"));
    }

    #[test]
    fn run_with_timeout_times_out() {
        let result = run_with_timeout("sleep 30", 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("timed out"));
    }
}
