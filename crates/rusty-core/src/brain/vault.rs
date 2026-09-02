//! Vault filesystem manager for brain pages.
//!
//! Manages the `~/.rusty/brain/` directory structure — creating type directories,
//! reading/writing markdown files, soft-deleting to `archive/`, and generating
//! slugs from titles.

use std::path::{Path, PathBuf};

/// Type directories and their corresponding page types.
const TYPE_DIRS: &[(&str, &str)] = &[
    ("person", "people"),
    ("company", "companies"),
    ("project", "projects"),
    ("concept", "concepts"),
    ("meeting", "meetings"),
    ("idea", "ideas"),
    ("daily", "daily"),
    ("inbox", "inbox"),
    ("conversation", "conversations"),
];

/// Every page type with the vault folder it lives in, in display order.
pub fn page_types() -> &'static [(&'static str, &'static str)] {
    TYPE_DIRS
}

/// Manages markdown files in the brain vault directory.
pub struct VaultManager {
    root: PathBuf,
    /// In-flight git-commit threads. The long-running server fires commits and
    /// forgets them; a short-lived process (rusty-cli) calls [`flush_commits`]
    /// before exit so the commit isn't dropped when the process ends.
    ///
    /// [`flush_commits`]: VaultManager::flush_commits
    pending_commits: std::sync::Mutex<Vec<std::thread::JoinHandle<()>>>,
}

impl VaultManager {
    /// Create a new VaultManager rooted at the given path.
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            pending_commits: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Ensure all type directories, `.templates/`, and `archive/` exist.
    /// Also initializes a git repo in the vault if one doesn't exist.
    pub fn ensure_dirs(&self) -> Result<(), String> {
        for (_, dir) in TYPE_DIRS {
            std::fs::create_dir_all(self.root.join(dir))
                .map_err(|e| format!("Failed to create {dir}/ directory: {e}"))?;
        }
        std::fs::create_dir_all(self.root.join(".templates"))
            .map_err(|e| format!("Failed to create .templates/ directory: {e}"))?;
        std::fs::create_dir_all(self.root.join("archive"))
            .map_err(|e| format!("Failed to create archive/ directory: {e}"))?;

        // Initialize git repo if not present
        if !self.root.join(".git").exists() {
            self.git_init();
        }

        Ok(())
    }

    /// Auto-commit all changes in the vault with a descriptive message.
    ///
    /// Runs git in a background thread so callers (especially the async server)
    /// don't block on the subprocess. The handle is tracked so a short-lived
    /// process can [`flush_commits`] before exiting; finished handles are pruned
    /// on each call to keep the tracking vector bounded in a long-running server.
    ///
    /// [`flush_commits`]: VaultManager::flush_commits
    pub fn git_commit(&self, message: &str) {
        let root = self.root.clone();
        let msg = message.to_string();
        let handle = std::thread::spawn(move || {
            let _ = std::process::Command::new("git")
                .args(["add", "-A"])
                .current_dir(&root)
                .output();
            let _ = std::process::Command::new("git")
                .args([
                    "commit",
                    "-m",
                    &msg,
                    "--allow-empty-message",
                    "--no-gpg-sign",
                ])
                .current_dir(&root)
                .output();
        });
        if let Ok(mut pending) = self.pending_commits.lock() {
            pending.retain(|h| !h.is_finished());
            pending.push(handle);
        }
    }

    /// Wait for all in-flight git-commit threads to finish.
    ///
    /// Call this from a short-lived process (e.g. rusty-cli) before exit so an
    /// auto-commit isn't lost when the process ends. The server never needs it.
    pub fn flush_commits(&self) {
        let handles: Vec<_> = match self.pending_commits.lock() {
            Ok(mut pending) => pending.drain(..).collect(),
            Err(_) => return,
        };
        for handle in handles {
            let _ = handle.join();
        }
    }

