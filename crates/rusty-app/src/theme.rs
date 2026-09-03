//! The `Theme` QML type: the Omarchy colours, terminal font and colour scheme the
//! shell binds to, plus which tab to open first. `reload()` re-reads the desktop, which
//! is what the theme-set hook will call when Omarchy changes theme.

use core::pin::Pin;
use std::time::Duration;

use cxx_qt::Threading;
use cxx_qt_lib::QString;
use notify::Watcher;

use crate::omarchy::Look;

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// Qt's string type.
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, background)]
        #[qproperty(QString, foreground)]
        #[qproperty(QString, accent)]
        #[qproperty(QString, cursor)]
        #[qproperty(QString, term_font)]
        #[qproperty(QString, term_scheme)]
        #[qproperty(QString, home_dir)]
        #[qproperty(QString, host_name)]
        #[qproperty(QString, facts)]
        #[qproperty(i32, start_tab)]
        #[qproperty(bool, debug)]
        #[qproperty(QString, surface)]
        #[qproperty(QString, surface_alt)]
        #[qproperty(QString, line)]
        #[qproperty(QString, muted)]
        #[qproperty(QString, faint)]
        #[qproperty(QString, code)]
        #[qproperty(QString, code_bg)]
        #[qproperty(QString, tag)]
        #[qproperty(QString, link)]
        #[qproperty(QString, hover)]
        #[qproperty(QString, active)]
        #[qproperty(QString, tokens)]
        #[qproperty(QString, shot_path)]
        #[qproperty(i32, shot_delay)]
        #[qproperty(QString, shot_scene)]
        type Theme = super::ThemeRust;

        /// Re-read the Omarchy theme, font and scheme.
        #[qinvokable]
        fn reload(self: Pin<&mut Theme>);

        /// Follow `omarchy theme set`: watch the current-theme link and reload on change.
        #[qinvokable]
        fn watch(self: Pin<&mut Theme>);
    }

    impl cxx_qt::Threading for Theme {}
}

/// The Rust side of [`qobject::Theme`].
pub struct ThemeRust {
    background: QString,
    foreground: QString,
    accent: QString,
    cursor: QString,
    term_font: QString,
    term_scheme: QString,
    home_dir: QString,
    host_name: QString,
    facts: QString,
    start_tab: i32,
    debug: bool,
    surface: QString,
    surface_alt: QString,
    line: QString,
    muted: QString,
    faint: QString,
    code: QString,
    code_bg: QString,
    tag: QString,
    link: QString,
    hover: QString,
    active: QString,
    /// Every colour token as a JSON object, for the highlighter and the renderer style.
    tokens: QString,
    /// `RUSTY_SHOT=<png>`: grab the window after `shot_delay` ms and quit.
    shot_path: QString,
    shot_delay: i32,
    /// `RUSTY_SHOT_SCENE`: what to show first (`switcher`, `palette`, `edit`,
    /// `right:agent`, `left:search`, ...), comma separated.
    shot_scene: QString,
}

