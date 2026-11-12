//! What:
//!  What a commit does about the index entries a landed batch still owes:
//! it records nothing about a path that still holds exactly the content the landing replaced,
//! and nothing
//!  about a path whose index entry moved after the commit copied it.
//! Why:
//!   A batch's branch moves before its index is installed,
//! so the worktree's index can describe the branch before a landing,
//! and a
//!  commit of that index would record the landed commit's own paths as their old content,
//! or as a staged deletion.
//! The question is not who owns an entry but what the commit is about to record:
//! the record names a path
//!  with the entry the landing replaced and the entry it landed,
//! and a path is left out exactly while the commit's own
//!  index still holds the entry the landing replaced.
//! That asks about content,
//! so an amend,
//! a `reset --hard`,
//! an attach from a
//!  detached head and an alternate index need no rule of their own.
//! The price is that a person's deliberate restoration of that same
//!  content is left out too,
//! with the notice that names the path and both ways on,
//! and their staging stays in the live index.
//! A path whose live entry moved after the commit copied the index is left out as well:
//! that snapshot is older
//!  than the index,
//! so what the commit would record there is not the person's current intent.
//! Nothing here writes the live index or removes a record.
//! What it reads is `owed_index_entries.rs`.
//!
//! The rule this file enforces:
//!   It may change only a path a record names,
//! and only while every fact it has about that path agrees that the index still holds the content the
//! landing replaced:
//! the commit's own index holds exactly the entry the landing's
//!  old tree held there,
//! the worktree does not match that entry (so the staging is not the person's own doing),
//! and the commit did not select the path by name.
//! On no evidence it changes nothing,
//! leaves the landed content where it belongs,
//! and says which path it left out;
//! when it cannot even make the change it decided on,
//! it stops the commit instead of letting stale content be recorded.
//! A path whose live entry moved after the commit copied the index is left out too,
//! on the same terms:
//! the copy,
//! not the staged index,
//! is what the live index is compared with.
//! Each of these steps has two faces:
//! the synchronous one a caller on a thread asks,
//! and the awaited one a task of
//!  the runtime asks (`exclude_what_a_landed_commit_left_awaiting`,
//! `exclude_with_note_awaiting`),
//! which awaits every Git child on the
//!  runtime and reads the records,
//! the worktree's paths and its own index as jobs of the blocking pool.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const left = await excludeWhatALandedCommitLeft(git, gitDir, liveIndex, commitIndex);
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than, run_blocking_job_within_the_budget};
use crate::owed_index_entries::{Entry, head_entries, head_entries_awaiting, index_entries, index_entries_awaiting, is_unmerged, is_zero_byte, moved_since_listing, moved_since_listing_awaiting, names_in, specs_of, tree_entries, tree_entries_awaiting};
use crate::owed_index_update::Owed;
use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::wait_ceilings::COMMIT_OWN_STATE_CEILING;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The paths one record's update covers,
/// derived from the two branch values when the batch names none.
fn names_of(git: &GitContext, repository: &Path, owed: &Owed) -> Result<Vec<Vec<u8>>, FailureCode> {
    if !owed.derived {
        return Ok(owed.paths.clone());
    }
    let names: Vec<u8> = crate::service_land_follow::derived_names(git, repository, owed.old.as_str(), owed.new.as_str())?;
    return Ok(names_in(names.as_slice()));
}

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

/// What:
///  The question `moved_since` asks first:
///  the tree the live index holds.
/// Why:
///  Both faces ask the one question,
/// so neither compares another tree with the copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['write-tree'], index: liveIndex };
/// ```
fn tree_request(repository: &Path, live_index: &Path) -> GitRequest {
    let mut tree: GitRequest = GitRequest::new(repository, &["write-tree"]);
    tree.index = Some(live_index.to_path_buf());
    return tree;
}

/// What:
///  The question `moved_since` asks second:
///  which paths the commit's own index holds differently from `held`.
/// Why:
///  Both faces ask the one question,
/// so a path one of them would report is reported by the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['diff-index', '--cached', '--name-only', '-z', held], index: commitIndex };
/// ```
fn moved_request(repository: &Path, commit_index: &Path, held: &str) -> GitRequest {
    let mut moved: GitRequest = GitRequest::new(repository, &["diff-index", "--cached", "--name-only", "-z", held]);
    moved.index = Some(commit_index.to_path_buf());
    return moved;
}

