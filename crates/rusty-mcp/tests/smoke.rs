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
    assert!(tools.len() >= 70, "{} tools", tools.len());
    for name in [
        "list_task_groups",
        "brain_capture",
        "obsidian_status",
        "settings_list",
        "brain_tree",
        "brain_render",
        "brain_rename",
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

    // The workspace path: a page in a folder, rendered, edited whole, moved with its
    // links rewritten, and the tree that shows it.
    let created = client
        .call_tool(
            CallToolRequestParams::new("brain_new_page").with_arguments(args(
                serde_json::json!({"folder": "projects", "name": "Smoke plan"}),
            )),
        )
        .await
        .unwrap();
    assert_eq!(
        text_of(&created),
        "\"projects/Smoke plan\"",
        "{}",
        text_of(&created)
    );
    let linked = client
        .call_tool(
            CallToolRequestParams::new("brain_new_page").with_arguments(args(
                serde_json::json!({"folder": "ideas", "name": "Linker"}),
            )),
        )
        .await
        .unwrap();
    assert!(!linked.is_error.unwrap_or(false), "{}", text_of(&linked));
    let written = client
        .call_tool(
            CallToolRequestParams::new("brain_write_page").with_arguments(args(serde_json::json!({
                "slug": "ideas/Linker",
                "content": "---\ntitle: Linker\ntype: idea\n---\n\nSee [[projects/Smoke plan|the plan]].\n\n> [!tip] Hint\n> - [ ] a task\n"
            }))),
        )
        .await
        .unwrap();
    assert!(!written.is_error.unwrap_or(false), "{}", text_of(&written));
    let rendered = client
        .call_tool(
            CallToolRequestParams::new("brain_render").with_arguments(args(serde_json::json!({
                "slug": "ideas/Linker",
                "style": {"accent": "#123456"}
            }))),
        )
        .await
        .unwrap();
    let rendered_text = text_of(&rendered);
    assert!(
        rendered_text.contains("rusty:page/projects/Smoke plan"),
        "{rendered_text}"
    );
    assert!(rendered_text.contains("Hint"), "{rendered_text}");
    assert!(rendered_text.contains("\"tasks\": 1"), "{rendered_text}");
    let moved = client
        .call_tool(
            CallToolRequestParams::new("brain_rename").with_arguments(args(serde_json::json!({
                "from": "projects/Smoke plan",
                "to": "concepts/Moved plan"
            }))),
        )
        .await
        .unwrap();
    let moved_text = text_of(&moved);
    assert!(
        moved_text.contains("\"pages_rewritten\": 1"),
        "{moved_text}"
    );
    let after = client
        .call_tool(
            CallToolRequestParams::new("brain_read_page")
                .with_arguments(args(serde_json::json!({"slug": "ideas/Linker"}))),
        )
        .await
        .unwrap();
    assert!(
        text_of(&after).contains("[[concepts/Moved plan|the plan]]"),
        "{}",
        text_of(&after)
    );
    let tree = client
        .call_tool(CallToolRequestParams::new("brain_tree"))
        .await
        .unwrap();
    let tree_text = text_of(&tree);
    assert!(
        tree_text.contains("\"path\": \"concepts/Moved plan\""),
        "{tree_text}"
    );
    let unresolved = client
        .call_tool(CallToolRequestParams::new("brain_unresolved"))
        .await
        .unwrap();
    assert!(
        text_of(&unresolved).trim() == "[]",
        "{}",
        text_of(&unresolved)
    );

    // Tags and properties: an inline tag reaches the tag list and the tag: search; a
    // property lands in the frontmatter with the body untouched, and leaves again.
    let tagged = client
        .call_tool(
            CallToolRequestParams::new("brain_write_page").with_arguments(args(serde_json::json!({
                "slug": "concepts/Moved plan",
                "content": "---\ntitle: Moved plan\ntype: concept\n---\n\nA plan with #smoke/test inside.\n"
            }))),
        )
        .await
        .unwrap();
    assert!(!tagged.is_error.unwrap_or(false), "{}", text_of(&tagged));
    let tags = client
        .call_tool(CallToolRequestParams::new("brain_tags"))
        .await
        .unwrap();
    let tags_text = text_of(&tags);
    assert!(
        tags_text.contains("\"tag\": \"smoke\"") && tags_text.contains("\"tag\": \"smoke/test\""),
        "{tags_text}"
    );
    let by_tag = client
        .call_tool(
            CallToolRequestParams::new("brain_search")
                .with_arguments(args(serde_json::json!({"query": "tag:smoke"}))),
        )
        .await
        .unwrap();
    assert!(
        text_of(&by_tag).contains("concepts/Moved plan"),
        "{}",
        text_of(&by_tag)
    );
    let set = client
        .call_tool(
            CallToolRequestParams::new("brain_set_property").with_arguments(args(
                serde_json::json!({
                    "slug": "concepts/Moved plan", "key": "status", "value": "active"
                }),
            )),
        )
        .await
        .unwrap();
    assert!(
        text_of(&set).contains("\"status\": \"active\""),
        "{}",
        text_of(&set)
    );
    let after = client
        .call_tool(
            CallToolRequestParams::new("brain_read_page")
                .with_arguments(args(serde_json::json!({"slug": "concepts/Moved plan"}))),
        )
        .await
        .unwrap();
    assert!(
        text_of(&after).contains("A plan with #smoke/test inside."),
        "{}",
        text_of(&after)
    );
    let removed = client
        .call_tool(
            CallToolRequestParams::new("brain_remove_property").with_arguments(args(
                serde_json::json!({"slug": "concepts/Moved plan", "key": "status"}),
            )),
        )
        .await
        .unwrap();
    assert!(
        !text_of(&removed).contains("\"status\""),
        "{}",
        text_of(&removed)
    );

    client.cancel().await.unwrap();
    let _ = std::fs::remove_dir_all(&home);
}
