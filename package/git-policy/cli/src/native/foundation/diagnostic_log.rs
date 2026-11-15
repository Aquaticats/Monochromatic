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
//! const l = tagged({ tag: 'monochromatic-git-policy-cli' }); l.debug(message); l.warn(message);
//! ```

/// The current time in `toISOString` form.
use crate::iso_time::now_iso;
/// Replaces the userinfo of every URL in a message,
/// as the relays of Git's and `git-lfs` words do.
use crate::lfs_output_mask::mask_url_secrets_text;
/// The trait that gives output streams `.write_all(..)`.
use std::io::Write;

/// What:
///  Whether debug diagnostics are on:
///  `MONOCHROMATIC_VERBOSE` is exactly `true`.
/// Why:
///   The switch is shared with the TypeScript tools (decision recorded in
///       `doc/handover/cli-git-rust-implementation.md`,
///  "Adopted without a question";
///  that section moved on 2026-11-03 to
///  `doc/planning/cli-git-rust-open-decisions.md`).
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
///       `[<level>] [<ISO time>] [monochromatic-git-policy-cli] [<tag>] <message>`,
///  with the userinfo of every URL in the message hidden and every control character of
///       the message replaced by a space.
/// Why:
///   The incumbent's tagged loggers print `[warn] [2026-10-06T21:40:39.524Z] [monochromatic-git-policy-cli] ...`;
///       one line per record keeps standard error parseable beside JSONL events.
///  Debug and warning lines carry Git's words (a failed fetch names its remote),
/// and a remote written as
///       `https://user:token@host` would put its token on the terminal;
/// every caller passes through here,
///  so the userinfo is hidden once,
/// before the control characters go,
/// so that no character inside the userinfo
///       decides where the hiding ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `[${level}] [${new Date().toISOString()}] [monochromatic-git-policy-cli] [${tag}] ${neutralize(maskUserinfo(message))}\n`
/// ```
pub fn diagnostic_line(level: &str, time: &str, tag: &str, message: &str) -> String {
    let masked: String = mask_url_secrets_text(message);
    // `mut` allows building the neutralized message one character at a time.
    let mut single: String = String::with_capacity(masked.len());
    for character in masked.chars() {
        if character.is_control() {
            single.push(' ');
        } else {
            single.push(character);
        }
    }
    return format!("[{level}] [{time}] [monochromatic-git-policy-cli] [{tag}] {single}\n");
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
    #[cfg(test)]
    remember_the_line(line);
    // `let _ =` discards the write result: there is nowhere left to report it.
    let _ = std::io::stderr().write_all(line.as_bytes());
}

#[cfg(test)]
thread_local! {
    /// The lines this thread wrote, for a control whose subject is the words a command says.
    static LINES_WRITTEN: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// What:
///  Remember one diagnostic line this thread wrote.
/// Why:
///   A control of a command's words has no other way to read them:
/// the line goes to standard error,
/// which a test process does not own.
///  Keeping it per thread means two controls running beside each other cannot read each other's words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// linesWritten.push(line);
/// ```
#[cfg(test)]
fn remember_the_line(line: &str) {
    LINES_WRITTEN.with(|lines| {
        lines.borrow_mut().push(line.to_owned());
    });
}

/// What:
///  Take and clear the diagnostic lines this thread has written so far.
/// Why:
///   A control clears them before the step under test and reads exactly that step's words after it.
///  A release build carries none of this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = linesWritten.splice(0);
/// ```
#[cfg(test)]
pub(crate) fn take_lines_written() -> Vec<String> {
    return LINES_WRITTEN.with(|lines| {
        return std::mem::take(&mut *lines.borrow_mut());
    });
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
