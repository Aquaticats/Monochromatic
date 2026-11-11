//! What:
//!  The first line of a push's output that gives a reason for its failure.
//! Why:
//!   Git prints the remote it pushed to above everything it says about the push
//! (`transport.c`,
//! `print_push_status`,
//! which writes `To %s` before any reference's line),
//! so the first line of a failed push's output is where the push went and not why it failed.
//! A record that named that line as the cause said only the destination:
//! 20 of the retry lines this host's service wrote on `2026-11-10` for the real worktrees of
//! `Monochromatic` said
//! `cause: To https://github.com/Aquaticats/Monochromatic`,
//! and the reason was on the line after it
//! (`doc/planning/git-policy-cli-push-jams-2026-11-10.md`).
//! A line that holds only a remote and no other line is still the answer,
//! so the words are never dropped.
//! The reading stands in a file of its own so that the retry's plan and its list stay short enough to read whole
//! (`push_retry.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! causeLine(text) // '! [remote rejected] main -> main (missing necessary objects)'
//! ```

/// The words with which Git names the remote it pushed to,
/// on a line of their own above the references'
///  lines.
pub const REMOTE_HEADER: &str = "To ";

/// What:
///  The first line of a push's output that gives a reason for its failure,
/// or nothing for output with no line at all.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// causeLine(text) // '! [remote rejected] main -> main (missing necessary objects)'
/// ```
pub fn cause_line(text: &str) -> Option<&str> {
    let mut remote: Option<&str> = None;
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with(REMOTE_HEADER) {
            remote = Some(trimmed);
            continue;
        }
        return Some(trimmed);
    }
    return remote;
}

/// The reading of a failure's cause is controlled in the test module.
#[cfg(test)]
#[path = "push_retry_cause_line_tests.rs"]
mod tests;
