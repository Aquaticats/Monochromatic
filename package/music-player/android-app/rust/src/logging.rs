//! The Android tracing subscriber:
//!  a logcat sink installed once per process.
//!
//! Android's stderr never reaches logcat,
//!  so every `tracing` event from this crate and the
//! shared `truepeak-core` is routed to logcat through `paranoid-android`'s subscriber layer.
//! The JNI create entries call `init()` at startup;
//!  a `OnceLock` makes repeated calls (each
//! JNI entry,
//!  each process re-attach) idempotent.
//!  The level is read from `RUST_LOG`
//! (default `info`).

/// What:
///      `use std::sync::OnceLock;`.
///  A write-once cell (sibling `Once`) guarding the
///           one-time global-subscriber install across concurrent JNI calls.
/// Why:
///       Installing the global default twice would error;
///  the cell runs the closure once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let installed = false; // set once, then skip
/// ```
use std::sync::OnceLock;

/// What:
///      `use tracing_subscriber::EnvFilter;`.
///  The `RUST_LOG`-driven level filter.
/// Why:
///       Lets a developer raise the level with `adb shell setprop`/env without a rebuild.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { EnvFilter } from "tracing-subscriber";
/// ```
use tracing_subscriber::EnvFilter;

/// What:
///      `use tracing_subscriber::prelude::*;`.
///  Brings the registry `.with(...)` combinator
///           and `.try_init()` into scope.
/// Why:
///       The layered subscriber is composed with these extension traits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "tracing-subscriber/prelude";
/// ```
use tracing_subscriber::prelude::*;

/// What:
///      `static LOGGING: OnceLock<()> = OnceLock::new();`.
///  The one-time install guard.
/// Why:
///       Every JNI create entry may call `init()`;
///  only the first does the work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const LOGGING = new Once();
/// ```
static LOGGING: OnceLock<()> = OnceLock::new();

/// What:
///      `pub fn init()`.
///  Install the logcat tracing subscriber once,
///  idempotently.
/// Why:
///       Android stderr never reaches logcat,
///  so without this a device build is silent;
///           `try_init` (not `init`) tolerates a subscriber another component already set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// logging.init(); // first JNI entry installs the logcat sink
/// ```
pub fn init() {
    LOGGING.get_or_init(install_subscriber);
}

/// What:
///      `fn install_subscriber()`.
///  Compose the env-filter with the logcat layer and install it as the global tracing subscriber.
///  Called at most once,
///  by `init` through its `OnceLock`.
/// Why:
///       `OnceLock::get_or_init` takes it by path,
///  so no closure is needed for the one-time work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installSubscriber(): void { tracing.install(envFilter(), logcatLayer("MusicPlayer")); }
/// ```
fn install_subscriber() {
    // The level filter from RUST_LOG, or `info` when unset or malformed.
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("info"),
    };
    // Compose the env-filter with the logcat layer under a fixed tag; `with_ansi(false)`
    // drops the color escapes that would otherwise clutter logcat. Ignore an error when a
    // global subscriber is already set (the OnceLock already prevents our own reruns).
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(paranoid_android::layer("MusicPlayer").with_ansi(false))
        .try_init();
}
