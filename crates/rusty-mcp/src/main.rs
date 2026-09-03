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
    schemars,
    service::{NotificationContext, Peer, RequestContext},
    tool, tool_handler, tool_router,
    transport::{
        stdio,
        streamable_http_server::{
            session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
        },
    },
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use rusty_core::events::AppEvent;
use rusty_core::obsidian::Obsidian;
use rusty_core::Core;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Every connected client, so a change can be announced to all of them.
type Peers = Arc<Mutex<Vec<Peer<RoleServer>>>>;

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
    /// Keep only pages whose text holds the words as typed, case included (a text search).
    pub case_sensitive: Option<bool>,
    /// Treat the words as a regular expression over the page text (a text search).
    pub regex: Option<bool>,
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

/// Parameters for `rename_task_group`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct RenameGroupParams {
    /// The list (task group) id.
    pub group_id: i64,
    /// The new name.
    pub name: String,
}

/// A list (task group) id.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct GroupIdParams {
    /// The list (task group) id.
    pub group_id: i64,
}

/// Parameters for `update_task_title`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateTaskTitleParams {
    /// The task id.
    pub id: i64,
    /// The new title.
    pub title: String,
}

/// Parameters for `create_note`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateNoteParams {
    /// Folder to create in, relative to the notes root; empty for the root.
    #[serde(default)]
    pub parent: String,
    /// File name (with `.md`) or folder name.
    pub name: String,
    /// Create a folder instead of a note.
    #[serde(default)]
    pub is_folder: bool,
}

/// Parameters for `rename_note`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct RenameNoteParams {
    /// The current relative path.
    pub path: String,
    /// The new name within the same folder.
    pub new_name: String,
}

/// A memory id.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct MemoryIdParams {
    /// The memory id, from `list_memories`.
    pub id: String,
}

/// Parameters for `brain_update_page`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct UpdatePageParams {
    /// The page slug.
    pub slug: String,
    /// The full new markdown body (frontmatter title is kept).
    pub content: String,
}

/// Parameters for `brain_get_timeline`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct TimelineParams {
    /// The page slug.
    pub slug: String,
    /// Maximum entries, newest first.
    pub limit: Option<usize>,
}

/// A partial slug or title to resolve.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ResolveSlugParams {
    /// A slug fragment or title words.
    pub partial: String,
}

/// Parameters for `search_conversations`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SearchConversationsParams {
    /// Words to look for in past prompts and results.
    pub query: String,
    /// Maximum results (default 10).
    pub limit: Option<usize>,
}

/// Parameters for `skill_create`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateSkillParams {
    /// Directory name, lowercase with dashes; it is the invocation name.
    pub name: String,
    /// One-line description; Claude uses it to decide when the skill applies.
    pub description: String,
    /// The SKILL.md body (markdown, no frontmatter).
    pub body: String,
    /// Stage it for approval instead of activating it directly.
    #[serde(default)]
    pub pending: bool,
    /// Overwrite an existing active skill of the same name.
    #[serde(default)]
    pub force: bool,
}

/// Parameters for `skill_approve`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ApproveSkillParams {
    /// The staged skill's name.
    pub name: String,
    /// Approve even if the safety scan reports findings.
    #[serde(default)]
    pub force: bool,
}

/// A secret's key.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SecretKeyParams {
    /// The vault key, such as `OPENAI_API_KEY`.
    pub key: String,
}

/// Parameters for `secret_set`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SecretSetParams {
    /// The vault key.
    pub key: String,
    /// The value; it is written to the vault and never echoed back.
    pub value: String,
}

/// A settings key.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SettingKeyParams {
    /// The setting key, such as `brain_vault_path`.
    pub key: String,
}

/// Parameters for `setting_set`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SettingSetParams {
    /// The setting key.
    pub key: String,
    /// The value, stored as a string.
    pub value: String,
}

/// Parameters for `reorder_tasks`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ReorderTasksParams {
    /// The list (task group) id.
    pub group_id: i64,
    /// Task ids in the wanted order; tasks left out follow in their current order.
    pub task_ids: Vec<i64>,
}

/// Parameters for `update_memory`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateMemoryParams {
    /// The memory id, from `list_memories`.
    pub id: String,
    /// New content; omit to keep.
    #[serde(default)]
    pub content: Option<String>,
    /// New category; omit to keep.
    #[serde(default)]
    pub category: Option<String>,
    /// New importance (`low`, `normal`, `high`); omit to keep.
    #[serde(default)]
    pub importance: Option<String>,
}

/// Parameters for `brain_daily_note`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct DailyNoteParams {
    /// `YYYY-MM-DD`; omit for today.
    #[serde(default)]
    pub date: Option<String>,
}

/// Parameters for `brain_capture`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CaptureParams {
    /// The note to append.
    pub text: String,
    /// `daily` (default) or `inbox`.
    #[serde(default)]
    pub target: Option<String>,
    /// `YYYY-MM-DD` for the daily page and the entry date; omit for today.
    #[serde(default)]
    pub date: Option<String>,
}

