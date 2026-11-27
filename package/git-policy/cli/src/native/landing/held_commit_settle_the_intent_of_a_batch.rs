//! What:
//!  The records a settlement reads before it decides anything about a held commit:
//!  the batch's own intent,
//!  the finished record of a batch whose intent is gone,
//!  and whether the branch already holds the commit the batch made.
//! Why:
//!   The intent is the batch's own record of where the branch was and where it goes;
//!  without it nothing is decided about the commit,
//!  and a record that cannot be read is said rather than guessed at.
//!  A kill between the end of a batch and the removal of its record leaves one whose intent is gone,
//!  and the branch says whether the commit is on it:
//!  a comparison that read only the commit the batch started with would say for ever that it ended
//!  without landing
//!  (`held_commit_settle_carrying_the_change.rs`).
//!  These reads sit in a file of their own because the settlement was over the limit the repository linter
//!  enforces (`monochromatic-lint.config.jsonc`,
//!  `rust/max-lines`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const intent = intentOf(held); // { intent } | { absent: true } | { damaged: true }
//! ```

use crate::held_commit_record::Held;
use crate::service_journal::{
    DONE_EXTENSION,
    Intent,
    OPEN_EXTENSION,
    batch_path,
    parse_intent,
};
use crate::transaction_git::{
    GitContext,
    GitRequest,
    run_git,
};
use std::path::PathBuf;

/// The exit status `git merge-base --is-ancestor` gives when the first commit is not below the second.
const NOT_AN_ANCESTOR: i32 = 1;

/// What:
///  The intent of a held batch,
///  `Ok(None)` when it is gone,
///  and the reason when it cannot be read.
/// Why:
///   The intent is the batch's own record of where the branch was and where it goes;
///  without it nothing is decided about the commit,
///  and a record that cannot be read is said rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readIntent(journal, seq) // { intent } | { absent: true } | { damaged: true }
/// ```
pub(crate) fn intent_of(held: &Held) -> Result<Option<Intent>, String> {
    let path: PathBuf = batch_path(held.journal.as_path(), held.seq, OPEN_EXTENSION);
    let Ok(text) = std::fs::read_to_string(path.as_path()) else {
        return Ok(None);
    };
    return match parse_intent(text.as_str()) {
        Some(intent) => Ok(Some(intent)),
        None => Err(format!(
            "the record of the batch at `{}` cannot be read",
            path.display()
        )),
    };
}

/// Whether the commit the branch names is the batch's own or lies above it,
/// so the batch landed and only its index update is owed.
pub(crate) fn landed_below(
    git: &GitContext,
    held: &Held,
    intent: &Intent,
    current: &str,
) -> bool {
    let request: GitRequest = GitRequest::new(
        held.repository.as_path(),
        &["merge-base", "--is-ancestor", intent.new.as_str(), current],
    );
    let Ok(output) = run_git(git, &request) else {
        return false;
    };
    if output.succeeded() {
        return true;
    }
    return output.code != Some(NOT_AN_ANCESTOR) && current == intent.new;
}

/// What:
///  The record of the batch this held record belongs to once that batch ended,
///  nothing when no finished record is there.
/// Why:
///   A settlement that carries the batch onto a branch that moved writes the commit it made in the batch's own record
///  before it moves the branch (`carry`),
///  and the finished record is what names that commit after the branch moved.
///  A kill between the finished record and the removal of this one leaves this one naming the commit the batch started
///  with,
///  so the finished record is the only place the commit that landed is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = readIntent(batchPath(journal, seq, 'done'))
/// ```
pub(crate) fn finished_of(held: &Held) -> Option<Intent> {
    let path: PathBuf = batch_path(held.journal.as_path(), held.seq, DONE_EXTENSION);
    let text: String = std::fs::read_to_string(path.as_path()).ok()?;
    return parse_intent(text.as_str());
}
