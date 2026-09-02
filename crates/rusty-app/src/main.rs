//! Rusty's desktop app: a QML shell for Omarchy with native Claude Code and Codex
//! terminals, and tabs for tasks, the brain, notes, memories, skills and settings.
//!
//! The window is `qml/Main.qml`, bundled into the binary as the QML module
//! `dev.ignibyte.rusty`. Rust supplies the `Theme` type (Omarchy colours, terminal font
//! and colour scheme) and, as the milestones land, the models behind each tab. Data
//! comes from `rusty-mcp` over local HTTP; the app holds no store of its own.

mod backend;
mod omarchy;
mod terminals;
mod theme;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QString, QUrl};

fn main() {
    // Wayland first on Omarchy; Qt still falls back to X11 when there is no compositor.
    if std::env::var_os("QT_QPA_PLATFORM").is_none() {
        std::env::set_var("QT_QPA_PLATFORM", "wayland;xcb");
    }
    // The terminal widget lists colour schemes from the directory named by this
    // variable, so the scheme generated from the Omarchy theme is found there.
    if std::env::var_os("COLORSCHEMES_DIR").is_none() {
        let dir = omarchy::scheme_dir();
        let _ = std::fs::create_dir_all(&dir);
        std::env::set_var("COLORSCHEMES_DIR", &dir);
    }
    let mut app = QGuiApplication::new();
    if let Some(mut app) = app.as_mut() {
        app.as_mut().set_application_name(&QString::from("rusty"));
        app.as_mut()
            .set_application_display_name(&QString::from("Rusty"));
        app.as_mut()
            .set_organization_name(&QString::from("Ignibyte"));
        app.as_mut()
            .set_organization_domain(&QString::from("ignibyte.com"));
    }
    let mut engine = QQmlApplicationEngine::new();
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/dev/ignibyte/rusty/qml/Main.qml"));
    }
    let code = match app.as_mut() {
        Some(app) => app.exec(),
        None => 1,
    };
    std::process::exit(code);
}