/// What:
///  The tree Git's answer to `tree_request` names.
/// Why:
///  Both faces read the name through this one function,
/// so a tree Git wrote and a step it refused cannot be told apart
///  differently by the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeOf(asked: GitOutput | Error): string; // throws FailureCode.Unavailable
/// ```
fn tree_of(asked: std::io::Result<GitOutput>) -> Result<String, FailureCode> {
    return match asked {
        Ok(answer) if answer.succeeded() => Ok(String::from_utf8_lossy(answer.stdout.as_slice()).trim().to_string()),
        _ => Err(FailureCode::Unavailable),
    };
}

/// What:
///  The paths Git's answer to `moved_request` names.
/// Why:
///  Both faces read the listing through this one function,
/// so a listing and a refusal are the same answer for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function movedOf(asked: GitOutput | Error): path[]; // throws FailureCode.Unavailable
/// ```
fn moved_of(asked: std::io::Result<GitOutput>) -> Result<Vec<Vec<u8>>, FailureCode> {
    return match asked {
        Ok(output) if output.succeeded() => Ok(names_in(output.stdout.as_slice())),
        _ => Err(FailureCode::Unavailable),
    };
}

/// What:
///  The paths in which a commit's own index differs from the live index.
/// Why:
///   A writer replaces the index file whole,
/// so a live index that is another file than the one the commit copied may
///  hold entries the commit's copy does not:
/// the paths that differ are where the copy is older.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// movedSince(liveIndex, commitIndex) // `git write-tree` then `diff-index --cached <tree>` on the copy
/// ```
fn moved_since(git: &GitContext, repository: &Path, live_index: &Path, commit_index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let held: String = tree_of(run_git(git, &tree_request(repository, live_index)))?;
    return moved_of(run_git(git, &moved_request(repository, commit_index, held.as_str())));
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
///  The two paths of one copy of an index aside,
/// owned so that a job of the blocking pool can take them.
/// Why:
///   Setting the commit's own index to the head keeps a copy of it first,
/// so a step that refuses leaves the file exactly as it was;
///  both copies wait in the kernel on the disk the repository is on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexCopy = { from: string; to: string };
/// ```
struct IndexCopy {
    /// The file the copy is made of,
    /// or the copy that is put back.
    from: PathBuf,
    /// The file the copy is written to,
    /// or the index the copy is put back into.
    to: PathBuf,
}

/// The copy one job of the pool makes:
/// `std::fs::copy` over the two paths of the question.
fn copy_of(work: IndexCopy) -> std::io::Result<()> {
    let _written: u64 = std::fs::copy(work.from.as_path(), work.to.as_path())?;
    return Ok(());
}

/// The removal one job of the pool makes:
/// the copy aside once the step it protected ended.
fn removed(path: PathBuf) -> std::io::Result<()> {
    return std::fs::remove_file(path.as_path());
}

/// What:
///  Sets an index to the worktree's head at `paths`,
/// removing whatever it held there and below them first,
/// and leaves the file exactly as it was when any step fails.
/// Why:
fn set_to_head_with(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>], _named: &Path) -> Result<(), FailureCode> {
    if paths.is_empty() {
        return Ok(());
    }
    let kept: PathBuf = index.with_extension("before-head");
    if let Err(error) = copy_of(IndexCopy { from: index.to_path_buf(), to: kept.clone() }) {
        crate::diagnostic_log::warn("servicePrepare", format!("the commit's own index cannot be copied aside before it is set to the head ({error})").as_str());
        return Err(FailureCode::Unavailable);
    }
    let restored: Result<(), FailureCode> = set_to_head_unchecked(git, repository, index, paths);
    if restored.is_err() && let Err(error) = copy_of(IndexCopy { from: kept.clone(), to: index.to_path_buf() }) {
        crate::diagnostic_log::warn("servicePrepare", format!("the commit's own index could not be put back after a refused step ({error})").as_str());
    }
    let _gone: std::io::Result<()> = removed(kept.clone());
    return restored;
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

/// What:
///  The question `set_to_head_unchecked` asks first:
///  the removal of whatever the index held at `paths` and below them.
/// Why:
///  Both faces ask the one question,
/// so neither prunes another set of entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['rm', '--cached', '-r', '--force', '--ignore-unmatch', '--quiet', '--', ...specs], index };
/// ```
fn prune_request(repository: &Path, index: &Path, paths: &[Vec<u8>]) -> GitRequest {
    let specs: Vec<OsString> = specs_of(paths);
    let mut prune_words: Vec<&OsStr> = vec![
        OsStr::new("rm"),
        OsStr::new("--cached"),
        OsStr::new("-r"),
        OsStr::new("--force"),
        OsStr::new("--ignore-unmatch"),
        OsStr::new("--quiet"),
        OsStr::new("--"),
    ];
    prune_words.extend(specs.iter().map(OsString::as_os_str));
    let mut prune: GitRequest = GitRequest::new(repository, prune_words.as_slice()).own_pathspecs();
    prune.index = Some(index.to_path_buf());
    return prune;
}

/// What:
///  The listing `set_to_head_unchecked` writes when Git's own `reset` refused:
///  the head's entry of each path,
///  and a removal for a path the head holds nothing at.
/// Why:
///  Both faces write the head's entries through this one function,
/// so a path the head does not hold and a path it does
///  are the same entry for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listing = head.has(path) ? `${mode} ${object} 0\t${path}\0` : `0 ${'0'.repeat(40)} 0\t${path}\0`;
/// ```
fn index_info_listing(head: &BTreeMap<Vec<u8>, Entry>, paths: &[Vec<u8>]) -> Vec<u8> {
    let mut listing: Vec<u8> = Vec::new();
    for path in paths {
        let (mode, object) = match head.get(path) {
            Some((mode, object)) => (mode.clone(), object.clone()),
            // The head holds nothing there, so the entry goes.
            None => (String::from("0"), String::from("0".repeat(40))),
        };
        listing.extend_from_slice(format!("{mode} {object} 0\t").as_bytes());
        listing.extend_from_slice(path.as_slice());
        listing.push(0);
    }
    return listing;
}

/// What:
///  The question `set_to_head_unchecked` asks last:
///  the entries written where the head's tree has them.
/// Why:
///  Both faces write the one listing to the index they were given,
/// so neither writes another entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['update-index', '-z', '--index-info'], index, input: listing };
/// ```
fn index_info_request(repository: &Path, index: &Path, listing: Vec<u8>) -> GitRequest {
    let mut write: GitRequest = GitRequest::new(repository, &["update-index", "-z", "--index-info"]);
    write.index = Some(index.to_path_buf());
    write.input = Some(listing);
    return write;
}

/// What:
///  Whether Git took one of the two writes this step makes.
/// Why:
///  Both faces read the answer through this one function,
/// so a write that could not be started and one Git refused are the
///  same answer for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tookIt(asked: GitOutput | Error): boolean;
/// ```
fn took_it(asked: std::io::Result<GitOutput>) -> bool {
    return matches!(asked, Ok(output) if output.succeeded());
}

/// The two steps of `set_to_head`,
/// each of which may refuse.
fn set_to_head_unchecked(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    if !took_it(run_git(git, &prune_request(repository, index, paths))) {
        return Err(FailureCode::Unavailable);
    }
    if crate::service_land_follow::reset_entries(git, repository, Some(index), "HEAD", paths).is_ok() {
        return Ok(());
    }
    // A `reset` this wrapper refuses is not the only way to the head: the entries are written where the head's tree has them.
    let head: BTreeMap<Vec<u8>, Entry> = head_entries(git, repository, paths)?;
    let listing: Vec<u8> = index_info_listing(&head, paths);
    return if took_it(run_git(git, &index_info_request(repository, index, listing))) { Ok(()) } else { Err(FailureCode::Unavailable) };
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

/// `set_to_head` for paths whose old entry is no record's,
/// with the worktree's head as the entry to take.
fn set_to_head_paths(git: &GitContext, repository: &Path, index: &Path, paths: &[Vec<u8>]) -> Result<(), FailureCode> {
    return set_to_head_with(git, repository, index, paths, index);
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

/// What:
///  Sets the commit's own index to the worktree's head for the paths whose entry it would record against that head,
///  and gives those paths.
/// Why:
///   This is the whole of what a commit does about a landed batch's owed index update.
/// The live index is never written,
///  so nothing that may be a program's staging is replaced,
/// and nothing here removes a record,
/// so no other command's
///  snapshot is left unprotected.
/// A path is left out while the commit's own index still holds the entry the landing
///  replaced,
/// or while its live entry moved after the copy:
/// one can be the landing's leftover,
/// the other is someone
///  else's newer intent,
/// and in both cases what the commit would record there is not the person's.
/// The paths are set to the
///  head rather than to the landed commit,
/// so the commit records nothing about them whatever the branch has done since.
/// An explicit-path commit names its own selection,
/// which is the person's statement,
/// so nothing is
///  taken from it and the notice alone is given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludeWhatALandedCommitLeft(git, gitDir, liveIndex, commitIndex) // the paths this commit leaves out
/// ```
pub fn exclude_what_a_landed_commit_left(
    git: &GitContext,
    repository: &Path,
    git_dir: &Path,
    live_index: &Path,
    commit_index: Option<&Path>,
    captured_index: Option<&Path>,
    captured: &crate::service_land_start::IndexWitness,
) -> Result<Vec<String>, String> {
    let mut note: Option<String> = None;
    let left: Vec<String> = exclude_with_note(git, repository, git_dir, live_index, commit_index, captured_index, captured, &mut note)?;
    return Ok(left);
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
///  `exclude_what_a_landed_commit_left`,
/// leaving the sentence a commit must print when an obligation could not be read,
/// and nothing otherwise.
/// Why:
///   The notice of a repair names the paths it left out;
/// when the reason is a record that cannot be read,
/// the person
///  must be told that too,
/// and the reason is only known here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludeWithNote(...) // [paths, note]
/// ```
pub fn exclude_with_note(
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
    let (owed_records, unreadable): (Vec<Owed>, usize) = crate::owed_index_update::records_and_unreadable(git_dir);
    // An obligation that cannot be read names no path, so nothing can be repaired for it: what is known is that a landed file could
    //  be recorded as deleted. The commit goes through with such a deletion left out, and says why.
    if unreadable > 0 {
        let at_risk: Vec<Vec<u8>> = match commit_index {
            None => Vec::new(),
            Some(index) => deletions_a_landing_could_own(git, repository, index)?,
        };
        if !at_risk.is_empty() {
            let index = commit_index.expect("the paths were asked of the commit's own index");
            if let Err(code) = set_to_head_paths(git, repository, index, at_risk.as_slice()) {
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
    if let (Some(index), Some(copy), Ok(now)) = (commit_index, captured_index, crate::service_land_start::witness_of(live_index))
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
            if let Ok(names) = names_of(git, repository, owed) {
                named.extend(names);
            }
        }
        let moved: Vec<Vec<u8>> = match moved_since(git, repository, live_index, copy) {
            Ok(moved) => moved,
            Err(_unwritable) => match moved_since_listing(git, repository, live_index, copy) {
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
            if let Err(code) = set_to_head_paths(git, repository, index, owned.as_slice()) {
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
        let names: Vec<Vec<u8>> = match names_of(git, repository, &owed) {
            Ok(names) => names,
            Err(code) => return Err(format!("the paths of the index update batch {} owes cannot be read ({code:?})", owed.seq)),
        };
        let replaced: BTreeMap<Vec<u8>, Entry> = match tree_entries(git, repository, owed.old.as_str(), names.as_slice()) {
            Ok(entries) => entries,
            Err(code) => return Err(format!("the tree the landing of batch {} replaced cannot be read ({code:?})", owed.seq)),
        };
        let held: BTreeMap<Vec<u8>, Entry> = match index_entries(git, repository, snapshot, names.as_slice()) {
            Ok(entries) => entries,
            Err(code) => return Err(format!("the commit's own index cannot be read for the paths of batch {} ({code:?})", owed.seq)),
        };
        // What the head holds there: a path whose copy already holds it is not left out, and is not said, because the commit
        //  records nothing about it either way.
        let head: BTreeMap<Vec<u8>, Entry> = match head_entries(git, repository, names.as_slice()) {
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
            let theirs: bool = match (entry, worktree_object(git, repository, name.as_slice())) {
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
        if let Err(code) = set_to_head_with(git, repository, snapshot, excluded.as_slice(), snapshot) {
            return Err(format!("the commit's own index cannot be set to the head for the entries batch {} owes ({code:?})", owed.seq));
        }
        for path in &excluded {
            left.push(crate::path_bytes::path_message_text(path.as_slice()));
        }
    }
    return Ok(left);
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

/// The path one worktree question names,
/// owned so that a job of the blocking pool can take it.
struct WorktreePath {
    /// The path in the worktree the entry belongs to.
    path: PathBuf,
}

/// What:
///  What one path of the worktree holds:
/// nothing,
/// or a file or a link with the mode Git records for it and the text of a link.
/// Why:
///  The repair compares the entry the index holds with what the worktree holds,
/// and both facts are read from the
///  same metadata,
/// so a file that is replaced between two looks cannot be read as two different entries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeFile = { kind: 'absent' } | { kind: 'present'; mode: string; linkTarget?: Uint8Array };
/// ```
enum WorktreeFile {
    /// Nothing is there.
    Absent,
    /// A file or a link is there,
    /// with the mode Git records and a link's target text.
    Present(WorktreeEntry),
}

/// One path of the worktree that is there:
/// its mode and a link's target.
struct WorktreeEntry {
    /// The mode Git records:
    /// `100644`,
    /// `100755` or `120000`.
    mode: String,
    /// The text of a link's target,
    /// which is what Git stores for a link.
    link_target: Option<Vec<u8>>,
}

/// What:
///  The storage facts one path of the worktree answers,
/// or the error of the look.
/// Why:
///   A file that is not there is absent,
/// which is evidence;
/// one that is there and cannot be read is no evidence at all.
/// Git stores a link as the text of its target,
///  so that text is read here,
/// where the file's kind is known,
/// instead of through the link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeFile(path): WorktreeFile | Error;
/// ```
fn worktree_file(work: WorktreePath) -> std::io::Result<WorktreeFile> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(work.path.as_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(WorktreeFile::Absent),
        Err(error) => return Err(error),
    };
    let linking: bool = metadata.file_type().is_symlink();
    let link_target: Option<Vec<u8>> = if linking {
        use std::os::unix::ffi::OsStringExt as _;
        Some(std::fs::read_link(work.path.as_path())?.into_os_string().into_vec())
    } else {
        None
    };
    #[cfg(unix)]
    let mode: String = {
        use std::os::unix::fs::PermissionsExt as _;
        if linking {
            String::from("120000")
        } else if metadata.permissions().mode() & 0o111 != 0 {
            String::from("100755")
        } else {
            String::from("100644")
        }
    };
    #[cfg(not(unix))]
    let mode: String = String::from("100644");
    return Ok(WorktreeFile::Present(WorktreeEntry { mode, link_target }));
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
///  The question `worktree_object` asks Git for one path:
///  the object of the file,
///  or of the text of a link's target.
/// Why:
///  Both faces ask the one question,
/// so a link's object is hashed from its target by both of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = linked ? { arguments: ['hash-object', '--stdin'], input: target } : { arguments: ['hash-object', '--', path] };
/// ```
fn hash_request(repository: &Path, path: &OsStr, link_target: Option<&[u8]>) -> GitRequest {
    return match link_target {
        Some(target) => {
            let mut hashing: GitRequest = GitRequest::new(repository, &[OsStr::new("hash-object"), OsStr::new("--stdin")]);
            hashing.input = Some(target.to_vec());
            hashing
        }
        None => GitRequest::new(repository, &[OsStr::new("hash-object"), OsStr::new("--"), path]),
    };
}

/// What:
///  The entry Git's answer to `hash_request` names,
/// with the mode the worktree file carries.
/// Why:
///  Both faces read the object through this one function,
/// so a start Git refused and an object it wrote are the same
///  answer for the two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function objectAnswered(asked: GitOutput | Error, mode: string): [mode, object]; // throws FailureCode.Unavailable
/// ```
fn object_answered(asked: std::io::Result<GitOutput>, mode: &str) -> Result<(String, String), FailureCode> {
    return match asked {
        Ok(answer) if answer.succeeded() => Ok((String::from(mode), String::from_utf8_lossy(answer.stdout.as_slice()).trim().to_string())),
        _ => Err(FailureCode::Unavailable),
    };
}

/// What:
///  The object the worktree holds at one path,
/// or nothing when it holds no file there.
/// Why:
///   A worktree that still holds the entry the index holds means the person staged what they have,
/// so the
///  entry is their own statement and nothing may be changed there;
/// a worktree that holds something
///  else is an index that no longer describes it,
/// which is what a failed index update leaves.
/// The object is
///  asked of the file itself,
/// so no stat cache can answer for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeObject(name) // `git hash-object -- <path>`
/// ```
fn worktree_object(git: &GitContext, repository: &Path, name: &[u8]) -> Result<Option<(String, String)>, FailureCode> {
    let path: OsString = <OsString as std::os::unix::ffi::OsStringExt>::from_vec(name.to_vec());
    let asked: WorktreePath = WorktreePath { path: repository.join(path.as_os_str()) };
    return match worktree_file(asked) {
        Ok(WorktreeFile::Absent) => Ok(None),
        Ok(WorktreeFile::Present(present)) => Ok(Some(object_answered(run_git(git, &hash_request(repository, path.as_os_str(), present.link_target.as_deref())), present.mode.as_str())?)),
        Err(_unreadable) => Err(FailureCode::Unavailable),
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
///  The one line a commit prints about the index entries it left out,
/// and nothing when it left none.
/// Why:
///   A commit whose content differs from the index must say why,
/// and name both ways on:
/// catching the index up with the
///  branch,
/// and taking the worktree's content for those paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludedNotice(paths)
/// ```
pub fn excluded_notice(paths: &[String]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }
    return Some(format!(
        "monochromatic-git-policy-cli: a commit that landed did not finish updating this worktree's index, and these entries differ from the branch. They were left out of this commit: {}. To take the branch's content: git reset -- <paths>. To commit the worktree's content for them: git add -- <paths>, then git commit.\n",
        paths.join(", ")
    ));
}

/// What:
///  The sentence a commit prints when an obligation of this worktree cannot be read.
/// Why:
///   The paths it left out are those a landing could have made it delete;
/// the person is told why they were left out,
///  and
/// that the record is still there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unreadableNotice(count, paths)
/// ```
pub fn unreadable_notice(count: usize, paths: &[String]) -> String {
    return format!(
        "monochromatic-git-policy-cli: {count} record(s) of what this worktree's index owes cannot be read, so the deletion this commit would have recorded of {} was left out, and the record is kept.",
        paths.join(", ")
    );
}

/// What:
///  The question `deletions_a_landing_could_own` asks Git:
///  the paths the commit's own index records as deleted against the head.
/// Why:
///  Both faces ask the one question,
/// so neither keeps another set of paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd: repository, arguments: ['diff-index', '--cached', '--name-only', '--diff-filter=D', '-z', 'HEAD'], index };
/// ```
fn deletions_request(repository: &Path, index: &Path) -> GitRequest {
    let mut request: GitRequest = GitRequest::new(repository, &["diff-index", "--cached", "--name-only", "--diff-filter=D", "-z", "HEAD"]);
    request.index = Some(index.to_path_buf());
    return request;
}

/// The names one listing of that question holds,
/// owned so that a job of the blocking pool can take them.
struct ListedNames {
    /// The names Git listed,
    /// in its order.
    names: Vec<Vec<u8>>,
    /// The worktree the names are looked for in.
    repository: PathBuf,
}

/// What:
///  The names of `work` whose file the worktree holds,
/// in the order Git listed them.
/// Why:
///   A deletion the person staged is a file the worktree does not hold,
/// and one a landing left is a file that is still there;
///  the look is a `stat` of every listed name,
/// which waits in the kernel on a mount that stopped answering.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function presentInTheWorktree({ names, repository }) { return names.filter((name) => existsSync(join(repository, name))) }
/// ```
fn present_in_the_worktree(work: ListedNames) -> Vec<Vec<u8>> {
    let mut at_risk: Vec<Vec<u8>> = Vec::new();
    for name in &work.names {
        let held: PathBuf = work.repository.join(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(name.clone()).as_os_str());
        if std::fs::symlink_metadata(held.as_path()).is_ok() {
            at_risk.push(name.clone());
        }
    }
    return at_risk;
}

/// The names one listing of `deletions_request` holds,
/// and the failure the repair says when Git gave none.
fn listed_deletions(asked: std::io::Result<GitOutput>) -> Result<Vec<Vec<u8>>, String> {
    let output: GitOutput = match asked {
        Ok(answer) if answer.succeeded() => answer,
        _ => return Err(String::from("the deletions this commit would record cannot be read")),
    };
    let mut names: Vec<Vec<u8>> = Vec::new();
    for name in output.stdout.split(is_zero_byte) {
        if name.is_empty() {
            continue;
        }
        names.push(name.to_vec());
    }
    return Ok(names);
}

/// What:
///  The paths this commit would record as deleted while the head holds an entry and the worktree holds the file:
/// what a landing whose index update never ran leaves.
/// Why:
///   An obligation that cannot be read names no path,
/// and the only state it can have left is a copy of the index older than the landing.
/// A deletion the person staged is a file the worktree does not hold,
/// so it is not this set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// deletionsALandingCouldOwn(index) // `diff-index --cached --diff-filter=D HEAD`, kept where the worktree has the file
/// ```
fn deletions_a_landing_could_own(git: &GitContext, repository: &Path, index: &Path) -> Result<Vec<Vec<u8>>, String> {
    let names: Vec<Vec<u8>> = listed_deletions(run_git(git, &deletions_request(repository, index)))?;
    return Ok(present_in_the_worktree(ListedNames { names, repository: repository.to_path_buf() }));
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
