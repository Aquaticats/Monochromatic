//! What:
//!  Where a work record is when a move of its claim finds nothing,
//!  and the words that say what was found out and what could not be.
//! Why:
//!  A settlement renames the claim's file out of `claimed` and the same failure,
//! "No such file or directory" or "the claim is not in claimed",
//!  stands for several orders that are not one:
//! another instance took the claim over when its lease ran out,
//! a recovery is moving the record,
//! a repository was removed under the service,
//!  or the directory the record was to move into is not there.
//! The record's own places say which of them happened,
//! so the words of a failed move are read from them
//!  instead of guessed at from the claim's absence.
//!  A guess is worse than nothing here:
//! it sends a reader looking for another instance that never existed,
//! while the work of a record that went with a removed repository is recorded nowhere.
//!  A place that could not be read is the third answer,
//! and it is never told as an absence:
//! a directory the process may not list answers with an error,
//! and saying that nothing is in it would state a removal nobody made
//! (`AGENTS.md`, the rule on user-facing diagnostics).
//! The words say that the look failed, name the place and its error,
//! and give the person the permission that has to change.
//!  An order that has two endings says both,
//!  and says which of them happened is read afterwards:
//! a recovery of a record queues it again or gives it up on its last try,
//! and the message names both places rather than the one the writer hopes for.
//! The words name no process:
//!  the claim protocol has no process identity,
//! and an instance's number means nothing to the person reading the journal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! wordsForAMoveThatFoundNothing(workDir, id, token) // "another instance ... its work is run again there"
//! ```

use crate::work_item::{
    CLAIMED, DONE, FAILED, PENDING, RECOVERING, entries_oldest_first_or_the_error, id_of_entry, names_claim, state_directory,
};
use std::path::{Path, PathBuf};

/// What:
///  The sentence that follows a failed move of `id`'s claim:
/// the place the record is in now, or what could not be found out about it.
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
    return match where_the_record_is(work_dir, id, token) {
        Ok(words) => words,
        // A place that could not be read is its own answer, and it is the whole sentence.
        Err(could_not_be_found_out) => could_not_be_found_out,
    };
}

/// What:
///  The sentence for a place that could not be read at all.
/// Why:
///   A directory the process may not list, or a name it may not look up, must not be told as an empty one:
/// the words name the place, keep the error the filesystem gave,
/// say that whether a record is there could not be found out,
///  and name the permission a person has to change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the directory ${place} could not be read (${error}), so whether a record of ${id} is in it could not be found out ...`
/// ```
pub(crate) fn a_directory_that_could_not_be_read(place: &Path, id: &str, error: &std::io::Error) -> String {
    return format!(
        "the directory {} could not be read ({error}), so whether a record of {id} is in it could not be found out and nothing of the record was moved here; the permissions of that directory are what to look at",
        place.display()
    );
}

/// What:
///  The sentence for a `claimed` directory that could not be read.
/// Why:
///   The claim's own directory is the one place the settlement was looking for,
/// so its failure says plainly that the claim's presence there is unknown,
/// which is the question the reader of the failure has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the directory ${place} could not be read (${error}), so whether the claim of ${id} is still in claimed could not be found out ...`
/// ```
pub(crate) fn the_claim_directory_could_not_be_read(place: &Path, id: &str, error: &std::io::Error) -> String {
    return format!(
        "the directory {} could not be read ({error}), so whether the claim of {id} is still in claimed, and where its record stands, could not be found out; nothing of the record was moved here, and the permissions of that directory are what to look at",
        place.display()
    );
}

/// The sentence for a state of one record that could not be looked at,
/// which is the same answer as a directory that could not be read:
/// whether the record is there is not known.
fn a_state_that_could_not_be_looked_at(place: &Path, id: &str, error: &std::io::Error) -> String {
    return format!(
        "the state {} could not be looked at ({error}), so whether a record of {id} is there could not be found out and nothing of the record was moved here; the permissions of that directory are what to look at",
        place.display()
    );
}

/// What:
///  Whether `path` is a directory, or the error that kept that from being found out.
/// Why:
///  `Path::is_dir` answers `false` for a directory whose parent could not be looked at,
/// and a work directory or a `claimed` directory is told absent only when the filesystem says it is absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isADirectory(path) // Result<boolean, Error>
/// ```
fn is_a_directory(path: &Path) -> std::io::Result<bool> {
    return match std::fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    };
}

/// Whether a record of `id` is at `path`, or the error that kept that from being found out.
fn is_a_record(path: &Path) -> std::io::Result<bool> {
    return match std::fs::metadata(path) {
        Ok(_metadata) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    };
}

