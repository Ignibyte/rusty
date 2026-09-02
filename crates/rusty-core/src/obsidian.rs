//! Bridge to the Obsidian desktop app through its official command-line interface.
//!
//! Obsidian 1.12.4 and later answer commands from a second `obsidian` process: that
//! process forwards its arguments over a local socket to the running app and prints
//! the reply. Rusty uses it for the things only the app knows or does well: which
//! pages link where as Obsidian resolves them, link-safe renames, and putting a page
//! on screen. Every call degrades to a plain error when the app is missing, not
//! running, or has the CLI switched off, so callers can report instead of guess.
//!
//! The app must be told about the vault once. [`Obsidian::register`] writes the
//! entry into Obsidian's own config file and turns the CLI on, which is the same
//! thing the vault picker and the settings toggle do by hand. Opening an unknown
//! folder through an `obsidian://open?path=` URL instead leaves the app stuck in the
//! picker for minutes, so registration is the supported route.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::process::Command;

/// Environment variable that overrides the CLI program (default `obsidian`).
pub const PROGRAM_ENV: &str = "RUSTY_OBSIDIAN_CLI";
const DEFAULT_PROGRAM: &str = "obsidian";
const SOCKET_NAME: &str = ".obsidian-cli.sock";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);
const LAUNCH_TIMEOUT: Duration = Duration::from_secs(45);

/// The Obsidian CLI bound to one vault.
#[derive(Debug, Clone)]
pub struct Obsidian {
    program: String,
    vault_path: PathBuf,
    config_path: PathBuf,
    socket_path: PathBuf,
}

/// What the app's config file says about the vault and the CLI toggle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registration {
    /// The vault path is listed in Obsidian's config.
    pub vault_registered: bool,
    /// The "Command line interface" setting is on.
    pub cli_enabled: bool,
}

/// A snapshot of everything the bridge depends on.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// The program the bridge runs.
    pub program: String,
    /// The program is on `PATH` (or exists at the configured path).
    pub installed: bool,
    /// The app is up: its CLI socket exists.
    pub running: bool,
    /// The vault folder the bridge targets.
    pub vault_path: String,
    /// The vault name Obsidian uses, the folder's basename.
    pub vault_name: String,
    /// What the config file says.
    pub registration: Registration,
    /// The app version when it answered, otherwise `None`.
    pub version: Option<String>,
    /// Why `version` is `None`, when the app was asked and did not answer.
    pub error: Option<String>,
}

/// One page that links to a target page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backlink {
    /// Vault-relative path of the linking page.
    pub file: String,
    /// How many links that page holds to the target.
    pub count: u32,
}

/// A wikilink whose target page does not exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unresolved {
    /// The link text as written.
    pub link: String,
    /// How many times it appears in the vault.
    pub count: u32,
    /// Vault-relative paths of the pages that carry it.
    pub sources: Vec<String>,
}

impl Obsidian {
    /// Bind the CLI to a vault, resolving the program, config file and socket from
    /// the environment the way the app itself does.
    pub fn new(vault_path: impl Into<PathBuf>) -> Self {
        let program = std::env::var(PROGRAM_ENV)
            .ok()
            .filter(|p| !p.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_PROGRAM.to_string());
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("obsidian")
            .join("obsidian.json");
        let socket_dir = if cfg!(target_os = "macos") {
            dirs::home_dir()
        } else {
            std::env::var_os("XDG_RUNTIME_DIR")
                .map(PathBuf::from)
                .or_else(dirs::home_dir)
        };
        let socket_path = socket_dir
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(SOCKET_NAME);
        Self::with_paths(program, vault_path, config_path, socket_path)
    }

