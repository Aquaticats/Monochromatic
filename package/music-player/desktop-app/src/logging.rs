//! The desktop tracing subscriber:
//!  a stderr sink with an env-filter,
//!  and the `log` bridge behind it.
//!
//! The binary installs this once at startup so every `tracing` event from this crate and
//! the shared `truepeak-core` reaches stderr,
//! and so every `log` record does too.
//!  Until issue 1010 only `tracing` events had a sink:
//!  the file dialog library
//! reports why a folder chooser could not open through `log`,
//!  so those reports went nowhere.
//!  Two ICU crates also write through `log`,
//!  and
//! their CJK-segmentation warnings must stay silent
//!  (see the `icu_provider` note in `Cargo.toml`),
//!  so the filter turns exactly those targets off
//!  and
//! an operator can turn them back on by naming them.
//!  The level is read from `RUST_LOG` (default
//! `info`).

/// What:
///      `use tracing_subscriber::EnvFilter;`.
///  The filter that reads `RUST_LOG`.
/// Why:
///       Lets an operator raise or lower the level without a rebuild,
///  and decides which targets are heard at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { EnvFilter } from "tracing-subscriber";
/// ```
use tracing_subscriber::EnvFilter;

/// What:
///      `use tracing_subscriber::util::SubscriberInitExt;`.
///  The extension trait whose `try_init` installs a subscriber as the global default
///           and,
/// because `tracing-subscriber`'s default features include the `tracing-log` bridge,
///  installs the `log` logger in front of
///           it.
/// Why:
///       One call does both,
///  so no crate has to be named to bridge the facade,
///  and a second call is a no-op.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { SubscriberInitExt } from "tracing-subscriber/util";
/// ```
use tracing_subscriber::util::SubscriberInitExt;

/// What:
///      `pub const SILENT_TARGETS: &str`.
///  The filter directives that keep ICU's segmentation warnings silent.
/// Why:
///       `icu_provider` writes those warnings through `log`,
///  which the bridge now carries,
///  and one appears per
///           text-layout pass of Chinese,
///  Japanese,
///  Thai,
///  Khmer,
///  Lao or Burmese text;
///  the silence is why this crate never bridged the facade
///           before (`doc/troubleshooting/slint-parley-icu-cjk-segmentation.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SILENT_TARGETS = "icu_provider=off,icu_segmenter=off";
/// ```
pub const SILENT_TARGETS: &str = "icu_provider=off,icu_segmenter=off";

/// What:
///      `pub const DEFAULT_LEVEL: &str`.
///  The level used when `RUST_LOG` is unset,
///  empty or malformed.
/// Why:
///       Warnings and errors from the whole tree,
///  and the information the player reports about its own steps,
///  are what
///           a user reading a bug report needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_LEVEL = "info";
/// ```
pub const DEFAULT_LEVEL: &str = "info";

/// What:
///      `pub fn directives(rust_log: Option<&str>) -> String`.
///  The filter text:
///  the silent targets first,
///  then the operator's own directives,
///  or the default
///  level when there are none.
/// Why:
///       `tracing-subscriber` keeps the later of two directives that name the same target
///  (`DirectiveSet::add`
///           replaces an equally specific entry),
///  so naming a silent target in `RUST_LOG` turns it back on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directives = (rustLog?: string) => [`${SILENT_TARGETS}`, rustLog?.trim() || DEFAULT_LEVEL].join(",");
/// ```
pub fn directives(rust_log: Option<&str>) -> String {
    let requested: Option<&str> = match rust_log.map(str::trim) {
        Some(text) if !text.is_empty() => Some(text),
        _ => None,
    };
    return match requested {
        Some(text) => format!("{SILENT_TARGETS},{text}"),
        None => format!("{SILENT_TARGETS},{DEFAULT_LEVEL}"),
    };
}

