//! The `Theme` QML type: every colour the shell binds to, the faces, the corner
//! radius and the CRT overlay switch, computed from the chosen skin (see
//! [`crate::skin`]) and from what the Omarchy desktop gives (the terminal font and
//! colour scheme, and the palette when the skin follows Omarchy). `select()` switches
//! the skin at runtime; `reload()` re-reads the desktop, which the watcher calls when
//! `omarchy theme set` runs.

use core::pin::Pin;
use std::collections::BTreeMap;
use std::time::Duration;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use notify::Watcher;

use crate::omarchy::{ansi_tokens, Look};
use crate::skin::{self, Choice, Roles};

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
        #[qproperty(QString, ui_font)]
        #[qproperty(QString, home_dir)]
        #[qproperty(QString, host_name)]
        #[qproperty(QString, facts)]
        #[qproperty(i32, start_tab)]
        #[qproperty(bool, debug)]
        #[qproperty(QString, surface)]
        #[qproperty(QString, surface_alt)]
        #[qproperty(QString, panel)]
        #[qproperty(QString, panel2)]
        #[qproperty(QString, panel3)]
        #[qproperty(QString, line)]
        #[qproperty(QString, line_bright)]
        #[qproperty(QString, muted)]
        #[qproperty(QString, faint)]
        #[qproperty(QString, bright)]
        #[qproperty(QString, accent_soft)]
        #[qproperty(QString, gold)]
        #[qproperty(QString, alive)]
        #[qproperty(QString, red)]
        #[qproperty(QString, code)]
        #[qproperty(QString, code_bg)]
        #[qproperty(QString, tag)]
        #[qproperty(QString, link)]
        #[qproperty(QString, hover)]
        #[qproperty(QString, active)]
        #[qproperty(QString, selection)]
        #[qproperty(i32, radius)]
        #[qproperty(bool, scanlines)]
        #[qproperty(bool, dark)]
        #[qproperty(QString, source)]
        #[qproperty(QString, theme_name)]
        #[qproperty(QString, theme_title)]
        #[qproperty(QString, choices)]
        #[qproperty(QString, tokens)]
        #[qproperty(QString, shot_path)]
        #[qproperty(i32, shot_delay)]
        #[qproperty(QString, shot_scene)]
        type Theme = super::ThemeRust;

        /// Re-read the Omarchy theme, font and scheme, keeping the chosen skin.
        #[qinvokable]
        fn reload(self: Pin<&mut Theme>);

        /// Follow `omarchy theme set`: watch the current-theme link and reload on change.
        #[qinvokable]
        fn watch(self: Pin<&mut Theme>);

        /// Switch the skin: a JSON object `{source, name, scanlines}` as the shell
        /// keeps it in the workspace state. Unknown names fall back to the first preset.
        #[qinvokable]
        fn select(self: Pin<&mut Theme>, choice: &QString);
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
    /// The face for every label: the skin's, or the terminal font.
    ui_font: QString,
    home_dir: QString,
    host_name: QString,
    facts: QString,
    start_tab: i32,
    debug: bool,
    surface: QString,
    surface_alt: QString,
    panel: QString,
    panel2: QString,
    panel3: QString,
    line: QString,
    line_bright: QString,
    muted: QString,
    faint: QString,
    bright: QString,
    accent_soft: QString,
    gold: QString,
    alive: QString,
    red: QString,
    code: QString,
    code_bg: QString,
    tag: QString,
    link: QString,
    hover: QString,
    active: QString,
    selection: QString,
    radius: i32,
    scanlines: bool,
    dark: bool,
    /// The chosen source: `preset`, `omarchy` or `file`.
    source: QString,
    /// The chosen preset id or file stem.
    theme_name: QString,
    /// What Settings shows for the choice.
    theme_title: QString,
    /// Every skin that can be picked, as JSON (see [`skin::choices_json`]).
    choices: QString,
    /// Every colour token as a JSON object, for the highlighter and the renderer style.
    tokens: QString,
    /// `RUSTY_SHOT=<png>`: grab the window after `shot_delay` ms and quit.
    shot_path: QString,
    shot_delay: i32,
    /// `RUSTY_SHOT_SCENE`: what to show first (`switcher`, `palette`, `edit`,
    /// `right:agent`, `left:search`, ...), comma separated.
    shot_scene: QString,
    /// The skin in force.
    choice: Choice,
}

