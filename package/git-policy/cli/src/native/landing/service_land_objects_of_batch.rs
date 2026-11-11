//! What:
//!  The objects a batch lands:
//! the commit Git built,
//! checked for its parents and its objects,
//! or the commits `fast-import` makes
//!  from the changes of the requests.
//! Why:
//!  It is the step of a landing that creates objects and the largest part of `land_batch_body`;
//! a file of its own keeps that
//!  function short.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { commits, newTip } = await objectsOfBatch(...)
//! ```

use crate::service_fault::reach;
use crate::service_built_checks::{BuiltCommit, adds_only_present_objects, read_commit};
use crate::service_land_follow::classify;
use crate::service_protocol::{CommitRequest, FailureCode, is_zero_object};
use crate::transaction_git::{GitContext, GitOutput};
use std::path::{Path, PathBuf};
use crate::service_land_git_steps_and_import_stream::{ASideRefWrite, commits_from_marks, import_stream, keep_on_the_scratch_reference, run};
use crate::service_scratch_ref_marker_of_a_batch::record_scratch_ref;

/// What:
///  Makes the commits a batch lands:
/// checks the commit Git built (its parents and its objects) or imports the changes of the
///  requests as new commits,
/// and returns the commits made with the new tip.
/// Why:
///  This is the one step of a landing that creates objects,
/// and the intent that records the batch names what it returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { commits, newTip } = await objectsOfBatch(git, journal, seq, first, scratch, old, batch, amendedParents, plainHistory, importFsync)
/// ```
pub(crate) fn objects_of_batch(
    git: &GitContext,
    journal: &Path,
    seq: u64,
    first: &CommitRequest,
    scratch: &str,
    old: &str,
    batch: &[&CommitRequest],
    amended_parents: &[String],
    plain_history: bool,
    import_fsync: Option<&str>,
) -> Result<(Vec<String>, String), FailureCode> {
    let repository: &Path = first.repository.as_path();
    let unborn: bool = is_zero_object(first.base.as_str());
    return if let Some(built) = first.commit.as_deref() {
        // A commit Git built elsewhere lands as it is; it must be a commit with the parents this request's rule gives:
        //  an amend has the replaced commit's parents, a commit on a branch that did not exist has none, and any other
        //  commit has the branch tip first (a merge conclusion has the merged commits after it).
        // The parents are read from the object store; a commit it cannot read as a plain commit is listed by Git, whose
        //  answer is the one that counts (a repository with grafts, a shallow history or replacement objects).
        let read: Option<BuiltCommit> = if plain_history { read_commit(first.common_dir.as_path(), built) } else { None };
        let (named_itself, parents): (bool, Vec<String>) = match &read {
            Some(commit) => (true, commit.parents.clone()),
            None => {
                let listing: GitOutput = run(git, repository, &["rev-list", "--parents", "--max-count=1", built], None)?;
                let text: String = String::from_utf8_lossy(listing.stdout.as_slice()).into_owned();
                let mut words = text.split_whitespace();
                let named: bool = listing.succeeded() && words.next() == Some(built);
                (named, words.map(str::to_string).collect())
            }
        };
        let acceptable: bool = if first.amend {
            parents == amended_parents
        } else if unborn {
            parents.is_empty()
        } else {
            parents.first() == Some(&first.base)
        };
        if !named_itself || !acceptable {
            return Err(FailureCode::Rejected);
        }
        // Git moves a branch to a commit without looking at what the commit holds, so a tree or a blob that a prune took
        //  while the commit object survived would put a damaged commit on the branch.
        reach("before-connectivity");
        // A commit whose one parent is the tip adds the objects that differ from the tip's tree, and the object store shows
        //  they are all there; any other commit, and any object that looks absent, is checked by the `rev-list` Git always ran.
        let proven: bool = match read.as_ref() {
            Some(commit) => !unborn && adds_only_present_objects(first.common_dir.as_path(), commit, old),
            None => false,
        };
        if !proven {
            let connected: Vec<&str> = if unborn { vec!["rev-list", "--objects", built] } else { vec!["rev-list", "--objects", built, "--not", old] };
            if !run(git, repository, connected.as_slice(), None)?.succeeded() {
                return Err(FailureCode::Unavailable);
            }
        }
        // A commit Git built elsewhere is an object no reference names until the branch write below moves one to it, and
        //  `git gc --prune=now` takes an object nothing names. It is kept on the batch's side reference here, as the import
        //  keeps the commits it makes, so a commit that is held behind a reference lock, or a landing a kill leaves, keeps what
        //  it made (`held_commit_record.rs`, `service_scratch_ref_marker_of_a_batch.rs`).
        // The write is answered by the reference, not by the wait for it: a wait that ends without its answer is told apart from a
        //  write that failed, so a slow host never tells a person their commit was not made (`keep_on_the_scratch_reference`).
        //  A reference that still does not name the commit after that is a write the repository did not take, and the batch ends
        //  with the person's index and worktree as they were, so the answer that it was not made is true of anything they can reach.
        //  A step a stop ended is the landing's own ending, and it keeps its code so that the commit is kept beside it
        //  (`service_land.rs`, `keep_what_an_ended_step_built`).
        record_scratch_ref(journal, seq, repository, first.git_dir.as_path());
        return match keep_on_the_scratch_reference(git, repository, scratch, built) {
            ASideRefWrite::Written => Ok((vec![built.to_string()], built.to_string())),
            ASideRefWrite::Refused(words) => {
                crate::service_error!("batch {seq}: the commit Git built cannot be kept on {scratch} ({words})");
                Err(FailureCode::Unavailable)
            }
            ASideRefWrite::Ended(code) => {
                crate::service_error!("batch {seq}: the commit Git built cannot be kept on {scratch}, because the write of it was ended ({code:?})");
                Err(code)
            }
        };
    } else {
        let marks: PathBuf = journal.join(format!("marks-{seq}"));
        let marks_argument: String = format!("--export-marks={}", marks.display());
        // The `core.fsync` of the import is a global option of this one start; it reaches the `unpack-objects` child too, where
        //  it names the same loose-object flush the repository has.
        let fsync_setting: String = match import_fsync {
            Some(value) => format!("core.fsync={value}"),
            None => String::new(),
        };
        // The import moves `refs/worktree/cli-git-service/<seq>` at its end, and a service killed between that and the intent
        //  leaves the reference with nothing that names its worktree; the record written here is what a restart finds it by
        //  (`service_scratch_ref_marker_of_a_batch.rs`).
        record_scratch_ref(journal, seq, first.repository.as_path(), first.git_dir.as_path());
        let mut import_arguments: Vec<&str> = Vec::new();
        if import_fsync.is_some() {
            import_arguments.extend(["-c", fsync_setting.as_str()]);
        }
        import_arguments.extend(["fast-import", "--done", marks_argument.as_str()]);
        let imported: GitOutput = run(git, repository, import_arguments.as_slice(), Some(import_stream(scratch, old, batch, amended_parents)))?;
        if !imported.succeeded() {
            return Err(classify(&imported));
        }
        // The import carries no `--quiet`, so its `--stats` report is kept in the service's log.
        crate::inner_git_words::record_words(&["fast-import", "--done"], &imported);
        let marks_text: String = match std::fs::read_to_string(&marks) {
            Ok(read_marks) => read_marks,
            Err(_unreadable) => return Err(FailureCode::Unavailable),
        };
        let _marks_removed = std::fs::remove_file(&marks);
        let made: Vec<String> = commits_from_marks(marks_text.as_str(), batch.len()).ok_or(FailureCode::Unavailable)?;
        let last: String = made.last().cloned().ok_or(FailureCode::Unavailable)?;
        Ok((made, last))
    };
}
