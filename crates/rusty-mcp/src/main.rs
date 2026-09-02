//! `rusty-mcp`: Rusty's back end as a Model Context Protocol server.
//!
//! One process, built on [`rusty_core::Core`], serves the agents over stdio and the
//! desktop app over Streamable HTTP on localhost. Every tool is a thin wrapper around
//! a manager call; the managers own the rules. Nothing is written to stdout except the
//! protocol, so all diagnostics go to stderr.
//!
//! ```text
//! rusty-mcp                     stdio, for Claude Code and Codex `mcpServers` entries
//! rusty-mcp --http [ADDR]       Streamable HTTP at http://ADDR/mcp (default 127.0.0.1:4174)
//! ```

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::{
        stdio,
        streamable_http_server::{
            session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
        },
    },
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use rusty_core::Core;
use std::sync::Arc;

/// Serialize any manager result as a JSON text block, or map its error.
fn json_result<T: serde::Serialize>(value: Result<T, String>) -> Result<CallToolResult, McpError> {
    let value = value.map_err(|e| McpError::internal_error(e, None))?;
    let text = serde_json::to_string_pretty(&value)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// Parameters for `list_tasks`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ListTasksParams {
    /// The list (task group) id, from `list_task_groups`.
    pub group_id: i64,
    /// Include archived tasks.
    #[serde(default)]
    pub include_archived: bool,
}

/// Parameters for `create_task`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateTaskParams {
    /// The list (task group) id.
    pub group_id: i64,
    /// The task title.
    pub title: String,
}

/// A task id.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct TaskIdParams {
    /// The task id.
    pub id: i64,
}

/// A list (task group) name.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct GroupNameParams {
    /// The new list's name.
    pub name: String,
}

/// Parameters for `brain_search`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct BrainSearchParams {
    /// Full-text query. All terms must match; plain words work best.
    pub query: String,
    /// Maximum results (default 10).
    pub limit: Option<usize>,
    /// Restrict to a page type such as `project`, `concept`, `person`.
    pub page_type: Option<String>,
}

/// A brain page slug, folder included, such as `projects/rusty`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SlugParams {
    /// The page slug.
    pub slug: String,
}

/// Parameters for `brain_list_pages`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ListPagesParams {
    /// Restrict to a page type.
    pub page_type: Option<String>,
    /// Maximum results.
    pub limit: Option<usize>,
}

/// Parameters for `brain_create_page`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreatePageParams {
    /// Page type: `project`, `concept`, `person`, `company`, `idea`, `meeting`.
    pub page_type: String,
    /// Page title; the slug derives from it.
    pub title: String,
    /// Markdown body.
    pub content: String,
}

/// Parameters for `brain_add_timeline`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct AddTimelineParams {
    /// The page slug.
    pub slug: String,
    /// One-line summary of what happened.
    pub summary: String,
    /// Optional longer detail.
    pub detail: Option<String>,
    /// Date as YYYY-MM-DD; defaults to today.
    pub date: Option<String>,
}

/// Parameters for `store_memory`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct StoreMemoryParams {
    /// The memory text.
    pub content: String,
    /// Category such as `preference`, `fact`, `context` (default `fact`).
    pub category: Option<String>,
    /// Importance `low`, `medium`, `high` (default `medium`).
    pub importance: Option<String>,
}

/// Parameters for `list_memories`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ListMemoriesParams {
    /// Restrict to one category.
    pub category: Option<String>,
}

/// A note path relative to the notes folder, such as `Misc.md`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct NotePathParams {
    /// The relative path.
    pub path: String,
}

/// Parameters for `write_note`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct WriteNoteParams {
    /// The relative path.
    pub path: String,
    /// The full new content.
    pub content: String,
}

/// Parameters for `skill_view`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SkillNameParams {
    /// The skill's directory name.
    pub name: String,
}

/// Parameters for `skill_list`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SkillListParams {
    /// Include skills awaiting approval.
    #[serde(default)]
    pub include_pending: bool,
}

/// The MCP server: the tool router plus a shared [`Core`].
#[derive(Clone)]
pub struct Rusty {
    core: Arc<Core>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Rusty {
    /// Wrap a ready [`Core`].
    pub fn new(core: Arc<Core>) -> Self {
        Self {
            core,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "List the to-do lists (task groups) with their ids")]
    fn list_task_groups(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.user_task_manager.list_headers())
    }

