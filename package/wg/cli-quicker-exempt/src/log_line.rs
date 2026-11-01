//! Writes lines to the watcher's standard error without ever panicking,
//! inside a byte budget,
//! and escapes untrusted text for the destination that receives it.
//! The standard `eprintln!` panics when a write fails,
//! for example on a full `/run` tmpfs,
//! and a panic in the watcher would end every policy it enforces without a word.
//! The log file lives on that tmpfs and has no other bound,
//! so a failure that repeats on every poll could fill it.

/// Writes to standard error.
use std::io::Write;
/// Shared budget state.
use std::sync::Mutex;
/// Window timing.
use std::time::{Duration, Instant};

/// Words that start every notice about the log file itself,
/// so the start of `up` relays them next to the loud reports:
/// a line dropped by a budget may have been a report.
pub const LOG_NOTICE_MARKER: &str = "LOG LIMIT";

/// Bytes the log file may receive per window.
pub const LOG_BYTES_PER_WINDOW: u64 = 262_144;
/// Length of one budget window.
pub const LOG_WINDOW: Duration = Duration::from_secs(3600);
/// Bytes the log file may receive over the whole life of the watcher,
/// so a watcher that runs for weeks cannot fill the tmpfs the log lives on.
pub const LOG_BYTES_TOTAL: u64 = 4_194_304;

/// What the budget decided for one line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Admission {
    /// Write the line,
    /// after a note about lines dropped in the previous window when that count is not zero.
    Write {
        /// Lines dropped since the last write.
        dropped_before: u64,
    },
    /// Write the line saying the budget of the window is reached,
    /// and drop the lines that follow until the window ends.
    BudgetReached,
    /// Write the line saying the budget for the whole life of the watcher is spent,
    /// and drop every line after it.
    TotalSpent,
    /// Drop the line.
    Drop,
}

/// Byte budget of one log file.
pub struct LogBudget {
    /// Bytes allowed per window.
    limit: u64,
    /// Bytes allowed in total.
    total_limit: u64,
    /// Bytes written in total.
    total: u64,
    /// Whether the budget for the whole life is spent.
    total_spent: bool,
    /// Length of a window.
    window: Duration,
    /// Start of the current window.
    window_start: Option<Instant>,
    /// Bytes written in the current window.
    used: u64,
    /// Lines dropped since the last note.
    dropped: u64,
}

/// Admission of lines.
impl LogBudget {
    /// Builds a budget of `limit` bytes per `window` and `total_limit` bytes in all.
    pub const fn new(limit: u64, window: Duration, total_limit: u64) -> LogBudget {
        return LogBudget {
            limit,
            total_limit,
            total: 0,
            total_spent: false,
            window,
            window_start: None,
            used: 0,
            dropped: 0,
        };
    }

    /// Decides whether a line of `bytes` bytes is written at `now`.
    pub fn admit(&mut self, bytes: u64, now: Instant) -> Admission {
        let expired = match self.window_start {
            Some(start) => now.duration_since(start) >= self.window,
            None => true,
        };
        if expired {
            self.window_start = Some(now);
            self.used = 0;
        }
        if self.total_spent {
            return Admission::Drop;
        }
        let window_fits = self.used.saturating_add(bytes) <= self.limit;
        let total_fits = self.total.saturating_add(bytes) <= self.total_limit;
        if window_fits && total_fits {
            self.used += bytes;
            self.total += bytes;
            let dropped_before = self.dropped;
            self.dropped = 0;
            return Admission::Write { dropped_before };
        }
        if window_fits {
            // The window has room and the whole life has not:
            // the log is spent for good, and this is said once.
            self.total_spent = true;
            return Admission::TotalSpent;
        }
        self.dropped += 1;
        if self.dropped == 1 {
            return Admission::BudgetReached;
        }
        return Admission::Drop;
    }
}

/// Budget of this process's log file.
static BUDGET: Mutex<LogBudget> = Mutex::new(LogBudget::new(LOG_BYTES_PER_WINDOW, LOG_WINDOW, LOG_BYTES_TOTAL));

/// Writes one line to standard error and ignores a failed write.
pub fn write_unchecked(line: &str) {
    let _ignored = writeln!(std::io::stderr().lock(), "{line}");
}

/// Writes one line to the log inside the byte budget.
/// A failed write is ignored,
/// because the failure reports have other channels and a panic here would end the watcher.
pub fn log_line(line: &str) {
    let admission = match BUDGET.lock() {
        Ok(mut budget) => budget.admit(line.len() as u64 + 1, Instant::now()),
        Err(_poisoned) => Admission::Write { dropped_before: 0 },
    };
    match admission {
        Admission::Write { dropped_before } => {
            if dropped_before > 0 {
                write_unchecked(&format!(
                    "wg-quicker-exempt: {LOG_NOTICE_MARKER}: {dropped_before} log line(s) were dropped because the log budget of {LOG_BYTES_PER_WINDOW} bytes per hour was reached"
                ));
            }
            write_unchecked(line);
        }
        Admission::BudgetReached => {
            write_unchecked(&format!(
                "wg-quicker-exempt: {LOG_NOTICE_MARKER}: the log budget of {LOG_BYTES_PER_WINDOW} bytes per hour is reached, so later lines this hour are dropped from this file; failures still reach the journal and the desktop"
            ));
        }
        Admission::TotalSpent => {
            write_unchecked(&format!(
                "wg-quicker-exempt: {LOG_NOTICE_MARKER}: the log file has received its whole budget of {LOG_BYTES_TOTAL} bytes for the life of this watcher, \
so nothing more is written to it, success lines included; failures still reach the journal and the desktop, and `watch-stop` and `watch-start` give a fresh budget"
            ));
        }
        Admission::Drop => {}
    }
}

/// Reports a character that can change how a terminal or a reader shows text:
/// control characters,
/// the line and paragraph separators,
/// the invisible direction and joining marks that reorder or hide text,
/// and the invisible tag characters.
fn is_unsafe_character(character: char) -> bool {
    return character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200b}'..='\u{200f}'
                | '\u{2028}'..='\u{2029}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{2064}'
                | '\u{2066}'..='\u{2069}'
                | '\u{feff}'
                | '\u{e0000}'..='\u{e007f}'
        );
}

/// Escapes untrusted text for a line of a log,
/// the journal,
/// or a terminal:
/// every character that could start an escape sequence,
/// end the line,
/// or reorder the text becomes `\u{XXXX}`.
pub fn sanitize(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        if is_unsafe_character(character) {
            escaped.push_str(&format!("\\u{{{:04x}}}", u32::from(character)));
        } else {
            escaped.push(character);
        }
    }
    return escaped;
}

/// Escapes text for the body of a desktop notification,
/// whose markup subset treats `<`,
/// `>` and `&` as syntax.
pub fn escape_markup(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    return escaped;
}
