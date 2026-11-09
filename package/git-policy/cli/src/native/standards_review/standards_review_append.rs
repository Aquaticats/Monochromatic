//! What:
//!  How the service adds a review entry to the repository's live shadow log,
//! which every worktree's service instance
//!  appends to at the same time:
//! one whole-entry write on an append-mode descriptor,
//! once per commit,
//! creating the
//!  directory and the file when missing.
//! Why:
//!  The live log is an untracked file in the common repository directory (`monochromatic-git-policy-cli/standards-review-shadow.md`),
//! and no
//!  lock may be taken.
//! One write of the whole entry on an `O_APPEND` descriptor lands at the end of the file in one step,
//!  so two instances (threads or processes,
//! each with its own descriptor) cannot interleave their entries.
//! Nothing else
//!  edits this file,
//! so the protection a tracked,
//! session-edited log needed is gone:
//! the per-worktree spool of delivered
//!  entries,
//! the bounded repair that appended again an entry another writer's whole-file rewrite had lost,
//! and the rule
//!  that the service never creates the log.
//! A write the operating system cuts short (a signal,
//! a full disk) is
//!  continued with the rest of the same entry,
//! so an entry is whole unless the disk refuses it.
//! A review counts as delivered only by the receipt line that ends its own entry,
//! compared as one whole
//!  line of the log:
//! text a remote answer or a subject happens to spell cannot pass for a receipt,
//! and an entry cut
//!  short before that line is not a finished review,
//! so a retry appends the whole entry and leaves the fragment where it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!(await readLog(log)).split('\n').includes(receiptLine(sha))) await appendFile(log, text); // one write on an 'a' handle
//! ```

use crate::private_storage::create_private_directory_all;
use crate::standards_review_entry::receipt_line;
use std::io::Write;
use std::path::Path;

/// What a delivery did.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivery {
    /// The entry was appended.
    Appended,
    /// The log already names this commit,
    /// so nothing was added.
    AlreadyThere,
}

/// What:
///  Whether the log's text carries the whole receipt line of `sha`.
/// Why:
///   A receipt is a line of its own,
/// compared whole,
/// so no text that merely names the commit can pass for one,
/// and an entry
///  the system cut short before its last line is not a finished review.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// text.split('\n').includes(receiptLine(sha))
/// ```
fn text_names_commit(text: &str, sha: &str) -> bool {
    let wanted: String = receipt_line(sha);
    for line in text.lines() {
        if line == wanted {
            return true;
        }
    }
    return false;
}

/// Whether the log names the commit already;
/// a log that cannot be read names nothing.
pub fn names_commit(log: &Path, sha: &str) -> bool {
    return match std::fs::read_to_string(log) {
        Ok(text) => text_names_commit(text.as_str(), sha),
        Err(_) => false,
    };
}

/// What:
///  Append `text` to the log in one write on an append-mode descriptor,
/// creating the log and its directory when missing.
/// Why:
///   The kernel positions an `O_APPEND` write at the end of the file and writes it whole,
/// so concurrent appenders never
///   interleave;
/// `write_all` continues a cut-short write with the rest of the same entry.
/// A final link is refused,
/// and a
///   created file is private because entries carry commit subjects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(dirname(log), { recursive: true, mode: 0o700 }); await appendFile(log, text, { mode: 0o600 });
/// ```
fn append_whole(log: &Path, text: &str) -> std::io::Result<()> {
    if let Some(directory) = log.parent() {
        create_private_directory_all(directory)?;
    }
    // `mut` allows the builder to be configured step by step.
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.append(true).create(true);
    #[cfg(unix)]
    {
        // The trait adds `.mode` and `.custom_flags` to open options on Unix.
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(crate::private_storage::PRIVATE_FILE_MODE).custom_flags(libc::O_NOFOLLOW);
    }
    let mut file: std::fs::File = options.open(log)?;
    return file.write_all(text.as_bytes());
}

/// What:
///  Deliver one entry for `sha` to `log`:
/// nothing if the log already carries the commit's receipt line,
/// otherwise append it whole.
/// Why:
///   Keeping the receipt test first makes a repeated item (a crash after the append,
/// before the item was settled)
///   add nothing.
/// A log that does not exist yet names nothing;
/// any other read failure is reported,
/// because appending
///   after it could repeat an entry.
/// A log whose entry for the commit was cut short before its receipt is appended to,
/// never taken as
///  finished and never rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deliver({ log, sha, text })
/// ```
pub fn deliver(log: &Path, sha: &str, text: &str) -> Result<Delivery, String> {
    let current: String = match std::fs::read_to_string(log) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("reading {} failed: {error}", log.display())),
    };
    if text_names_commit(current.as_str(), sha) {
        return Ok(Delivery::AlreadyThere);
    }
    if let Err(error) = append_whole(log, text) {
        return Err(format!("appending to {} failed: {error}", log.display()));
    }
    return Ok(Delivery::Appended);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "standards_review_append_tests.rs"]
mod tests;
