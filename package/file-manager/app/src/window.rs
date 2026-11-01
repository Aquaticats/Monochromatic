//! Top-level application window construction.

/// What:
///  imports the GTK widget-extension traits (builders,
///  `present`).
/// Why:
///  the shell presents an `ApplicationWindow` via prelude trait methods.
use gtk4::prelude::*;
/// What:
///  imports the concrete GTK application and window types.
/// Why:
///  named explicitly so window construction reads without a glob import.
use gtk4::{Application, ApplicationWindow};

/// What:
///  imports the owned path type.
/// Why:
///  the start directory is computed as a `PathBuf`.
use std::path::PathBuf;

/// What:
///  imports the verification env names and default window-geometry constants.
/// Why:
///  the window is sized from a single source of truth,
///  and the test hooks are gated on env.
use crate::constants::{
    AUTOPREVIEW_ENV, AUTOSPAWN_ENV, DEFAULT_HEIGHT, DEFAULT_WIDTH, START_DIR_ENV,
};
/// What:
///  imports the pane-strip controller.
/// Why:
///  the window's content is the strip's scroller;
///  the controller is returned to be kept alive.
use crate::strip::StripController;

/// What:
///  build the top-level window over a strip rooted at the start directory,
///  present it,
///  and
///       return the controller so the caller keeps it (and the strip state) alive for the app.
/// Why:
///  the Pane-strip milestone:
///  the window shows the fixed-canvas strip.
///  The controller must
///      outlive this function because the pane activation closures hold only a weak reference.
pub(crate) fn build_window(app: &Application) -> StripController {
    let start = start_directory();
    let controller = StripController::new(&start);
    if std::env::var_os(AUTOSPAWN_ENV).is_some() {
        controller.autospawn_first_dir_for_test();
    }
    if std::env::var_os(AUTOPREVIEW_ENV).is_some() {
        controller.autopreview_first_image_for_test();
    }
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Monochromatic File Manager")
        .default_width(DEFAULT_WIDTH)
        .default_height(DEFAULT_HEIGHT)
        .child(&controller.widget())
        .build();
    crate::dnd::install_drop_target(&window);
    window.present();
    tracing::info!(path = %start.display(), "presented top-level window");
    return controller
}

/// What:
///  choose the directory the app opens on:
///  `FM_START_DIR`,
///  else `$HOME`,
///  else the current
///       directory,
///  else `.`.
/// Why:
///  the env override lets a verification run point at a fixture directory;
///  otherwise `$HOME` is
///      a sensible default until OS-open integration and a path argument arrive.
fn start_directory() -> PathBuf {
    if let Some(dir) = std::env::var_os(START_DIR_ENV) {
        return PathBuf::from(dir);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home);
    }
    return std::env::current_dir().unwrap_or_else(|_| return PathBuf::from("."))
}
