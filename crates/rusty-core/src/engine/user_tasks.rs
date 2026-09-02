//! User task management with grouped lists.
//!
//! Stores task groups (headers) and individual tasks in SQLite.
//! Separate from the Claude CLI `tasks` table — these are user-managed to-do items.

use crate::engine::db::Database;
use std::sync::Arc;

/// A task group header.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskHeader {
    /// Auto-increment ID.
    pub id: i64,
    /// Group name.
    pub name: String,
    /// Sort position.
    pub sort_order: i64,
}

/// A user task item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserTask {
    /// Auto-increment ID.
    pub id: i64,
    /// Parent group ID.
    pub header_id: i64,
    /// Task title.
    pub title: String,
    /// Whether the task is completed.
    pub completed: bool,
    /// Whether the task is archived.
    pub archived: bool,
    /// Sort position within the group.
    pub sort_order: i64,
    /// Creation timestamp (Unix seconds).
    pub created_at: i64,
}

/// Manages user task groups and items in SQLite.
pub struct UserTaskManager {
    db: Arc<Database>,
}

impl UserTaskManager {
    /// Create a new UserTaskManager backed by the given database.
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    // ── Headers ────────────────────────────────────────────────────────

    /// List all task group headers ordered by sort_order.
    pub fn list_headers(&self) -> Result<Vec<TaskHeader>, String> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare("SELECT id, name, sort_order FROM task_headers ORDER BY sort_order ASC")
            .map_err(|e| format!("Query error: {e}"))?;

        let headers = stmt
            .query_map([], |row| {
                Ok(TaskHeader {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    sort_order: row.get(2)?,
                })
            })
            .map_err(|e| format!("Query error: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {e}"))?;