/// What:
///  The sentence for where the record of `id` was found,
///  or the sentence for the place that could not be read before it was found.
/// Why:
///   The three answers are values here,
/// so a caller cannot take the third for the second:
/// each place is asked in the order the record could have left in,
/// and the first look that fails ends the search with its own sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whereTheRecordIs(workDir, id, token): Result<string, string>
/// ```
fn where_the_record_is(work_dir: &Path, id: &str, token: &str) -> Result<String, String> {
    // A work directory that is not there answers every other look with nothing, so it is asked first.
    match is_a_directory(work_dir) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(format!(
                "the work directory {} is not there, so no state of {id} can be read: the record went with it and its work is recorded nowhere",
                work_dir.display()
            ));
        }
        Err(error) => return Err(a_directory_that_could_not_be_read(work_dir, id, &error)),
    }
    let claimed: PathBuf = state_directory(work_dir, CLAIMED);
    match is_a_directory(claimed.as_path()) {
        Ok(true) => {}
        Ok(false) => {
            return Ok(format!(
                "the {CLAIMED} directory is not there under {}, so no state of {id} can be read: the record went with it and its work is recorded nowhere",
                work_dir.display()
            ));
        }
        Err(error) => return Err(the_claim_directory_could_not_be_read(claimed.as_path(), id, &error)),
    }
    // The claim's own directory: the file the move was for, or another instance's claim of the same record.
    let mut another: Option<String> = None;
    let claims = entries_oldest_first_or_the_error(work_dir, CLAIMED).map_err(|error| the_claim_directory_could_not_be_read(claimed.as_path(), id, &error))?;
    for (name, _modified) in claims {
        if id_of_entry(name.as_str()) != id {
            continue;
        }
        if names_claim(name.as_str(), id, token) {
            return Ok(format!(
                "the claim is still in {CLAIMED}/{name}, so nothing of the record moved here and another look runs its work again when its lease runs out"
            ));
        }
        if another.is_none() {
            another = Some(name);
        }
    }
    if let Some(name) = another {
        return Ok(format!(
            "another instance took the claim of {id} back when its lease ran out and claimed it again as {CLAIMED}/{name}, so its work is run again there and nothing of it was moved here"
        ));
    }
    let recovering: PathBuf = state_directory(work_dir, RECOVERING);
    match is_a_directory(recovering.as_path()) {
        Ok(true) => {
            let taking = entries_oldest_first_or_the_error(work_dir, RECOVERING)
                .map_err(|error| a_directory_that_could_not_be_read(recovering.as_path(), id, &error))?;
            for (name, _modified) in taking {
                if id_of_entry(name.as_str()) != id {
                    continue;
                }
                // A recovery has two endings, and this holder cannot tell which one this record meets.
                return Ok(format!(
                    "another instance is taking the claim of {id} out of {CLAIMED} into {RECOVERING}/{name}, and that recovery has two endings: it leaves the record in {PENDING}/{id} when the record still has tries left, where the next look that claims it does its work, and in {FAILED}/{id} when its tries are all used, where nothing looks at it again and its work happens only if a person does it; where the record stands when the recovery ends is which of the two happened, and nothing of it was moved here"
                ));
            }
        }
        Ok(false) => {}
        Err(error) => return Err(a_directory_that_could_not_be_read(recovering.as_path(), id, &error)),
    }
    let queued: PathBuf = state_directory(work_dir, PENDING).join(id);
    match is_a_record(queued.as_path()) {
        Ok(true) => {
            return Ok(format!(
                "another instance took the claim of {id} back when its lease ran out and left it queued as {PENDING}/{id}, so its work is run again by the next look that claims it and nothing of it was moved here"
            ));
        }
        Ok(false) => {}
        Err(error) => return Err(a_state_that_could_not_be_looked_at(queued.as_path(), id, &error)),
    }
    let settled: PathBuf = state_directory(work_dir, DONE).join(id);
    match is_a_record(settled.as_path()) {
        Ok(true) => {
            return Ok(format!("the record of {id} is already in {DONE}/{id}, so a settlement moved it before this one and its work is there as finished"));
        }
        Ok(false) => {}
        Err(error) => return Err(a_state_that_could_not_be_looked_at(settled.as_path(), id, &error)),
    }
    let given_up: PathBuf = state_directory(work_dir, FAILED).join(id);
    match is_a_record(given_up.as_path()) {
        Ok(true) => {
            return Ok(format!("the record of {id} is already in {FAILED}/{id}, so it was given up on before this one and no later look runs its work"));
        }
        Ok(false) => {}
        Err(error) => return Err(a_state_that_could_not_be_looked_at(given_up.as_path(), id, &error)),
    }
    return Ok(format!(
        "no file of {id} is in {CLAIMED}, {RECOVERING}, {PENDING}, {DONE} or {FAILED} under {}, and every one of those places was looked at, so the record is in none of them: something other than a move of this queue removed it, and nothing of it was moved here",
        work_dir.display()
    ));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "work_record_where_it_went_tests.rs"]
mod tests;
