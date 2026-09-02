//! Build script: generates the cxx-qt bridge and bundles the QML module
//! `dev.ignibyte.rusty` (Main.qml plus the Rust-backed types) into the binary.

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    let builder = CxxQtBuilder::new_qml_module(
        QmlModule::new("dev.ignibyte.rusty")
            .version(1, 0)
            .qml_file("qml/Main.qml"),
    )
    .file("src/theme.rs")
    .qt_module("Quick")
    .qt_module("Qml");
    // SAFETY: the closure only adds a diagnostic flag. It touches no include paths,
    // defines or sources the generated code depends on. GCC 16 warns about Qt's own
    // headers (-Wsfinae-incomplete), which is not ours to fix.
    let builder = unsafe {
        builder.cc_builder(|cc| {
            cc.flag_if_supported("-Wno-sfinae-incomplete");
        })
    };
    builder.build();
}
