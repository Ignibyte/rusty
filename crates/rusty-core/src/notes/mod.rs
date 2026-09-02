//! Markdown notes manager backed by the filesystem.
//!
//! Notes are stored as `.md` files in `~/.rusty/notes/`. Supports hierarchical
//! folders, soft-delete to a `.deleted/` directory, and safe file naming.

use std::path::{Path, PathBuf};

/// A node in the note tree (file or folder).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NoteTreeEntry {
    /// Display name (filename without `.md` extension).
    pub name: String,
    /// Relative path from the notes root (e.g., "folder/note.md").
    pub path: String,
    /// Whether this is a "file" or "folder".
    pub entry_type: String,
    /// Child entries (only populated for folders).
    pub children: Vec<NoteTreeEntry>,
}

/// Manages markdown notes on disk.
pub struct NotesManager {
    root: PathBuf,
}

impl NotesManager {
    /// Create a new NotesManager rooted at `~/.rusty/notes/`.
    ///
    /// Creates the directory if it doesn't exist.
    pub fn new() -> Result<Self, String> {
        let root = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".rusty")
            .join("notes");
        Self::with_root(root)
    }

    /// Create a NotesManager rooted at a custom path.
    pub fn with_root(root: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&root)
            .map_err(|e| format!("Failed to create notes directory: {e}"))?;

        // Ensure .deleted directory exists
        std::fs::create_dir_all(root.join(".deleted"))
            .map_err(|e| format!("Failed to create .deleted directory: {e}"))?;

        Ok(Self { root })
    }

    /// Build a tree of all notes and folders.
    pub fn list_tree(&self) -> Result<Vec<NoteTreeEntry>, String> {
        self.build_tree(&self.root, "")
    }

    /// Read the content of a note.
    pub fn read_note(&self, relative_path: &str) -> Result<String, String> {
        let full_path = self.resolve_safe_path(relative_path)?;
        std::fs::read_to_string(&full_path).map_err(|e| format!("Failed to read note: {e}"))
    }

    /// Save content to a note (creates or overwrites).
    pub fn save_note(&self, relative_path: &str, content: &str) -> Result<(), String> {
        let full_path = self.resolve_safe_path(relative_path)?;

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        std::fs::write(&full_path, content).map_err(|e| format!("Failed to save note: {e}"))
    }

    /// Create a new note or folder.
    ///
    /// For files, creates an empty `.md` file. For folders, creates the directory.
    pub fn create_note(
        &self,
        parent_path: &str,
        name: &str,
        is_folder: bool,
    ) -> Result<String, String> {
        let safe_name = sanitize_name(name)?;

        let parent = if parent_path.is_empty() {
            self.root.clone()
        } else {
            self.resolve_safe_path(parent_path)?
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| self.root.clone())
        };

        if is_folder {
            let folder_path = parent.join(&safe_name);
            if folder_path.exists() {
                return Err("Folder already exists".to_string());
            }
            std::fs::create_dir_all(&folder_path)
                .map_err(|e| format!("Failed to create folder: {e}"))?;

            let relative = folder_path
                .strip_prefix(&self.root)
                .unwrap_or(&folder_path)
                .to_string_lossy()
                .to_string();
            Ok(relative)
        } else {
            let file_path = parent.join(format!("{safe_name}.md"));
            if file_path.exists() {
                return Err("Note already exists".to_string());
            }
            std::fs::write(&file_path, "").map_err(|e| format!("Failed to create note: {e}"))?;

            let relative = file_path
                .strip_prefix(&self.root)
                .unwrap_or(&file_path)
                .to_string_lossy()
                .to_string();
            Ok(relative)
        }
    }

    /// Soft-delete a note or folder by moving it to `.deleted/`.
    pub fn delete_note(&self, relative_path: &str) -> Result<(), String> {
        let full_path = self.resolve_safe_path(relative_path)?;
        if !full_path.exists() {
            return Err("Note not found".to_string());
        }

        let deleted_dir = self.root.join(".deleted");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let file_name = full_path.file_name().unwrap_or_default().to_string_lossy();
        let dest = deleted_dir.join(format!("{file_name}_{timestamp}"));

        std::fs::rename(&full_path, &dest).map_err(|e| format!("Failed to delete note: {e}"))
    }

    /// Rename a note or folder.
    pub fn rename_note(&self, relative_path: &str, new_name: &str) -> Result<String, String> {
        let full_path = self.resolve_safe_path(relative_path)?;
        if !full_path.exists() {
            return Err("Note not found".to_string());
        }

        let safe_name = sanitize_name(new_name)?;
        let parent = full_path.parent().unwrap_or(&self.root);

        let new_path = if full_path.is_dir() {
            parent.join(&safe_name)
        } else {
            parent.join(format!("{safe_name}.md"))
        };

        if new_path.exists() {
            return Err("A note with that name already exists".to_string());
        }

        std::fs::rename(&full_path, &new_path)
            .map_err(|e| format!("Failed to rename note: {e}"))?;

        let relative = new_path
            .strip_prefix(&self.root)
            .unwrap_or(&new_path)
            .to_string_lossy()
            .to_string();
        Ok(relative)
    }

    /// Resolve a relative path to an absolute path, validating it stays within root.
    fn resolve_safe_path(&self, relative_path: &str) -> Result<PathBuf, String> {
        if relative_path.contains("..") {
            return Err("Invalid path: directory traversal not allowed".to_string());
        }

        let full = self.root.join(relative_path);
        if !full.starts_with(&self.root) {
            return Err("Invalid path: outside notes directory".to_string());
        }

        Ok(full)
    }

    /// Recursively build the file tree.
    fn build_tree(&self, dir: &Path, prefix: &str) -> Result<Vec<NoteTreeEntry>, String> {
        let mut entries = Vec::new();

        let read_dir =
            std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {e}"))?;

        let mut items: Vec<_> = read_dir
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                !name.starts_with('.')
            })
            .collect();

        // Sort: folders first, then alphabetical
        items.sort_by(|a, b| {
            let a_dir = a.path().is_dir();
            let b_dir = b.path().is_dir();
            match (a_dir, b_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.file_name().cmp(&b.file_name()),
            }
        });

        for item in items {
            let path = item.path();
            let name = item.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                let rel_path = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{prefix}/{name}")
                };
                let children = self.build_tree(&path, &rel_path)?;
                entries.push(NoteTreeEntry {
                    name,
                    path: rel_path,
                    entry_type: "folder".to_string(),
                    children,
                });
            } else if name.ends_with(".md") {
                let display_name = name.trim_end_matches(".md").to_string();
                let rel_path = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{prefix}/{name}")
                };
                entries.push(NoteTreeEntry {
                    name: display_name,
                    path: rel_path,
                    entry_type: "file".to_string(),
                    children: Vec::new(),
                });
            }
        }

        Ok(entries)
    }
}