/// What:
///      `pub fn filter(rust_log: Option<&str>) -> EnvFilter`.
///  The filter built from `directives`,
///  falling back to the default level when the operator's text has an invalid directive.
/// Why:
///       A typo in `RUST_LOG` must not silence the application or report itself on the operator's stderr;
///  `EnvFilter::try_new`
///           refuses it,
///  and the fallback keeps the silence directives either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function filter(rustLog?: string): EnvFilter { return EnvFilter.tryNew(directives(rustLog)) ?? EnvFilter.new(directives(undefined)); }
/// ```
pub fn filter(rust_log: Option<&str>) -> EnvFilter {
    if let Ok(filter) = EnvFilter::try_new(directives(rust_log)) {
        return filter;
    }
    return EnvFilter::new(directives(None));
}

/// What:
///      `pub fn init()`.
///  Install the global tracing subscriber once,
///  to stderr.
/// Why:
///       One startup call gives every crate's events a sink;
///  `try_init` also installs the `log` bridge,
///  so the file dialog library's
///           reports about a folder chooser reach that same sink (issue 1010).
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
///  Same as `init`,
/// with ANSI colors on or off as the caller measured.
/// Why:
///       `tracing-subscriber` turns colors on whenever `NO_COLOR` is empty,
/// whatever stderr is.
/// The Windows build
///           measures whether stderr is a console that interprets escape sequences (`console_color.rs`) and passes the
///           answer here,
/// so a file or a console without VT processing gets plain text.
pub fn init_with_ansi(ansi: bool) {
    install(Some(ansi));
}

/// What:
///      `fn install(ansi: Option<bool>)`.
///  Installs the pipeline over stderr;
/// `None` keeps `tracing-subscriber`'s color default.
/// Why:
///       One builder for both entry points,
/// so they differ only in the color setting.
fn install(ansi: Option<bool>) {
    // The answer is ignored: a second init (tests, a library that installed its own) leaves the first in place.
    let _installed: bool = install_with(std::io::stderr, ansi);
}

/// What:
///      `fn install_with(writer: W, ansi: Option<bool>) -> bool`.
///  Builds the subscriber over `writer` and installs it,
/// with the `log` bridge,
/// as the global default;
///  answers
///           whether it did.
///  `W` is any sink the formatter can write a record through:
///  stderr in production,
///  a buffer in a test.
/// Why:
///       The tests install this same pipeline over a buffer,
///  so a change here cannot pass them unnoticed;
///  `install`
///           is the caller that writes to stderr.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installWith(writer: MakeWriter, ansi?: boolean): boolean { return buildSubscriber(filter(process.env.RUST_LOG), writer, ansi).tryInit(); }
/// ```
pub(crate) fn install_with<W>(writer: W, ansi: Option<bool>) -> bool
where
    W: for<'writer> tracing_subscriber::fmt::MakeWriter<'writer> + Send + Sync + 'static,
{
    // The level filter from RUST_LOG, or the default when the variable is unset, empty or malformed.
    let env_filter: EnvFilter = filter(std::env::var("RUST_LOG").ok().as_deref());
    // A fmt subscriber writing to `writer`.
    let builder = tracing_subscriber::fmt().with_env_filter(env_filter).with_writer(writer);
    // `.finish()` returns the subscriber; `try_init` installs it together with the `log` bridge.
    let subscriber = match ansi {
        Some(colors) => builder.with_ansi(colors).finish(),
        None => builder.finish(),
    };
    return subscriber.try_init().is_ok();
}

/// What:
///  `mod log_capture;` the buffer the tests read records from (`src/log_capture.rs`).
/// Why:
///  The bridge and the filter are only observable through a sink,
///  and the tests need one that is not stderr.
#[cfg(test)]
#[path = "log_capture.rs"]
pub(crate) mod log_capture;

/// What:
/// The filter's directives and the targets they silence,
///  checked under a scoped subscriber that writes into a buffer.
/// Why:
///  The bridge is what changed;
///  a test must show a `log`-shaped record reaching the sink
///  while ICU's warnings stay
///  out.
#[cfg(test)]
#[path = "logging_tests.rs"]
mod tests;
