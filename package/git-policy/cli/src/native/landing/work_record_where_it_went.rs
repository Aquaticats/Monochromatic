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
    CLAIMED, DONE, FAILED, PENDING, RECOVERING, WhatTheWalkReached, id_of_entry, names_claim, state_directory, the_names_in,
};
use std::path::{Path, PathBuf};

/// What:
///  The walk one look makes over a state directory,
///  given as a value so a control can hand in its own.
/// Why:
///   A look is a sequence of walks,
/// and the states a record moves through are read in an order;
/// a control that makes a walk stop part-way,
/// or that lets a move finish between two of the walks,
/// reads the words the shipped walks would never let it see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AWalkOf = (workDir: string, state: string) => WhatTheWalkReached
/// ```
pub(crate) type AWalkOf<'a> = &'a dyn Fn(&Path, &str) -> WhatTheWalkReached;

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
    return words_from(work_dir, id, token, &the_names_in);
}

/// What:
///  The same words, read through the walk a control hands in.
/// Why:
///   The states of a record are read in an order and each walk can stop part-way;
/// a control that arranges either reads the sentence here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordsFromTheWalk(workDir, id, token, walk)
/// ```
#[cfg(test)]
pub(crate) fn words_from_the_walk(work_dir: &Path, id: &str, token: &str, walk: AWalkOf<'_>) -> String {
    return words_from(work_dir, id, token, walk);
}

/// The body of the words, over the walk `walk`.
fn words_from(work_dir: &Path, id: &str, token: &str, walk: AWalkOf<'_>) -> String {
    return match where_the_record_is(work_dir, id, token, walk) {
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
        "the directory {} could not be read ({error}), so whether a record of {id} is in it could not be found out and nothing of the record was moved here{}",
        place.display(),
        or_the_permission(error)
    );
}

/// What:
///  What a walk that stopped says about the place it walked.
/// Why:
///   A directory that is not there, a name something else has taken,
/// and a directory that answered with an error are three different answers,
/// and a person is sent to permissions only by the third:
/// the first two are absences the filesystem answered,
/// and a claim behind one of them has nothing left to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whatTheFailureMeans(error) // 'the place is not there' | 'something else is at the place' | 'could not be told'
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WhatTheFailureMeans {
    /// The place is not there, so what it held went with it.
    ThePlaceIsNotThere,
    /// Something else holds the place's name, so the queue is not there any more either.
    SomethingElseIsAtThePlace,
    /// The place answered with an error, so nothing is known about it either way.
    CouldNotBeTold,
}

///  What a walk that stopped says about the place it walked.
pub(crate) fn what_the_failure_means(error: &std::io::Error) -> WhatTheFailureMeans {
    return match error.kind() {
        std::io::ErrorKind::NotFound => WhatTheFailureMeans::ThePlaceIsNotThere,
        std::io::ErrorKind::NotADirectory => WhatTheFailureMeans::SomethingElseIsAtThePlace,
        _ => WhatTheFailureMeans::CouldNotBeTold,
    };
}

///  How many names a walk reached, said as a person reads it.
fn a_count_of_names(count: usize) -> String {
    return if count == 1 { String::from("1 name") } else { format!("{count} names") };
}

///  The permission to look at, for the one error that is about a permission.
fn or_the_permission(error: &std::io::Error) -> &'static str {
    return match error.kind() {
        std::io::ErrorKind::PermissionDenied => "; the permissions of that directory are what to look at",
        _ => "",
    };
}

/// What:
///  The sentence for a walk over `claimed` that stopped, and what the stop means.
/// Why:
///   The claim's own directory is the one place the settlement was looking for,
/// so its failure says plainly what is known about the claim there:
/// a directory that is not there took the record with it,
/// and one that answered with an error leaves the claim's presence unknown.
///  A walk that stopped after some names says how far it got,
/// because the claim may be among the names it never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theClaimDirectoryWalkFailed(workDir, id, error, namesReached) // [meaning, words]
/// ```
pub(crate) fn the_claim_directory_walk_failed(work_dir: &Path, id: &str, error: &std::io::Error, names_reached: usize) -> (WhatTheFailureMeans, String) {
    let claimed: PathBuf = state_directory(work_dir, CLAIMED);
    let meaning: WhatTheFailureMeans = what_the_failure_means(error);
    let said: String = match meaning {
        WhatTheFailureMeans::ThePlaceIsNotThere => format!(
            "the {CLAIMED} directory is not there under {}, so no state of {id} can be read: the record went with it and its work is recorded nowhere",
            work_dir.display()
        ),
        WhatTheFailureMeans::SomethingElseIsAtThePlace => format!(
            "a file is at {}, where the {CLAIMED} directory was, so no state of {id} can be read: the record went with it and its work is recorded nowhere",
            claimed.display()
        ),
        WhatTheFailureMeans::CouldNotBeTold if names_reached > 0 => format!(
            "the walk over {} stopped after {}, so whether the claim of {id} is among the rest, and where its record stands, could not be found out; nothing of the record was moved here{}",
            claimed.display(),
            a_count_of_names(names_reached),
            or_the_permission(error)
        ),
        WhatTheFailureMeans::CouldNotBeTold => format!(
            "the directory {} could not be read ({error}), so whether the claim of {id} is still in claimed, and where its record stands, could not be found out; nothing of the record was moved here{}",
            claimed.display(),
            or_the_permission(error)
        ),
    };
    return (meaning, said);
}