/// Everything the properties are set from, computed in one place for the constructor
/// and for `reload`/`select` alike.
struct Computed {
    look: Look,
    roles: Roles,
    title: String,
    tokens: BTreeMap<String, String>,
}

fn compute(choice: &Choice) -> Computed {
    let look = Look::gather();
    let (roles, title) = skin::resolve(choice, &look.palette, &ansi_tokens());
    let tokens = skin::tokens(&roles);
    Computed {
        look,
        roles,
        title,
        tokens,
    }
}

impl Default for ThemeRust {
    fn default() -> Self {
        let choice = startup_choice();
        let c = compute(&choice);
        let token = |key: &str| QString::from(c.tokens.get(key).map(String::as_str).unwrap_or(""));
        Self {
            background: token("background"),
            foreground: token("foreground"),
            accent: token("accent"),
            cursor: QString::from(&c.look.palette.cursor),
            term_font: QString::from(&c.look.font),
            term_scheme: QString::from(&c.look.scheme),
            ui_font: QString::from(&ui_font(&c.roles, &c.look.font)),
            home_dir: QString::from(
                &dirs::home_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            ),
            host_name: QString::from(&host_name()),
            facts: QString::from(&facts_text(&c)),
            start_tab: start_tab_from_env(),
            debug: std::env::var_os("RUSTY_DEBUG").is_some_and(|v| !v.is_empty()),
            surface: token("surface"),
            surface_alt: token("surface-alt"),
            panel: token("panel"),
            panel2: token("panel2"),
            panel3: token("panel3"),
            line: token("line"),
            line_bright: token("line-bright"),
            muted: token("muted"),
            faint: token("faint"),
            bright: token("bright"),
            accent_soft: token("accent-soft"),
            gold: token("gold"),
            alive: token("alive"),
            red: token("red"),
            code: token("code"),
            code_bg: token("code-bg"),
            tag: token("tag"),
            link: token("link"),
            hover: token("hover"),
            active: token("active"),
            selection: token("selection"),
            radius: c.roles.radius,
            scanlines: choice.scanlines,
            dark: c.tokens.get("dark").is_some_and(|d| d == "true"),
            source: QString::from(&choice.source),
            theme_name: QString::from(&choice.name),
            theme_title: QString::from(&c.title),
            choices: QString::from(&skin::choices_json()),
            tokens: QString::from(&tokens_json(&c.tokens)),
            shot_path: QString::from(&std::env::var("RUSTY_SHOT").unwrap_or_default()),
            shot_delay: std::env::var("RUSTY_SHOT_DELAY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2500),
            shot_scene: QString::from(&std::env::var("RUSTY_SHOT_SCENE").unwrap_or_default()),
            choice,
        }
    }
}

/// The skin's face, or the terminal font when the skin names none.
fn ui_font(roles: &Roles, term_font: &str) -> String {
    if roles.font.trim().is_empty() {
        term_font.to_string()
    } else {
        roles.font.clone()
    }
}

/// The lines Settings shows under "This machine".
fn facts_text(c: &Computed) -> String {
    let mut facts: Vec<String> = c
        .look
        .facts
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect();
    facts.push(format!("skin: {}", c.title));
    facts.push(format!(
        "theme files: {}",
        skin::themes_dir().to_string_lossy()
    ));
    facts.join("\n")
}

/// The tokens as a JSON object.
fn tokens_json(tokens: &BTreeMap<String, String>) -> String {
    serde_json::to_string(tokens).unwrap_or_else(|_| "{}".to_string())
}

/// The skin the shell saved last time, read from the workspace state before the
/// window exists, so the first paint and the application font are already right.
/// `RUSTY_THEME=<source>:<name>` overrides it (screenshots of another skin).
pub fn startup_choice() -> Choice {
    if let Some(spec) = std::env::var("RUSTY_THEME").ok().filter(|v| !v.is_empty()) {
        let (source, name) = spec.split_once(':').unwrap_or((spec.as_str(), ""));
        return Choice {
            source: source.to_string(),
            name: name.to_string(),
            scanlines: std::env::var("RUSTY_SCANLINES").map_or(true, |v| v != "0"),
        };
    }
    let text = std::fs::read_to_string(crate::terminals::state_path()).unwrap_or_default();
    serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|state| state.get("theme")?.as_str().map(str::to_string))
        .and_then(|inner| serde_json::from_str::<Choice>(&inner).ok())
        .unwrap_or_default()
}

