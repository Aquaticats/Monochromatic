//! The desktop tracing subscriber:
//!  a stderr sink with an env-filter.
//!
//! The binary installs this once at startup so every `tracing` event from this crate and
//! the shared `truepeak-core` reaches stderr.
//!  The level is read from `RUST_LOG` (default
//! `info`).
//!  We deliberately do NOT bridge the `log` facade:
//!  ICU's `log`-routed
//! CJK-segmentation warnings must stay silent,
//!  exactly as they are today with no `log`
//! subscriber installed (see the `icu_provider` note in Cargo.toml).

/// What:
///      `use tracing_subscriber::EnvFilter;`.
///  The filter that reads `RUST_LOG`.
/// Why:
///       Lets an operator raise or lower the level without a rebuild.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { EnvFilter } from "tracing-subscriber";
/// ```
use tracing_subscriber::EnvFilter;

/// What:
///      `pub fn init()`.
///  Install the global tracing subscriber once,
///  to stderr.
/// Why:
///       One startup call gives every crate's events a sink;
///  using `set_global_default`
///           rather than `.init()` leaves the `log` facade unbridged so ICU's silenced
///           CJK-segmentation warnings do not revive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// music_player.logging.init(); // first line of main
/// ```
pub fn init() {
    install(None);
}

/// What:
///      `pub fn init_with_ansi(ansi: bool)`.
///  Same as `init`, with ANSI colors on or off as the caller measured.
/// Why:
///       `tracing-subscriber` turns colors on whenever `NO_COLOR` is empty, whatever stderr is. The Windows build
///           measures whether stderr is a console that interprets escape sequences (`console_color.rs`) and passes the
///           answer here, so a file or a console without VT processing gets plain text.
pub fn init_with_ansi(ansi: bool) {
    install(Some(ansi));
}

/// What:
///      `fn install(ansi: Option<bool>)`.
///  Builds the stderr subscriber and installs it as the global default; `None` keeps `tracing-subscriber`'s color default.
/// Why:
///       One builder for both entry points, so they differ only in the color setting.
fn install(ansi: Option<bool>) {
    // The level filter from RUST_LOG, or `info` when the variable is unset or malformed.
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| return EnvFilter::new("info"));
    // A fmt subscriber writing to stderr.
    let builder = tracing_subscriber::fmt().with_env_filter(filter).with_writer(std::io::stderr);
    // `.finish()` returns the subscriber without touching `log`.
    let subscriber = match ansi {
        Some(colors) => builder.with_ansi(colors).finish(),
        None => builder.finish(),
    };
    // Ignore the error when a subscriber is already set (repeated init, tests).
    let _ = tracing::subscriber::set_global_default(subscriber);
}