    /// Bind the CLI with every path given explicitly. Tests use this.
    pub fn with_paths(
        program: impl Into<String>,
        vault_path: impl Into<PathBuf>,
        config_path: impl Into<PathBuf>,
        socket_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            program: program.into(),
            vault_path: vault_path.into(),
            config_path: config_path.into(),
            socket_path: socket_path.into(),
        }
    }

    /// The program the bridge runs.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// The vault folder.
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    /// The name Obsidian gives the vault: the folder's basename.
    pub fn vault_name(&self) -> String {
        self.vault_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "vault".to_string())
    }

    /// Whether the program can be found.
    pub fn is_installed(&self) -> bool {
        let program = Path::new(&self.program);
        if program.components().count() > 1 {
            return program.is_file();
        }
        std::env::var_os("PATH")
            .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(&self.program).is_file()))
            .unwrap_or(false)
    }

    /// Whether the app is up. The app creates its CLI socket at startup and removes
    /// it on exit, so the socket is the cheapest liveness signal there is.
    pub fn is_running(&self) -> bool {
        self.socket_path.exists()
    }

    /// Read the vault and CLI entries from the app's config file.
    pub fn registration(&self) -> Registration {
        let Some(config) = read_config(&self.config_path) else {
            return Registration::default();
        };
        let vault_registered = config
            .get("vaults")
            .and_then(|v| v.as_object())
            .map(|vaults| {
                vaults.values().any(|entry| {
                    entry.get("path").and_then(|p| p.as_str())
                        == Some(&*self.vault_path.to_string_lossy())
                })
            })
            .unwrap_or(false);
        let cli_enabled = config.get("cli").and_then(|c| c.as_bool()).unwrap_or(false);
        Registration {
            vault_registered,
            cli_enabled,
        }
    }

    /// Register the vault with the app and switch the CLI on, by editing the app's
    /// config file in place. Refuses while the app runs, because the app rewrites
    /// that file from memory and would drop the change.
    pub fn register(&self) -> Result<Registration, String> {
        if self.is_running() {
            return Err(
                "Obsidian is running; quit it first so it does not overwrite the config"
                    .to_string(),
            );
        }
        let mut config = read_config(&self.config_path).unwrap_or_else(|| serde_json::json!({}));
        if !config.is_object() {
            config = serde_json::json!({});
        }
        merge_registration(&mut config, &self.vault_path);
        if let Some(dir) = self.config_path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
        }
        let text = serde_json::to_string(&config).map_err(|e| e.to_string())?;
        std::fs::write(&self.config_path, text)
            .map_err(|e| format!("write {}: {e}", self.config_path.display()))?;
        self.configure_vault()?;
        Ok(self.registration())
    }

    /// The vault settings Rusty relies on, written into `<vault>/.obsidian/app.json`:
    /// renames rewrite links without the "Update links?" dialog, and new links are
    /// shortest-path wikilinks, the style the vault already uses. Existing keys are
    /// kept; nothing is written when the values already match. Obsidian picks up the
    /// file live, so this is safe while the app runs.
    pub fn configure_vault(&self) -> Result<(), String> {
        let dir = self.vault_path.join(".obsidian");
        let path = dir.join("app.json");
        let mut config = read_config(&path).unwrap_or_else(|| serde_json::json!({}));
        if !config.is_object() {
            config = serde_json::json!({});
        }
        let wanted = [
            ("alwaysUpdateLinks", serde_json::Value::Bool(true)),
            ("useMarkdownLinks", serde_json::Value::Bool(false)),
            (
                "newLinkFormat",
                serde_json::Value::String("shortest".into()),
            ),
        ];
        let object = config.as_object_mut().expect("app.json is an object");
        let mut changed = false;
        for (key, value) in wanted {
            if object.get(key) != Some(&value) {
                object.insert(key.to_string(), value);
                changed = true;
            }
        }
        if !changed && path.exists() {
            return Ok(());
        }
        std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
        let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
        std::fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))
    }

    /// Ask the running app for its version.
    pub async fn version(&self) -> Result<String, String> {
        self.run("version", &[], &[]).await
    }

    /// Everything the bridge depends on, in one look.
    pub async fn status(&self) -> Status {
        let mut status = Status {
            program: self.program.clone(),
            installed: self.is_installed(),
            running: self.is_running(),
            vault_path: self.vault_path.to_string_lossy().into_owned(),
            vault_name: self.vault_name(),
            registration: self.registration(),
            version: None,
            error: None,
        };
        if status.installed && status.running {
            match self.version().await {
                Ok(v) => status.version = Some(v),
                Err(e) => status.error = Some(e),
            }
        }
        status
    }

    /// Start the app detached and wait until its CLI socket appears. Registers the
    /// vault first when the config does not know it yet, so the app opens straight
    /// into the vault instead of the picker.
    pub async fn launch(&self) -> Result<(), String> {
        if !self.is_installed() {
            return Err(format!("`{}` is not installed", self.program));
        }
        if self.is_running() {
            return Ok(());
        }
        let registration = self.registration();
        if !registration.vault_registered || !registration.cli_enabled {
            self.register()?;
        } else {
            self.configure_vault()?;
        }
        let mut command = std::process::Command::new(&self.program);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }
        command
            .spawn()
            .map_err(|e| format!("start `{}`: {e}", self.program))?;
        let deadline = tokio::time::Instant::now() + LAUNCH_TIMEOUT;
        while tokio::time::Instant::now() < deadline {
            if self.is_running() {
                // The socket exists a moment before the app answers on it.
                tokio::time::sleep(Duration::from_millis(500)).await;
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        Err(format!(
            "Obsidian did not come up within {}s",
            LAUNCH_TIMEOUT.as_secs()
        ))
    }

    /// Launch the app when it is not running, then return.
    pub async fn ensure_running(&self) -> Result<(), String> {
        if self.is_running() {
            Ok(())
        } else {
            self.launch().await
        }
    }

    /// Run one CLI command against the vault. `args` become `key=value` pairs and
    /// `flags` bare words, in the CLI's own syntax.
    pub async fn run(
        &self,
        command: &str,
        args: &[(&str, &str)],
        flags: &[&str],
    ) -> Result<String, String> {
        if !self.is_running() {
            return Err(format!(
                "Obsidian is not running (no CLI socket at {})",
                self.socket_path.display()
            ));
        }
        let mut argv: Vec<String> =
            vec![format!("vault={}", self.vault_name()), command.to_string()];
        argv.extend(args.iter().map(|(k, v)| format!("{k}={v}")));
        argv.extend(flags.iter().map(|f| f.to_string()));
        let child = Command::new(&self.program)
            .args(&argv)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .output();
        let output = tokio::time::timeout(COMMAND_TIMEOUT, child)
            .await
            .map_err(|_| format!("`{} {command}` timed out", self.program))?
            .map_err(|e| format!("run `{} {command}`: {e}", self.program))?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !output.status.success() && stdout.is_empty() {
            return Err(if stderr.is_empty() {
                format!("`{} {command}` failed ({})", self.program, output.status)
            } else {
                stderr
            });
        }
        classify(stdout)
    }

    /// Show a page in the app, launching the app first when needed.
    pub async fn open(&self, path: &str, new_tab: bool) -> Result<String, String> {
        self.ensure_running().await?;
        let path = note_path(path);
        let flags: &[&str] = if new_tab { &["newtab"] } else { &[] };
        let reply = self.run("open", &[("path", &path)], flags).await?;
        Ok(if reply.is_empty() {
            format!("opened {path}")
        } else {
            reply
        })
    }

    /// Pages linking to `path`, with per-page counts, as Obsidian resolves them.
    pub async fn backlinks(&self, path: &str) -> Result<Vec<Backlink>, String> {
        let path = note_path(path);
        let out = self
            .run(
                "backlinks",
                &[("path", &path), ("format", "json")],
                &["counts"],
            )
            .await?;
        parse_backlinks(&out)
    }

    /// Outgoing links of `path`, as vault-relative paths.
    pub async fn links(&self, path: &str) -> Result<Vec<String>, String> {
        let path = note_path(path);
        let out = self.run("links", &[("path", &path)], &[]).await?;
        Ok(parse_lines(&out))
    }

    /// Every wikilink in the vault whose target does not exist.
    pub async fn unresolved(&self) -> Result<Vec<Unresolved>, String> {
        let out = self
            .run("unresolved", &[("format", "json")], &["counts", "verbose"])
            .await?;
        parse_unresolved(&out)
    }

    /// Rename or move a page. Obsidian rewrites every link to it across the vault.
    /// `to` is a vault-relative path with `.md`, or a folder to move into.
    pub async fn rename(&self, from: &str, to: &str) -> Result<String, String> {
        // Without this Obsidian pops an "Update links?" dialog and the CLI returns
        // before anyone answers it, leaving the links stale.
        self.configure_vault()?;
        let from = note_path(from);
        let reply = self
            .run("move", &[("path", &from), ("to", to)], &[])
            .await?;
        Ok(if reply.is_empty() {
            format!("moved {from} to {to}")
        } else {
            reply
        })
    }

    /// The vault-relative path of today's daily note, whether or not it exists.
    pub async fn daily_path(&self) -> Result<String, String> {
        self.run("daily:path", &[], &[]).await
    }
}