/// The sentence for a state of one record that could not be looked at,
/// which is the same answer as a directory that could not be read:
/// whether the record is there is not known.
fn a_state_that_could_not_be_looked_at(place: &Path, id: &str, error: &std::io::Error) -> String {
    return format!(
        "the state {} could not be looked at ({error}), so whether a record of {id} is there could not be found out and nothing of the record was moved here{}",
        place.display(),
        or_the_permission(error)
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
fn where_the_record_is(work_dir: &Path, id: &str, token: &str, walk: AWalkOf<'_>) -> Result<String, String> {
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
    // The claim's own directory: the file the move was for, or another instance's claim of the same record.
    // A directory that is not there is the walk's answer and not a permission problem, so the walk alone decides.
    let claims: WhatTheWalkReached = walk(work_dir, CLAIMED);
    let names_reached: usize = claims.names.len();
    let mut another: Option<String> = None;
    for (name, _modified) in claims.names {
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
        return Ok(another_claim_holds_it(id, name.as_str()));
    }
    if let Some(error) = claims.stopped_by.as_ref() {
        // The claim is not among the names the walk reached and the walk did not reach the end,
        // so the claim is either gone with the directory or in the names that were never read, and the words say which.
        let (_meaning, said): (WhatTheFailureMeans, String) = the_claim_directory_walk_failed(work_dir, id, error, names_reached);
        return Err(said);
    }
    // The other place a move of this record passes through: a take's staging and a recovery's own file both sit here.
    let recovering: PathBuf = state_directory(work_dir, RECOVERING);
    let taking: WhatTheWalkReached = walk(work_dir, RECOVERING);
    for (name, _modified) in taking.names {
        if id_of_entry(name.as_str()) != id {
            continue;
        }
        // A file of this record under a generation of its own is between two steps of a move that has three endings,
        // and this holder cannot tell which one the record meets.
        return Ok(format!(
            "another instance has this record's file in {RECOVERING}/{name}, under a generation of its own, so it is between two steps of a move of its own and this instance's claim of the record is not the file in that directory: a take publishes it as a claim in {CLAIMED}, where that instance runs the work there; a recovery queues it in {PENDING}/{id} when the record still has tries left, where the next look that claims it does its work; and a recovery leaves it in {FAILED}/{id} when its tries are all used, where nothing looks at it again and its work happens only if a person does it. Where the record stands after that move is which of the three happened, and nothing of it was moved here"
        ));
    }
    if let Some(error) = taking.stopped_by.as_ref() {
        // The directory is one a queue need not have, so only a name something else took or an error is worth a sentence.
        if what_the_failure_means(error) != WhatTheFailureMeans::ThePlaceIsNotThere {
            return Err(a_directory_that_could_not_be_read(recovering.as_path(), id, error));
        }
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
    // A take publishes its staged claim in `claimed`, and that can finish between two of the looks above.
    // The claim's own directory is therefore asked once more, so the words do not call a move under way a removal.
    let again: WhatTheWalkReached = walk(work_dir, CLAIMED);
    let names_reached_again: usize = again.names.len();
    for (name, _modified) in again.names {
        if id_of_entry(name.as_str()) != id {
            continue;
        }
        if names_claim(name.as_str(), id, token) {
            return Ok(format!(
                "the claim is still in {CLAIMED}/{name}, so nothing of the record moved here and another look runs its work again when its lease runs out"
            ));
        }
        return Ok(another_claim_holds_it(id, name.as_str()));
    }
    if let Some(error) = again.stopped_by.as_ref() {
        let (_meaning, said): (WhatTheFailureMeans, String) = the_claim_directory_walk_failed(work_dir, id, error, names_reached_again);
        return Err(said);
    }
    return Ok(format!(
        "no file of {id} was in {CLAIMED}, {RECOVERING}, {PENDING}, {DONE} or {FAILED} under {}, and every one of those places was looked at, so either a move of this queue finished between those looks or something other than a move of this queue removed the record; nothing of it was moved here",
        work_dir.display()
    ));
}

///  The sentence for a claim of this record that is not this instance's: another instance took it back when its lease ran out
///  and claimed it again, and that claim's work is its own to run.
fn another_claim_holds_it(id: &str, name: &str) -> String {
    return format!(
        "another instance took the claim of {id} back when its lease ran out and claimed it again as {CLAIMED}/{name}, so its work is run again there and nothing of it was moved here"
    );
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "work_record_where_it_went_tests.rs"]
mod tests;
