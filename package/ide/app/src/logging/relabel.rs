//! helix-lsp's ERROR records for a healthy server,
//!  re-labelled before they reach the log.
//!
//! helix-lsp (pinned revision,
//!  `helix-lsp/src/transport.rs`) writes three kinds of record at
//! ERROR for servers that work as intended
//! (`doc/troubleshooting/helix-lsp-transport-error-level-records.md`):
//!
//! - every line a server writes to its standard error:
//!    `{name} err <- "{line}"`;
//! - the end of that stream,
//!    on every server exit:
//!    `{name} err: <- StreamClosed`;
//! - every error answer,
//!    including the ones the protocol uses for a request that became moot:
//!   `{name} <- ServerError(-32801): content modified`.
//!
//! The application installs [`Relabel`] as the `log` crate's logger.
//!  It lowers exactly those
//! records to a level that says what they are,
//!  keeps their text,
//!  target,
//!  and source location,
//! and passes every other record through at its own level.
//!  An error answer with any other code is
//! a request failure the user is told about,
//!  so it keeps ERROR.

/// What:
///  `log` is the logging facade helix-lsp writes through;
///  `tracing-log` re-exports it and
///       converts its records into the application's `tracing` events.
///  `AsLog` and `AsTrace`
///       convert levels and metadata between the two.
/// Why:
///  Re-labelling happens on the `log` side,
///  before the conversion,
///  with the record's text in hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { log, formatTrace } from 'tracing-log';
/// ```
use tracing_log::{AsLog, AsTrace, log};

/// Each server's last standard-error lines are kept from the records seen here.
use super::stderr_tail;

/// The module that writes all three record shapes.
const TRANSPORT_TARGET: &str = "helix_lsp::transport";

/// Between a server's name and the quoted line it wrote to standard error.
const STDERR_LINE: &str = " err <- \"";

/// After a server's name when its standard error ended.
const END_OF_STDERR: &str = " err: <- StreamClosed";

/// Between a server's name and an error answer whose request became moot:
///  `-32801` (content
/// modified) and `-32800` (request cancelled).
///  The worker asks again for these and shows nothing
/// (`src/language/request/answer.rs`).
///  helix-lsp prints codes outside JSON-RPC's own range this way.
const MOOT_ANSWERS: [&str; 2] = [" <- ServerError(-32801): ", " <- ServerError(-32800): "];

/// What:
///  The record shapes that are lowered.
///  A plain `enum` is a closed set of names.
/// Why:
///  Each shape gets its own level,
///  and tests name the shape they expect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shape = 'serverStderrLine' | 'endOfServerStderr' | 'mootAnswer';
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// One line the server wrote to its standard error.
    ServerStderrLine,
    /// The server's standard error ended,
    ///  which happens whenever the process ends.
    EndOfServerStderr,
    /// An error answer for a request whose result no longer matters.
    MootAnswer,
}

/// Levels and their reasons.
impl Shape {
    /// What:
    ///  The level a record of this shape is logged at.
    ///  `log::Level` lists ERROR,
    ///  WARN,
    ///  INFO,
    ///       DEBUG,
    ///  and TRACE,
    ///  most severe first.
    /// Why:
    ///  A server's standard error is something the server said,
    ///  like the protocol messages
    ///      helix-lsp logs at INFO.
    ///  The end of that stream and a moot answer are routine steps of
    ///      a lifetime;
    ///  helix-lsp itself logs comparable shutdown steps at DEBUG.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// level(shape: Shape): Level { return shape === 'serverStderrLine' ? 'info' : 'debug'; }
    /// ```
    pub fn level(self) -> log::Level {
        return match self {
            Shape::ServerStderrLine => log::Level::Info,
            Shape::EndOfServerStderr | Shape::MootAnswer => log::Level::Debug,
        };
    }
}

/// What:
///  One record that has a shape,
///  with the parts of its text the worker needs.
///  `'text` says
///       the fields borrow the record's text and live no longer than it.
/// Why:
///  The server's name keys its kept standard-error lines (`stderr_tail.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Matched = { shape: Shape; server: string; line: string };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Matched<'text> {
    /// Which shape the record has.
    pub shape: Shape,
    /// The server's configured name,
    ///  which helix-lsp writes first.
    pub server: &'text str,
    /// For a standard-error line,
    ///  the line in helix-lsp's quoted form without the outer quotes;
    /// empty for the other shapes.
    pub line: &'text str,
}

/// What:
///  Name the shape of one record,
///  or nothing when it keeps its level.
/// Why:
///  Callers that need only the level ask this;
///  see [`matched`].
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shape = (target: string, level: Level, text: string) => matched(target, level, text)?.shape;
/// ```
pub fn shape(target: &str, level: log::Level, text: &str) -> Option<Shape> {
    let found = matched(target, level, text)?;
    return Some(found.shape);
}