/// Parameters for `skill_update`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SkillUpdateParams {
    /// The skill's directory name.
    pub name: String,
    /// New description; omit to keep.
    #[serde(default)]
    pub description: Option<String>,
    /// New markdown body; omit to keep.
    #[serde(default)]
    pub body: Option<String>,
}

/// One row of `settings_list`.
#[derive(serde::Serialize)]
pub struct SettingEntry {
    /// The setting key.
    pub key: String,
    /// The value, masked when the key looks like a credential.
    pub value: String,
}

/// What `skill_update` returns.
#[derive(serde::Serialize)]
pub struct SkillUpdateResult {
    /// The skill as re-read from disk.
    pub skill: rusty_core::skills::Skill,
    /// Safety-scan findings on the new file; empty when clean.
    pub findings: Vec<String>,
}

/// Whether a settings key names something that should not be echoed back.
fn looks_secret(key: &str) -> bool {
    let k = key.to_ascii_lowercase();
    ["key", "token", "secret", "password", "passwd"]
        .iter()
        .any(|needle| k.contains(needle))
}

/// Parameters for `brain_reembed`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ReembedParams {
    /// Embed every page again, not only the stale ones.
    #[serde(default)]
    pub force: bool,
}

/// What `brain_semantic_status` returns.
#[derive(serde::Serialize)]
pub struct SemanticStatus {
    /// `provider:model` in use, or `None` for full-text only.
    pub provider: Option<String>,
    /// Pages and chunks with vectors.
    pub stats: rusty_core::brain::semantic::SemanticStats,
    /// Pages whose vectors are missing or older than the page.
    pub stale: usize,
}

/// Parameters for the Obsidian tools that name one page.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ObsidianPathParams {
    /// Brain slug or vault-relative path, e.g. `projects/emmett-hub` or `projects/emmett-hub.md`.
    pub path: String,
}

/// Parameters for `obsidian_open`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ObsidianOpenParams {
    /// Brain slug or vault-relative path of the page to show.
    pub path: String,
    /// Open in a new tab instead of the current one.
    #[serde(default)]
    pub new_tab: bool,
}

/// Parameters for `obsidian_rename_page`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ObsidianRenameParams {
    /// Brain slug or vault-relative path of the page to move.
    pub from: String,
    /// New vault-relative path ending in `.md`, or a folder to move the page into.
    pub to: String,
}

/// Parameters for `brain_render`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct RenderParams {
    /// The page slug, folder included.
    pub slug: String,
    /// Colours and fonts for the HTML, any subset of the renderer's style keys
    /// (`text`, `muted`, `link`, `unresolved`, `accent`, `code`, `code_bg`, `mono`,
    /// `mark_bg`, `line`, `tag`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`,
    /// `headings`, `size`); the rest take defaults.
    #[serde(default)]
    pub style: Option<serde_json::Value>,
}

/// Parameters for `brain_write_page`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct WritePageParams {
    /// The page slug, folder included.
    pub slug: String,
    /// The whole file: frontmatter, body and timeline, exactly as it should be on disk.
    pub content: String,
}

/// Parameters for `brain_new_page`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct NewPageParams {
    /// The folder to create the page in; empty for the vault root.
    #[serde(default)]
    pub folder: String,
    /// The file name without `.md`; omitted means `Untitled`, `Untitled 1`, ...
    pub name: Option<String>,
}

/// A vault-relative folder path.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct FolderParams {
    /// The folder path, e.g. `projects/archive`.
    pub path: String,
}

/// Parameters for `brain_rename`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct RenameParams {
    /// The page slug or folder path to move.
    pub from: String,
    /// The new slug or folder path; a value ending in `/` moves into that folder under
    /// the same name.
    pub to: String,
}

/// Parameters for `brain_graph`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct GraphParams {
    /// Tags as nodes, with an edge from every page carrying them.
    #[serde(default)]
    pub tags: bool,
    /// Unresolved link targets as nodes.
    #[serde(default)]
    pub unresolved: bool,
    /// Only the neighbourhood of this page slug (a local graph).
    pub around: Option<String>,
    /// How many links away the neighbourhood reaches (default 1).
    pub depth: Option<usize>,
}

/// Parameters for `brain_set_property`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SetPropertyParams {
    /// The page slug, folder included.
    pub slug: String,
    /// The frontmatter key.
    pub key: String,
    /// The value: text, a number, true or false, a `YYYY-MM-DD` date as text, or a list
    /// of strings.
    pub value: serde_json::Value,
}

/// Parameters for `brain_remove_property`.
#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct RemovePropertyParams {
    /// The page slug, folder included.
    pub slug: String,
    /// The frontmatter key to remove.
    pub key: String,
}

/// The MCP server: the tool router, a shared [`Core`], and the connected peers.
#[derive(Clone)]
pub struct Rusty {
    core: Arc<Core>,
    peers: Peers,
    obsidian: Arc<Obsidian>,
    tool_router: ToolRouter<Self>,
}

/// A parsed `rusty://` resource URI.
#[derive(Debug, PartialEq)]
enum ResourceUri {
    Tasks,
    TaskGroup(i64),
    Memories,
    Skills,
    Notes,
    Note(String),
    Brain,
    BrainPage(String),
}

