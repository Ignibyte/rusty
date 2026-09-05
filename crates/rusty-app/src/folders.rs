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

        // Part two (TICKET-019): the writes. Each answers JSON — `{"ok":true,"path":…}` or
        // `{"ok":false,"error":…}` — so the explorer can refresh or show the reason.

        /// An empty file `name` inside `dir`; refuses a name that exists.
        #[qinvokable]
        fn create_file(self: &Folders, dir: &QString, name: &QString) -> QString;
        /// A folder `name` inside `dir`; refuses a name that exists.
        #[qinvokable]
        fn create_dir(self: &Folders, dir: &QString, name: &QString) -> QString;
        /// Rename the entry at `path` to `name` in the same folder.
        #[qinvokable]
        fn rename_entry(self: &Folders, path: &QString, name: &QString) -> QString;
        /// Move the entry at `path` into the folder `into`, keeping its name.
        #[qinvokable]
        fn move_entry(self: &Folders, path: &QString, into: &QString) -> QString;
        /// Move the entry at `path` to the XDG trash, with a record of where it came from.
        #[qinvokable]
        fn trash(self: &Folders, path: &QString) -> QString;
        /// Write `text` to `path` atomically (a sibling temp file, then a rename).
        #[qinvokable]
        fn write_text(self: &Folders, path: &QString, text: &QString) -> QString;
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

    /// See the bridge.
    pub fn create_file(&self, dir: &QString, name: &QString) -> QString {
        outcome(create_file(Path::new(&dir.to_string()), &name.to_string()))
    }

    /// See the bridge.
    pub fn create_dir(&self, dir: &QString, name: &QString) -> QString {
        outcome(create_dir(Path::new(&dir.to_string()), &name.to_string()))
    }

    /// See the bridge.
    pub fn rename_entry(&self, path: &QString, name: &QString) -> QString {
        outcome(rename_entry(
            Path::new(&path.to_string()),
            &name.to_string(),
        ))
    }

    /// See the bridge.
    pub fn move_entry(&self, path: &QString, into: &QString) -> QString {
        outcome(move_entry(
            Path::new(&path.to_string()),
            Path::new(&into.to_string()),
        ))
    }

    /// See the bridge.
    pub fn trash(&self, path: &QString) -> QString {
        outcome(trash_to(Path::new(&path.to_string()), &trash_root()))
    }

    /// See the bridge.
    pub fn write_text(&self, path: &QString, text: &QString) -> QString {
        outcome(write_atomic(
            Path::new(&path.to_string()),
            &text.to_string(),
        ))
    }
}

/// The JSON a write answers with.
fn outcome(result: Result<PathBuf, String>) -> QString {
    let value = match result {
        Ok(path) => serde_json::json!({ "ok": true, "path": path.to_string_lossy() }),
        Err(error) => serde_json::json!({ "ok": false, "error": error }),
    };
    QString::from(&value.to_string())
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

/// A name for a new or renamed entry: one path segment, and not `.` or `..`.
pub fn validate_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("a name is needed".to_string());
    }
    if name == "." || name == ".." {
        return Err(format!("{name:?} is not a name"));
    }
    if name.contains('/') || name.contains('\0') {
        return Err("a name cannot contain a slash".to_string());
    }
    Ok(name)
}

/// `dir/name`, provided nothing is there yet (a broken symlink counts as something).
fn fresh(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    if std::fs::symlink_metadata(&path).is_ok() {
        return Err(format!("{} exists", path.display()));
    }
    Ok(path)
}

/// An empty file `name` in `dir`.
pub fn create_file(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let path = fresh(dir, validate_name(name)?)?;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("create {}: {e}", path.display()))?;
    Ok(path)
}

/// A folder `name` in `dir`.
pub fn create_dir(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let path = fresh(dir, validate_name(name)?)?;
    std::fs::create_dir(&path).map_err(|e| format!("create {}: {e}", path.display()))?;
    Ok(path)
}

