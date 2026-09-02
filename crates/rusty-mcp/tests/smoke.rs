//! Drives the built `rusty-mcp` binary over stdio with the rmcp client, in a scratch
//! HOME, the way Claude Code or Codex would. Runs in CI.

use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::transport::TokioChildProcess;
use rmcp::ServiceExt;
use tokio::process::Command;

fn scratch_home() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("rusty-mcp-smoke-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(dir.join("run")).unwrap();
    dir
}

fn text_of(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect()
}

fn args(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    value.as_object().cloned().expect("a JSON object")
}

#[tokio::test]
async fn a_real_client_can_list_call_and_read() {
    let home = scratch_home();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rusty-mcp"));
    cmd.env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_RUNTIME_DIR", home.join("run"))
        .env("RUSTY_OBSIDIAN_CLI", "rusty-no-obsidian-here");
    let client =
        ().serve(TokioChildProcess::new(cmd).expect("spawn rusty-mcp"))
            .await
            .expect("initialize");

    let info = client.peer_info().expect("server info");
    let server_name = info
        .server_info
        .as_ref()
        .map(|i| i.name.to_string())
        .unwrap_or_default();
    assert!(server_name.contains("rusty"), "{server_name}");

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.len() >= 57, "{} tools", tools.len());
    for name in [
        "list_task_groups",
        "brain_capture",
        "obsidian_status",
        "settings_list",
    ] {
        assert!(tools.iter().any(|t| t.name == name), "missing {name}");
    }

    let created = client
        .call_tool(
            CallToolRequestParams::new("create_task_group")
                .with_arguments(args(serde_json::json!({"name": "Smoke"}))),
        )
        .await
        .unwrap();
    assert!(!created.is_error.unwrap_or(false), "{}", text_of(&created));
    let groups = client
        .call_tool(CallToolRequestParams::new("list_task_groups"))
        .await
        .unwrap();
    assert!(text_of(&groups).contains("Smoke"), "{}", text_of(&groups));

    // Without Obsidian the bridge reports itself rather than failing the call.
    let status = client
        .call_tool(CallToolRequestParams::new("obsidian_status"))
        .await
        .unwrap();
    let status_text = text_of(&status);
    assert!(
        status_text.contains("\"installed\": false"),
        "{status_text}"
    );
    assert!(status_text.contains("\"running\": false"), "{status_text}");

    let resources = client.list_all_resources().await.unwrap();
    assert!(resources.iter().any(|r| r.uri == "rusty://tasks"));

    client.cancel().await.unwrap();
    let _ = std::fs::remove_dir_all(&home);
}