/// Map a `rusty://` URI onto what it names, or `None` for anything else.
fn parse_resource_uri(uri: &str) -> Option<ResourceUri> {
    let rest = uri.strip_prefix("rusty://")?.trim_end_matches('/');
    let parsed = match rest {
        "tasks" => ResourceUri::Tasks,
        "memories" => ResourceUri::Memories,
        "skills" => ResourceUri::Skills,
        "notes" => ResourceUri::Notes,
        "brain" => ResourceUri::Brain,
        other => {
            if let Some(id) = other.strip_prefix("tasks/") {
                ResourceUri::TaskGroup(id.parse().ok()?)
            } else if let Some(path) = other.strip_prefix("notes/") {
                ResourceUri::Note(path.to_string())
            } else {
                ResourceUri::BrainPage(other.strip_prefix("brain/")?.to_string())
            }
        }
    };
    Some(parsed)
}

#[tool_router]
impl Rusty {
    /// Wrap a ready [`Core`]; `peers` is shared by every session of one process.
    pub fn new(core: Arc<Core>, peers: Peers) -> Self {
        let obsidian = Arc::new(Obsidian::new(core.brain_path.clone()));
        Self {
            core,
            peers,
            obsidian,
            tool_router: Self::tool_router(),
        }
    }

    /// Like [`json_result`], and announces the change to connected clients on success.
    fn mutate<T: serde::Serialize>(
        &self,
        value: Result<T, String>,
    ) -> Result<CallToolResult, McpError> {
        if value.is_ok() {
            self.core.events.emit(AppEvent::DataChanged);
        }
        json_result(value)
    }

