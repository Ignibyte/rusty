//! Semaphore-based task queue for concurrent Claude CLI execution.
//!
//! Wraps the existing `process_manager::execute_task` with a concurrency limiter
//! using `tokio::sync::Semaphore`. Tasks wait for an available slot before executing.

use crate::brain::BrainManager;
use crate::engine::memory_manager::MemoryManager;
use crate::engine::process_manager;
use crate::engine::settings_manager::SettingsManager;
use crate::engine::task_manager::TaskManager;
use crate::engine::tool_registry::ToolRegistry;
use crate::events::EventBus;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// Default maximum concurrent Claude CLI tasks.
const DEFAULT_MAX_CONCURRENT: usize = 3;

/// Manages concurrent task execution with a semaphore-based limiter.
pub struct TaskQueue {
    semaphore: Arc<Semaphore>,
    active_tasks: Arc<Mutex<HashSet<String>>>,
    max_concurrent: usize,
}

/// Current status of the task queue.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QueueStatus {
    /// Number of currently executing tasks.
    pub active_count: usize,
    /// Maximum concurrent tasks allowed.
    pub max_concurrent: usize,
    /// IDs of currently executing tasks.
    pub active_task_ids: Vec<String>,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    /// Create a new task queue with the default concurrency limit.
    pub fn new() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(DEFAULT_MAX_CONCURRENT)),
            active_tasks: Arc::new(Mutex::new(HashSet::new())),
            max_concurrent: DEFAULT_MAX_CONCURRENT,
        }
    }

    /// Submit a task for execution. Acquires a semaphore permit before spawning.
    ///
    /// The task will wait if the concurrency limit is reached, then execute
    /// via `process_manager::execute_task` when a slot becomes available.
    #[allow(clippy::too_many_arguments)] // Mirrors process_manager::execute_task signature
    pub fn submit(
        &self,
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
        let semaphore = Arc::clone(&self.semaphore);
        let active_tasks = Arc::clone(&self.active_tasks);
        let tid = task_id.clone();

        tokio::spawn(async move {
            // Track as active
            {
                let mut active = active_tasks.lock().await;
                active.insert(tid.clone());
            }

            // Wait for an available slot
            let permit = semaphore.acquire().await;

            // Execute the task with tool registry for AI function calling
            process_manager::execute_task(
                events,
                task_manager,
                memory_manager,
                tool_registry,
                brain_manager,
                settings_manager,
                task_id,
                prompt,
                resume_session_id,
                max_turns,
                max_budget_usd,
            )
            .await;

            // Remove from active set
            {
                let mut active = active_tasks.lock().await;
                active.remove(&tid);
            }

            // Release permit
            drop(permit);
        });
    }

    /// Get the current queue status.
    pub async fn status(&self) -> QueueStatus {
        let active = self.active_tasks.lock().await;
        QueueStatus {
            active_count: active.len(),
            max_concurrent: self.max_concurrent,
            active_task_ids: active.iter().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn queue_status_initially_empty() {
        let queue = TaskQueue::new();
        let status = queue.status().await;
        assert_eq!(status.active_count, 0);
        assert_eq!(status.max_concurrent, DEFAULT_MAX_CONCURRENT);
        assert!(status.active_task_ids.is_empty());
    }

    #[tokio::test]
    async fn queue_tracks_active_tasks() {
        let queue = TaskQueue::new();

        // Manually insert to test tracking (without actually running tasks)
        {
            let mut active = queue.active_tasks.lock().await;
            active.insert("task-1".to_string());
            active.insert("task-2".to_string());
        }

        let status = queue.status().await;
        assert_eq!(status.active_count, 2);
        assert!(status.active_task_ids.contains(&"task-1".to_string()));
        assert!(status.active_task_ids.contains(&"task-2".to_string()));

        // Remove one
        {
            let mut active = queue.active_tasks.lock().await;
            active.remove("task-1");
        }

        let status = queue.status().await;
        assert_eq!(status.active_count, 1);
    }

    #[tokio::test]
    async fn queue_status_reports_max_concurrent() {
        let queue = TaskQueue::new();
        let status = queue.status().await;
        assert_eq!(status.max_concurrent, 3);
    }
}
