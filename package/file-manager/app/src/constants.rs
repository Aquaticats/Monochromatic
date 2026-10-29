//! Compile-time constants shared across the file-manager shell.

/// What:
///  GTK application id,
///  reused verbatim as the Wayland `app_id` on the `xdg_toplevel`.
/// Why:
///  GTK requires a valid reverse-DNS application id,
///  and the compositor keys window rules
///      and taskbar identity off the same string,
///  so one source of truth avoids drift.
pub(crate) const APP_ID: &str = "dev.monochromatic.FileManager";

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
///  environment variable naming a millisecond budget after which the app self-quits.
/// Why:
///  lets an unattended verification run open the window,
///  prove it renders,
///  and exit
///      without a human closing it;
///  unset in normal use so the app runs until closed.
pub(crate) const QUIT_MS_ENV: &str = "FM_QUIT_MS";

/// What:
///  environment variable that,
///  when set,
///  programmatically spawns the first sub-directory's
///       child pane at startup.
/// Why:
///  exercises the real spawn+reconcile path end to end for unattended verification,
///  without a
///      human clicking;
///  unset in normal use.
pub(crate) const AUTOSPAWN_ENV: &str = "FM_AUTOSPAWN";

/// What:
///  fixed pane width in pixels on the canvas.
/// Why:
///  panes tile at fixed positions so a pan re-composites cached render nodes instead of
///      re-running layout;
///  a fixed width is what makes the column grid deterministic.
pub(crate) const PANE_WIDTH: i32 = 320;

/// What:
///  fixed pane height in pixels on the canvas.
/// Why:
///  bounds each pane's virtualized list so an outer pan never re-virtualizes it.
pub(crate) const PANE_HEIGHT: i32 = 520;

/// What:
///  gap in pixels between adjacent panes on the canvas.
/// Why:
///  separates columns and stacked panes without overlap;
///  named so it is not a magic literal.
pub(crate) const PANE_GAP: i32 = 12;

/// What:
///  maximum edge length in pixels a decoded thumbnail is scaled to fit within.
/// Why:
///  bounds decode work and texture memory;
///  a preview does not need the full-resolution image.
pub(crate) const THUMB_SIZE: u32 = 256;

/// What:
///  soft cap in bytes on the total size of cached thumbnail textures.
/// Why:
///  the cache evicts the least-recently-used entries to stay under this,
///  so browsing many
///      images keeps decoded memory bounded (evicted previews re-decode on scroll-back).
pub(crate) const THUMB_CACHE_BYTES: usize = 64 * 1024 * 1024;

/// What:
///  environment variable overriding the directory the app opens on.
/// Why:
///  lets a verification run point the app at a fixture directory of known contents.
pub(crate) const START_DIR_ENV: &str = "FM_START_DIR";

/// What:
///  environment variable that,
///  when set,
///  spawns a preview pane for the first image file in the
///       start directory at startup.
/// Why:
///  exercises the off-thread decode + cache path end to end for unattended verification.
pub(crate) const AUTOPREVIEW_ENV: &str = "FM_AUTOPREVIEW";

/// What:
///  environment variable that,
///  when set,
///  tints each structural layer (columns,
///  panes,
///  headers,
///       list areas) a distinct hue.
/// Why:
///  a debugging aid so the layout and scroll layers can be named precisely;
///  off in normal use.
pub(crate) const DEBUG_TINT_ENV: &str = "FM_DEBUG_TINT";
