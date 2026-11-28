//! What:
//!  The words one diagnostic line is made of:
//!  the two switches a level answers to,
//!  the line the incumbent's tagged logger prints,
//!  the sentence the diagnostics' queue says when its reader stops taking lines,
//!  and the record of the lines this thread wrote,
//!  which a control of a command's words reads.
//! Why:
//!  A line is said by something that holds a lock or a claim
//!  (`diagnostic_lines_off_the_calling_thread.rs`),
//!  and the notice that the queue's reader stopped
//!  is said by that queue itself,
//!  so the words of both are written here,
//!  below the queue that says them,
//!  and the door over them (`diagnostic_log.rs`) says nothing of its own.
//!  The record of the lines written is read by a control through the door's own name,
//!  because a line goes to standard error,
//!  which a test process does not own.
//!  A release build carries the words and none of the record.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const DEBUG_ON = (env: Env) => env.MONOCHROMATIC_VERBOSE === 'true';
//! const line = (level: string, time: string, tag: string, message: string) => `[${level}] [${time}] [monochromatic-git-policy-cli] [${tag}] ${neutralize(maskUserinfo(message))}\n`;
//! ```

/// Replaces the userinfo of every URL in a message,
/// as the relays of Git's and `git-lfs` words do.
use crate::lfs_output_mask::mask_url_secrets_text;

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
pub fn diagnostic_line(
    level: &str,
    time: &str,
    tag: &str,
    message: &str,
) -> String {
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
///  The sentence the diagnostics' queue says when its reader took no line within the bound
///  the queue's caller gave it.
/// Why:
///   A person whose diagnostics stopped reaching them reads how long the reader was given and
///       that nothing waits for it any more,
///  so a gap in what a command says is a gap the command states.
///  The bound is the queue's own,
///  which is why the sentence is built from it here rather than
///       written beside the wait:
///  the words are the queue's,
///  and the wait is where they are said
///       (`diagnostic_lines_off_the_calling_thread.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readerStopped = (within: Duration) => `the reader of this queue's stream took no line within ${within}, so the command stops waiting for it`;
/// ```
pub(crate) fn the_reader_stopped_sentence(within: std::time::Duration) -> String {
    return format!(
        "the reader of this queue's stream took no line within {within:?}, so the command stops waiting for it"
    );
}

#[cfg(test)]
thread_local! {
    /// The lines this thread wrote,
    /// for a control whose subject is the words a command says.
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
pub(crate) fn remember_the_line(line: &str) {
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

/// Line formatting controls stay out of the release executable.
#[cfg(test)]
#[path = "the_words_of_a_diagnostic_line_and_the_lines_this_thread_wrote_tests.rs"]
mod tests;
