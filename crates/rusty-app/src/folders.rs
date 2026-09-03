//! Folders on the machine for the explorer: list a directory, say what kind of file
//! something is, read a text file, hand a path to the desktop. The disk is not the
//! store: nothing here reaches the back end, and the vault's rules stay with it
//! (TICKET-016, part one).

use std::path::{Path, PathBuf};

use cxx_qt_lib::QString;

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, home)]
        type Folders = super::FoldersRust;

        /// The entries of a directory as JSON: `[{name, path, kind, size}]`, folders
        /// first, names without case, dotfiles skipped; `[]` when it cannot be read.
        #[qinvokable]
        fn list(self: &Folders, path: &QString) -> QString;
        /// `markdown`, `image`, `text` or `other`.
        #[qinvokable]
        fn kind_of(self: &Folders, path: &QString) -> QString;
        /// The text of a file, up to a megabyte; empty when it is not text.
        #[qinvokable]
        fn read_text(self: &Folders, path: &QString) -> QString;
        /// The last path segment.
        #[qinvokable]
        fn base_name(self: &Folders, path: &QString) -> QString;
        /// Hand a path to the desktop's handler (`xdg-open`), detached.
        #[qinvokable]
        fn open_externally(self: &Folders, path: &QString) -> bool;
    }
}

/// The most of a text file the viewer reads.
pub const MAX_TEXT: u64 = 1 << 20;

/// One directory entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

/// The folder reader exposed to QML.
pub struct FoldersRust {
    home: QString,
}

impl Default for FoldersRust {
    fn default() -> Self {
        Self {
            home: QString::from(&home_dir().to_string_lossy().to_string()),
        }
    }
}

impl qobject::Folders {
    /// See the bridge.
    pub fn list(&self, path: &QString) -> QString {
        QString::from(&list_json(Path::new(&path.to_string())))
    }

    /// See the bridge.
    pub fn kind_of(&self, path: &QString) -> QString {
        QString::from(kind_for(Path::new(&path.to_string())))
    }

    /// See the bridge.
    pub fn read_text(&self, path: &QString) -> QString {
        QString::from(&read_text(Path::new(&path.to_string()), MAX_TEXT).unwrap_or_default())
    }

    /// See the bridge.
    pub fn base_name(&self, path: &QString) -> QString {
        QString::from(base_name(&path.to_string()))
    }

    /// See the bridge.
    pub fn open_externally(&self, path: &QString) -> bool {
        std::process::Command::new("xdg-open")
            .arg(path.to_string())
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .is_ok()
    }
}

/// The user's home, from `HOME` or `/`.
pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// The last segment of a path, or the path itself.
pub fn base_name(path: &str) -> &str {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return path;
    }
    trimmed
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(trimmed)
}

/// The entries of `dir`: folders first, then names compared without case, dotfiles
/// skipped. A symlink counts as a folder when it points at one.
pub fn list_dir(dir: &Path) -> Result<Vec<Entry>, String> {
    let read = std::fs::read_dir(dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    let mut entries = Vec::new();
    for item in read.flatten() {
        let name = item.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = item.path();
        let meta = std::fs::metadata(&path).ok();
        let is_dir = meta.as_ref().is_some_and(|m| m.is_dir());
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        entries.push(Entry {
            name,
            path,
            is_dir,
            size,
        });
    }
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(entries)
}

/// `list_dir` as the JSON the explorer reads; an unreadable folder lists as empty.
pub fn list_json(dir: &Path) -> String {
    let entries = list_dir(dir).unwrap_or_default();
    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "name": e.name,
                "path": e.path.to_string_lossy(),
                "kind": if e.is_dir { "folder" } else { "file" },
                "size": e.size,
            })
        })
        .collect();
    serde_json::Value::Array(items).to_string()
}

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico"];
const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown"];

