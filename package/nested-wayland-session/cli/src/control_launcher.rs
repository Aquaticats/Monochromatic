//! Answers the taskbar-progress control verbs from the launcher listener's history.
//!
//! Each verb is a plain function of the listener handle, so the no-bus refusal is testable
//!  without a display.

/// Imports the monotonic start time that every timestamp is measured from.
use std::time::Instant;

/// Imports the answer formatters, the listener handle, and the wire response.
use crate::{
    launcher_format::{format_latest, format_log},
    launcher_listen::LauncherLog,
    protocol::Response,
};

/// The refusal every taskbar verb gives when no private bus was started.
///
/// What:
///      `const NO_BUS_MESSAGE: &str`.
///  The text names both options that start the bus.
/// Why:
///       A session started without either option has no bus to listen on,
///  and the caller needs the one way to fix that.
const NO_BUS_MESSAGE: &str = "no private session bus (start with --launcher-log or --color-scheme)";

/// Reports the newest taskbar progress update, or `none` before any arrived.
///
/// What:
///      `pub fn latest(log: Option<&LauncherLog>, start_time: Instant) -> Response`.
///  `None` means the session has no private bus.
/// Why:
///       `ok none` and the refusal are different answers: the first means the bus is running
///  and quiet, the second means nothing can be observed at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function latest(log, startTime): Response { ... }
/// ```
pub fn latest(log: Option<&LauncherLog>, start_time: Instant) -> Response {
    let Some(listener) = log else {
        return Response::Err(NO_BUS_MESSAGE.to_string());
    };
    // What:     `let now_ms = start_time.elapsed().as_millis() as u64;`. The age reference,
    //           taken on the same clock the listener stamps updates with.
    // Why:      `age-ms` is only meaningful when both ends use one clock.
    let now_ms = start_time.elapsed().as_millis() as u64;
    let history = listener.lock();
    match history.latest() {
        Some(update) => return Response::OkWith(format_latest(update, now_ms)),
        None => return Response::OkWith("none".to_string()),
    }
}

/// Lists the newest `count` taskbar progress updates, oldest first.
///
/// What:
///      `pub fn log(log: Option<&LauncherLog>, count: usize) -> Response`.
/// Why:
///       A test compares the list with the values it emitted, in the order it emitted them.
pub fn log(log: Option<&LauncherLog>, count: usize) -> Response {
    let Some(listener) = log else {
        return Response::Err(NO_BUS_MESSAGE.to_string());
    };
    let history = listener.lock();
    let records = history.last_n(count);
    return Response::OkWith(format_log(&records))
}

/// Empties the taskbar progress history.
///
/// What:
///      `pub fn clear(log: Option<&LauncherLog>) -> Response`.
/// Why:
///       A test clears between steps so its next `launcher-log` lists only its own updates.
pub fn clear(log: Option<&LauncherLog>) -> Response {
    let Some(listener) = log else {
        return Response::Err(NO_BUS_MESSAGE.to_string());
    };
    listener.lock().clear();
    return Response::Ok
}

/// Unit tests for the no-bus refusal.
#[cfg(test)]
#[path = "control_launcher_tests.rs"]
mod tests;