impl Default for ThemeRust {
    fn default() -> Self {
        let mut theme = Self {
            background: QString::default(),
            foreground: QString::default(),
            accent: QString::default(),
            cursor: QString::default(),
            term_font: QString::default(),
            term_scheme: QString::default(),
            home_dir: QString::from(
                &dirs::home_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ),
            host_name: QString::from(&host_name()),
            facts: QString::default(),
            start_tab: start_tab_from_env(),
            debug: std::env::var_os("RUSTY_DEBUG").is_some_and(|v| !v.is_empty()),
            surface: QString::default(),
            surface_alt: QString::default(),
            line: QString::default(),
            muted: QString::default(),
            faint: QString::default(),
            code: QString::default(),
            code_bg: QString::default(),
            tag: QString::default(),
            link: QString::default(),
            hover: QString::default(),
            active: QString::default(),
            tokens: QString::default(),
            shot_path: QString::from(&std::env::var("RUSTY_SHOT").unwrap_or_default()),
            shot_delay: std::env::var("RUSTY_SHOT_DELAY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2500),
            shot_scene: QString::from(&std::env::var("RUSTY_SHOT_SCENE").unwrap_or_default()),
        };
        theme.apply(&Look::gather());
        theme
    }
}

impl ThemeRust {
    fn apply(&mut self, look: &Look) {
        self.background = QString::from(&look.palette.background);
        self.foreground = QString::from(&look.palette.foreground);
        self.accent = QString::from(&look.palette.accent);
        self.cursor = QString::from(&look.palette.cursor);
        self.term_font = QString::from(&look.font);
        self.term_scheme = QString::from(&look.scheme);
        let facts: Vec<String> = look
            .facts
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect();
        self.facts = QString::from(&facts.join("\n"));
        let token =
            |key: &str| QString::from(look.tokens.get(key).map(String::as_str).unwrap_or(""));
        self.surface = token("surface");
        self.surface_alt = token("surface-alt");
        self.line = token("line");
        self.muted = token("muted");
        self.faint = token("faint");
        self.code = token("code");
        self.code_bg = token("code-bg");
        self.tag = token("tag");
        self.link = token("link");
        self.hover = token("hover");
        self.active = token("active");
        self.tokens = QString::from(&tokens_json(&look.tokens));
    }
}

/// The tokens as a JSON object.
fn tokens_json(tokens: &std::collections::BTreeMap<String, String>) -> String {
    serde_json::to_string(tokens).unwrap_or_else(|_| "{}".to_string())
}

/// This machine's host name, which tmux uses as the default terminal title.
fn host_name() -> String {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_default()
}

/// `RUSTY_TAB=3` opens the fourth tab; unset means -1, and the shell then opens the tab
/// it remembers from last time.
fn start_tab_from_env() -> i32 {
    std::env::var("RUSTY_TAB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(-1)
}

impl qobject::Theme {
    /// Watch `~/.config/omarchy/current` from a background thread. Omarchy repoints the
    /// `theme` link there on `omarchy theme set`; every change is coalesced for a moment
    /// and then `reload()` runs on the Qt thread. Safe to call once per object.
    pub fn watch(self: Pin<&mut Self>) {
        let qt_thread = self.qt_thread();
        let dir = crate::omarchy::theme_dir()
            .parent()
            .map(std::path::Path::to_path_buf);
        let Some(dir) = dir else {
            return;
        };
        std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();
            let Ok(mut watcher) = notify::recommended_watcher(move |event| {
                let _ = tx.send(event);
            }) else {
                return;
            };
            if watcher
                .watch(&dir, notify::RecursiveMode::NonRecursive)
                .is_err()
            {
                return;
            }
            while rx.recv().is_ok() {
                // Omarchy touches several files per theme switch; wait for the burst to end.
                std::thread::sleep(Duration::from_millis(400));
                while rx.try_recv().is_ok() {}
                let _ = qt_thread.queue(|theme| theme.reload());
            }
        });
    }

    /// Re-read the desktop and push every property, so bindings update.
    pub fn reload(mut self: Pin<&mut Self>) {
        let look = Look::gather();
        self.as_mut()
            .set_background(QString::from(&look.palette.background));
        self.as_mut()
            .set_foreground(QString::from(&look.palette.foreground));
        self.as_mut()
            .set_accent(QString::from(&look.palette.accent));
        self.as_mut()
            .set_cursor(QString::from(&look.palette.cursor));
        self.as_mut().set_term_font(QString::from(&look.font));
        self.as_mut().set_term_scheme(QString::from(&look.scheme));
        let facts: Vec<String> = look
            .facts
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect();
        self.as_mut().set_facts(QString::from(&facts.join("\n")));
        let token =
            |key: &str| QString::from(look.tokens.get(key).map(String::as_str).unwrap_or(""));
        self.as_mut().set_surface(token("surface"));
        self.as_mut().set_surface_alt(token("surface-alt"));
        self.as_mut().set_line(token("line"));
        self.as_mut().set_muted(token("muted"));
        self.as_mut().set_faint(token("faint"));
        self.as_mut().set_code(token("code"));
        self.as_mut().set_code_bg(token("code-bg"));
        self.as_mut().set_tag(token("tag"));
        self.as_mut().set_link(token("link"));
        self.as_mut().set_hover(token("hover"));
        self.as_mut().set_active(token("active"));
        self.as_mut()
            .set_tokens(QString::from(&tokens_json(&look.tokens)));
    }
}