/// `markdown`, `image`, `text` or `other`, by extension first and then by a look at the
/// first eight kilobytes: a NUL byte or invalid UTF-8 means `other`.
pub fn kind_for(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if MARKDOWN_EXTENSIONS.contains(&ext.as_str()) {
        return "markdown";
    }
    if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return "image";
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return "other";
    };
    let mut head = [0u8; 8192];
    let read = {
        use std::io::Read;
        let mut got = 0;
        loop {
            match file.read(&mut head[got..]) {
                Ok(0) => break,
                Ok(n) => got += n,
                Err(_) => return "other",
            }
            if got == head.len() {
                break;
            }
        }
        got
    };
    let head = &head[..read];
    if head.contains(&0) {
        return "other";
    }
    match std::str::from_utf8(head) {
        Ok(_) => "text",
        // A multi-byte character cut by the window's edge is still text.
        Err(e) if e.error_len().is_none() && read == 8192 => "text",
        Err(_) => "other",
    }
}

/// The text of a file, at most `max` bytes (a note marks the cut); `None` when it is
/// not text.
pub fn read_text(path: &Path, max: u64) -> Option<String> {
    if matches!(kind_for(path), "image" | "other") {
        return None;
    }
    let file = std::fs::File::open(path).ok()?;
    let size = file.metadata().ok()?.len();
    let mut text = String::new();
    {
        use std::io::Read;
        let mut limited = file.take(max);
        let mut bytes = Vec::new();
        limited.read_to_end(&mut bytes).ok()?;
        text.push_str(&String::from_utf8_lossy(&bytes));
    }
    if size > max {
        text.push_str("\n… the rest of the file is not shown (over a megabyte)\n");
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("rusty_folders_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("b")).unwrap();
        std::fs::create_dir_all(dir.join("a")).unwrap();
        std::fs::write(dir.join("Zeta.txt"), "one\ntwo\n").unwrap();
        std::fs::write(dir.join("alpha.md"), "# Alpha\n").unwrap();
        std::fs::write(dir.join(".hidden"), "x").unwrap();
        std::fs::write(dir.join("pic.png"), [0x89, b'P', b'N', b'G', 0, 1]).unwrap();
        std::fs::write(dir.join("bin.dat"), [1, 2, 0, 3]).unwrap();
        dir
    }

    #[test]
    fn list_dir_sorts_folders_first_and_skips_dotfiles() {
        let dir = tree("list");
        let names: Vec<String> = list_dir(&dir)
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect();
        assert_eq!(
            names,
            ["a", "b", "alpha.md", "bin.dat", "pic.png", "Zeta.txt"]
        );
        let json = list_json(&dir);
        assert!(json.contains("\"kind\":\"folder\"") && json.contains("\"name\":\"Zeta.txt\""));
        assert_eq!(list_json(&dir.join("missing")), "[]");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn kind_for_reads_the_extension_then_sniffs() {
        let dir = tree("kind");
        assert_eq!(kind_for(&dir.join("alpha.md")), "markdown");
        assert_eq!(kind_for(&dir.join("pic.png")), "image");
        assert_eq!(kind_for(&dir.join("Zeta.txt")), "text");
        assert_eq!(kind_for(&dir.join("bin.dat")), "other");
        assert_eq!(kind_for(&dir.join("missing")), "other");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn read_text_stops_at_binary_and_marks_a_cut() {
        let dir = tree("read");
        assert_eq!(
            read_text(&dir.join("Zeta.txt"), MAX_TEXT).unwrap(),
            "one\ntwo\n"
        );
        assert!(read_text(&dir.join("bin.dat"), MAX_TEXT).is_none());
        let cut = read_text(&dir.join("Zeta.txt"), 3).unwrap();
        assert!(cut.starts_with("one") && cut.contains("not shown"), "{cut}");
        assert_eq!(base_name("/srv/stacks/rusty-v3/"), "rusty-v3");
        assert_eq!(base_name("/"), "/");
        let _ = std::fs::remove_dir_all(dir);
    }
}