/// A vault-relative page path for a brain slug or an existing path: `.md` is added
/// when the last segment has no extension.
pub fn note_path(slug_or_path: &str) -> String {
    let trimmed = slug_or_path.trim().trim_start_matches('/');
    let last = trimmed.rsplit('/').next().unwrap_or(trimmed);
    if last.contains('.') {
        trimmed.to_string()
    } else {
        format!("{trimmed}.md")
    }
}

/// The vault id Obsidian's config keys entries by: 16 hex characters. The app picks
/// random ones; deriving ours from the path keeps re-registration idempotent.
fn vault_id(vault_path: &Path) -> String {
    let digest = Sha256::digest(vault_path.to_string_lossy().as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    hex[..16].to_string()
}

/// Add or refresh the vault entry and switch the CLI on. Returns the vault id.
fn merge_registration(config: &mut serde_json::Value, vault_path: &Path) -> String {
    let path = vault_path.to_string_lossy().into_owned();
    let object = config.as_object_mut().expect("config is an object");
    let vaults = object
        .entry("vaults")
        .or_insert_with(|| serde_json::json!({}));
    if !vaults.is_object() {
        *vaults = serde_json::json!({});
    }
    let vaults = vaults.as_object_mut().expect("vaults is an object");
    let existing = vaults
        .iter()
        .find(|(_, v)| v.get("path").and_then(|p| p.as_str()) == Some(path.as_str()))
        .map(|(k, _)| k.clone());
    let id = existing.unwrap_or_else(|| vault_id(vault_path));
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let entry = vaults
        .entry(id.clone())
        .or_insert_with(|| serde_json::json!({}));
    if let Some(entry) = entry.as_object_mut() {
        entry.insert("path".into(), serde_json::Value::String(path));
        entry.insert("ts".into(), serde_json::json!(ts));
        entry.insert("open".into(), serde_json::Value::Bool(true));
    }
    object.insert("cli".into(), serde_json::Value::Bool(true));
    id
}

fn read_config(path: &Path) -> Option<serde_json::Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// Turn the CLI's reply into a result. The CLI exits 0 even on failure and puts the
/// reason on stdout, so the text is the only signal.
fn classify(output: String) -> Result<String, String> {
    let head = output.lines().next().unwrap_or("").trim();
    if head.starts_with("Error:")
        || head == "Vault not found."
        || head.starts_with("Command line interface is not enabled")
        || head.starts_with("Uncaught Exception")
    {
        Err(output)
    } else {
        Ok(output)
    }
}

/// Non-empty lines of a plain-text reply; a "No ... found." reply is an empty list.
fn parse_lines(output: &str) -> Vec<String> {
    if output.starts_with("No ") && output.ends_with("found.") {
        return Vec::new();
    }
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// Counts come back as strings (`"count": "4"`) today; accept numbers too.
fn count_of(value: Option<&serde_json::Value>) -> u32 {
    match value {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as u32,
        Some(serde_json::Value::String(s)) => s.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

fn parse_json_rows(output: &str) -> Result<Option<Vec<serde_json::Value>>, String> {
    if output.starts_with("No ") && output.ends_with("found.") {
        return Ok(None);
    }
    serde_json::from_str::<Vec<serde_json::Value>>(output)
        .map(Some)
        .map_err(|e| {
            format!(
                "unexpected CLI output ({e}): {}",
                output.chars().take(120).collect::<String>()
            )
        })
}

fn parse_backlinks(output: &str) -> Result<Vec<Backlink>, String> {
    let Some(rows) = parse_json_rows(output)? else {
        return Ok(Vec::new());
    };
    Ok(rows
        .iter()
        .filter_map(|row| {
            let file = row.get("file")?.as_str()?.to_string();
            let count = count_of(row.get("count")).max(1);
            Some(Backlink { file, count })
        })
        .collect())
}

fn parse_unresolved(output: &str) -> Result<Vec<Unresolved>, String> {
    let Some(rows) = parse_json_rows(output)? else {
        return Ok(Vec::new());
    };
    Ok(rows
        .iter()
        .filter_map(|row| {
            let link = row.get("link")?.as_str()?.to_string();
            let count = count_of(row.get("count")).max(1);
            let sources = row
                .get("sources")
                .and_then(|s| s.as_str())
                .map(|s| {
                    s.split(',')
                        .map(str::trim)
                        .filter(|p| !p.is_empty())
                        .map(str::to_string)
                        .collect()
                })
                .unwrap_or_default();
            Some(Unresolved {
                link,
                count,
                sources,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rusty-obsidian-{tag}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn note_path_adds_md_to_slugs_only() {
        assert_eq!(note_path("projects/emmett-hub"), "projects/emmett-hub.md");
        assert_eq!(
            note_path("projects/emmett-hub.md"),
            "projects/emmett-hub.md"
        );
        assert_eq!(note_path("/inbox/x"), "inbox/x.md");
        assert_eq!(note_path("assets/logo.png"), "assets/logo.png");
        assert_eq!(note_path("daily/2026.09.02"), "daily/2026.09.02");
    }

    #[test]
    fn replies_that_are_errors_become_errors() {
        assert!(classify("Error: File \"x.md\" not found.".into()).is_err());
        assert!(classify("Vault not found.".into()).is_err());
        assert!(classify(
            "Command line interface is not enabled. Please turn it on in Settings > General > Advanced.".into()
        )
        .is_err());
        assert_eq!(
            classify("1.13.7 (installer 1.13.7)".into()).unwrap(),
            "1.13.7 (installer 1.13.7)"
        );
        assert_eq!(classify(String::new()).unwrap(), "");
    }

    #[test]
    fn backlinks_parse_with_string_counts_and_empty_replies() {
        let json = r#"[{"file":"companies/ignibyte.md","count":"4"},{"file":"ideas/x.md"}]"#;
        let parsed = parse_backlinks(json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].file, "companies/ignibyte.md");
        assert_eq!(parsed[0].count, 4);
        assert_eq!(parsed[1].count, 1);
        assert!(parse_backlinks("No backlinks found.").unwrap().is_empty());
        assert!(parse_backlinks("garbage").is_err());
    }

    #[test]
    fn unresolved_parse_splits_sources() {
        let json =
            r#"[{"link":"/lab/build","count":"2","sources":"projects/a.md, projects/b.md"}]"#;
        let parsed = parse_unresolved(json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].link, "/lab/build");
        assert_eq!(parsed[0].count, 2);
        assert_eq!(parsed[0].sources, vec!["projects/a.md", "projects/b.md"]);
        assert!(parse_unresolved("No unresolved links found.")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn links_parse_lines() {
        assert_eq!(parse_lines("a.md\nb.md\n"), vec!["a.md", "b.md"]);
        assert!(parse_lines("No links found.").is_empty());
    }

    #[test]
    fn registration_round_trips_through_the_config_file() {
        let dir = temp_dir("register");
        let vault = dir.join("brain");
        std::fs::create_dir_all(&vault).unwrap();
        let config = dir.join("obsidian").join("obsidian.json");
        let socket = dir.join("missing.sock");
        let obsidian = Obsidian::with_paths("obsidian", &vault, &config, &socket);

        assert_eq!(obsidian.vault_name(), "brain");
        assert!(!obsidian.is_running());
        assert_eq!(obsidian.registration(), Registration::default());

        let first = obsidian.register().unwrap();
        assert!(first.vault_registered && first.cli_enabled);
        let saved: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        let vaults = saved["vaults"].as_object().unwrap();
        assert_eq!(vaults.len(), 1);
        let (id, entry) = vaults.iter().next().unwrap();
        assert_eq!(id.len(), 16);
        assert_eq!(entry["path"], vault.to_string_lossy().as_ref());
        assert_eq!(entry["open"], true);
        assert_eq!(saved["cli"], true);

        // A second registration keeps the id and leaves other keys alone.
        let mut with_extra = saved.clone();
        with_extra["frame"] = serde_json::json!("hidden");
        std::fs::write(&config, with_extra.to_string()).unwrap();
        obsidian.register().unwrap();
        let again: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config).unwrap()).unwrap();
        assert_eq!(again["vaults"].as_object().unwrap().len(), 1);
        assert!(again["vaults"].get(id).is_some());
        assert_eq!(again["frame"], "hidden");

        // While the app runs, registration is refused rather than racing the app.
        std::fs::write(&socket, b"").unwrap();
        assert!(obsidian.is_running());
        assert!(obsidian.register().is_err());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn vault_settings_are_merged_into_app_json() {
        let dir = temp_dir("vault-settings");
        let vault = dir.join("brain");
        let app_json = vault.join(".obsidian").join("app.json");
        std::fs::create_dir_all(app_json.parent().unwrap()).unwrap();
        std::fs::write(
            &app_json,
            r#"{"alwaysUpdateLinks":false,"readableLineLength":true}"#,
        )
        .unwrap();
        let obsidian = Obsidian::with_paths(
            "obsidian",
            &vault,
            dir.join("obsidian.json"),
            dir.join("missing.sock"),
        );

        obsidian.configure_vault().unwrap();
        let saved: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&app_json).unwrap()).unwrap();
        assert_eq!(saved["alwaysUpdateLinks"], true);
        assert_eq!(saved["useMarkdownLinks"], false);
        assert_eq!(saved["newLinkFormat"], "shortest");
        assert_eq!(saved["readableLineLength"], true, "unrelated keys survive");

        // A second pass changes nothing on disk.
        let before = std::fs::metadata(&app_json).unwrap().modified().unwrap();
        obsidian.configure_vault().unwrap();
        assert_eq!(
            std::fs::metadata(&app_json).unwrap().modified().unwrap(),
            before
        );

        // A vault without .obsidian/ gets one.
        let fresh = dir.join("fresh");
        std::fs::create_dir_all(&fresh).unwrap();
        Obsidian::with_paths("obsidian", &fresh, dir.join("o.json"), dir.join("s.sock"))
            .configure_vault()
            .unwrap();
        assert!(fresh.join(".obsidian").join("app.json").exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn commands_fail_cleanly_when_the_app_is_down() {
        let dir = temp_dir("down");
        let obsidian = Obsidian::with_paths(
            "obsidian",
            dir.join("brain"),
            dir.join("obsidian.json"),
            dir.join("missing.sock"),
        );
        let err = obsidian.links("projects/x").await.unwrap_err();
        assert!(err.contains("not running"), "{err}");
        let status = obsidian.status().await;
        assert!(!status.running);
        assert_eq!(status.version, None);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[tokio::test]
    async fn launch_reports_a_missing_program() {
        let dir = temp_dir("missing");
        let obsidian = Obsidian::with_paths(
            "rusty-no-such-program-xyz",
            dir.join("brain"),
            dir.join("obsidian.json"),
            dir.join("missing.sock"),
        );
        assert!(!obsidian.is_installed());
        let err = obsidian.launch().await.unwrap_err();
        assert!(err.contains("not installed"), "{err}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
