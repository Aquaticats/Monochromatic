//! What:
//!  The repair's own reads as faces a task of the runtime may await:
//!  the records a batch still owes,
//!  the witness of the live index,
//!  the worktree path and the object it holds,
//!  the deletions a landing could own,
//!  and every Git step that decides what a commit leaves out.
//! Why:
//!   A task of the runtime may neither wait on a file system nor enter the runtime to wait for a
//! child,
//! and the repair reads both before the branch moves.
//! The questions,
//! the answers and the words of every stop are the synchronous faces',
//! which `owed_index_repair.rs` still holds,
//! so a path is left out on the same evidence whichever face asked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await excludeWhatALandedCommitLeft(git, gitDir, liveIndex, commitIndex);
//! ```

use super::requests::{
    IndexCopy, ListedNames, WorktreeFile, WorktreePath, copy_of, deletions_request, hash_request, index_info_listing, index_info_request,
    listed_deletions, moved_of, moved_request, object_answered, present_in_the_worktree, prune_request, removed, took_it, tree_of, tree_request, worktree_file,
};
use super::unreadable_notice;
use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than, run_blocking_job_within_the_budget};
use crate::owed_index_entries::{Entry, head_entries_awaiting, index_entries_awaiting, is_unmerged, moved_since_listing_awaiting, names_in, tree_entries_awaiting};
use crate::owed_index_update::Owed;
use crate::service_protocol::FailureCode;
use crate::transaction_git::GitContext;
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::COMMIT_OWN_STATE_CEILING;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// `names_of` with its Git child awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function namesOfAwaiting(git, repository, owed): Promise<path[]>;
/// ```
///
/// # Errors
///
/// As `names_of`.
async fn names_of_awaiting(git: &GitContext, repository: &Path, owed: &Owed) -> Result<Vec<Vec<u8>>, FailureCode> {
    if !owed.derived {
        return Ok(owed.paths.clone());
    }
    let names: Vec<u8> = crate::service_land_follow::derived_names_awaiting(git, repository, owed.old.as_str(), owed.new.as_str()).await?;
    return Ok(names_in(names.as_slice()));
}
/// `moved_since` with both of its Git children awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function movedSinceAwaiting(git, repository, liveIndex, commitIndex): Promise<path[]>;
/// ```
///
/// # Errors
///
/// As `moved_since`.
async fn moved_since_awaiting(git: &GitContext, repository: &Path, live_index: &Path, commit_index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let held: String = tree_of(run_git_awaiting(git, &tree_request(repository, live_index)).await)?;
    return moved_of(run_git_awaiting(git, &moved_request(repository, commit_index, held.as_str())).await);
}
/// What:
///  `set_to_head_with` with its storage steps jobs of the blocking pool and every Git child awaited on the
///  runtime.
/// Why:
///   A task of the runtime may neither wait on a filesystem nor enter the runtime to wait for a child,
/// and the
///  copy,
/// the put back and the removal above wait in the kernel on the disk the repository is on.
/// The copies keep
///  no ceiling:
/// a copy that was abandoned half-written is a file nothing waits for,
/// so the caller waits for it to end,
///  as the job of the pool holds its place.
/// The order of the steps,
/// the words they log and every refusal are the synchronous face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setToHeadWithAwaiting(git, repository, index, paths): Promise<void>;
/// ```
///
/// # Errors
///
/// As `set_to_head_with`.
async fn set_to_head_with_awaiting(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    if paths.is_empty() {
        return Ok(());
    }
    let kept: PathBuf = index.with_extension("before-head");
    let copied: IndexCopy = IndexCopy { from: index.to_path_buf(), to: kept.clone() };
    if let Err(error) = run_blocking_job_within_the_budget(move || { return copy_of(copied); }).await {
        crate::diagnostic_log::warn("servicePrepare", format!("the commit's own index cannot be copied aside before it is set to the head ({error})").as_str());
        return Err(FailureCode::Unavailable);
    }
    let restored: Result<(), FailureCode> = set_to_head_unchecked_awaiting(git, repository, index, paths).await;
    if restored.is_err() {
        let back: IndexCopy = IndexCopy { from: kept.clone(), to: index.to_path_buf() };
        if let Err(error) = run_blocking_job_within_the_budget(move || { return copy_of(back); }).await {
            crate::diagnostic_log::warn("servicePrepare", format!("the commit's own index could not be put back after a refused step ({error})").as_str());
        }
    }
    let going: PathBuf = kept.clone();
    let _gone: Result<std::io::Result<()>, crate::blocking_job::BlockingJobError> = run_blocking_job_within_the_budget(move || { return removed(going); }).await;
    return restored;
}
/// `set_to_head_unchecked` with its Git children awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setToHeadUncheckedAwaiting(git, repository, index, paths): Promise<void>;
/// ```
///
/// # Errors
///
/// As `set_to_head_unchecked`.
async fn set_to_head_unchecked_awaiting(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    if !took_it(run_git_awaiting(git, &prune_request(repository, index, paths)).await) {
        return Err(FailureCode::Unavailable);
    }
    if crate::service_land_follow::reset_entries_awaiting(git, repository, Some(index), "HEAD", paths).await.is_ok() {
        return Ok(());
    }
    // A `reset` this wrapper refuses is not the only way to the head: the entries are written where the head's tree has them.
    let head: BTreeMap<Vec<u8>, Entry> = head_entries_awaiting(git, repository, paths).await?;
    let listing: Vec<u8> = index_info_listing(&head, paths);
    return if took_it(run_git_awaiting(git, &index_info_request(repository, index, listing)).await) { Ok(()) } else { Err(FailureCode::Unavailable) };
}
/// `set_to_head_paths` with its storage steps jobs of the blocking pool and its Git awaited on the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setToHeadPathsAwaiting(git, repository, index, paths): Promise<void>;
/// ```
///
/// # Errors
///
/// As `set_to_head_paths`.
async fn set_to_head_paths_awaiting(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    return set_to_head_with_awaiting(git, repository, index, paths).await;
}
/// `exclude_what_a_landed_commit_left` with every Git child awaited on the runtime and every storage step a job of
///  the blocking pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await excludeWhatALandedCommitLeftAwaiting(...) // the paths this commit leaves out
/// ```
///
/// # Errors
///
/// As `exclude_what_a_landed_commit_left`.
pub async fn exclude_what_a_landed_commit_left_awaiting(
    git: &GitContext,
    repository: &Path,
    git_dir: &Path,
    live_index: &Path,
    commit_index: Option<&Path>,
    captured_index: Option<&Path>,
    captured: &crate::service_land_start::IndexWitness,
) -> Result<Vec<String>, String> {
    let mut note: Option<String> = None;
    let left: Vec<String> = exclude_with_note_awaiting(git, repository, git_dir, live_index, commit_index, captured_index, captured, &mut note).await?;
    return Ok(left);
}
/// The Git directory one question about the owed records names,
/// owned so that a job of the blocking pool can take it.
struct RecordsQuestion {
    /// The worktree's own Git directory,
    /// which holds the records.
    git_dir: PathBuf,
}
/// The look one job of the pool does:
/// `records_and_unreadable` over the question's own directory.
fn records_here(question: RecordsQuestion) -> (Vec<Owed>, usize) {
    return crate::owed_index_update::records_and_unreadable(question.git_dir.as_path());
}
/// What:
///  The owed records of one worktree and how many cannot be read,
/// as a job of the blocking pool awaited no longer than
///  `ceiling`,
///  and nothing when the look gave no answer.
/// Why:
///   The look lists a directory and reads one file per record,
/// which waits in the kernel on a disk or a mount that stopped
///  answering,
/// and a task of the runtime may not wait there.
/// A look that gave nothing in time is told apart from "nothing is
///  owed",
/// because "nothing is owed" lets the commit record the branch as it stands,
/// and a look nobody answered must not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recordsWithin(ceilingMs, gitDir): Promise<[Owed[], number] | undefined>;
/// ```
async fn records_within(ceiling: Duration, git_dir: &Path) -> Option<(Vec<Owed>, usize)> {
    let question: RecordsQuestion = RecordsQuestion { git_dir: git_dir.to_path_buf() };
    return match run_a_job_awaited_no_longer_than(ceiling, question, records_here).await {
        TheAnswerOfAJobAwaited::Ran(found) => Some(found),
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}
/// The index file one witness question names,
/// owned so that a job of the blocking pool can take it.
struct WitnessQuestion {
    /// The live index the copy is compared with.
    path: PathBuf,
}
/// The look one job of the pool does:
/// `witness_of` over the question's own file.
fn witness_here(question: WitnessQuestion) -> std::io::Result<crate::service_land_start::IndexWitness> {
    return crate::service_land_start::witness_of(question.path.as_path());
}
/// What:
///  The witness of the live index,
/// as a job of the blocking pool awaited no longer than `ceiling`,
/// and nothing when the look gave no
///  answer.
/// Why:
///   The witness opens the index and reads its end,
/// which waits in the kernel on a disk or a mount that stopped answering,
///  and a task of the runtime may not wait there.
/// Nothing is the answer a witness the synchronous face cannot read stands for,
/// so
///  the comparison is not made,
/// which changes nothing in the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function witnessWithin(ceilingMs, index): Promise<IndexWitness | Error | undefined>;
/// ```
async fn witness_within(ceiling: Duration, index: &Path) -> Option<std::io::Result<crate::service_land_start::IndexWitness>> {
    let question: WitnessQuestion = WitnessQuestion { path: index.to_path_buf() };
    return match run_a_job_awaited_no_longer_than(ceiling, question, witness_here).await {
        TheAnswerOfAJobAwaited::Ran(asked) => Some(asked),
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}
/// What:
///  `exclude_with_note` with every Git child awaited on the runtime and every storage step a job of the
///  blocking pool.
/// Why:
///   A task of the runtime may neither enter the runtime to wait for a child nor wait on a filesystem,
/// and the
///  repair reads the records of what the index owes,
/// the witness of the live index,
/// the worktree's paths and
///  the commit's own index.
/// The order of the steps,
/// the paths left out and every sentence a step says are the
///  synchronous face's,
/// and the three answers a look can give are kept apart:
/// a set of records that cannot be read is
///  the answer it already has,
/// a witness that gives no answer is compared with nothing,
/// and a look that gave no answer in
///  time stops the commit rather than letting it record what it never read.
/// The wait each storage step is granted is `COMMIT_OWN_STATE_CEILING`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await excludeWithNoteAwaiting(...) // [paths, note]
/// ```
///
/// # Errors
///
/// As `exclude_with_note`,
/// and the reason a record that cannot be looked at gives.
pub async fn exclude_with_note_awaiting(
    git: &GitContext,
    repository: &Path,
    git_dir: &Path,
    live_index: &Path,
    commit_index: Option<&Path>,
    captured_index: Option<&Path>,
    captured: &crate::service_land_start::IndexWitness,
    note: &mut Option<String>,
) -> Result<Vec<String>, String> {
    return exclude_with_note_within(COMMIT_OWN_STATE_CEILING, git, repository, git_dir, live_index, commit_index, captured_index, captured, note).await;
}
/// What:
///  `exclude_with_note_awaiting` with the wait `ceiling` grants each of its storage steps.
/// Why:
///   A control needs a wait small enough to see the answer a step that gives nothing in time gives,
/// and the
///  value the shipped face grants is `COMMIT_OWN_STATE_CEILING`;
/// every step,
/// every path left out and every sentence is the awaited
///  face's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await excludeWithNoteWithin(100, ...)
/// ```
///
/// # Errors
///
/// As `exclude_with_note_awaiting`.
pub(crate) async fn exclude_with_note_within(
    ceiling: Duration,
    git: &GitContext,
    repository: &Path,
    git_dir: &Path,
    live_index: &Path,
    commit_index: Option<&Path>,
    captured_index: Option<&Path>,
    captured: &crate::service_land_start::IndexWitness,
    note: &mut Option<String>,
) -> Result<Vec<String>, String> {
    let mut left: Vec<String> = Vec::new();
    let Some((owed_records, unreadable)): Option<(Vec<Owed>, usize)> = records_within(ceiling, git_dir).await else {
        return Err(String::from(
            "the records of what this worktree's index owes could not be looked at in time, so this commit was not made and nothing of the index was changed",
        ));
    };
    // An obligation that cannot be read names no path, so nothing can be repaired for it: what is known is that a landed file could
    //  be recorded as deleted. The commit goes through with such a deletion left out, and says why.
    if unreadable > 0 {
        let at_risk: Vec<Vec<u8>> = match commit_index {
            None => Vec::new(),
            Some(index) => deletions_a_landing_could_own_within(ceiling, git, repository, index).await?,
        };
        if !at_risk.is_empty() {
            let index = commit_index.expect("the paths were asked of the commit's own index");
            if let Err(code) = set_to_head_paths_awaiting(git, repository, index, at_risk.as_slice()).await {
                return Err(format!("the entries an obligation that cannot be read may owe cannot be set to the head ({code:?})"));
            }
        }
        for path in &at_risk {
            left.push(crate::path_bytes::path_message_text(path.as_slice()));
        }
        *note = Some(unreadable_notice(unreadable, left.as_slice()));
    }
    // Nothing at all is done unless this worktree has an owed record: a commit that owes nothing is never compared, so no staging of
    //  its own can be mistaken for a move.
    if owed_records.is_empty() {
        return Ok(left);
    }
    // What the commit copied is not what the index holds any more: the paths whose live entry moved since the copy are left out,
    //  because what the snapshot would record there is not the person's current intent. The copy, not the commit's staged index, is
    //  what the live index is compared with: the commit's own staging (`--all`) is the person's and must never be reported as moved.
    let witness: Option<crate::service_land_start::IndexWitness> = match witness_within(ceiling, live_index).await {
        Some(Ok(found)) => Some(found),
        // A witness that cannot be read, or that gave no answer in time, is compared with nothing, which is what the synchronous
        //  face does with the error of its own look.
        Some(Err(_)) | None => None,
    };
    if let (Some(index), Some(copy), Some(now)) = (commit_index, captured_index, witness)
        && now != *captured
    {
        // Only a path a record names may be changed: a move another command made to a path no record names is that command's own
        //  staging, and the commit keeps what it selected for it.
        let mut named: Vec<Vec<u8>> = Vec::new();
        for owed in &owed_records {
            // A record of another index file (`GIT_INDEX_FILE`) names no path of this commit's index.
            if owed.index != live_index {
                continue;
            }
            if let Ok(names) = names_of_awaiting(git, repository, owed).await {
                named.extend(names);
            }
        }
        let moved: Vec<Vec<u8>> = match moved_since_awaiting(git, repository, live_index, copy).await {
            Ok(moved) => moved,
            Err(_unwritable) => match moved_since_listing_awaiting(git, repository, live_index, copy).await {
                Ok(moved) => moved,
                Err(code) => return Err(format!("the entries the index moved since this commit copied it cannot be read ({code:?})")),
            },
        };
        let mut owned: Vec<Vec<u8>> = Vec::with_capacity(moved.len());
        for path in moved {
            if named.contains(&path) {
                owned.push(path);
            }
        }
        if !owned.is_empty() {
            if let Err(code) = set_to_head_paths_awaiting(git, repository, index, owned.as_slice()).await {
                return Err(format!("the entries the index moved since this commit copied it cannot be set to the head ({code:?})"));
            }
            for path in &owned {
                left.push(crate::path_bytes::path_message_text(path.as_slice()));
            }
        }
    }
    for owed in owed_records {
        // The record names the index file whose update it owes; a commit of another one is another question.
        if owed.index != live_index {
            continue;
        }
        let snapshot: &Path = commit_index.unwrap_or(live_index);
        let names: Vec<Vec<u8>> = match names_of_awaiting(git, repository, &owed).await {
            Ok(names) => names,
            Err(code) => return Err(format!("the paths of the index update batch {} owes cannot be read ({code:?})", owed.seq)),
        };
        let replaced: BTreeMap<Vec<u8>, Entry> = match tree_entries_awaiting(git, repository, owed.old.as_str(), names.as_slice()).await {
            Ok(entries) => entries,
            Err(code) => return Err(format!("the tree the landing of batch {} replaced cannot be read ({code:?})", owed.seq)),
        };
        let held: BTreeMap<Vec<u8>, Entry> = match index_entries_awaiting(git, repository, snapshot, names.as_slice()).await {
            Ok(entries) => entries,
            Err(code) => return Err(format!("the commit's own index cannot be read for the paths of batch {} ({code:?})", owed.seq)),
        };
        // What the head holds there: a path whose copy already holds it is not left out, and is not said, because the commit
        //  records nothing about it either way.
        let head: BTreeMap<Vec<u8>, Entry> = match head_entries_awaiting(git, repository, names.as_slice()).await {
            Ok(entries) => entries,
            Err(code) => return Err(format!("the worktree's head cannot be read for the paths of batch {} ({code:?})", owed.seq)),
        };
        // A path the person selected is never changed by a guess.
        if commit_index.is_none() {
            for name in &names {
                if held.get(name) == replaced.get(name) && !is_unmerged(held.get(name)) {
                    left.push(crate::path_bytes::path_message_text(name.as_slice()));
                }
            }
            continue;
        }
        // The worktree is asked whether the staging is the person's own: a worktree that holds what the index holds means they
        //  staged what they have, so it is their statement and never the landing's leftover.
        let mut excluded: Vec<Vec<u8>> = Vec::new();
        for name in &names {
            let entry: Option<&Entry> = held.get(name);
            if is_unmerged(entry) {
                // A merge in progress is the person's own act.
                continue;
            }
            if entry != replaced.get(name) {
                continue;
            }
            let theirs: bool = match (entry, worktree_object_within(ceiling, git, repository, name.as_slice()).await) {
                (Some((mode, object)), Ok(Some((worktree_mode, mine)))) => *object == mine && *mode == worktree_mode,
                // Nothing is proof of the person's staging here: a file that is not there with no entry in the copy is what a
                //  landing whose index update failed leaves, so the landed entry stays and the path is said. A deletion the person
                //  staged in that window waits for the next commit.
                _ => false,
            };
            if !theirs && head.get(name) != entry {
                excluded.push(name.clone());
            }
        }
        if excluded.is_empty() {
            continue;
        }
        if let Err(code) = set_to_head_with_awaiting(git, repository, snapshot, excluded.as_slice()).await {
            return Err(format!("the commit's own index cannot be set to the head for the entries batch {} owes ({code:?})", owed.seq));
        }
        for path in &excluded {
            left.push(crate::path_bytes::path_message_text(path.as_slice()));
        }
    }
    return Ok(left);
}
/// What:
///  `worktree_file` as a job of the blocking pool,
/// awaited no longer than `ceiling`,
/// and nothing when the look gave no answer.
/// Why:
///   A task of the runtime may not wait on a filesystem:
/// the look and the link read wait in the kernel on a mount that stopped
///  answering,
/// and no timer on that worker can end it.
/// A look that gave nothing in time is no evidence,
/// which is the answer a
///  path that is there and cannot be read gives already,
/// so the entry the index holds is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function worktreeFileWithin(ceilingMs, path): WorktreeFile | undefined;
/// ```
async fn worktree_file_within(ceiling: Duration, path: &Path) -> Option<WorktreeFile> {
    let asked: WorktreePath = WorktreePath { path: path.to_path_buf() };
    return match run_a_job_awaited_no_longer_than(ceiling, asked, worktree_file).await {
        TheAnswerOfAJobAwaited::Ran(Ok(found)) => Some(found),
        TheAnswerOfAJobAwaited::Ran(Err(_unreadable)) => None,
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}
/// What:
///  `worktree_object` with its look and its link read jobs of the blocking pool awaited no longer than `ceiling`,
///  and its Git awaited on the runtime.
/// Why:
///   A task of the runtime may neither wait on a filesystem nor enter the runtime to wait for a child,
/// and the look
///  and the link read of one path wait in the kernel on a mount that stopped answering.
/// The ceiling is the face's,
///  so a control can watch a look that gives nothing in time;
/// the shipped caller grants `COMMIT_OWN_STATE_CEILING`.
/// A look that gave nothing in
///  time is no evidence of the person's staging,
/// which is the answer the synchronous face gives for a file it cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function worktreeObjectWithin(ceilingMs, git, repository, name): Promise<[mode, object] | undefined>;
/// ```
///
/// # Errors
///
/// As `worktree_object`.
async fn worktree_object_within(ceiling: Duration, git: &GitContext, repository: &Path, name: &[u8]) -> Result<Option<(String, String)>, FailureCode> {
    let path: OsString = <OsString as std::os::unix::ffi::OsStringExt>::from_vec(name.to_vec());
    let held: PathBuf = repository.join(path.as_os_str());
    return match worktree_file_within(ceiling, held.as_path()).await {
        Some(WorktreeFile::Absent) => Ok(None),
        Some(WorktreeFile::Present(present)) => Ok(Some(object_answered(run_git_awaiting(git, &hash_request(repository, path.as_os_str(), present.link_target.as_deref())).await, present.mode.as_str())?)),
        // No answer is no evidence of the person's staging, which is what the synchronous face answers for a file it cannot read.
        None => Err(FailureCode::Unavailable),
    };
}
/// What:
///  `deletions_a_landing_could_own` with its Git awaited on the runtime and its look at every listed name a job of the
///  blocking pool awaited no longer than `ceiling`.
/// Why:
///   A task of the runtime may neither enter the runtime to wait for a child nor wait on a filesystem,
/// and the look at
///  every listed name waits in the kernel on a mount that stopped answering.
/// A look that gives nothing in time is the failure this step already
///  answers for a listing it cannot read,
/// so nothing claims a deletion was checked that never was.
/// The ceiling is the face's,
/// so a control can watch a look that gives nothing in time;
/// the shipped caller grants `COMMIT_OWN_STATE_CEILING`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await deletionsALandingCouldOwnWithin(ceilingMs, git, repository, index)
/// ```
///
/// # Errors
///
/// As `deletions_a_landing_could_own`.
async fn deletions_a_landing_could_own_within(ceiling: Duration, git: &GitContext, repository: &Path, index: &Path) -> Result<Vec<Vec<u8>>, String> {
    let names: Vec<Vec<u8>> = listed_deletions(run_git_awaiting(git, &deletions_request(repository, index)).await)?;
    let listing: ListedNames = ListedNames { names, repository: repository.to_path_buf() };
    return match run_a_job_awaited_no_longer_than(ceiling, listing, present_in_the_worktree).await {
        TheAnswerOfAJobAwaited::Ran(at_risk) => Ok(at_risk),
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => Err(String::from("the deletions this commit would record cannot be read")),
    };
}

/// Controls for the awaited faces of this repair stay out of the release executable.
#[cfg(test)]
#[path = "owed_index_repair_awaiting_tests.rs"]
mod awaiting_tests;
