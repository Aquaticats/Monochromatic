//! What:
//!  The entries of a stopped instance's journal,
//! and what each name in it says about keeping the journal:
//! the record of a batch that landed,
//! the record of a commit that has not landed,
//! a claim watch note,
//! and a name this release does not explain.
//! Why:
//!  A journal goes only when every entry in it is the record of a batch that landed
//! and no client of that batch can still read the record (`service_files_of_a_stopped_instance.rs`),
//! because work a person may still want is never discarded.
//! This part says what one name means,
//! so the removal that lists the journal stays one decision per listing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! keptEntryOf(journal, now) // { name, reason, forAReader } or undefined
//! ```

use crate::claim_time::ClaimTime;
use crate::push_retry::PLAN_DIRECTORY;
use crate::service_claim_watch::{
    WATCH_DIRECTORY,
    WatchedRepositories,
    watched_repositories,
};
use crate::service_journal::{
    CLAIM_EXTENSION,
    CONCLUSION_COPY_EXTENSION,
    DONE_EXTENSION,
    INDEXED_EXTENSION,
    OPEN_EXTENSION,
    SCRATCH_EXTENSION,
    START_EXTENSION,
    TEMPORARY_EXTENSION,
};
use crate::service_journal_record_age::{
    a_client_may_still_read,
    a_client_may_still_read_finished_batch,
    modified_time_of,
};
use crate::service_rules_precompile::is_the_remembered_repository;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  A journal that stayed after its instance's units were stopped,
///  the entry that kept it,
///  and what that entry is.
/// Why:
///   The caller decides where the line goes:
/// the command a person runs reports it on standard error,
/// and the service logs it to its own journal,
/// so a notice a person needs never depends on a warning being enabled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeptJournal {
    /// The journal folder that stays.
    pub journal: PathBuf,
    /// The entry that keeps it.
    pub entry: String,
    /// What that entry is,
    /// in words.
    pub reason: String,
}

/// What:
///  What became of the journal of an instance whose units were stopped.
/// Why:
///   Two of the three answers ask different things of the caller:
/// a journal that holds unresolved work is work a person may still want and is named to them with the file that holds it,
/// while a journal that stays only because a client may still read it asks nothing of them,
/// and the caller keeps the note and the socket file so a later sweep finishes the removal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JournalAfterStop {
    /// Nothing of the journal is left.
    Removed,
    /// The journal holds something a person may still want,
    /// and the caller names it to them.
    HoldsWork(KeptJournal),
    /// The journal stays because a client of its instance may still read it,
    /// or because the service that writes it may still run,
    /// and nothing in it is the person's business yet.
    StillRead(KeptJournal),
}

/// The directory of the record a landing keeps inside the journal (`service_busy.rs`),
/// which is empty once no landing is in flight.
pub(crate) const LANDING_RECORD_DIRECTORY: &str = "busy";

/// What a kept journal keeps the journal for.
pub(crate) enum Kept {
    /// Something a person may still want,
    /// named with the words that say what it is.
    Work(String, String),
    /// A finished batch a client of the instance may still read.
    Read(String, &'static str),
}

/// Why a finished batch's record keeps the journal:
/// the client of its request may still look for the commit it became.
pub(crate) const A_LOST_ANSWER_MAY_READ_IT: &str =
    "a batch that landed too recently for every client of it to be gone";

/// Whether every byte of the text is a decimal digit.
fn are_digits(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    for byte in text.bytes() {
        if !byte.is_ascii_digit() {
            return false;
        }
    }
    return true;
}

/// Whether a file name is the record of a batch that landed,
/// which a stopped instance's journal may lose once no client of it can still look.
fn is_finished_memory(name: &str) -> bool {
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return false;
    };
    if !are_digits(stem) {
        return false;
    }
    return extension == DONE_EXTENSION;
}

/// The extension of a file name,
/// and nothing when it has none.
fn extension_of(name: &str) -> &str {
    return match name.rsplit_once('.') {
        Some((_stem, extension)) => extension,
        None => "",
    };
}

/// Why a file of a journal that is not a finished batch keeps the journal.
fn reason_of_a_kept_file(name: &str) -> &'static str {
    let extension: &str = extension_of(name);
    if extension == OPEN_EXTENSION {
        return "the record of a commit that has not landed";
    }
    if extension == SCRATCH_EXTENSION {
        return "the record of a batch whose objects a scratch reference may hold";
    }
    if extension == CLAIM_EXTENSION
        || extension == INDEXED_EXTENSION
        || extension == START_EXTENSION
    {
        return "the record of a batch that was being landed";
    }
    // A `.tmp` file can hold a record that was written whole and never published, so it is no remains.
    if extension == TEMPORARY_EXTENSION {
        return "a record that was written and not published";
    }
    return "a file this release does not know";
}

/// What:
///  Whether a folder inside a journal is one batch's copy of the conclusion state that batch removed.
/// Why:
///   The copy is named `<batch>.cleanup` (`conclusion_state_copy.rs`),
/// and it is a copy the service made:
/// the
///  worktree's own state files are what the cleanup removes,
/// and this journal belongs to a worktree that is gone.
///  A folder whose name only ends that way is not one:
/// the batch number before it is digits and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copy = /^[0-9]+\.cleanup$/.test(name);
/// ```
fn is_a_conclusion_copy(name: &str) -> bool {
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return false;
    };
    return extension == CONCLUSION_COPY_EXTENSION
        && !stem.is_empty()
        && stem.bytes().all(|byte| return byte.is_ascii_digit());
}

