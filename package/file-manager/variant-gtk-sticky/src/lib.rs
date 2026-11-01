//! Library entry point for the sticky-band variant of the Monochromatic file manager.
//!
//! Same product model as `package/file-manager/app` (whose public `model`/`fs`/`types`
//! modules this crate reuses verbatim),
//!  with the original's 400-plus-line lane engine and
//! collision solver replaced by the pure `band` module:
//!  one stateless clamp per pane,
//!  the rule
//! CSS `position: sticky` applies.
//!  The derivation and behavioral deltas are recorded in
//! `doc/audit/file-manager-sticky-flow.md`.

/// What:
///  the shared-constants module (application id,
///  geometry,
///  env var names).
/// Why:
///  one source of truth for magic values used across the shell modules.
mod constants;
/// What:
///  the pure sticky-band math:
///  bands,
///  positions,
///  pinning,
///  overlap counting.
/// Why:
///  the whole layout policy,
///  public so it unit-tests and the audit doc can cite it.
pub mod band;
/// What:
///  the stateless GTK layout adapter over the band math.
/// Why:
///  owns GTK plumbing only;
///  every position decision is a pure `band` call.
mod layout;
/// What:
///  the directory-listing and preview pane widgets.
/// Why:
///  renders the shared model's snapshots;
///  trimmed from the original (no thumbs,
///  no DnD).
mod pane;
/// What:
///  the pane-strip controller over the shared model.
/// Why:
///  mutates spawn/dedup/close state and delegates GTK placement to `layout`.
mod strip;
/// What:
///  keyboard navigation for the strip.
/// Why:
///  Left/Right column focus and Backspace close,
///  matching the Electron prototype.
mod keys;
/// What:
///  observed-state output for boundary tests.
/// Why:
///  mirrors the same shallow JSON schema the Electron prototype writes.
mod state_out;
/// What:
///  the application stylesheet (black background,
///  debug rails).
/// Why:
///  applies a low-glare theme override and the opt-in rail tint once at startup.
mod style;
/// What:
///  the top-level window construction module.
/// Why:
///  keeps window assembly out of `run`.
mod window;

/// What:
///  unit tests for the pure band math.
/// Why:
///  compiled only under test;
///  kept in an exempt `_tests.rs` sibling.
#[cfg(test)]
mod band_tests;

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
///  `FM_STICKY_QUIT_MS` drives unattended runs.
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
    tracing::info!("monochromatic file manager (sticky variant) starting");

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
///  if `FM_STICKY_QUIT_MS` holds a millisecond count,
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
