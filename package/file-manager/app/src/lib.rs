//! Library entry point for the Monochromatic file manager.
//!
//! The product is a Niri-like infinite horizontal strip of columns,
//!  each column stacking panes
//! vertically (see doc/planning/file-manager.md for the full interaction model,
//!  and
//! doc/handover/file-manager-gtk-build.md for build state).
//!  `run` installs non-blocking
//! logging,
//!  creates the GTK `Application`,
//!  and drives its event loop;
//!  `main.rs` is a thin bin
//! over it so the plain-Rust domain modules stay unit-testable without spinning up GTK.

/// What:
///  the shared-constants module (application id,
///  default geometry,
///  env var names).
/// Why:
///  one source of truth for magic values used across the shell modules.
mod constants;
/// What:
///  the top-level window construction module.
/// Why:
///  keeps window assembly out of `run`,
///  each file under the max-lines budget.
mod window;
/// What:
///  the directory-listing pane widget.
/// Why:
///  renders a `DirectorySnapshot` as a virtualized icon+name list.
mod pane;
/// What:
///  deep GTK layout adapter for the detached-column pane strip.
/// Why:
///  centralizes horizontal scroll,
///  static canvases,
///  app vertical scroll,
///  lane sticky offsets,
///      and reveal behind one interface.
mod layout;
/// What:
///  the pane-strip controller.
/// Why:
///  mutates the pane model and delegates GTK placement/scrolling to `layout`.
mod strip;
/// What:
///  keyboard column navigation for the strip.
/// Why:
///  Left/Right move focus between columns;
///  split out to keep `strip.rs` under max-lines.
mod keys;
/// What:
///  debug-only tint labels and overlay wrappers.
/// Why:
///  layout screenshots need stable three-character codes naming each visible region.
mod debug_tint;
/// What:
///  the application stylesheet (black background).
/// Why:
///  applies a low-glare theme override once at startup.
mod style;
/// What:
///  the off-thread thumbnail decoder and bounded texture cache.
/// Why:
///  preview panes decode images on a worker thread and cache them with LRU eviction.
mod thumbs;
/// What:
///  the file drag-and-drop wiring (inbound drop target,
///  outbound drag source/shims).
/// Why:
///  native inbound over `GdkFileList`;
///  outbound native on Wayland,
///  shims on Windows/macOS.
mod dnd;
/// What:
///  the Windows-only native OLE outbound-drag shim.
/// Why:
///  GDK's Win32 drag source cannot deliver files to Explorer.
#[cfg(windows)]
mod win_drag;
/// What:
///  the macOS-only native AppKit outbound-drag shim.
/// Why:
///  GDK's Quartz drag source cannot deliver files to Finder.
#[cfg(target_os = "macos")]
mod mac_drag;

/// What:
///  the core domain types (ids,
///  entries,
///  locations,
///  snapshots).
/// Why:
///  plain-Rust model shared across the shell;
///  public so it unit-tests without GTK.
pub mod types;
/// What:
///  filesystem reads producing sorted directory snapshots.
/// Why:
///  isolates I/O so it is testable against throwaway directories.
pub mod fs;
/// What:
///  the pane-strip state machine (spawn/dedup/focus/close).
/// Why:
///  the Niri interaction rules live here,
///  tested independently of GTK.
pub mod model;

/// What:
///  unit tests for the filesystem reads.
/// Why:
///  compiled only under test;
///  kept in an exempt `_tests.rs` sibling.
#[cfg(test)]
mod fs_tests;
/// What:
///  unit tests for the pane-strip state machine.
/// Why:
///  compiled only under test;
///  kept in an exempt `_tests.rs` sibling.
#[cfg(test)]
mod model_tests;

/// What:
///  imports the GTK application-extension traits (`connect_activate`,
///  `run`,
///  `quit`).
/// Why:
///  `run` drives an `Application` through those trait methods,
///  which live in the prelude.
use gtk4::prelude::*;
/// What:
///  imports the GTK `Application` type and the `glib` module (`ExitCode`,
///  timers).
/// Why:
///  `run` constructs an `Application` and returns the `glib::ExitCode` GTK yields.
use gtk4::{Application, glib};

/// What:
///  imports the tracing subscriber's env-filter type.
/// Why:
///  log verbosity is driven by `RUST_LOG`,
///  defaulting to `info` when it is unset.
use tracing_subscriber::EnvFilter;

/// What:
///  imports the application-id constant and the self-quit env var name.
/// Why:
///  the `Application` is identified by `APP_ID`;
///  `QUIT_MS_ENV` drives unattended runs.
use crate::constants::{APP_ID, QUIT_MS_ENV};

/// What:
///  install non-blocking tracing,
///  build the GTK application,
///  and run its event loop.
/// Why:
///  the `Application` owns the GDK backend;
///  `run` blocks until the last window closes.
///  The
///      tracing writer guard is held for the whole run so a slow stderr never blocks the UI
///      thread.
///  Returns GTK's process exit code for `main` to propagate.
pub fn run() -> glib::ExitCode {
    let (writer, _guard) = tracing_appender::non_blocking(std::io::stderr());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| return EnvFilter::new("info")))
        .with_writer(writer)
        .init();
    enable_windows_gpu_rendering();
    tracing::info!("monochromatic file manager starting");

    let app = Application::builder().application_id(APP_ID).build();
    // Keep each window's strip controller alive for the app's lifetime; the pane activation
    // closures hold only a weak reference, so without this the strip state would drop at the end
    // of `build_window` and spawning would stop working.
    let controllers = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    app.connect_activate(move |activated| {
        style::install();
        controllers.borrow_mut().push(window::build_window(activated));
        schedule_self_quit(activated);
    });
    return app.run()
}

/// What:
///  on Windows,
///  opt into DirectComposition so GTK's GL renderer runs on the GPU.
/// Why:
///  gvsbuild patches DirectComposition to opt-in and GTK's GL renderer hard-requires a
///      DComp device;
///  without it GTK falls back to the Cairo software renderer at 3-6 fps (see
///      doc/troubleshooting/gtk4-windows-gvsbuild-directcomposition.md).
///  Set before GDK init,
///      and only when unset so an explicit `GDK_DEBUG` from the environment still wins.
#[cfg(windows)]
fn enable_windows_gpu_rendering() {
    if std::env::var_os("GDK_DEBUG").is_none() {
        // SAFETY: called at startup before GTK/GDK initialization while still single-threaded,
        // so no other thread can be reading the environment concurrently.
        unsafe {
            std::env::set_var("GDK_DEBUG", "dcomp");
        }
        tracing::info!("set GDK_DEBUG=dcomp for GPU rendering on Windows");
    }
}

/// What:
///  no-op stand-in for the Windows GPU-rendering opt-in on other platforms.
/// Why:
///  keeps `run` platform-agnostic;
///  the Wayland and Quartz backends need no such flag.
#[cfg(not(windows))]
fn enable_windows_gpu_rendering() {}

/// What:
///  if `FM_QUIT_MS` holds a millisecond count,
///  quit `app` after that delay.
/// Why:
///  an unattended verification run opens the window,
///  proves it renders,
///  then exits itself.
///      An absent or unparsable value leaves the app running normally until the window closes.
fn schedule_self_quit(app: &Application) {
    let Some(raw) = std::env::var_os(QUIT_MS_ENV) else {
        return;
    };
    let Some(ms) = raw.to_str().and_then(|value| return value.parse::<u64>().ok()) else {
        return;
    };
    let owned_app = app.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(ms), move || {
        tracing::info!(ms, "self-quit timer elapsed, quitting");
        owned_app.quit();
    });
}
