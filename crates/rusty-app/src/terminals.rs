//! The `Terminals` QML type: everything the agent tabs need from the system. Tabs are a
//! QML list; this object persists them, names their tmux sessions, lists the sessions
//! tmux already has, says which agents are installed, and ends a session on request.
//!
//! One tab is one tmux session. Closing a tab detaches; the session keeps running until
//! the user ends it, which is what makes the terminals survive an app restart.

use std::path::PathBuf;
use std::process::Command;

use cxx_qt_lib::{QList, QString, QStringList};

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt's string type.
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        /// Qt's list of strings.
        type QStringList = cxx_qt_lib::QStringList;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, tabs_path)]
        type Terminals = super::TerminalsRust;

        /// The saved tabs as JSON (`[{name, session, program, cwd}]`), or the defaults.
        #[qinvokable]
        fn load(self: &Terminals) -> QString;

        /// Persist the tabs JSON the QML side holds.
        #[qinvokable]
        fn save(self: &Terminals, json: &QString);

        /// The tmux sessions that exist right now.
        #[qinvokable]
        fn sessions(self: &Terminals) -> QStringList;

        /// The agents this machine can run (`claude`, `codex`, `gemini`, `aider`, `opencode`
        /// when on `PATH`) and always `shell`.
        #[qinvokable]
        fn programs(self: &Terminals) -> QStringList;

        /// A tmux session name for a tab label: `rusty-<slug>`, unique among `taken`.
        #[qinvokable]
        fn session_name(self: &Terminals, label: &QString, taken: &QStringList) -> QString;

        /// The command a program name stands for (`shell` becomes the user's shell).
        #[qinvokable]
        fn command_for(self: &Terminals, program: &QString) -> QString;

        /// End a tmux session. Returns whether tmux agreed.
        #[qinvokable]
        fn end_session(self: &Terminals, session: &QString) -> bool;

        /// Show a desktop notification (through `notify-send`, so mako shows it on Omarchy).
        #[qinvokable]
        fn notify(self: &Terminals, title: &QString, body: &QString);
    }
}

/// A saved tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// Rail label.
    pub name: String,
    /// tmux session name.
    pub session: String,
    /// `claude`, `codex` or `shell`.
    pub program: String,
    /// Working directory the session starts in; empty means the home directory.
    pub cwd: String,
}

/// The Rust side of [`qobject::Terminals`].
pub struct TerminalsRust {
    tabs_path: QString,
}

impl Default for TerminalsRust {
    fn default() -> Self {
        Self {
            tabs_path: QString::from(&tabs_path().to_string_lossy().into_owned()),
        }
    }
}

/// `~/.config/rusty/tabs.json`.
pub fn tabs_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config/rusty/tabs.json")
}

/// The agent command-line tools the launch bar looks for, in the order they are shown.
/// Each entry is the binary name; the QML side gives them their labels.
pub const AGENT_CANDIDATES: &[&str] = &["claude", "codex", "gemini", "aider", "opencode"];

/// The tabs a fresh install opens with: one tab for the first agent that is installed, or
/// a shell when none is. The launch bar covers the rest.
pub fn default_tabs(installed: &[String]) -> Vec<Tab> {
    let program = AGENT_CANDIDATES
        .iter()
        .find(|c| installed.iter().any(|i| i == *c))
        .map(|c| c.to_string())
        .unwrap_or_else(|| "shell".to_string());
    let name = tab_label(&program);
    vec![Tab {
        session: session_for(&name, &[]),
        name,
        program,
        cwd: String::new(),
    }]
}