/// The entry at `path`, renamed to `name` in the same folder.
pub fn rename_entry(path: &Path, name: &str) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "nothing to rename".to_string())?;
    let to = fresh(parent, validate_name(name)?)?;
    std::fs::rename(path, &to).map_err(|e| format!("rename {}: {e}", path.display()))?;
    Ok(to)
}

/// The entry at `path`, moved into the folder `into` under its own name. A folder is
/// refused a move into itself or below itself.
pub fn move_entry(path: &Path, into: &Path) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "nothing to move".to_string())?;
    if !into.is_dir() {
        return Err(format!("{} is not a folder", into.display()));
    }
    if into.starts_with(path) {
        return Err("a folder cannot move into itself".to_string());
    }
    let to = fresh(into, name)?;
    std::fs::rename(path, &to).map_err(|e| format!("move {}: {e}", path.display()))?;
    Ok(to)
}

/// The XDG home trash: `$XDG_DATA_HOME/Trash`, or `~/.local/share/Trash`.
pub fn trash_root() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| home_dir().join(".local/share"))
        .join("Trash")
}

/// Percent-encode a path for a `.trashinfo` record, as the spec asks: everything but
/// the unreserved characters and the slash.
fn percent_encode(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for byte in path.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/');
        if keep {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn copy_tree(from: &Path, to: &Path) -> Result<(), String> {
    let meta = std::fs::symlink_metadata(from).map_err(|e| format!("{}: {e}", from.display()))?;
    if meta.is_dir() {
        std::fs::create_dir(to).map_err(|e| format!("{}: {e}", to.display()))?;
        let read = std::fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))?;
        for entry in read.flatten() {
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else if meta.file_type().is_symlink() {
        let target = std::fs::read_link(from).map_err(|e| format!("{}: {e}", from.display()))?;
        std::os::unix::fs::symlink(target, to).map_err(|e| format!("{}: {e}", to.display()))?;
    } else {
        std::fs::copy(from, to).map_err(|e| format!("{}: {e}", from.display()))?;
    }
    Ok(())
}

fn remove_tree(path: &Path) -> Result<(), String> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let result = if meta.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    result.map_err(|e| format!("{}: {e}", path.display()))
}

/// Move `path` into the trash under `root` the way the XDG trash spec lays it out: the
/// entry goes to `files/`, and `info/<name>.trashinfo` says where it came from and when.
/// A name already in `files/` takes a numeric suffix. The record is written first, so a
/// failure between the two leaves a record without a file — which file managers
/// tolerate — rather than a file nobody can restore. Across devices, where `rename`
/// cannot, the entry is copied and then removed.
pub fn trash_to(path: &Path, root: &Path) -> Result<PathBuf, String> {
    use std::os::unix::fs::MetadataExt;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "nothing to delete".to_string())?;
    let source = std::fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let files = root.join("files");
    let info = root.join("info");
    std::fs::create_dir_all(&files).map_err(|e| format!("trash {}: {e}", files.display()))?;
    std::fs::create_dir_all(&info).map_err(|e| format!("trash {}: {e}", info.display()))?;
    let mut chosen = name.to_string();
    let mut n = 1;
    while std::fs::symlink_metadata(files.join(&chosen)).is_ok()
        || info.join(format!("{chosen}.trashinfo")).exists()
    {
        chosen = format!("{name}.{n}");
        n += 1;
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let record = format!(
        "[Trash Info]\nPath={}\nDeletionDate={}\n",
        percent_encode(&absolute.to_string_lossy()),
        chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
    );
    let record_path = info.join(format!("{chosen}.trashinfo"));
    std::fs::write(&record_path, record)
        .map_err(|e| format!("trash record {}: {e}", record_path.display()))?;
    let dest = files.join(&chosen);
    let same_device = std::fs::metadata(&files)
        .map(|m| m.dev() == source.dev())
        .unwrap_or(true);
    if same_device {
        std::fs::rename(path, &dest).map_err(|e| format!("trash {}: {e}", path.display()))?;
    } else {
        copy_tree(path, &dest)?;
        remove_tree(path)?;
    }
    Ok(dest)
}

/// Write `text` to `path` through a sibling temp file and a rename, so a crash mid-write
/// leaves the old file whole rather than a truncated one. The original's permissions are
/// kept, so an edited script stays executable.
pub fn write_atomic(path: &Path, text: &str) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "nowhere to write".to_string())?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "nothing to write".to_string())?;
    let tmp = parent.join(format!(".{name}.rusty-tmp"));
    std::fs::write(&tmp, text).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    if let Ok(meta) = std::fs::metadata(path) {
        let _ = std::fs::set_permissions(&tmp, meta.permissions());
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("write {}: {e}", path.display()));
    }
    Ok(path.to_path_buf())
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

    #[test]
    fn validate_name_refuses_separators_and_dots() {
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
        assert!(validate_name(".").is_err());
        assert!(validate_name("..").is_err());
        assert!(validate_name("a/b").is_err());
        assert_eq!(validate_name("  notes.md ").unwrap(), "notes.md");
    }

    #[test]
    fn create_file_and_dir_refuse_what_exists() {
        let dir = tree("create");
        let f = create_file(&dir, "new.txt").unwrap();
        assert!(f.is_file() && std::fs::read(&f).unwrap().is_empty());
        assert!(create_file(&dir, "new.txt").unwrap_err().contains("exists"));
        let d = create_dir(&dir, "c").unwrap();
        assert!(d.is_dir());
        assert!(create_dir(&dir, "a").unwrap_err().contains("exists"));
        assert!(create_file(&dir, "x/y").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_and_move_stay_inside_the_tree() {
        let dir = tree("move");
        let renamed = rename_entry(&dir.join("Zeta.txt"), "zeta2.txt").unwrap();
        assert_eq!(renamed, dir.join("zeta2.txt"));
        assert!(!dir.join("Zeta.txt").exists());
        assert!(rename_entry(&dir.join("alpha.md"), "zeta2.txt")
            .unwrap_err()
            .contains("exists"));
        let moved = move_entry(&dir.join("zeta2.txt"), &dir.join("b")).unwrap();
        assert_eq!(moved, dir.join("b").join("zeta2.txt"));
        assert!(move_entry(&dir.join("b"), &dir.join("b"))
            .unwrap_err()
            .contains("itself"));
        assert!(move_entry(&dir.join("alpha.md"), &dir.join("alpha.md"))
            .unwrap_err()
            .contains("not a folder"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn trash_moves_the_entry_and_records_where_it_came_from() {
        let dir = tree("trash");
        let root = dir.join("trash-root");
        let gone = trash_to(&dir.join("Zeta.txt"), &root).unwrap();
        assert_eq!(gone, root.join("files").join("Zeta.txt"));
        assert!(!dir.join("Zeta.txt").exists());
        assert_eq!(std::fs::read_to_string(&gone).unwrap(), "one\ntwo\n");
        let record = std::fs::read_to_string(root.join("info").join("Zeta.txt.trashinfo")).unwrap();
        assert!(record.starts_with("[Trash Info]\n"), "{record}");
        assert!(
            record.contains(&format!(
                "Path={}",
                percent_encode(&dir.join("Zeta.txt").to_string_lossy())
            )),
            "{record}"
        );
        assert!(record.contains("DeletionDate=20"), "{record}");
        // A folder goes whole, and a second entry of the same name takes a suffix.
        trash_to(&dir.join("a"), &root).unwrap();
        std::fs::create_dir(dir.join("a")).unwrap();
        let again = trash_to(&dir.join("a"), &root).unwrap();
        assert_eq!(again, root.join("files").join("a.1"));
        assert!(root.join("info").join("a.1.trashinfo").is_file());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_atomic_replaces_whole_and_keeps_the_mode() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tree("write");
        let script = dir.join("run.sh");
        std::fs::write(&script, "#!/bin/sh\necho old\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        write_atomic(&script, "#!/bin/sh\necho new\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(&script).unwrap(),
            "#!/bin/sh\necho new\n"
        );
        assert_eq!(
            std::fs::metadata(&script).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert!(!dir.join(".run.sh.rusty-tmp").exists());
        assert!(write_atomic(&dir.join("missing").join("x"), "y").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
