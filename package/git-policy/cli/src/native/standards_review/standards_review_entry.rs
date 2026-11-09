//! What:
//!  The log entries of the standards-review shadow run,
//! rendered as the root `mise.toml` task `shadow-review` writes
//!  them.
//! Why:
//!  `doc/audit/standards-review-gate-shadow.md` already holds that task's entries,
//! and readers and scorers parse them,
//!  so the port writes the same lines;
//! `standards_review_append.rs` appends each entry,
//! whole,
//! to the live log.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! entry('- Date:'); entry(`  ${new Date().toISOString()}`); // ...
//! ```

use crate::standards_review_request::ROUTE;
use crate::standards_review_input::CAP_LINES;
use crate::standards_review_text::on_one_line;

/// The label of the line that records a finished review,
/// and the only place a full object name appears in an entry.
pub const RECEIPT_LABEL: &str = "reviewed";

/// Leading characters of the commit name the log prints.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const SHORT_SHA_LENGTH: usize = 12;

/// What:
///  Everything one review entry prints.
/// Why:
///   The renderer is a pure function of these values,
/// so a test can fix the clock and the elapsed time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReviewEntry = { date: string; sha: string; subject: string; codeFiles: number; diffLines: number; truncated: boolean; inputUnits: number; seconds: string; top: string; outcome: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReviewEntry {
    /// `toISOString` text of the moment the entry is written.
    pub date: String,
    /// Full commit name.
    pub sha: String,
    /// Subject line.
    pub subject: String,
    /// Code files in the commit.
    pub code_files: usize,
    /// Lines of the whole patch.
    pub diff_lines: usize,
    /// Whether the patch was over the cap.
    pub truncated: bool,
    /// Reported model input length in UTF-16 units.
    pub input_units: usize,
    /// One-decimal seconds of the gateway call.
    pub seconds: String,
    /// The judged choice,
    /// or empty.
    pub top: String,
    /// The `Findings kept` line.
    pub outcome: String,
}

/// What:
///  The line every entry ends with:
/// this review of `sha` finished,
/// named by the whole object name.
/// Why:
///   It is the log's receipt and the once-per-commit test reads it as one whole line,
/// so only an entry
///  that was written whole can carry it,
/// and no text an entry's own fields repeat can pass for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receiptLine = (sha: string) => `  ${RECEIPT_LABEL}: ${sha}`;
/// ```
pub fn receipt_line(sha: &str) -> String {
    return format!("  {RECEIPT_LABEL}: {sha}");
}

/// What:
///  The first `SHORT_SHA_LENGTH` characters of a commit name.
/// Why:
///   The log names commits by that prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sha.slice(0, 12)
/// ```
fn short(sha: &str) -> String {
    return sha.chars().take(SHORT_SHA_LENGTH).collect();
}

/// What:
///  The entry for a commit with no code files:
/// its skip lines and the receipt.
/// Why:
///   Coverage stays visible:
/// every skipped landing leaves a line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entry('- Skipped:'); entry(`  ${sha.slice(0, 12)} (no code files):`); entry(`  \`${subject}\``); entry(receiptLine(sha));
/// ```
pub fn render_skipped(sha: &str, subject: &str) -> String {
    return format!("- Skipped:\n  {} (no code files):\n  `{}`\n{}\n", short(sha), on_one_line(subject), receipt_line(sha));
}

/// What:
///  The review entry:
/// its fields and the receipt line of the commit.
/// Why:
///   `Named superlinear cost` is the choice for `yes` and `no`,
/// the two-line gesture for `partial`,
/// and `unjudged` for
///   anything else,
/// as the script prints it.
/// The receipt is the entry's last line,
/// so only an entry written whole carries one and the
///  once-per-commit test cannot find it in an entry the system cut short.
/// The entry ends with one line feed and no blank line:
/// the write of the next entry supplies the line feed
///  that separates them,
/// so two entries always sit one blank line apart and the last one ends the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entry('- Date:'); // ... entry(receiptLine(entry.sha)); entry('')
/// ```
pub fn render_review(entry: &ReviewEntry) -> String {
    let capped: String = if entry.truncated { format!(", capped at {CAP_LINES}") } else { String::new() };
    let judged: String = match entry.top.as_str() {
        "partial" => String::from("  partial (cost gestured as unverified,\n  not named)"),
        "yes" | "no" => format!("  {}", entry.top),
        _ => String::from("  unjudged"),
    };
    return format!(
        "- Date:\n  {date}\n  Subject:\n  {sha} `{subject}`\n  Input size:\n  `{files} code files, {lines} diff lines{capped}, luna input {units} chars`\n  Reviewer route:\n  `{ROUTE}`\n  Wall-clock cost:\n  {seconds} s\n  Named superlinear cost:\n{judged}\n  Findings kept:\n    `````text\n    {outcome}\n    `````\n{receipt}\n",
        date = entry.date,
        sha = short(entry.sha.as_str()),
        subject = on_one_line(entry.subject.as_str()),
        files = entry.code_files,
        lines = entry.diff_lines,
        units = entry.input_units,
        seconds = entry.seconds,
        outcome = on_one_line(entry.outcome.as_str()),
        receipt = receipt_line(entry.sha.as_str()),
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_entry_tests.rs"]
mod tests;
