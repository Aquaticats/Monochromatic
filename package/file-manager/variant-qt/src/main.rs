//! What:
//!  entry point;
//!  boots a QGuiApplication + QQmlApplicationEngine and loads the QML UI.
//! Why:
//!  cxx-qt requires a QGuiApplication before any QObject exists;
//!  the engine then loads
//!      the qrc-embedded QML,
//!  which instantiates the Rust `AppBridge` QObject.
//!  This minimal
//!      shell proves the Rust <-> Qt/QtQuick pipeline and a native Wayland window before the
//!      directory model and drag-and-drop handlers land.

/// What:
///  the cxx-qt bridge module holding the app's root QObject.
/// Why:
///  declared here (not a separate lib) because this is a binary crate;
///  build.rs points
///      at the same file so the C++ shim is generated.
pub mod cxxqt_object;

/// What:
///  Qt-to-tracing logging bridge (forwards Qt's own messages off-thread).
/// Why:
///  keeps Qt-internal logs off the synchronous default handler;
///  see qt_log.rs.
mod qt_log;

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

/// What:
///  install the process-wide non-blocking tracing subscriber and return its guard.
/// Why:
///  the app logs extensively (repo LOG rule);
///  tracing-appender's NonBlocking writer
///      drains to stderr on a dedicated thread,
///  so logging never blocks the UI/render
///      thread.
///  The returned guard must outlive all logging:
///  dropping it flushes buffered
///      lines and stops the worker.
///  RUST_LOG overrides the default `info` filter.
fn setup_tracing() -> WorkerGuard {
    let (writer, guard) = tracing_appender::non_blocking(std::io::stderr());
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| return EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(writer)
        .init();
    return guard
}

/// What:
///  build the Qt application and QML engine,
///  load the UI,
///  and run the event loop.
/// Why:
///  `QGuiApplication` owns the platform integration (the Wayland QPA plugin on this KWin
///      session);
///  `QQmlApplicationEngine::load` resolves the module URI to the qrc path that
///      build.rs embedded;
///  `exec` blocks until the last window closes.
fn main() {
    // Hold the guard for the whole process: dropping it flushes and stops the log worker.
    let _log_guard = setup_tracing();
    qt_log::install();
    tracing::info!("file-manager-qt starting");

    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    if let Some(qml_engine) = engine.as_mut() {
        qml_engine.load(&QUrl::from(
            "qrc:/qt/qml/dev/monochromatic/file_manager/qml/main.qml",
        ));
    }

    if let Some(gui_app) = app.as_mut() {
        gui_app.exec();
    }
}
