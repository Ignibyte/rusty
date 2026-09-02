//! The wired-up manager layer: one [`Core`] holds every manager, the event bus
//! and the paths the file watcher observes. Transport-agnostic by design: the
//! MCP server, the CLI and the desktop app all build the same `Core`.

use crate::brain::BrainManager;
use crate::engine::agent_manager::AgentManager;
use crate::engine::db::Database;
use crate::engine::memory_manager::MemoryManager;
use crate::engine::secrets_manager::SecretsManager;
use crate::engine::settings_manager::SettingsManager;
use crate::engine::task_manager::TaskManager;
use crate::engine::task_queue::TaskQueue;
use crate::engine::tool_registry::ToolRegistry;
use crate::engine::user_tasks::UserTaskManager;
use crate::events::EventBus;
use crate::notes::NotesManager;
use crate::skills::SkillsManager;
use std::path::PathBuf;
use std::sync::Arc;

/// Every manager Rusty has, ready to use.
pub struct Core {
    /// Broadcasts [`crate::events::AppEvent`]s to whoever is listening.
    pub events: EventBus,
    /// Claude CLI task lifecycle (the agent runs).
    pub task_manager: Arc<TaskManager>,
    /// Long-term memories.
    pub memory_manager: Arc<MemoryManager>,
    /// Markdown notes.
    pub notes_manager: Arc<NotesManager>,
    /// The to-do lists.
    pub user_task_manager: Arc<UserTaskManager>,
    /// Background task queue.
    pub task_queue: Arc<TaskQueue>,
    /// In-process AI tool registry.
    pub tool_registry: Arc<ToolRegistry>,
    /// Dispatched background agents.
    pub agent_manager: Arc<AgentManager>,
    /// Key/value settings.
    pub settings_manager: Arc<SettingsManager>,
    /// The secrets vault.
    pub secrets_manager: Arc<SecretsManager>,
    /// The brain vault and its index.
    pub brain_manager: Arc<BrainManager>,
    /// The skills store.
    pub skills_manager: Arc<SkillsManager>,
    /// Resolved notes path (watched for changes).
    pub notes_path: PathBuf,
    /// Resolved brain vault path (watched for changes).
    pub brain_path: PathBuf,
    /// Resolved skills root (watched for changes).
    pub skills_root: PathBuf,
}

impl Core {
    /// Open the database, resolve the configured paths, build every manager and
    /// register the AI function tools. Panics only when the database or the
    /// brain vault cannot be initialized, which nothing can recover from.
    pub fn init() -> Self {
        let db = Arc::new(Database::open().expect("Failed to open database"));
        let settings_manager = Arc::new(SettingsManager::new(Arc::clone(&db)));

        let default_home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let default_notes = default_home
            .join(".rusty")
            .join("notes")
            .to_string_lossy()
            .to_string();
        let default_brain = default_home
            .join(".rusty")
            .join("brain")
            .to_string_lossy()
            .to_string();
        let notes_path = settings_manager
            .get_or_default("notes_path", &default_notes)
            .unwrap_or(default_notes);
        let brain_path = settings_manager
            .get_or_default("brain_vault_path", &default_brain)
            .unwrap_or(default_brain);
        let secrets_path = default_home.join(".rusty").join(".secret");

        crate::skills::bootstrap(&settings_manager);
        let skills_root = crate::skills::resolve_root(&settings_manager);

        let events = EventBus::new();
        let task_manager = Arc::new(TaskManager::new(Arc::clone(&db)));
        let memory_manager = Arc::new(MemoryManager::new(Arc::clone(&db)));
        let notes_manager = Arc::new(
            NotesManager::with_root(PathBuf::from(&notes_path)).expect("Failed to init notes"),
        );
        let user_task_manager = Arc::new(UserTaskManager::new(Arc::clone(&db)));
        let task_queue = Arc::new(TaskQueue::new());
        let tool_registry = Arc::new(ToolRegistry::new());
        let agent_manager = Arc::new(AgentManager::new(Arc::clone(&db)));
        let brain_manager = {
            let bm = BrainManager::new(Arc::clone(&db), PathBuf::from(&brain_path));
            bm.ensure_vault().expect("Failed to initialize brain vault");
            Arc::new(bm)
        };
        let skills_manager = Arc::new(SkillsManager::new(skills_root.clone()));
        let secrets_manager = Arc::new(SecretsManager::new(secrets_path));

        crate::engine::tools::notes::register(&tool_registry, Arc::clone(&notes_manager));
        crate::engine::tools::tasks::register(&tool_registry, Arc::clone(&user_task_manager));
        crate::engine::tools::memory::register(&tool_registry, Arc::clone(&memory_manager));
        crate::engine::tools::navigation::register(&tool_registry, events.clone());
        crate::engine::tools::desktop::register(&tool_registry);
        crate::engine::tools::conversations::register(&tool_registry, Arc::clone(&task_manager));
        crate::engine::tools::agents::register(
            &tool_registry,
            Arc::clone(&agent_manager),
            Arc::clone(&brain_manager),
            Arc::clone(&settings_manager),
            events.clone(),
        );
        crate::engine::tools::brain::register(&tool_registry, Arc::clone(&brain_manager));

        Core {
            events,
            task_manager,
            memory_manager,
            notes_manager,
            user_task_manager,
            task_queue,
            tool_registry,
            agent_manager,
            settings_manager,
            secrets_manager,
            brain_manager,
            skills_manager,
            notes_path: PathBuf::from(&notes_path),
            brain_path: PathBuf::from(&brain_path),
            skills_root,
        }
    }
}