    #[tool(description = "Create a to-do list; returns its id")]
    fn create_task_group(
        &self,
        Parameters(p): Parameters<GroupNameParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.user_task_manager.create_header(&p.name))
    }

    #[tool(description = "List the tasks in one to-do list")]
    fn list_tasks(
        &self,
        Parameters(p): Parameters<ListTasksParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(
            self.core
                .user_task_manager
                .list_tasks(p.group_id, p.include_archived),
        )
    }

    #[tool(description = "Add a task to a to-do list; returns its id")]
    fn create_task(
        &self,
        Parameters(p): Parameters<CreateTaskParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(
            self.core
                .user_task_manager
                .create_task(p.group_id, &p.title),
        )
    }

    #[tool(description = "Toggle a task between done and not done; returns the new state")]
    fn toggle_task(
        &self,
        Parameters(p): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.user_task_manager.toggle_complete(p.id))
    }

    #[tool(description = "Archive a task (hidden from the list, not deleted)")]
    fn archive_task(
        &self,
        Parameters(p): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(
            self.core
                .user_task_manager
                .archive_task(p.id)
                .map(|_| "archived"),
        )
    }

    #[tool(
        description = "Full-text search across the brain vault; results are slugs with snippets"
    )]
    fn brain_search(
        &self,
        Parameters(p): Parameters<BrainSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.search(
            &p.query,
            p.limit.or(Some(10)),
            p.page_type.as_deref(),
        ))
    }

    #[tool(description = "Read one brain page by slug (folder included, e.g. projects/rusty)")]
    fn brain_read_page(
        &self,
        Parameters(p): Parameters<SlugParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.read_page(&p.slug))
    }

    #[tool(description = "List brain pages, newest first, optionally by type")]
    fn brain_list_pages(
        &self,
        Parameters(p): Parameters<ListPagesParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(
            self.core
                .brain_manager
                .list_pages(p.page_type.as_deref(), p.limit),
        )
    }

    #[tool(description = "Create a brain page of a given type with a markdown body")]
    fn brain_create_page(
        &self,
        Parameters(p): Parameters<CreatePageParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(
            self.core
                .brain_manager
                .create_page(&p.page_type, &p.title, &p.content),
        )
    }

    #[tool(description = "Append a dated timeline entry to a brain page")]
    fn brain_add_timeline(
        &self,
        Parameters(p): Parameters<AddTimelineParams>,
    ) -> Result<CallToolResult, McpError> {
        let date = p.date.unwrap_or_else(chrono_today);
        json_result(self.core.brain_manager.add_timeline(
            &p.slug,
            &date,
            "mcp",
            &p.summary,
            p.detail.as_deref(),
        ))
    }

    #[tool(description = "Brain vault statistics: pages, links, tags, timeline entries")]
    fn brain_stats(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.stats())
    }

    #[tool(description = "List long-term memories, optionally by category")]
    fn list_memories(
        &self,
        Parameters(p): Parameters<ListMemoriesParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.memory_manager.list(p.category.as_deref()))
    }

    #[tool(description = "Store a long-term memory")]
    fn store_memory(
        &self,
        Parameters(p): Parameters<StoreMemoryParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.memory_manager.store(
            p.category.as_deref().unwrap_or("fact"),
            p.importance.as_deref().unwrap_or("medium"),
            &p.content,
            "mcp",
        ))
    }

    #[tool(description = "List the notes folder as a tree")]
    fn list_notes(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.notes_manager.list_tree())
    }

    #[tool(description = "Read a note by its relative path")]
    fn read_note(
        &self,
        Parameters(p): Parameters<NotePathParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.notes_manager.read_note(&p.path))
    }

    #[tool(description = "Replace a note's content")]
    fn write_note(
        &self,
        Parameters(p): Parameters<WriteNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(
            self.core
                .notes_manager
                .save_note(&p.path, &p.content)
                .map(|_| "saved"),
        )
    }

    #[tool(description = "List the skills in Rusty's store")]
    fn skill_list(
        &self,
        Parameters(p): Parameters<SkillListParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(Ok::<_, String>(
            self.core.skills_manager.list(p.include_pending),
        ))
    }

    #[tool(description = "Read one skill, frontmatter and body, by name")]
    fn skill_view(
        &self,
        Parameters(p): Parameters<SkillNameParams>,
    ) -> Result<CallToolResult, McpError> {
        match self.core.skills_manager.get(&p.name) {
            Some(skill) => json_result(Ok::<_, String>(skill)),
            None => Err(McpError::invalid_params(
                format!("no skill named {}", p.name),
                None,
            )),
        }
    }
}

/// Who we are, on top of whatever the build environment fills in.
fn server_identity() -> Implementation {
    let mut identity = Implementation::from_build_env();
    identity.name = "rusty-mcp".into();
    identity.version = env!("CARGO_PKG_VERSION").into();
    identity
}

/// Today's date as YYYY-MM-DD in local time.
fn chrono_today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

#[tool_handler]
impl ServerHandler for Rusty {
    fn get_info(&self) -> ServerInfo {
        let tool_count = self.tool_router.list_all().len();
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(server_identity())
            .with_instructions(format!(
                "Rusty is the user's local assistant store: to-do lists, notes, long-term \
                 memories, the brain vault (a markdown wiki with a full-text index) and \
                 skills, exposed as {tool_count} tools. Slugs include their folder \
                 (projects/name). Search is all-terms; use plain words."
            ))
    }
}

/// Default address for the HTTP transport: loopback only, the port v2 used.
const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:4174";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let core = Arc::new(Core::init());
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => {
            let service = Rusty::new(core).serve(stdio()).await?;
            service.waiting().await?;
        }
        Some("--http") => {
            let addr = args.next().unwrap_or_else(|| DEFAULT_HTTP_ADDR.to_string());
            let service = StreamableHttpService::new(
                move || Ok(Rusty::new(Arc::clone(&core))),
                LocalSessionManager::default().into(),
                StreamableHttpServerConfig::default(),
            );
            let router = axum::Router::new().nest_service("/mcp", service);
            let listener = tokio::net::TcpListener::bind(&addr).await?;
            eprintln!("rusty-mcp: Streamable HTTP at http://{addr}/mcp");
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await?;
        }
        Some(other) => {
            anyhow::bail!("unknown argument {other}: use no arguments for stdio, or --http [ADDR]")
        }
    }
    Ok(())
}
