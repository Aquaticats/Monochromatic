//! Compile-time constants shared across the sticky-variant shell.
//!
//! Geometry mirrors `package/file-manager/app/src/constants.rs` verbatim so the two apps
//! render the same pane grid and the audit doc can compare them pixel for pixel.

/// What:
///  GTK application id,
///  reused verbatim as the Wayland `app_id` on the `xdg_toplevel`.
/// Why:
///  distinct from the original's id so both apps can run side by side under one compositor.
pub(crate) const APP_ID: &str = "dev.monochromatic.FileManagerSticky";

/// What:
///  initial top-level window width in pixels.
/// Why:
///  a wide default suits the horizontal column strip that fills the window.
pub(crate) const DEFAULT_WIDTH: i32 = 1280;

/// What:
///  initial top-level window height in pixels.
/// Why:
///  tall enough to show a full column of vertically stacked panes.
pub(crate) const DEFAULT_HEIGHT: i32 = 800;

/// What:
///  fixed pane width in pixels on the canvas.
/// Why:
///  same deterministic column grid as the original.
pub(crate) const PANE_WIDTH: i32 = 320;

/// What:
///  fixed pane height in pixels on the canvas.
/// Why:
///  same fixed pane box as the original.
pub(crate) const PANE_HEIGHT: i32 = 520;

/// What:
///  gap in pixels between adjacent panes on the canvas.
/// Why:
///  same spacing as the original.
pub(crate) const PANE_GAP: i32 = 12;

/// What:
///  environment variable overriding the directory the app opens on.
/// Why:
///  lets a verification run point the app at a fixture directory of known contents.
pub(crate) const START_DIR_ENV: &str = "FM_STICKY_START_DIR";

/// What:
///  environment variable naming a millisecond budget after which the app self-quits.
/// Why:
///  lets an unattended run open the window,
///  prove it renders,
///  and exit itself.
pub(crate) const QUIT_MS_ENV: &str = "FM_STICKY_QUIT_MS";

/// What:
///  environment variable pointing at a JSON file where boundary tests observe app state.
/// Why:
///  the nested-Wayland test polls this file for the same shallow schema the Electron
///      prototype's main process writes,
///  so both boundary tests share assertions.
pub(crate) const STATE_PATH_ENV: &str = "FM_STICKY_STATE_PATH";

/// What:
///  environment variable that,
///  when set,
///  draws each pane's sticky band as a green rail.
/// Why:
///  screenshots then name the sticky containing bands,
///  matching the original's `Y6L` lanes
///      and the Electron prototype's `.debug-tint .rail` outlines.
pub(crate) const DEBUG_TINT_ENV: &str = "FM_STICKY_DEBUG_TINT";