        Ok(headers)
    }

    /// Create a new task group. Returns the new header ID.
    pub fn create_header(&self, name: &str) -> Result<i64, String> {
        let conn = self.db.conn()?;
        let max_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM task_headers",
                [],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "INSERT INTO task_headers (name, sort_order) VALUES (?1, ?2)",
            rusqlite::params![name, max_order + 1],
        )
        .map_err(|e| format!("Failed to create header: {e}"))?;

        Ok(conn.last_insert_rowid())
    }

    /// Rename a task group.
    pub fn rename_header(&self, id: i64, name: &str) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE task_headers SET name = ?1 WHERE id = ?2",
            rusqlite::params![name, id],
        )
        .map_err(|e| format!("Failed to rename header: {e}"))?;
        Ok(())
    }

    /// Delete a task group and all its tasks (CASCADE).
    pub fn delete_header(&self, id: i64) -> Result<(), String> {
        let conn = self.db.conn()?;
        // Delete tasks first (in case foreign keys aren't enforced)
        conn.execute(
            "DELETE FROM user_tasks WHERE header_id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to delete tasks: {e}"))?;
        conn.execute(
            "DELETE FROM task_headers WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to delete header: {e}"))?;
        Ok(())
    }

    // ── Tasks ──────────────────────────────────────────────────────────

    /// List tasks in a group. If `show_archived` is false, only active tasks are returned.
    pub fn list_tasks(&self, header_id: i64, show_archived: bool) -> Result<Vec<UserTask>, String> {
        let conn = self.db.conn()?;
        let sql = if show_archived {
            "SELECT id, header_id, title, completed, archived, sort_order, created_at FROM user_tasks WHERE header_id = ?1 ORDER BY sort_order ASC"
        } else {
            "SELECT id, header_id, title, completed, archived, sort_order, created_at FROM user_tasks WHERE header_id = ?1 AND archived = 0 ORDER BY sort_order ASC"
        };

        let mut stmt = conn.prepare(sql).map_err(|e| format!("Query error: {e}"))?;

        let tasks = stmt
            .query_map(rusqlite::params![header_id], |row| {
                Ok(UserTask {
                    id: row.get(0)?,
                    header_id: row.get(1)?,
                    title: row.get(2)?,
                    completed: row.get::<_, i64>(3)? != 0,
                    archived: row.get::<_, i64>(4)? != 0,
                    sort_order: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .map_err(|e| format!("Query error: {e}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Row error: {e}"))?;

        Ok(tasks)
    }

    /// Create a new task in a group. Returns the task ID.
    pub fn create_task(&self, header_id: i64, title: &str) -> Result<i64, String> {
        let conn = self.db.conn()?;
        let now = chrono_now();
        let max_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) FROM user_tasks WHERE header_id = ?1",
                rusqlite::params![header_id],
                |row| row.get(0),
            )
            .unwrap_or(-1);

        conn.execute(
            "INSERT INTO user_tasks (header_id, title, sort_order, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![header_id, title, max_order + 1, now],
        )
        .map_err(|e| format!("Failed to create task: {e}"))?;

        Ok(conn.last_insert_rowid())
    }

    /// Toggle a task's completed status. Returns the new status.
    pub fn toggle_complete(&self, id: i64) -> Result<bool, String> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE user_tasks SET completed = CASE WHEN completed = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to toggle task: {e}"))?;

        let completed: bool = conn
            .query_row(
                "SELECT completed FROM user_tasks WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get::<_, i64>(0).map(|v| v != 0),
            )
            .map_err(|e| format!("Query error: {e}"))?;

        Ok(completed)
    }

    /// Archive a task.
    pub fn archive_task(&self, id: i64) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE user_tasks SET archived = 1 WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to archive task: {e}"))?;
        Ok(())
    }

    /// Unarchive a task.
    pub fn unarchive_task(&self, id: i64) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE user_tasks SET archived = 0 WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to unarchive task: {e}"))?;
        Ok(())
    }

    /// Update a task's title.
    pub fn update_title(&self, id: i64, title: &str) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "UPDATE user_tasks SET title = ?1 WHERE id = ?2",
            rusqlite::params![title, id],
        )
        .map_err(|e| format!("Failed to update title: {e}"))?;
        Ok(())
    }

    /// Put a group's tasks in the given order. Every id must belong to the group; tasks
    /// the list leaves out keep their relative order after the listed ones.
    pub fn reorder(&self, header_id: i64, ordered_ids: &[i64]) -> Result<(), String> {
        let current = self.list_tasks(header_id, true)?;
        let known: std::collections::HashSet<i64> = current.iter().map(|t| t.id).collect();
        if let Some(bad) = ordered_ids.iter().find(|id| !known.contains(id)) {
            return Err(format!("Task {bad} is not in group {header_id}"));
        }
        let mut seen = std::collections::HashSet::new();
        let mut order: Vec<i64> = ordered_ids
            .iter()
            .copied()
            .filter(|id| seen.insert(*id))
            .collect();
        order.extend(current.iter().map(|t| t.id).filter(|id| !seen.contains(id)));
        let conn = self.db.conn()?;
        for (position, id) in order.iter().enumerate() {
            conn.execute(
                "UPDATE user_tasks SET sort_order = ?1 WHERE id = ?2 AND header_id = ?3",
                rusqlite::params![position as i64, id, header_id],
            )
            .map_err(|e| format!("Failed to reorder tasks: {e}"))?;
        }
        Ok(())
    }

    /// Delete a task permanently.
    pub fn delete_task(&self, id: i64) -> Result<(), String> {
        let conn = self.db.conn()?;
        conn.execute(
            "DELETE FROM user_tasks WHERE id = ?1",
            rusqlite::params![id],
        )
        .map_err(|e| format!("Failed to delete task: {e}"))?;
        Ok(())
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
    fn create_and_list_headers() {
        let db = test_db();
        let tm = UserTaskManager::new(db);

        let id = tm.create_header("Work").unwrap();
        assert!(id > 0);

        let headers = tm.list_headers().unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, "Work");
    }

    #[test]
    fn rename_header() {
        let db = test_db();
        let tm = UserTaskManager::new(db);

        let id = tm.create_header("Old Name").unwrap();
        tm.rename_header(id, "New Name").unwrap();

        let headers = tm.list_headers().unwrap();
        assert_eq!(headers[0].name, "New Name");
    }

    #[test]
    fn delete_header_cascades() {
        let db = test_db();
        let tm = UserTaskManager::new(db);

        let hid = tm.create_header("Temp").unwrap();
        tm.create_task(hid, "task 1").unwrap();
        tm.create_task(hid, "task 2").unwrap();

        tm.delete_header(hid).unwrap();
        let headers = tm.list_headers().unwrap();
        assert!(headers.is_empty());
        let tasks = tm.list_tasks(hid, true).unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn create_and_list_tasks() {
        let db = test_db();
        let tm = UserTaskManager::new(db);

        let hid = tm.create_header("Group").unwrap();
        tm.create_task(hid, "First").unwrap();
        tm.create_task(hid, "Second").unwrap();

        let tasks = tm.list_tasks(hid, false).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].title, "First");
        assert_eq!(tasks[1].title, "Second");
        assert!(!tasks[0].completed);
    }

    #[test]
    fn toggle_complete() {
        let db = test_db();
        let tm = UserTaskManager::new(db);

        let hid = tm.create_header("G").unwrap();
        let tid = tm.create_task(hid, "T").unwrap();

        let completed = tm.toggle_complete(tid).unwrap();
        assert!(completed);

        let completed = tm.toggle_complete(tid).unwrap();
        assert!(!completed);
    }

    #[test]
    fn archive_hides_from_default_list() {
        let db = test_db();
        let tm = UserTaskManager::new(db);

        let hid = tm.create_header("G").unwrap();
        let tid = tm.create_task(hid, "T").unwrap();

        tm.archive_task(tid).unwrap();

        let active = tm.list_tasks(hid, false).unwrap();
        assert!(active.is_empty());

        let all = tm.list_tasks(hid, true).unwrap();
        assert_eq!(all.len(), 1);
        assert!(all[0].archived);

        tm.unarchive_task(tid).unwrap();
        let active = tm.list_tasks(hid, false).unwrap();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn update_title() {
        let db = test_db();
        let tm = UserTaskManager::new(db);

        let hid = tm.create_header("G").unwrap();
        let tid = tm.create_task(hid, "Old").unwrap();

        tm.update_title(tid, "New").unwrap();
        let tasks = tm.list_tasks(hid, false).unwrap();
        assert_eq!(tasks[0].title, "New");
    }

    #[test]
    fn delete_task() {
        let db = test_db();
        let tm = UserTaskManager::new(db);

        let hid = tm.create_header("G").unwrap();
        let tid = tm.create_task(hid, "T").unwrap();

        tm.delete_task(tid).unwrap();
        let tasks = tm.list_tasks(hid, false).unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn reorder_moves_listed_tasks_first_and_keeps_the_rest() {
        let tm = UserTaskManager::new(test_db());
        let g = tm.create_header("Work").unwrap();
        let a = tm.create_task(g, "a").unwrap();
        let b = tm.create_task(g, "b").unwrap();
        let c = tm.create_task(g, "c").unwrap();
        tm.reorder(g, &[c, a]).unwrap();
        let ids: Vec<i64> = tm
            .list_tasks(g, true)
            .unwrap()
            .iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec![c, a, b]);
        let other = tm.create_header("Home").unwrap();
        let x = tm.create_task(other, "x").unwrap();
        assert!(tm.reorder(g, &[x]).is_err());
    }
}