/// The face and size the application font should take at start, from the saved skin.
pub fn startup_font() -> (String, i32) {
    let choice = startup_choice();
    let c = compute(&choice);
    (ui_font(&c.roles, &c.look.font), 12)
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
        let choice = self.rust().choice.clone();
        self.as_mut().apply(&choice);
    }

    /// See the bridge: switch the skin and push every property.
    pub fn select(mut self: Pin<&mut Self>, choice: &QString) {
        let parsed: Choice = serde_json::from_str(&choice.to_string()).unwrap_or_default();
        self.as_mut().rust_mut().choice = parsed.clone();
        self.as_mut().apply(&parsed);
    }

    fn apply(mut self: Pin<&mut Self>, choice: &Choice) {
        let c = compute(choice);
        let token = |key: &str| QString::from(c.tokens.get(key).map(String::as_str).unwrap_or(""));
        self.as_mut().set_background(token("background"));
        self.as_mut().set_foreground(token("foreground"));
        self.as_mut().set_accent(token("accent"));
        self.as_mut()
            .set_cursor(QString::from(&c.look.palette.cursor));
        self.as_mut().set_term_font(QString::from(&c.look.font));
        self.as_mut().set_term_scheme(QString::from(&c.look.scheme));
        self.as_mut()
            .set_ui_font(QString::from(&ui_font(&c.roles, &c.look.font)));
        self.as_mut().set_facts(QString::from(&facts_text(&c)));
        self.as_mut().set_surface(token("surface"));
        self.as_mut().set_surface_alt(token("surface-alt"));
        self.as_mut().set_panel(token("panel"));
        self.as_mut().set_panel2(token("panel2"));
        self.as_mut().set_panel3(token("panel3"));
        self.as_mut().set_line(token("line"));
        self.as_mut().set_line_bright(token("line-bright"));
        self.as_mut().set_muted(token("muted"));
        self.as_mut().set_faint(token("faint"));
        self.as_mut().set_bright(token("bright"));
        self.as_mut().set_accent_soft(token("accent-soft"));
        self.as_mut().set_gold(token("gold"));
        self.as_mut().set_alive(token("alive"));
        self.as_mut().set_red(token("red"));
        self.as_mut().set_code(token("code"));
        self.as_mut().set_code_bg(token("code-bg"));
        self.as_mut().set_tag(token("tag"));
        self.as_mut().set_link(token("link"));
        self.as_mut().set_hover(token("hover"));
        self.as_mut().set_active(token("active"));
        self.as_mut().set_selection(token("selection"));
        self.as_mut().set_radius(c.roles.radius);
        self.as_mut().set_scanlines(choice.scanlines);
        self.as_mut()
            .set_dark(c.tokens.get("dark").is_some_and(|d| d == "true"));
        self.as_mut().set_source(QString::from(&choice.source));
        self.as_mut().set_theme_name(QString::from(&choice.name));
        self.as_mut().set_theme_title(QString::from(&c.title));
        self.as_mut()
            .set_choices(QString::from(&skin::choices_json()));
        self.as_mut()
            .set_tokens(QString::from(&tokens_json(&c.tokens)));
    }
}
