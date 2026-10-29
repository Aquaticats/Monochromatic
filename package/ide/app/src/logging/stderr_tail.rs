//! The last lines each language server wrote to its standard error,
//!  kept for the record the
//! language worker writes when a server ends unexpectedly or fails to start.
//!
//! helix-lsp owns every server's standard-error pipe and passes each line on only as a `log`
//! record,
//!  at ERROR,
//!  so the log bridge (`relabel.rs`) sees every line whatever the log level shows
//! and hands it here,
//!  together with the end of the stream.
//!  Nothing here reaches a user-facing note:
//! the lines go only into the log,
//!  because a server's own output can hold paths and other details
//! of the machine.
//!
//! helix-lsp names a line's server by its configured name only,
//!  so lines are kept by name,
//!  and two
//! processes of the same name share one tail.
//!  A tail is removed when its lines are reported or the
//! worker stops the server itself;
//!  lines the same process still writes afterwards are dropped until
//! its stream ends,
//!  so they are never reported as the words of a later process of that name.

/// What:
///  `BTreeMap` is a sorted map (sibling:
///  `HashMap`,
///  whose constructor cannot run in a
///       `static`);
///  `VecDeque` is a list with cheap removal at the front (sibling:
///  `Vec`).
/// Why:
///  The map lives in a `static` shared by the log bridge and the worker;
///  each tail drops its
///      oldest line when a new one arrives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tails = new Map<string, { lines: string[]; closed: boolean; retired: boolean }>();
/// ```
use std::collections::{BTreeMap, VecDeque};
/// What:
///  `Mutex` guards a value that several threads change;
///  `MutexGuard` is the held lock.
/// Why:
///  The log bridge runs on whichever thread logs;
///  the worker reads from its own thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // one lock around `tails`
/// ```
use std::sync::{Mutex, MutexGuard};

/// Lines kept per server:
///  a panic message with its location,
///  or the top of a short stack trace.
pub const KEPT_LINES: usize = 8;

/// Longest kept line in bytes;
///  a longer line is cut at a character boundary and marked.
pub const LINE_BYTES: usize = 512;

/// Marker appended to a cut line.
pub const CUT_MARK: &str = " [cut]";

/// What:
///  One server's kept lines and where its stream stands.
/// Why:
///  The worker waits for the end of the stream before it reports,
///  and lines of a process
///      whose words were already reported must not be kept.
#[derive(Default)]
struct Tail {
    /// Oldest first.
    lines: VecDeque<String>,
    /// The stream ended after the newest line.
    closed: bool,
    /// The lines were reported or dropped while the stream was still open;
    ///  everything until the
    /// end of that stream is dropped.
    retired: bool,
}

/// Every server's tail,
///  by server name.
static TAILS: Mutex<BTreeMap<String, Tail>> = Mutex::new(BTreeMap::new());

/// What:
///  Lock the tails.
///  A panic while the lock was held leaves it "poisoned";
///  the data is still
///       consistent here,
///  so the guard is taken either way.
/// Why:
///  Missing log detail must never stop the worker or a logging thread.
fn tails() -> MutexGuard<'static, BTreeMap<String, Tail>> {
    return match TAILS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
}

/// What:
///  Turn helix-lsp's quoted form of a line back into the line,
///  in one pass over the
///       characters.
///  helix-lsp writes the line with Rust's debug quoting (`{line:?}`),
///  which
///       escapes `\`,
///  `"`,
///  tab,
///  return,
///  newline,
///  NUL,
///  and other unprintable characters as `\u{...}`.
/// Why:
///  The worker's record quotes the lines once more;
///  without this,
///  they would carry two layers
///      of escapes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unescape(quoted: string): string // '\\n' -> '\n', '\\u{1b}' -> '\x1b', ...
/// ```
pub fn unescape(quoted: &str) -> String {
    let mut text = String::with_capacity(quoted.len());
    // `chars()` walks the text character by character; `peekable` allows looking one ahead.
    let mut rest = quoted.chars().peekable();
    while let Some(current) = rest.next() {
        if current != '\\' {
            text.push(current);
            continue;
        }
        match rest.next() {
            Some('n') => text.push('\n'),
            Some('r') => text.push('\r'),
            Some('t') => text.push('\t'),
            Some('0') => text.push('\0'),
            Some('u') if rest.peek() == Some(&'{') => {
                rest.next();
                let mut digits = String::new();
                // `peek` looks at the next character without taking it; the digits end at the closing brace.
                while let Some(digit) = rest.peek().copied() {
                    if digit == '}' {
                        break;
                    }
                    rest.next();
                    digits.push(digit);
                }
                rest.next();
                // An escape that names no character is kept as written.
                match u32::from_str_radix(&digits, 16)
                    .ok()
                    .and_then(char::from_u32)
                {
                    Some(decoded) => text.push(decoded),
                    None => text.push_str(&format!("\\u{{{digits}}}")),
                }
            }
            Some(other) => text.push(other),
            None => text.push('\\'),
        }
    }
    return text;
}