    /// The text of one resource, as JSON or markdown.
    fn resource_text(&self, uri: &ResourceUri) -> Result<String, String> {
        fn pretty<T: serde::Serialize>(v: T) -> Result<String, String> {
            serde_json::to_string_pretty(&v).map_err(|e| e.to_string())
        }
        match uri {
            ResourceUri::Tasks => {
                let mut lists = Vec::new();
                for header in self.core.user_task_manager.list_headers()? {
                    let tasks = self.core.user_task_manager.list_tasks(header.id, false)?;
                    lists.push(serde_json::json!({ "group": header, "tasks": tasks }));
                }
                pretty(lists)
            }
            ResourceUri::TaskGroup(id) => {
                pretty(self.core.user_task_manager.list_tasks(*id, false)?)
            }
            ResourceUri::Memories => pretty(self.core.memory_manager.list(None)?),
            ResourceUri::Skills => pretty(self.core.skills_manager.list(true)),
            ResourceUri::Notes => pretty(self.core.notes_manager.list_tree()?),
            ResourceUri::Note(path) => self.core.notes_manager.read_note(path),
            ResourceUri::Brain => pretty(self.core.brain_manager.list_pages(None, None)?),
            ResourceUri::BrainPage(slug) => match self.core.brain_manager.read_page(slug)? {
                Some(page) => pretty(page),
                None => Err(format!("no page {slug}")),
            },
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
        self.mutate(self.core.user_task_manager.create_header(&p.name))
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
        self.mutate(
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
        self.mutate(self.core.user_task_manager.toggle_complete(p.id))
    }

    #[tool(description = "Archive a task (hidden from the list, not deleted)")]
    fn archive_task(
        &self,
        Parameters(p): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .user_task_manager
                .archive_task(p.id)
                .map(|_| "archived"),
        )
    }

    #[tool(
        description = "Search the brain vault: full text (and vectors when a provider is set). Operators narrow it: `tag:<name>` (the tag or one nested under it), `path:<part>` (the slug), `file:<part>` (the file name), `type:<type>`; a value in quotes may hold spaces, a leading `-` excludes, and operator terms alone list the matching pages. `case_sensitive` matches the words as typed and `regex` treats them as a pattern; both are text searches"
    )]
    async fn brain_search(
        &self,
        Parameters(p): Parameters<BrainSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let core = Arc::clone(&self.core);
        let options = rusty_core::brain::SearchOptions {
            limit: p.limit.or(Some(10)),
            page_type: p.page_type.clone(),
            case_sensitive: p.case_sensitive.unwrap_or(false),
            regex: p.regex.unwrap_or(false),
        };
        let results = tokio::task::spawn_blocking(move || match core.embedder() {
            Some(embedder) => {
                core.brain_manager
                    .search_hybrid_with(&p.query, &options, embedder.as_ref())
            }
            None => core.brain_manager.search_with(&p.query, &options),
        })
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r);
        json_result(results)
    }

    #[tool(
        description = "Semantic index state: the provider in use (none means full-text only), the model, pages and chunks indexed, pages waiting"
    )]
    async fn brain_semantic_status(&self) -> Result<CallToolResult, McpError> {
        let core = Arc::clone(&self.core);
        let status = tokio::task::spawn_blocking(move || {
            let embedder = core.embedder();
            let index = core.brain_manager.semantic();
            let stats = index.stats()?;
            let stale = match &embedder {
                Some(e) => index.stale_slugs(&e.id())?.0.len(),
                None => 0,
            };
            Ok::<_, String>(SemanticStatus {
                provider: embedder.map(|e| e.id()),
                stats,
                stale,
            })
        })
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r);
        json_result(status)
    }

    #[tool(
        description = "Embed brain pages now: the stale ones, or every page with force; needs an embedding provider"
    )]
    async fn brain_reembed(
        &self,
        Parameters(p): Parameters<ReembedParams>,
    ) -> Result<CallToolResult, McpError> {
        let core = Arc::clone(&self.core);
        let report = tokio::task::spawn_blocking(move || {
            let embedder = core.embedder().ok_or_else(|| {
                "no embedding provider: set embedding_provider to ollama, or to openai with openai_api_key in the vault".to_string()
            })?;
            core.brain_manager.index_stale(embedder.as_ref(), p.force)
        })
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r);
        json_result(report)
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
        self.mutate(
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
        self.mutate(self.core.brain_manager.add_timeline(
            &p.slug,
            &date,
            "mcp",
            &p.summary,
            p.detail.as_deref(),
        ))
    }

    #[tool(
        description = "Is Obsidian installed, running, registered for the brain vault, and answering on its CLI?"
    )]
    async fn obsidian_status(&self) -> Result<CallToolResult, McpError> {
        json_result(Ok(self.obsidian.status().await))
    }

    #[tool(description = "Show a brain page in Obsidian; starts the app when it is not running")]
    async fn obsidian_open(
        &self,
        Parameters(p): Parameters<ObsidianOpenParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.obsidian.open(&p.path, p.new_tab).await)
    }

    #[tool(description = "Pages that link to a brain page, as Obsidian resolves them, with counts")]
    async fn obsidian_backlinks(
        &self,
        Parameters(p): Parameters<ObsidianPathParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.obsidian.backlinks(&p.path).await)
    }

    #[tool(description = "Outgoing links of a brain page, as Obsidian resolves them")]
    async fn obsidian_links(
        &self,
        Parameters(p): Parameters<ObsidianPathParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.obsidian.links(&p.path).await)
    }

    #[tool(description = "Wikilinks anywhere in the vault whose target page does not exist")]
    async fn obsidian_unresolved(&self) -> Result<CallToolResult, McpError> {
        json_result(self.obsidian.unresolved().await)
    }

    #[tool(
        description = "Rename or move a brain page through Obsidian, which rewrites every link to it"
    )]
    async fn obsidian_rename_page(
        &self,
        Parameters(p): Parameters<ObsidianRenameParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.obsidian.rename(&p.from, &p.to).await)
    }

    #[tool(
        description = "Put a list's tasks in the given order; tasks left out follow in their current order"
    )]
    fn reorder_tasks(
        &self,
        Parameters(p): Parameters<ReorderTasksParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .user_task_manager
                .reorder(p.group_id, &p.task_ids)
                .map(|_| "reordered"),
        )
    }

    #[tool(description = "Change a memory's content, category or importance; omitted fields stay")]
    fn update_memory(
        &self,
        Parameters(p): Parameters<UpdateMemoryParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.memory_manager.update(
            &p.id,
            p.content.as_deref(),
            p.category.as_deref(),
            p.importance.as_deref(),
        ))
    }

    #[tool(
        description = "Every setting with its value; values of keys that look like credentials are masked"
    )]
    fn settings_list(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.settings_manager.list().map(|all| {
            all.into_iter()
                .map(|(key, value)| SettingEntry {
                    value: if looks_secret(&key) {
                        "•••".to_string()
                    } else {
                        value
                    },
                    key,
                })
                .collect::<Vec<_>>()
        }))
    }

    #[tool(description = "Today's daily page (or a given YYYY-MM-DD), created when missing")]
    fn brain_daily_note(
        &self,
        Parameters(p): Parameters<DailyNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        let date = p
            .date
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
            .map(str::to_string)
            .unwrap_or_else(chrono_today);
        let existed = self
            .core
            .brain_manager
            .read_page(&format!("daily/{date}"))
            .map(|page| page.is_some())
            .unwrap_or(false);
        let page = self.core.brain_manager.daily_page(Some(&date));
        if existed {
            json_result(page)
        } else {
            self.mutate(page)
        }
    }

    #[tool(
        description = "Append a quick note to today's daily page (default) or the inbox page, creating it when needed"
    )]
    fn brain_capture(
        &self,
        Parameters(p): Parameters<CaptureParams>,
    ) -> Result<CallToolResult, McpError> {
        let target = match p.target.as_deref() {
            Some(t) => match rusty_core::brain::CaptureTarget::parse(t) {
                Ok(t) => t,
                Err(e) => return Err(McpError::invalid_params(e, None)),
            },
            None => rusty_core::brain::CaptureTarget::Daily,
        };
        self.mutate(
            self.core
                .brain_manager
                .capture(&p.text, target, p.date.as_deref(), "mcp"),
        )
    }

    #[tool(description = "The page types the vault knows, with their folders and page counts")]
    fn brain_page_types(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.page_types())
    }

    #[tool(
        description = "Rewrite a skill's description and/or body in place; other frontmatter keys stay; returns the safety scan"
    )]
    fn skill_update(
        &self,
        Parameters(p): Parameters<SkillUpdateParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .core
            .skills_manager
            .update_skill(&p.name, p.description.as_deref(), p.body.as_deref())
            .map(|(skill, findings)| {
                self.core
                    .skills_manager
                    .git_commit(&format!("skill: update {}", p.name));
                SkillUpdateResult { skill, findings }
            });
        self.mutate(result)
    }

    #[tool(description = "Brain vault statistics: pages, links, tags, timeline entries")]
    fn brain_stats(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.stats())
    }

    #[tool(
        description = "The vault as a tree: folders first, then pages and other files, with page counts; dot-folders left out"
    )]
    fn brain_tree(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.tree())
    }

    #[tool(
        description = "A brain page rendered as rich-text HTML (Obsidian flavour: wikilinks, embeds, callouts, tasks, tables, footnotes), with its outline, links, unresolved targets, task and word counts, properties and raw file"
    )]
    fn brain_render(
        &self,
        Parameters(p): Parameters<RenderParams>,
    ) -> Result<CallToolResult, McpError> {
        let style = match p.style {
            Some(value) => serde_json::from_value(value)
                .map_err(|e| McpError::invalid_params(format!("style: {e}"), None))?,
            None => rusty_core::brain::render::Style::default(),
        };
        json_result(self.core.brain_manager.render_page(&p.slug, &style))
    }

    #[tool(
        description = "Replace a brain page's whole file (frontmatter, body, timeline) the way an editor saves; the previous text is kept as a version"
    )]
    fn brain_write_page(
        &self,
        Parameters(p): Parameters<WritePageParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.brain_manager.write_raw(&p.slug, &p.content))
    }

    #[tool(
        description = "Create a page in a folder (the root when empty), named as given or Untitled, typed after the folder; returns the slug"
    )]
    fn brain_new_page(
        &self,
        Parameters(p): Parameters<NewPageParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .brain_manager
                .new_page(&p.folder, p.name.as_deref()),
        )
    }

    #[tool(description = "Create a folder in the vault; returns its path")]
    fn brain_new_folder(
        &self,
        Parameters(p): Parameters<FolderParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.brain_manager.new_folder(&p.path))
    }

    #[tool(
        description = "Soft-delete a folder and everything in it into archive/; returns where it went"
    )]
    fn brain_delete_folder(
        &self,
        Parameters(p): Parameters<FolderParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.brain_manager.delete_folder(&p.path))
    }

    #[tool(
        description = "Rename or move a page or folder; every link to it in the vault is rewritten and the index follows"
    )]
    fn brain_rename(
        &self,
        Parameters(p): Parameters<RenameParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.brain_manager.rename(&p.from, &p.to))
    }

    #[tool(description = "Every wikilink in the vault whose target is no page, with its line")]
    fn brain_unresolved(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.unresolved())
    }

    #[tool(
        description = "The vault as a graph: page nodes (title, type, folder, tags) and edges from resolved links; tags and unresolved targets as nodes on request; `around` with `depth` keeps one page's neighbourhood"
    )]
    fn brain_graph(
        &self,
        Parameters(p): Parameters<GraphParams>,
    ) -> Result<CallToolResult, McpError> {
        let options = rusty_core::brain::GraphOptions {
            tags: p.tags,
            unresolved: p.unresolved,
            around: p.around,
            depth: p.depth,
        };
        json_result(self.core.brain_manager.graph(&options))
    }

    #[tool(
        description = "Every tag in the vault (frontmatter and inline #tags) with its page count; a nested tag a/b counts under a too. Search by tag with `tag:<name>` in brain_search"
    )]
    fn brain_tags(&self) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.tags())
    }

    #[tool(
        description = "Set one frontmatter property on a brain page (text, number, true/false, a YYYY-MM-DD date as text, or a list of strings); other keys keep their order and the body is untouched"
    )]
    fn brain_set_property(
        &self,
        Parameters(p): Parameters<SetPropertyParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .brain_manager
                .set_property(&p.slug, &p.key, p.value),
        )
    }

    #[tool(
        description = "Remove one frontmatter property from a brain page; the body is untouched"
    )]
    fn brain_remove_property(
        &self,
        Parameters(p): Parameters<RemovePropertyParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.brain_manager.remove_property(&p.slug, &p.key))
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
        self.mutate(self.core.memory_manager.store(
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
        self.mutate(
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

    #[tool(description = "Rename a to-do list")]
    fn rename_task_group(
        &self,
        Parameters(p): Parameters<RenameGroupParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .user_task_manager
                .rename_header(p.group_id, &p.name)
                .map(|_| "renamed"),
        )
    }

    #[tool(description = "Delete a to-do list and everything in it")]
    fn delete_task_group(
        &self,
        Parameters(p): Parameters<GroupIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .user_task_manager
                .delete_header(p.group_id)
                .map(|_| "deleted"),
        )
    }

    #[tool(description = "Change a task's title")]
    fn update_task_title(
        &self,
        Parameters(p): Parameters<UpdateTaskTitleParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .user_task_manager
                .update_title(p.id, &p.title)
                .map(|_| "updated"),
        )
    }

    #[tool(description = "Bring an archived task back")]
    fn unarchive_task(
        &self,
        Parameters(p): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .user_task_manager
                .unarchive_task(p.id)
                .map(|_| "unarchived"),
        )
    }

    #[tool(description = "Delete a task for good")]
    fn delete_task(
        &self,
        Parameters(p): Parameters<TaskIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .user_task_manager
                .delete_task(p.id)
                .map(|_| "deleted"),
        )
    }

    #[tool(
        description = "Create a note or a folder under the notes root; returns its relative path"
    )]
    fn create_note(
        &self,
        Parameters(p): Parameters<CreateNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .notes_manager
                .create_note(&p.parent, &p.name, p.is_folder),
        )
    }

    #[tool(description = "Rename a note or folder; returns the new relative path")]
    fn rename_note(
        &self,
        Parameters(p): Parameters<RenameNoteParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.notes_manager.rename_note(&p.path, &p.new_name))
    }

    #[tool(description = "Delete a note (moved to the notes .deleted folder)")]
    fn delete_note(
        &self,
        Parameters(p): Parameters<NotePathParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .notes_manager
                .delete_note(&p.path)
                .map(|_| "deleted"),
        )
    }

    #[tool(description = "Delete a long-term memory")]
    fn delete_memory(
        &self,
        Parameters(p): Parameters<MemoryIdParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.memory_manager.delete(&p.id).map(|_| "deleted"))
    }

    #[tool(description = "Outbound links and backlinks of a brain page")]
    fn brain_get_links(
        &self,
        Parameters(p): Parameters<SlugParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.get_links(&p.slug))
    }

    #[tool(description = "Replace a brain page's body; frontmatter and timeline are kept")]
    fn brain_update_page(
        &self,
        Parameters(p): Parameters<UpdatePageParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.brain_manager.update_page(&p.slug, &p.content))
    }

    #[tool(description = "Delete a brain page and its index entries")]
    fn brain_delete_page(
        &self,
        Parameters(p): Parameters<SlugParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .brain_manager
                .delete_page(&p.slug)
                .map(|_| "deleted"),
        )
    }

    #[tool(description = "A brain page's timeline entries, newest first")]
    fn brain_get_timeline(
        &self,
        Parameters(p): Parameters<TimelineParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.get_timeline(&p.slug, p.limit))
    }

    #[tool(description = "Resolve a partial slug or title to matching page slugs")]
    fn brain_resolve_slug(
        &self,
        Parameters(p): Parameters<ResolveSlugParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.brain_manager.resolve_slug(&p.partial))
    }

    #[tool(description = "Search past agent conversations (prompts and results) by keyword")]
    fn search_conversations(
        &self,
        Parameters(p): Parameters<SearchConversationsParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(
            self.core
                .task_manager
                .search_conversations(&p.query, p.limit.unwrap_or(10)),
        )
    }

    #[tool(description = "Create a skill (a SKILL.md in the store), active or staged for approval")]
    fn skill_create(
        &self,
        Parameters(p): Parameters<CreateSkillParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = if p.pending {
            self.core
                .skills_manager
                .create_pending_skill(&p.name, &p.description, &p.body)
        } else {
            self.core
                .skills_manager
                .create_skill(&p.name, &p.description, &p.body, p.force)
        };
        self.mutate(result)
    }

    #[tool(description = "Delete a skill, active or staged")]
    fn skill_delete(
        &self,
        Parameters(p): Parameters<SkillNameParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .skills_manager
                .delete_skill(&p.name)
                .map(|_| "deleted"),
        )
    }

    #[tool(description = "Run the safety scan on a skill; returns the findings, empty when clean")]
    fn skill_scan(
        &self,
        Parameters(p): Parameters<SkillNameParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.skills_manager.scan(&p.name))
    }

    #[tool(description = "Approve a staged skill so Claude Code can load it")]
    fn skill_approve(
        &self,
        Parameters(p): Parameters<ApproveSkillParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .skills_manager
                .approve(&p.name, p.force)
                .map(|_| "approved"),
        )
    }

    #[tool(description = "Reject and remove a staged skill")]
    fn skill_reject(
        &self,
        Parameters(p): Parameters<SkillNameParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.skills_manager.reject(&p.name).map(|_| "rejected"))
    }

    #[tool(description = "List the keys in the secrets vault; values are never returned")]
    fn secret_list(&self) -> Result<CallToolResult, McpError> {
        json_result(
            self.core
                .secrets_manager
                .list()
                .map(|secrets| secrets.into_iter().map(|s| s.key).collect::<Vec<_>>()),
        )
    }

    #[tool(description = "Set a secret in the vault")]
    fn secret_set(
        &self,
        Parameters(p): Parameters<SecretSetParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .secrets_manager
                .set(&p.key, &p.value)
                .map(|_| "set"),
        )
    }

    #[tool(description = "Delete a secret from the vault")]
    fn secret_delete(
        &self,
        Parameters(p): Parameters<SecretKeyParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(self.core.secrets_manager.delete(&p.key).map(|_| "deleted"))
    }

    #[tool(description = "Read one setting; null when unset")]
    fn setting_get(
        &self,
        Parameters(p): Parameters<SettingKeyParams>,
    ) -> Result<CallToolResult, McpError> {
        json_result(self.core.settings_manager.get(&p.key))
    }

    #[tool(description = "Write one setting")]
    fn setting_set(
        &self,
        Parameters(p): Parameters<SettingSetParams>,
    ) -> Result<CallToolResult, McpError> {
        self.mutate(
            self.core
                .settings_manager
                .set(&p.key, &p.value)
                .map(|_| "set"),
        )
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
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_resources_list_changed()
                .build(),
        )
        .with_server_info(server_identity())
        .with_instructions(format!(
            "Rusty is the user's local assistant store: to-do lists, notes, long-term \
                 memories, the brain vault (a markdown wiki with a full-text index) and \
                 skills, exposed as {tool_count} tools. Slugs include their folder \
                 (projects/name). Search is all-terms; use plain words. Resources under \
                 rusty:// mirror the same data and a list_changed notification follows every \
                 change."
        ))
    }

    async fn on_initialized(&self, context: NotificationContext<RoleServer>) {
        self.peers.lock().await.push(context.peer);
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let entries = [
            (
                "rusty://tasks",
                "tasks",
                "Every to-do list with its open tasks (JSON)",
            ),
            ("rusty://memories", "memories", "Long-term memories (JSON)"),
            (
                "rusty://skills",
                "skills",
                "Skills in the store, active and staged (JSON)",
            ),
            (
                "rusty://notes",
                "notes",
                "The notes folder as a tree (JSON)",
            ),
            ("rusty://brain", "brain", "Brain pages, newest first (JSON)"),
        ];
        let resources = entries
            .into_iter()
            .map(|(uri, name, description)| {
                let mut r = Resource::new(uri, name);
                r.description = Some(description.into());
                r
            })
            .collect();
        Ok(ListResourcesResult {
            resources,
            ..Default::default()
        })
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        let templates = [
            (
                "rusty://tasks/{group_id}",
                "tasks-in-list",
                "Open tasks in one list (JSON)",
            ),
            (
                "rusty://brain/{slug}",
                "brain-page",
                "One brain page with its frontmatter (JSON)",
            ),
            ("rusty://notes/{path}", "note", "One note's markdown"),
        ]
        .into_iter()
        .map(|(uri, name, description)| {
            let mut t = ResourceTemplate::new(uri, name);
            t.description = Some(description.into());
            t
        })
        .collect();
        Ok(ListResourceTemplatesResult {
            resource_templates: templates,
            ..Default::default()
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let uri = request.uri.clone();
        let parsed = parse_resource_uri(&uri).ok_or_else(|| {
            McpError::resource_not_found(
                "resource_not_found",
                Some(serde_json::json!({ "uri": uri })),
            )
        })?;
        let text = self.resource_text(&parsed).map_err(|e| {
            McpError::resource_not_found(e, Some(serde_json::json!({ "uri": uri })))
        })?;
        Ok(ReadResourceResult::new(vec![ResourceContents::text(text, uri)]).into())
    }
}

/// Keep the vector index current: once at start and after every burst of changes,
/// embed the pages whose vectors are missing or stale with the configured provider.
/// Quiet when there is no provider, so full-text-only setups cost nothing.
fn spawn_indexer(core: Arc<Core>, mut events: tokio::sync::broadcast::Receiver<AppEvent>) {
    tokio::spawn(async move {
        loop {
            let worker = Arc::clone(&core);
            let outcome = tokio::task::spawn_blocking(move || {
                // Files changed by another program (Obsidian, an editor, git) reach the
                // index here; the tools index what they write themselves.
                if let Err(e) = worker.brain_manager.sync_all() {
                    eprintln!("rusty-mcp: vault sync: {e}");
                }
                worker
                    .embedder()
                    .map(|e| worker.brain_manager.index_stale(e.as_ref(), false))
            })
            .await;
            match outcome {
                Ok(Some(Ok(r)))
                    if r.pages_indexed > 0 || r.pages_removed > 0 || !r.pages_failed.is_empty() =>
                {
                    eprintln!(
                        "rusty-mcp: semantic index ({}): {} pages embedded, {} chunks, {} removed, {} failed",
                        r.model,
                        r.pages_indexed,
                        r.chunks_written,
                        r.pages_removed,
                        r.pages_failed.len()
                    );
                    for failure in r.pages_failed.iter().take(5) {
                        eprintln!("rusty-mcp:   {failure}");
                    }
                }
                Ok(Some(Err(e))) => eprintln!("rusty-mcp: semantic index: {e}"),
                Err(e) => eprintln!("rusty-mcp: semantic index task: {e}"),
                _ => {}
            }
            // Wait for the next change (or ten minutes), then let the burst settle.
            loop {
                match tokio::time::timeout(std::time::Duration::from_secs(600), events.recv()).await
                {
                    Ok(Ok(AppEvent::DataChanged)) => break,
                    Ok(Ok(_)) => continue,
                    Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => break,
                    Ok(Err(_)) => return,
                    Err(_) => break,
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            while events.try_recv().is_ok() {}
        }
    });
}

/// Forward every data change on the event bus to every connected client as a
/// `resources/list_changed` notification, dropping peers that have gone away.
fn spawn_change_notifier(mut events: tokio::sync::broadcast::Receiver<AppEvent>, peers: Peers) {
    tokio::spawn(async move {
        loop {
            match events.recv().await {
                Ok(AppEvent::DataChanged) => {
                    let mut peers = peers.lock().await;
                    let mut alive = Vec::with_capacity(peers.len());
                    for peer in peers.drain(..) {
                        if peer.notify_resource_list_changed().await.is_ok() {
                            alive.push(peer);
                        }
                    }
                    *peers = alive;
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

/// Default address for the HTTP transport: loopback only, the port v2 used.
const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:4174";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let core = Arc::new(Core::init());
    let peers: Peers = Arc::default();
    rusty_core::start_data_watcher(
        core.events.clone(),
        core.notes_path.clone(),
        core.brain_path.clone(),
        core.skills_root.clone(),
    );
    spawn_indexer(Arc::clone(&core), core.events.subscribe());
    spawn_change_notifier(core.events.subscribe(), Arc::clone(&peers));
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => {
            let service = Rusty::new(core, peers).serve(stdio()).await?;
            service.waiting().await?;
        }
        Some("--http") => {
            let addr = args.next().unwrap_or_else(|| DEFAULT_HTTP_ADDR.to_string());
            let service = StreamableHttpService::new(
                move || Ok(Rusty::new(Arc::clone(&core), Arc::clone(&peers))),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tool the router advertises; keep in sync with the README.
    const EXPECTED: &[&str] = &[
        "list_task_groups",
        "create_task_group",
        "rename_task_group",
        "delete_task_group",
        "list_tasks",
        "create_task",
        "toggle_task",
        "archive_task",
        "unarchive_task",
        "update_task_title",
        "delete_task",
        "list_notes",
        "read_note",
        "write_note",
        "create_note",
        "rename_note",
        "delete_note",
        "list_memories",
        "store_memory",
        "delete_memory",
        "brain_search",
        "brain_read_page",
        "brain_list_pages",
        "brain_create_page",
        "brain_update_page",
        "brain_delete_page",
        "brain_add_timeline",
        "brain_get_timeline",
        "brain_get_links",
        "brain_resolve_slug",
        "brain_stats",
        "search_conversations",
        "skill_list",
        "skill_view",
        "skill_create",
        "skill_delete",
        "skill_scan",
        "skill_approve",
        "skill_reject",
        "secret_list",
        "secret_set",
        "secret_delete",
        "setting_get",
        "setting_set",
        "obsidian_status",
        "obsidian_open",
        "obsidian_backlinks",
        "obsidian_links",
        "obsidian_unresolved",
        "obsidian_rename_page",
        "reorder_tasks",
        "update_memory",
        "settings_list",
        "brain_daily_note",
        "brain_capture",
        "brain_page_types",
        "skill_update",
        "brain_semantic_status",
        "brain_reembed",
        "brain_tree",
        "brain_render",
        "brain_write_page",
        "brain_new_page",
        "brain_new_folder",
        "brain_delete_folder",
        "brain_rename",
        "brain_unresolved",
        "brain_tags",
        "brain_set_property",
        "brain_remove_property",
        "brain_graph",
    ];

    #[test]
    fn router_advertises_every_tool_once() {
        let router = Rusty::tool_router();
        let mut names: Vec<String> = router
            .list_all()
            .into_iter()
            .map(|t| t.name.to_string())
            .collect();
        names.sort();
        let mut expected: Vec<String> = EXPECTED.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(names, expected);
    }

    #[test]
    fn resource_uris_parse() {
        assert_eq!(
            parse_resource_uri("rusty://tasks"),
            Some(ResourceUri::Tasks)
        );
        assert_eq!(
            parse_resource_uri("rusty://tasks/5"),
            Some(ResourceUri::TaskGroup(5))
        );
        assert_eq!(parse_resource_uri("rusty://tasks/x"), None);
        assert_eq!(
            parse_resource_uri("rusty://brain/projects/rusty"),
            Some(ResourceUri::BrainPage("projects/rusty".into()))
        );
        assert_eq!(
            parse_resource_uri("rusty://notes/Misc.md"),
            Some(ResourceUri::Note("Misc.md".into()))
        );
        assert_eq!(parse_resource_uri("rusty://nope"), None);
        assert_eq!(parse_resource_uri("http://x"), None);
    }

    #[test]
    fn every_tool_has_a_description() {
        for tool in Rusty::tool_router().list_all() {
            let description = tool.description.as_deref().unwrap_or("");
            assert!(description.len() > 10, "{} has no description", tool.name);
        }
    }
}