/// Validate and sanitize a file/folder name.
fn sanitize_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        return Err("Name contains invalid characters".to_string());
    }
    // Allow alphanumeric, spaces, dashes, underscores
    let safe: String = trimmed
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    if safe.is_empty() {
        return Err("Name contains only invalid characters".to_string());
    }
    Ok(safe)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_notes_dir(test_name: &str) -> (PathBuf, NotesManager) {
        let dir = std::env::temp_dir().join(format!(
            "rusty_notes_test_{}_{test_name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(dir.join(".deleted")).unwrap();
        let manager = NotesManager { root: dir.clone() };
        (dir, manager)
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn list_tree_empty() {
        let (dir, nm) = test_notes_dir("list_tree_empty");
        let tree = nm.list_tree().unwrap();
        assert!(tree.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn create_and_read_note() {
        let (dir, nm) = test_notes_dir("create_and_read");
        let path = nm.create_note("", "Test Note", false).unwrap();
        assert_eq!(path, "Test Note.md");

        nm.save_note(&path, "# Hello World").unwrap();
        let content = nm.read_note(&path).unwrap();
        assert_eq!(content, "# Hello World");
        cleanup(&dir);
    }

    #[test]
    fn create_folder_and_note_inside() {
        let (dir, nm) = test_notes_dir("folder_and_note");
        let folder = nm.create_note("", "My Folder", true).unwrap();
        assert_eq!(folder, "My Folder");

        nm.save_note("My Folder/child.md", "child content").unwrap();
        let content = nm.read_note("My Folder/child.md").unwrap();
        assert_eq!(content, "child content");

        let tree = nm.list_tree().unwrap();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].entry_type, "folder");
        assert_eq!(tree[0].children.len(), 1);
        cleanup(&dir);
    }

    #[test]
    fn delete_note_soft_deletes() {
        let (dir, nm) = test_notes_dir("delete_soft");
        nm.create_note("", "To Delete", false).unwrap();
        assert!(dir.join("To Delete.md").exists());

        nm.delete_note("To Delete.md").unwrap();
        assert!(!dir.join("To Delete.md").exists());
        let deleted_files: Vec<_> = fs::read_dir(dir.join(".deleted"))
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(deleted_files.len(), 1);
        cleanup(&dir);
    }

    #[test]
    fn rename_note() {
        let (dir, nm) = test_notes_dir("rename");
        nm.create_note("", "Original", false).unwrap();
        let new_path = nm.rename_note("Original.md", "Renamed").unwrap();
        assert_eq!(new_path, "Renamed.md");
        assert!(!dir.join("Original.md").exists());
        assert!(dir.join("Renamed.md").exists());
        cleanup(&dir);
    }

    #[test]
    fn create_duplicate_fails() {
        let (dir, nm) = test_notes_dir("duplicate");
        nm.create_note("", "Unique", false).unwrap();
        let result = nm.create_note("", "Unique", false);
        assert!(result.is_err());
        cleanup(&dir);
    }

    #[test]
    fn path_traversal_rejected() {
        let (dir, nm) = test_notes_dir("traversal");
        assert!(nm.read_note("../etc/passwd").is_err());
        assert!(nm.save_note("../../evil.md", "hack").is_err());
        cleanup(&dir);
    }

    #[test]
    fn sanitize_name_valid() {
        assert_eq!(sanitize_name("My Note").unwrap(), "My Note");
        assert_eq!(sanitize_name("hello-world_2").unwrap(), "hello-world_2");
    }

    #[test]
    fn sanitize_name_strips_special() {
        assert_eq!(sanitize_name("hello!@#world").unwrap(), "helloworld");
    }

    #[test]
    fn sanitize_name_rejects_empty() {
        assert!(sanitize_name("").is_err());
        assert!(sanitize_name("!!!").is_err());
    }

    #[test]
    fn tree_sorts_folders_first() {
        let (dir, nm) = test_notes_dir("sort_order");
        fs::write(dir.join("zebra.md"), "").unwrap();
        fs::create_dir(dir.join("alpha")).unwrap();

        let tree = nm.list_tree().unwrap();
        assert_eq!(tree[0].name, "alpha");
        assert_eq!(tree[0].entry_type, "folder");
        assert_eq!(tree[1].name, "zebra");
        assert_eq!(tree[1].entry_type, "file");
        cleanup(&dir);
    }
}