/// What:
///  Shorten a line to `LINE_BYTES` bytes at a character boundary and mark the cut.
///       `is_char_boundary` tells whether a byte index starts a character.
/// Why:
///  One endless line must not fill the record.
pub fn cut(line: &str) -> String {
    if line.len() <= LINE_BYTES {
        return line.to_string();
    }
    let mut end = LINE_BYTES;
    while !line.is_char_boundary(end) {
        end -= 1;
    }
    return format!("{}{CUT_MARK}", &line[..end]);
}

/// What:
///  Keep one line a server wrote,
///  given in helix-lsp's quoted form without the outer quotes.
/// Why:
///  Called by the log bridge for every standard-error line.
pub fn remember(server: &str, quoted: &str) {
    let text = unescape(quoted);
    // A line arrives with its newline; the record lists lines, so the newline is dropped.
    let line = cut(text.strip_suffix('\n').unwrap_or(&text));
    let mut all = tails();
    let tail = all.entry(server.to_string()).or_default();
    if tail.retired {
        return;
    }
    // A line after the end of a stream comes from a new process of the same server.
    if tail.closed {
        tail.lines.clear();
        tail.closed = false;
    }
    tail.lines.push_back(line);
    while tail.lines.len() > KEPT_LINES {
        tail.lines.pop_front();
    }
}

/// What:
///  Note that a server's standard error ended.
///  Called by the log bridge.
/// Why:
///  The worker reports an ended server once this happened;
///  a retired tail is finished then.
pub fn ended(server: &str) {
    let mut all = tails();
    let tail = all.entry(server.to_string()).or_default();
    if tail.retired {
        all.remove(server);
        return;
    }
    tail.closed = true;
}

/// Whether a server's standard error ended after its newest line.
pub fn is_closed(server: &str) -> bool {
    if let Some(tail) = tails().get(server) {
        return tail.closed && !tail.retired;
    }
    return false;
}

/// What:
///  Copy a server's kept lines,
///  oldest first,
///  and leave them where they are.
/// Why:
///  The worker shows why a start failed while the lines are still to be reported once
///      as the log record of the ended server.
pub fn lines(server: &str) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    if let Some(tail) = tails().get(server)
        && !tail.retired
    {
        for line in tail.lines.iter() {
            kept.push(line.clone());
        }
    }
    return kept;
}

/// What:
///  Remove and return a server's kept lines,
///  oldest first;
///  while its stream is still open,
///       leave a retired tail that drops the rest of that stream.
/// Why:
///  Each line is reported once,
///  and only as the words of the process that wrote it.
pub fn take(server: &str) -> Vec<String> {
    let mut all = tails();
    let (lines, closed) = match all.remove(server) {
        Some(tail) => (tail.lines.into_iter().collect(), tail.closed),
        None => (Vec::new(), false),
    };
    if !closed {
        all.insert(
            server.to_string(),
            Tail {
                retired: true,
                ..Tail::default()
            },
        );
    }
    return lines;
}

/// What:
///  Drop a server's kept lines without reporting them.
/// Why:
///  The worker stopped the server itself;
///  nothing it wrote is a report.
pub fn forget(server: &str) {
    let _not_reported = take(server);
}

/// Unescaping,
///  cutting,
///  the line count,
///  and stream ends,
///  with server names no other test uses.
#[cfg(test)]
#[path = "stderr_tail_tests.rs"]
mod tests;
