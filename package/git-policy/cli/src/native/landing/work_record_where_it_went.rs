//! What:
//!  Where a work record is when a move of its claim finds nothing,
//!  and the words that say which order happened and what became of its work.
//! Why:
//!  A settlement renames the claim's file out of `claimed` and the same failure,
//! "No such file or directory" or "the claim is not in claimed",
//!  stands for several orders that are not one:
//! another instance took the claim over when its lease ran out,
//! a recovery gave up on the record,
//! a repository was removed under the service,
//!  or the directory the record was to move into is not there.
//! The record's own places say which of them happened,
//! so the words of a failed move are read from them
//!  instead of guessed at from the claim's absence.
//!  A guess is worse than nothing here:
//! it sends a reader looking for another instance that never existed,
//! while the work of a record that went with a removed repository is recorded nowhere,
//!  and a record that waits in `pending` is run again by the next look that claims it.
//! The words name no process:
//!  the claim protocol has no process identity,
//! and an instance's number means nothing to the person reading the journal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! wordsForAMoveThatFoundNothing(workDir, id, token) // "another instance ... its work is run again there"
//! ```

use crate::work_item::{CLAIMED, DONE, FAILED, PENDING, RECOVERING, entries_oldest_first, id_of_entry, names_claim, state_directory};
use std::path::Path;

/// What:
///  The sentence that follows a failed move of `id`'s claim:
/// the place the record is in now, and what became of its work.
/// Why:
///   Every order that leaves a record somewhere else has its own sentence,
/// and each one is read from the directory the record is found in,
/// so a reader of the journal learns which order happened
///  and whether the work will be run again,
/// is already recorded as finished,
/// or had its record taken away with the repository.
///  `token` is the generation the failing move was made for:
/// a name of the same id under another generation is another instance's claim of the record,
/// which is the one order in which two instances have believed they held one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!isDirectory(workDir)) return "the work directory ... is not there, so ..."
/// for (const name of namesIn(workDir, 'claimed')) if (sameId(name)) return sameClaim(name, token) ? "still in claimed" : "claimed it again"
/// ```
pub(crate) fn words_for_a_move_that_found_nothing(work_dir: &Path, id: &str, token: &str) -> String {
    // A work directory that is not there answers every other look with nothing, so it is asked first.
    if !work_dir.is_dir() {
        return format!(
            "the work directory {} is not there, so no state of {id} can be read: the record went with it and its work is recorded nowhere",
            work_dir.display()
        );
    }
    if !state_directory(work_dir, CLAIMED).is_dir() {
        return format!(
            "the {CLAIMED} directory is not there under {}, so no state of {id} can be read: the record went with it and its work is recorded nowhere",
            work_dir.display()
        );
    }
    // The claim's own directory: the file the move was for, or another instance's claim of the same record.
    let mut another: Option<String> = None;
    for (name, _modified) in entries_oldest_first(work_dir, CLAIMED) {
        if id_of_entry(name.as_str()) != id {
            continue;
        }
        if names_claim(name.as_str(), id, token) {
            return format!(
                "the claim is still in {CLAIMED}/{name}, so nothing of the record moved here and another look runs its work again when its lease runs out"
            );
        }
        if another.is_none() {
            another = Some(name);
        }
    }
    if let Some(name) = another {
        return format!(
            "another instance took the claim of {id} back when its lease ran out and claimed it again as {CLAIMED}/{name}, so its work is run again there and nothing of it was moved here"
        );
    }
    for (name, _modified) in entries_oldest_first(work_dir, RECOVERING) {
        if id_of_entry(name.as_str()) != id {
            continue;
        }
        return format!(
            "another instance is taking the claim of {id} out of {CLAIMED} to queue it again ({RECOVERING}/{name}), so its work is run again and nothing of it was moved here"
        );
    }
    if state_directory(work_dir, PENDING).join(id).exists() {
        return format!(
            "another instance took the claim of {id} back when its lease ran out and left it queued as {PENDING}/{id}, so its work is run again by the next look that claims it and nothing of it was moved here"
        );
    }
    if state_directory(work_dir, DONE).join(id).exists() {
        return format!("the record of {id} is already in {DONE}/{id}, so a settlement moved it before this one and its work is there as finished");
    }
    if state_directory(work_dir, FAILED).join(id).exists() {
        return format!("the record of {id} is already in {FAILED}/{id}, so it was given up on before this one and no later look runs its work");
    }
    return format!(
        "no file of {id} is in {CLAIMED}, {RECOVERING}, {PENDING} or {FAILED} under {}, so something other than the queue moved or removed it and no claim of it is left for this instance to settle",
        work_dir.display()
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "work_record_where_it_went_tests.rs"]
mod tests;