    /// Initialize a git repo in the vault.
    fn git_init(&self) {
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&self.root)
            .output();
        // Create .gitignore
        let gitignore = self.root.join(".gitignore");
        if !gitignore.exists() {
            let _ = std::fs::write(&gitignore, ".templates/\narchive/\n");
        }
        // Initial commit
        let _ = std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.root)
            .output();
        let _ = std::process::Command::new("git")
            .args([
                "commit",
                "-m",
                "init: brain vault",
                "--allow-empty-message",
                "--no-gpg-sign",
            ])
            .current_dir(&self.root)
            .output();
    }

    /// Write a page's content to its markdown file.
    ///
    /// The slug determines the file path (e.g., `people/sarah-chen` → `people/sarah-chen.md`).
    pub fn write_page(&self, slug: &str, content: &str) -> Result<(), String> {
        let path = self.resolve_path(slug)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
        std::fs::write(&path, content).map_err(|e| format!("Failed to write page: {e}"))
    }

    /// Read a page's raw markdown content from disk.
    pub fn read_page(&self, slug: &str) -> Result<Option<String>, String> {
        let path = self.resolve_path(slug)?;
        if !path.exists() {
            return Ok(None);
        }
        let content =
            std::fs::read_to_string(&path).map_err(|e| format!("Failed to read page: {e}"))?;
        Ok(Some(content))
    }

    /// Soft-delete a page by moving it to `archive/` with a timestamp suffix.
    pub fn delete_page(&self, slug: &str) -> Result<(), String> {
        let path = self.resolve_path(slug)?;
        if !path.exists() {
            return Err(format!("Page not found: {slug}"));
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let dest = self
            .root
            .join("archive")
            .join(format!("{file_name}_{timestamp}"));

        std::fs::rename(&path, &dest).map_err(|e| format!("Failed to archive page: {e}"))
    }

    /// Check if a page file exists on disk.
    pub fn page_exists(&self, slug: &str) -> bool {
        self.resolve_path(slug).map(|p| p.exists()).unwrap_or(false)
    }

    /// List all markdown files in the vault as (slug, path) pairs.
    #[allow(dead_code)] // Public API for sync_all in future phases
    pub fn list_all_files(&self) -> Result<Vec<(String, PathBuf)>, String> {
        let mut files = Vec::new();
        for (_, dir_name) in TYPE_DIRS {
            let dir = self.root.join(dir_name);
            if !dir.exists() {
                continue;
            }
            let entries =
                std::fs::read_dir(&dir).map_err(|e| format!("Failed to read {dir_name}/: {e}"))?;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("md") {
                    let stem = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let slug = format!("{dir_name}/{stem}");
                    files.push((slug, path));
                }
            }
        }
        Ok(files)
    }

    /// Resolve a slug to its absolute filesystem path.
    ///
    /// Validates the path stays within the vault root.
    pub fn resolve_path(&self, slug: &str) -> Result<PathBuf, String> {
        if slug.contains("..") {
            return Err("Invalid slug: path traversal not allowed".to_string());
        }
        let path = self.root.join(format!("{slug}.md"));
        if !path.starts_with(&self.root) {
            return Err("Invalid slug: outside vault directory".to_string());
        }
        Ok(path)
    }

    /// Get the vault root path.
    #[allow(dead_code)] // Public API for future phases (frontend vault browsing)
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Convert a title string to a URL-safe slug.
///
/// "Sarah Chen" → "sarah-chen"
/// "O'Brien (CEO)" → "obrien-ceo"
pub fn title_to_slug(title: &str) -> Result<String, String> {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c == ' ' || c == '_' || c == '-' {
                '-'
            } else {
                // Strip other characters
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect();

    // Collapse multiple hyphens
    let mut collapsed = String::with_capacity(slug.len());
    let mut last_was_hyphen = false;
    for c in slug.chars() {
        if c == '-' {
            if !last_was_hyphen {
                collapsed.push(c);
            }
            last_was_hyphen = true;
        } else {
            collapsed.push(c);
            last_was_hyphen = false;
        }
    }

    let result = collapsed.trim_matches('-').to_string();
    if result.is_empty() {
        return Err("Title produces empty slug".to_string());
    }
    Ok(result)
}

/// Map a page type to its directory name.
///
/// "person" → "people", "company" → "companies"
pub fn type_to_dir(page_type: &str) -> Result<&'static str, String> {
    TYPE_DIRS
        .iter()
        .find(|(t, _)| *t == page_type)
        .map(|(_, d)| *d)
        .ok_or_else(|| {
            format!(
                "Unknown page type: {page_type}. Valid types: {}",
                TYPE_DIRS
                    .iter()
                    .map(|(t, _)| *t)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_vault(name: &str) -> (PathBuf, VaultManager) {
        let dir =
            std::env::temp_dir().join(format!("rusty_brain_vault_{}_{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let vm = VaultManager::new(dir.clone());
        vm.ensure_dirs().unwrap();
        (dir, vm)
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn title_to_slug_basic() {
        assert_eq!(title_to_slug("Sarah Chen").unwrap(), "sarah-chen");
        assert_eq!(title_to_slug("hello world").unwrap(), "hello-world");
        assert_eq!(title_to_slug("MCP Protocol").unwrap(), "mcp-protocol");
    }

    #[test]
    fn title_to_slug_special_chars() {
        assert_eq!(title_to_slug("O'Brien (CEO)").unwrap(), "obrien-ceo");
        assert_eq!(title_to_slug("hello---world").unwrap(), "hello-world");
        assert_eq!(title_to_slug("  spaces  ").unwrap(), "spaces");
    }

    #[test]
    fn title_to_slug_empty_rejected() {
        assert!(title_to_slug("!!!").is_err());
        assert!(title_to_slug("").is_err());
    }

    #[test]
    fn type_to_dir_pluralization() {
        assert_eq!(type_to_dir("person").unwrap(), "people");
        assert_eq!(type_to_dir("company").unwrap(), "companies");
        assert_eq!(type_to_dir("project").unwrap(), "projects");
        assert_eq!(type_to_dir("daily").unwrap(), "daily");
        assert!(type_to_dir("invalid").is_err());
    }

    #[test]
    fn ensure_dirs_creates_all() {
        let (dir, _vm) = test_vault("ensure_dirs");
        assert!(dir.join("people").is_dir());
        assert!(dir.join("companies").is_dir());
        assert!(dir.join("projects").is_dir());
        assert!(dir.join("concepts").is_dir());
        assert!(dir.join("meetings").is_dir());
        assert!(dir.join("ideas").is_dir());
        assert!(dir.join("daily").is_dir());
        assert!(dir.join("inbox").is_dir());
        assert!(dir.join(".templates").is_dir());
        assert!(dir.join("archive").is_dir());
        cleanup(&dir);
    }

    #[test]
    fn write_and_read_page() {
        let (dir, vm) = test_vault("write_read");
        vm.write_page("people/test-person", "# Test Person\n\nContent here.\n")
            .unwrap();
        let content = vm.read_page("people/test-person").unwrap().unwrap();
        assert_eq!(content, "# Test Person\n\nContent here.\n");
        assert!(vm.page_exists("people/test-person"));
        assert!(!vm.page_exists("people/nonexistent"));
        cleanup(&dir);
    }

    #[test]
    fn delete_moves_to_archive() {
        let (dir, vm) = test_vault("delete_archive");
        vm.write_page("people/to-delete", "content").unwrap();
        assert!(vm.page_exists("people/to-delete"));

        vm.delete_page("people/to-delete").unwrap();
        assert!(!vm.page_exists("people/to-delete"));

        let archive_files: Vec<_> = fs::read_dir(dir.join("archive"))
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(archive_files.len(), 1);
        cleanup(&dir);
    }

    #[test]
    fn list_all_files() {
        let (dir, vm) = test_vault("list_all");
        vm.write_page("people/alice", "# Alice").unwrap();
        vm.write_page("concepts/rust", "# Rust").unwrap();

        let files = vm.list_all_files().unwrap();
        let slugs: Vec<&str> = files.iter().map(|(s, _)| s.as_str()).collect();
        assert!(slugs.contains(&"people/alice"));
        assert!(slugs.contains(&"concepts/rust"));
        assert_eq!(files.len(), 2);
        cleanup(&dir);
    }

    #[test]
    fn path_traversal_rejected() {
        let (dir, vm) = test_vault("traversal");
        assert!(vm.resolve_path("../etc/passwd").is_err());
        assert!(vm.write_page("../../evil", "hack").is_err());
        cleanup(&dir);
    }
}