/// What:
///  Match one record against the shapes and split out the server's name.
///  `&str` arguments
///       are borrowed text (sibling:
///  `String`,
///  which owns its text;
///  nothing here needs to keep it).
/// Why:
///  Only ERROR records from helix-lsp's transport are candidates,
///  and only these exact forms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function matched(target: string, level: Level, text: string): Matched | undefined
/// ```
pub fn matched<'text>(target: &str, level: log::Level, text: &'text str) -> Option<Matched<'text>> {
    if level != log::Level::Error || target != TRANSPORT_TARGET {
        return None;
    }
    // What: `split_once` returns the text before and after the first separator, or `None`.
    // Why: A server name comes first; a record without one is not one of these shapes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const at = text.indexOf(separator); if (at > 0 && text.endsWith('"')) return 'serverStderrLine';
    // ```
    if let Some((name, quoted)) = text.split_once(STDERR_LINE)
        && !name.is_empty()
        && let Some(line) = quoted.strip_suffix('"')
    {
        return Some(Matched {
            shape: Shape::ServerStderrLine,
            server: name,
            line,
        });
    }
    if let Some(name) = text.strip_suffix(END_OF_STDERR)
        && !name.is_empty()
    {
        return Some(Matched {
            shape: Shape::EndOfServerStderr,
            server: name,
            line: "",
        });
    }
    for marker in MOOT_ANSWERS {
        if let Some((name, _message)) = text.split_once(marker)
            && !name.is_empty()
        {
            return Some(Matched {
                shape: Shape::MootAnswer,
                server: name,
                line: "",
            });
        }
    }
    return None;
}

/// What:
///  The `log` crate's logger for the whole process.
///  A unit `struct` has no fields.
/// Why:
///  The `log` crate allows exactly one logger,
///  so re-labelling and the conversion to
///      `tracing` events live in the same place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const relabel: Logger = { log(record) { formatTrace(withLevel(record, levelFor(record))); } };
/// ```
pub struct Relabel;

/// Re-labelling,
///  then the conversion every record goes through.
impl log::Log for Relabel {
    /// What:
    ///  Whether a record at this level and target could be written at all.
    /// Why:
    ///  Callers that ask before building an expensive record get the subscriber's answer for
    ///      the record's own level,
    ///  which is the most severe level it can end up with.
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        // `Dispatch::default()` is a handle to the current subscriber, the one `get_default` lends.
        let dispatch = tracing::Dispatch::default();
        return dispatch.enabled(&metadata.as_trace());
    }

    /// What:
    ///  Convert one record to a `tracing` event,
    ///  at the lowered level when it has one of the
    ///       shapes.
    ///  `format_trace` checks the subscriber's filter for the level it is given.
    ///       A standard-error line and the end of that stream are also handed to `stderr_tail`,
    ///       whether or not the filter writes them.
    /// Why:
    ///  Only an ERROR record from the transport is formatted to text for the check;
    ///  every
    ///      other record goes through unchanged and without extra work.
    fn log(&self, record: &log::Record<'_>) {
        if record.level() != log::Level::Error || record.target() != TRANSPORT_TARGET {
            forward(record);
            return;
        }
        // `to_string` formats the record's message once, for the shape check and the new record.
        let text = record.args().to_string();
        let Some(found) = matched(record.target(), record.level(), &text) else {
            forward(record);
            return;
        };
        match found.shape {
            Shape::ServerStderrLine => stderr_tail::remember(found.server, found.line),
            Shape::EndOfServerStderr => stderr_tail::ended(found.server),
            Shape::MootAnswer => {}
        }
        // What: Build the same record with another level. `format_args!` must be used inside the
        //       expression that consumes it, so the record is built and forwarded in one statement.
        // Why: Text, target, module, file, and line stay exactly as helix-lsp wrote them.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // forward({ ...record, level: levelOf(found), message: text });
        // ```
        forward(
            &log::Record::builder()
                .args(format_args!("{text}"))
                .level(found.shape.level())
                .target(record.target())
                .module_path(record.module_path())
                .file(record.file())
                .line(record.line())
                .build(),
        );
    }

    /// Records are written as they arrive;
    ///  nothing is held here.
    fn flush(&self) {}
}

/// What:
///  Hand one record to the current `tracing` subscriber.
///  `format_trace` returns `io::Result`
///       but never fails.
/// Why:
///  One place converts records,
///  whether re-labelled or not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function forward(record: LogRecord): void { formatTrace(record); }
/// ```
fn forward(record: &log::Record<'_>) {
    let _conversion_cannot_fail = tracing_log::format_trace(record);
}

/// The one instance installed as the process's logger.
static RELABEL: Relabel = Relabel;

/// What:
///  Install [`Relabel`] as the process's `log` logger and let through every level the
///       `tracing` subscriber might write.
///  Call after the subscriber is installed.
/// Why:
///  `log` skips a record above its maximum level before any logger sees it,
///  which keeps
///      helix-lsp's message bodies free when the filter hides them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function install(): void { log.setLogger(relabel); log.setMaxLevel(currentTracingMaxLevel()); }
/// ```
pub fn install() -> anyhow::Result<()> {
    // A failure of the `log` crate becomes an error that names what failed.
    if let Err(error) = log::set_logger(&RELABEL) {
        return Err(anyhow::anyhow!(
            "Cannot install the helix-lsp log bridge: {error}"
        ));
    }
    log::set_max_level(tracing::level_filters::LevelFilter::current().as_log());
    return Ok(());
}

/// Shapes,
///  near misses,
///  and the re-labelled record as the subscriber writes it.
#[cfg(test)]
#[path = "relabel_tests.rs"]
mod tests;
