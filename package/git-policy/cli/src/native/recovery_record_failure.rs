//! What:
//!  The recovery failure for a journal record that cannot be read, with the file, the transaction and the ways out
//!  named.
//! Why:
//!  A strict parser knows only the bytes it was given, so its message names the kind of record and the field.
//!  The person who sees it needs the path of the file, the transaction directory, whether the file can be removed or
//!  restored, and what the wrapper does next; every guarded command stops with exit status 2 until then.
//!  The caller that read the file knows its path, so it wraps the parser's failure here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! try { return parseLandingRecord(bytes); } catch (error) { throw nameRecordFailure({ error, path, kind: 'landing' }); }
//! ```

/// The fail-closed recovery failure.
use super::recovery_error::RecoveryError;
/// The steps that finish a retained transaction by hand.
use super::recovery_manual_steps::finish_by_hand_steps;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Which journal record failed to read.
/// Why:
///   Each record has its own way out: what removing or restoring it makes the wrapper do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordKind = 'preparing' | 'landing' | 'index-lock' | 'ref-updated';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordKind {
    /// `preparing.json`, the first record of a transaction.
    Preparing,
    /// `landing-<n>.json`, written before the branch moves.
    Landing,
    /// `index-lock-<n>.json`, the identity of the real index lock the transaction created.
    IndexLock,
    /// `ref-updated.json`, written after the branch moved.
    RefUpdated,
}

/// What:
///  The parser's failure, extended with the path of the file, the transaction directory and what the person can do.
/// Why:
///   See the module comment.
///  The parser's own sentence stays in the message as the detail, so the field that failed is still named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw nameRecordFailure({ error, path, kind });
/// ```
pub fn name_record_failure(error: RecoveryError, path: &Path, kind: RecordKind) -> RecoveryError {
    let directory: &Path = path.parent().unwrap_or(path);
    let transaction: String = directory
        .file_name()
        .map_or_else(|| { return String::from("(unnamed)"); }, |name| { return name.to_string_lossy().into_owned(); });
    let way_out: String = match kind {
        RecordKind::Preparing => format!(
            "preparing.json is the first record of a transaction. If the directory holds no landing-<n>.json and no \
             ref-updated.json, no commit was made: remove {}, and the next guarded command discards the transaction \
             and goes on; if an index.lock file remains in the repository's Git directory after that and no Git \
             command is running, remove it as well. If it holds either file, the commit may already be on the branch: run `git log` to check. {}",
            path.display(),
            finish_by_hand_steps(directory, None)
        ),
        RecordKind::Landing => format!(
            "A landing record names the post-index and the index lock the landing installs, and recovery cannot decide \
             from the other records what to install. If you have a copy of this file, restore it at {} and the next \
             guarded command recovers the transaction. Otherwise run `git log` to see whether the transaction's commit \
             is on the branch. {}",
            path.display(),
            finish_by_hand_steps(directory, None)
        ),
        RecordKind::IndexLock => format!(
            "This record names the index.lock file the transaction created. Remove {}: the next guarded command then \
             handles the transaction from its other records, but it cannot prove that an index.lock file on disk is \
             the transaction's and leaves it. If index.lock still exists in the repository's Git directory (or the \
             linked worktree's Git directory) and no Git command is running, remove it yourself.",
            path.display()
        ),
        RecordKind::RefUpdated => format!(
            "This record is written after the branch moved. Remove {}: the next guarded command then looks in the \
             branch's reflog for this transaction's entry to decide whether the commit landed, and finishes the \
             transaction from its landing record.",
            path.display()
        ),
    };
    return RecoveryError(format!(
        "Cannot recover transaction {transaction}, so every guarded command stops with exit status 2 until it is \
         resolved: {} cannot be read ({}). Recovery retained the transaction at {}. {way_out}",
        path.display(),
        error.0.trim_end_matches('.'),
        directory.display()
    ));
}

/// Record failure message controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_record_failure_tests.rs"]
mod tests;
