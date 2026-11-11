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
//!  line of the log with the line feed that finishes it:
//! text a remote answer or a subject happens to
//!  spell cannot pass for a receipt,
//! a line feed cannot appear in either,
//! and a receipt the write never reached
//!  the end of is not one,
//! so a retry appends the whole entry and leaves the fragment where it is.
//! Every write begins with a line feed of its own,
//! whatever the log holds,
//! so an entry is never joined to a line the
//!  write before it cut short and the file begins with the first entry's line feed;
//! the entry's own text ends with
//!  one line feed,
//! so two entries are separated by exactly one blank line.
//!  A full log moves aside to the next free generation,
//! and a receipt is looked for in the log and in every generation of it,
//! because the
//!  history is one log to a reader:
//! a commit whose entry rotated away is still reviewed once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = receiptLine(sha) + '\n'; const held = (await readLogWithGenerations(log)).splitAfter('\n').includes(line);
//! if (!held) await appendEntry(log, '\n' + text, UNPINNED_LOG); // one write of the whole entry
//! ```

use crate::earlier_log_lines_added_to_current_log::numbered_copies;
use crate::log_generations_kept_to_a_bound::UNPINNED_LOG;
use crate::log_lines_appended_without_a_lock::append_entry;
use crate::standards_review_entry::receipt_line;
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
///  Whether one byte is the line feed that finishes a line of the log.
///  `&u8` borrows the byte,
/// as `position` hands it over.
/// Why:
///   `position` takes this function by path where a closure would otherwise stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isLineFeed = (byte: number) => byte === '\n'.charCodeAt(0);
/// ```
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether the log's bytes carry the receipt line of `sha` as a finished line,
/// line feed included.
/// Why:
///   A receipt is a line of its own,
/// compared whole,
/// so no text that merely names the commit can pass for one;
/// and the line feed is part of it,
/// because a write the system cut short can stop between the commit name
///  and the end of its line,
/// and bytes the write never finished are not evidence of delivery.
///  The comparison is over the log's bytes,
/// so a byte that is not UTF-8 anywhere in the log cannot hide a receipt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const line of log.splitAfter('\n')) if (line === receiptLine(sha) + '\n') return true;
/// ```
fn bytes_name_commit(log: &[u8], sha: &str) -> bool {
    let wanted: String = format!("{}\n", receipt_line(sha));
    let mut start: usize = 0;
    while let Some(relative) = log[start..].iter().position(is_line_feed_byte) {
        let end: usize = start + relative + 1;
        if &log[start..end] == wanted.as_bytes() {
            return true;
        }
        start = end;
    }
    // What is left has no line feed, so it is a line the write never finished.
    return false;
}

/// Whether the log carries the commit's receipt already;
/// a log that cannot be read names nothing.
pub fn names_commit(log: &Path, sha: &str) -> bool {
    return history_names_commit(log, sha).unwrap_or(false);
}

/// What:
///  Whether the log at `log`,
/// or any older generation of it,
///  carries the commit's receipt line.
/// Why:
///   A rotation keeps every entry,
/// so the generations are the same log to a reader;
/// a receipt the live file no longer holds still
///  names the commit,
/// and a commit whose entry rotated away must not be reviewed a second time.
/// A generation that cannot be read is one
///  this answer does not know about;
/// the live log's own failure is returned,
/// because appending after it could repeat an entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const files = [log, ...numberedCopies(log)]; files.some((f) => splitAfter(read(f), '\n').includes(wanted));
/// ```
///
/// # Errors
/// The system's error when the live log exists and cannot be read.
fn history_names_commit(log: &Path, sha: &str) -> std::io::Result<bool> {
    let current: Vec<u8> = match std::fs::read(log) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error),
    };
    if bytes_name_commit(current.as_slice(), sha) {
        return Ok(true);
    }
    let (Some(parent), Some(name)) = (log.parent(), log.file_name()) else {
        return Ok(false);
    };
    let current_name: String = name.to_string_lossy().into_owned();
    for copy in numbered_copies(parent, current_name.as_str()) {
        if let Ok(bytes) = std::fs::read(copy)
            && bytes_name_commit(bytes.as_slice(), sha)
        {
            return Ok(true);
        }
    }
    return Ok(false);
}

/// What:
///  Append `text` to the log as one whole entry,
/// beginning with a line feed of its own,
/// through the appender every log of the
///  wrapper shares.
/// Why:
///   The kernel positions an `O_APPEND` write at the end of the file and writes it whole,
/// so concurrent appenders never
///   interleave,
/// and a write the system cut short is continued with the rest of the same entry.
/// The entry's line feed is part of the write
///   and is never decided from what the log holds:
/// a reader can see another appender's entry while its write is still arriving,
/// so a separator that depended on the last byte observed would land
///   inside that entry,
/// while an unconditional one keeps every entry on a line of its own.
/// The appender creates the folder private and refuses a
///   final link,
/// because entries carry commit subjects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(dirname(log), { recursive: true, mode: 0o700 }); await appendEntry(log, '\n' + text, retention, { mode: 0o600, noFollow: true });
/// ```
fn append_whole(log: &Path, text: &str) -> std::io::Result<()> {
    let mut entry: Vec<u8> = Vec::with_capacity(text.len() + 1);
    entry.push(b'\n');
    entry.extend_from_slice(text.as_bytes());
    return append_entry(log, entry.as_slice(), &UNPINNED_LOG);
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
///  finished and never rewritten,
/// and the log is read as bytes,
/// so a torn character or a torn line in it never stops a later entry
///  from being written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deliver({ log, sha, text })
/// ```
pub fn deliver(log: &Path, sha: &str, text: &str) -> Result<Delivery, String> {
    let already_there: bool = match history_names_commit(log, sha) {
        Ok(found) => found,
        Err(error) => return Err(format!("reading {} failed: {error}", log.display())),
    };
    if already_there {
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
