//! The `Theme` QML type: the Omarchy colours, terminal font and colour scheme the
//! shell binds to, plus which tab to open first. `reload()` re-reads the desktop, which
//! is what the theme-set hook will call when Omarchy changes theme.

use core::pin::Pin;
use cxx_qt_lib::QString;

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
    }
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