/// The rail label for a program: `claude` is "Claude", `shell` is "Shell".
pub fn tab_label(program: &str) -> String {
    let mut chars = program.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The agent binaries found on `PATH`, in [`AGENT_CANDIDATES`] order.
pub fn installed_agents() -> Vec<String> {
    AGENT_CANDIDATES
        .iter()
        .filter(|p| on_path(p))
        .map(|p| p.to_string())
        .collect()
}

/// Tabs as the JSON the QML side reads. Hand-rolled: four string fields, no serde
/// needed in this crate.
pub fn tabs_to_json(tabs: &[Tab]) -> String {
    let items: Vec<String> = tabs
        .iter()
        .map(|t| {
            format!(
                "{{\"name\":{},\"session\":{},\"program\":{},\"cwd\":{}}}",
                json_string(&t.name),
                json_string(&t.session),
                json_string(&t.program),
                json_string(&t.cwd)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// `rusty-<slug>` for a label; digits are appended while the name is taken.
pub fn session_for(label: &str, taken: &[String]) -> String {
    let mut slug: String = label
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    let slug = slug.trim_matches('-');
    let base = if slug.is_empty() {
        "rusty-tab".to_string()
    } else {
        format!("rusty-{slug}")
    };
    if !taken.iter().any(|t| t == &base) {
        return base;
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|candidate| !taken.iter().any(|t| t == candidate))
        .expect("an unused suffix exists")
}

/// Session names from `tmux list-sessions` output, one per line.
pub fn parse_sessions(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
        .unwrap_or(false)
}

/// The command behind a program name.
pub fn command_for(program: &str) -> String {
    match program {
        "shell" => std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
        other => other.to_string(),
    }
}

fn to_qstringlist(items: &[String]) -> QStringList {
    let mut list = QList::<QString>::default();
    for item in items {
        list.append(QString::from(item));
    }
    QStringList::from(&list)
}

impl qobject::Terminals {
    /// The saved tabs as JSON, or the defaults when nothing was saved yet.
    pub fn load(&self) -> QString {
        match std::fs::read_to_string(tabs_path()) {
            Ok(text) if text.trim_start().starts_with('[') => QString::from(&text),
            _ => QString::from(&tabs_to_json(&default_tabs(&installed_agents()))),
        }
    }

    /// Persist the tabs JSON.
    pub fn save(&self, json: &QString) {
        let path = tabs_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, json.to_string());
    }

    /// Live tmux sessions.
    pub fn sessions(&self) -> QStringList {
        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output();
        let names = match output {
            Ok(o) if o.status.success() => parse_sessions(&String::from_utf8_lossy(&o.stdout)),
            _ => Vec::new(),
        };
        to_qstringlist(&names)
    }

    /// Installed agents plus `shell`, in launch-bar order.
    pub fn programs(&self) -> QStringList {
        let mut names = installed_agents();
        names.push("shell".to_string());
        to_qstringlist(&names)
    }

    /// A unique `rusty-<slug>` session name.
    pub fn session_name(&self, label: &QString, taken: &QStringList) -> QString {
        let taken: Vec<String> = QList::<QString>::from(taken)
            .iter()
            .map(|s| s.to_string())
            .collect();
        QString::from(&session_for(&label.to_string(), &taken))
    }

    /// The command behind a program name.
    pub fn command_for(&self, program: &QString) -> QString {
        QString::from(&command_for(&program.to_string()))
    }

    /// Desktop notification; quietly does nothing where `notify-send` is missing.
    pub fn notify(&self, title: &QString, body: &QString) {
        let _ = Command::new("notify-send")
            .args([
                "--app-name=Rusty",
                "--icon=com.ignibyte.rusty",
                &title.to_string(),
                &body.to_string(),
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    /// End a tmux session.
    pub fn end_session(&self, session: &QString) -> bool {
        Command::new("tmux")
            .args(["kill-session", "-t", &session.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_follow_what_is_installed() {
        let both = default_tabs(&["codex".to_string(), "claude".to_string()]);
        assert_eq!(
            tabs_to_json(&both),
            r#"[{"name":"Claude","session":"rusty-claude","program":"claude","cwd":""}]"#
        );
        let codex_only = default_tabs(&["codex".to_string()]);
        assert_eq!(codex_only[0].program, "codex");
        assert_eq!(codex_only[0].name, "Codex");
        let none = default_tabs(&[]);
        assert_eq!(none[0].program, "shell");
        assert_eq!(none[0].session, "rusty-shell");
        assert_eq!(tab_label("opencode"), "Opencode");
        assert_eq!(json_string("a \"quoted\" tab\n"), r#""a \"quoted\" tab\n""#);
    }

    #[test]
    fn session_names_are_slugs_and_unique() {
        assert_eq!(session_for("Claude", &[]), "rusty-claude");
        assert_eq!(
            session_for("  Work / Ignibyte  ", &[]),
            "rusty-work-ignibyte"
        );
        assert_eq!(session_for("***", &[]), "rusty-tab");
        let taken = vec!["rusty-claude".to_string(), "rusty-claude-2".to_string()];
        assert_eq!(session_for("Claude", &taken), "rusty-claude-3");
    }

    #[test]
    fn tmux_output_parses_and_shell_resolves() {
        assert_eq!(
            parse_sessions("ai\nrusty-claude\n\n"),
            vec!["ai", "rusty-claude"]
        );
        assert!(!command_for("shell").is_empty());
        assert_eq!(command_for("codex"), "codex");
    }
}