/// Why a folder inside a journal keeps it.
fn reason_of_a_kept_folder(name: &str) -> &'static str {
    if name == LANDING_RECORD_DIRECTORY {
        return "a record of a landing or of a withdrawal";
    }
    // The pushes of the instance: a service of this worktree reads them when it starts again,
    // so a worktree that comes back at the same path finds its pending push.
    if name == PLAN_DIRECTORY {
        return "the pushes this service was still to try again, which a service of this worktree reads when it starts again";
    }
    return "a folder this release does not know";
}

/// The folder's entries,
/// and the words that say why they could not be read.
fn entries_of(folder: &Path) -> Result<std::fs::ReadDir, String> {
    return match std::fs::read_dir(folder) {
        Ok(listing) => Ok(listing),
        Err(error) => Err(format!(
            "{} could not be listed ({error})",
            folder.display()
        )),
    };
}

/// Whether a folder holds no entry at all.
fn holds_nothing(folder: &Path) -> Result<bool, String> {
    let mut listing = entries_of(folder)?;
    return Ok(listing.next().is_none());
}

/// What:
///  The first entry of a stopped instance's journal that keeps the journal,
/// with the words that say why,
/// and nothing when every entry may go.
/// Why:
///   Only the record of a batch that landed,
/// and a folder the service made and left empty (`busy`,
/// the landing's record),
/// are provably of no use once the instance is stopped;
/// so is the repository the service itself remembered for its rules warm,
/// which names a repository and no record of a commit,
/// and so is a claim watch note that names a repository whose work directory is gone,
/// which is the directory a look would list.
/// Every other name either holds a commit a person may still want
/// or is a name this release does not explain,
/// and guessing about one of those would discard work.
/// A finish record is no use either,
/// but only once every client of it is gone:
/// a client whose answer was lost reads it to learn which commit its request became,
/// so a journal written within `A_FINISHED_RECORD_IS_READ` is kept for the clients that can still read it,
/// and one that holds work a person may want is kept for them however old it is.
/// The answer comes from one listing,
/// so a journal that grows while it is read is kept,
/// not half-removed.
/// The age of a finished record is read from the clocks of the record itself,
/// so the answer is the same ones the clock a client waits on gives
/// (`service_journal_record_age.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keptEntryOf(journal, now) // { name, reason, forAReader } or undefined
/// ```
pub(crate) fn kept_entry_of(
    journal: &Path,
    now: &ClaimTime,
) -> Result<Option<Kept>, String> {
    let listing = match entries_of(journal) {
        Ok(listing) => listing,
        Err(reason) => {
            // A journal that is not there has nothing to keep.
            if !journal.exists() {
                return Ok(None);
            }
            return Err(reason);
        }
    };
    // The folder's own time says when a record was last published into it, which is when a batch finished:
    // a batch that began long ago and landed now leaves a record whose own time is old.
    let folder_name: String = match journal.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => journal.display().to_string(),
    };
    let mut readable: Option<Kept> =
        match a_client_may_still_read(None, modified_time_of(journal), now) {
            true => Some(Kept::Read(folder_name, A_LOST_ANSWER_MAY_READ_IT)),
            false => None,
        };
    for listed in listing {
        let entry = match listed {
            Ok(entry) => entry,
            Err(error) => {
                return Err(format!(
                    "{} could not be listed ({error})",
                    journal.display()
                ));
            }
        };
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let path: PathBuf = entry.path();
        let kind = match entry.file_type() {
            Ok(kind) => kind,
            Err(error) => return Err(format!("{} could not be read ({error})", path.display())),
        };
        if kind.is_dir() {
            // The copy of a batch's conclusion state is the service's own, made before the branch moved, and it is a
            // copy of what the landing compares and of the worktree's scratch state, never the state itself: the
            // worktree this journal belongs to is gone, so nothing reads it again.
            if is_a_conclusion_copy(name.as_str()) {
                continue;
            }
            if holds_nothing(path.as_path())? {
                continue;
            }
            // A claim watch note is a pointer: a look at it lists the work directory of the repository it names, so a
            //  note that names this worktree, or any repository whose work directory is gone, leads a service to
            //  nothing. A note that names a repository which still holds its work directory keeps the journal and is
            //  named, because a service of this instance reads it when the worktree is back at that path.
            if name == WATCH_DIRECTORY {
                match watched_repositories(path.as_path()) {
                    WatchedRepositories::AllGone => continue,
                    WatchedRepositories::StillThere {
                        git_dir,
                        note,
                    } => {
                        return Ok(Some(Kept::Work(
                            format!("{name}/{note}"),
                            format!(
                                "a note of the repository {}, which still holds the work directory a look reads, and which a service of this instance reads when its worktree is back at that path",
                                git_dir.display()
                            ),
                        )));
                    }
                    WatchedRepositories::Unexplained {
                        entry: unreadable,
                        reason,
                    } => return Ok(Some(Kept::Work(format!("{name}/{unreadable}"), reason))),
                }
            }
            return Ok(Some(Kept::Work(
                name.clone(),
                String::from(reason_of_a_kept_folder(name.as_str())),
            )));
        }
        if is_finished_memory(name.as_str()) {
            if a_client_may_still_read_finished_batch(path.as_path(), now) {
                readable = Some(Kept::Read(name, A_LOST_ANSWER_MAY_READ_IT));
            }
            continue;
        }
        // The repository the service remembered for its rules warm names a repository and carries no record of a
        //  commit, so nothing in it is work a person may still want, and it goes with the finished batches.
        if is_the_remembered_repository(name.as_str()) {
            continue;
        }
        return Ok(Some(Kept::Work(
            name.clone(),
            String::from(reason_of_a_kept_file(name.as_str())),
        )));
    }
    return Ok(readable);
}
