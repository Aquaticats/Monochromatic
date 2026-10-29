//! What:
//!  Line-oriented debug and warning diagnostics on standard error,
//!  switched like the
//!       incumbent's console sink.
//! Why:
//!  The incumbent prints debug records only when `MONOCHROMATIC_VERBOSE=true` and warnings
//!      unless `MONOCHROMATIC_WARN=false` (`package/logger/core/src/sink/console.ts`),
//!  and its
//!      spec requires that diagnostics never corrupt a JSONL stream:
//!  each diagnostic is one
//!      complete line written in one call (`SPEC.md`,
//!  "Debug logging").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const l = tagged({ tag: 'cli-git' }); l.debug(message); l.warn(message);
//! ```

/// The current time in `toISOString` form.
use super::iso_time::now_iso;
/// The trait that gives output streams `.write_all(..)`.
use std::io::Write;

/// What:
///  Whether debug diagnostics are on:
///  `MONOCHROMATIC_VERBOSE` is exactly `true`.
/// Why:
///   The switch is shared with the TypeScript tools (decision recorded in
///       `doc/handover/cli-git-rust-implementation.md`,
///  "Adopted without a question").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.env.MONOCHROMATIC_VERBOSE === 'true'
/// ```
pub fn verbose_enabled() -> bool {
    return std::env::var_os("MONOCHROMATIC_VERBOSE").is_some_and(is_true);
}

/// What:
///  Whether an environment value is exactly `true`.
/// Why:
///   A named function keeps the check out of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (value: string) => value === 'true'
/// ```
fn is_true(value: std::ffi::OsString) -> bool {
    return value == "true";
}

/// What:
///  Whether warnings are suppressed:
///  `MONOCHROMATIC_WARN` is exactly `false`.
/// Why:
///   Machine-protocol consumers silence warnings this way in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.env.MONOCHROMATIC_WARN === 'false'
/// ```
pub fn warnings_suppressed() -> bool {
    return std::env::var_os("MONOCHROMATIC_WARN").is_some_and(is_false);
}

/// What:
///  Whether an environment value is exactly `false`.
/// Why:
///   A named function keeps the check out of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (value: string) => value === 'false'
/// ```
fn is_false(value: std::ffi::OsString) -> bool {
    return value == "false";
}

/// What:
///  One diagnostic line in the incumbent's console format,
///       `[<level>] [<ISO time>] [cli-git] [<tag>] <message>`,
///  with every control character of
///       the message replaced by a space.
/// Why:
///   The incumbent's tagged loggers print `[warn] [2026-10-06T21:40:39.524Z] [cli-git] ...`;
///       one line per record keeps standard error parseable beside JSONL events.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `[${level}] [${new Date().toISOString()}] [cli-git] [${tag}] ${neutralize(message)}\n`
/// ```
pub fn diagnostic_line(level: &str, time: &str, tag: &str, message: &str) -> String {
    // `mut` allows building the neutralized message one character at a time.
    let mut single: String = String::with_capacity(message.len());
    for character in message.chars() {
        if character.is_control() {
            single.push(' ');
        } else {
            single.push(character);
        }
    }
    return format!("[{level}] [{time}] [cli-git] [{tag}] {single}\n");
}

/// What:
///  Write one complete line to standard error,
///  ignoring a closed stream.
/// Why:
///   A diagnostic must never turn into a failure of the command it describes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.stderr.write(line);
/// ```
fn write_line(line: &str) {
    // `let _ =` discards the write result: there is nowhere left to report it.
    let _ = std::io::stderr().write_all(line.as_bytes());
}

/// What:
///  Print a debug diagnostic when verbose diagnostics are on.
///       `tag` names the function or module boundary,
///  as the incumbent's tagged loggers do.
/// Why:
///   Decisions such as "owner alive,
///  skipping" are invisible otherwise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tagged({ tag }).debug(message);
/// ```
pub fn debug(tag: &str, message: &str) {
    if verbose_enabled() {
        write_line(diagnostic_line("debug", now_iso().as_str(), tag, message).as_str());
    }
}

/// What:
///  Print a warning unless warnings are suppressed.
/// Why:
///   Conditions the user should know about that do not fail the command,
///  such as a
///       landed-capture record that could not be written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// tagged({ tag }).warn(message);
/// ```
pub fn warn(tag: &str, message: &str) {
    if !warnings_suppressed() {
        write_line(diagnostic_line("warn", now_iso().as_str(), tag, message).as_str());
    }
}

/// Line formatting controls stay out of the release executable.
#[cfg(test)]
#[path = "diagnostic_log_tests.rs"]
mod tests;
