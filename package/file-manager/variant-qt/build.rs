//! What:
//!  cxx-qt build script;
//!  registers the QML module and compiles the Rust bridge.
//! Why:
//!  pure-Cargo cxx-qt build (no CMake).
//!  `QmlModule` embeds qml/main.qml as a qrc
//!      resource and registers the module URI;
//!  `.files` lists the Rust sources holding
//!      `#[cxx_qt::bridge]` blocks so their C++ shims are generated and linked.
//!  build.rs
//!      is exempt from the require-rustdoc / max-lines linters.

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("dev.monochromatic.file_manager").qml_file("qml/main.qml"),
    )
    // Qt Core is always linked; Qt Gui/Qml come from cxx-qt-lib's `full` feature. Qt Qml
    // requires linking Qt Network on macOS, so declare it here for cross-platform builds.
    .qt_module("Network")
    .files(["src/cxxqt_object.rs"])
    // Hand-written C++ shim installing the Qt -> tracing message handler (src/qt_log.cpp).
    .cpp_file("src/qt_log.cpp")
    .build();
}
