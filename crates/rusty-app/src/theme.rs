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
        #[qproperty(QString, facts)]
        #[qproperty(i32, start_tab)]
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
    facts: QString,
    start_tab: i32,
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
            facts: QString::default(),
            start_tab: start_tab_from_env(),
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
    }
}

/// `RUSTY_TAB=3` opens the fourth tab; anything else opens the first.
fn start_tab_from_env() -> i32 {
    std::env::var("RUSTY_TAB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
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
    }
}
