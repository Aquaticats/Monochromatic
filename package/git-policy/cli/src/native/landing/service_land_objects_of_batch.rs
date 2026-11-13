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
use crate::service_land_git_steps_and_import_stream::{commits_from_marks, import_stream, run};
use crate::service_land_settings::Settings;
use crate::service_land_side_reference_write::{ASideRefWrite, keep_on_the_scratch_reference, remember_the_tip_a_side_reference_does_not_hold};
use crate::service_land_without_its_scratch_reference::remember_why_it_has_none;
use crate::service_scratch_ref_marker_of_a_batch::record_scratch_ref;
use crate::side_reference_written_here::{SideReferenceWrite, SideReferenceWritePlan, write_side_reference};

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
    settings: &Settings<'_>,
) -> Result<(Vec<String>, String), FailureCode> {
    let repository: &Path = first.repository.as_path();
    let unborn: bool = is_zero_object(first.base.as_str());
    return if let Some(built) = first.commit.as_deref() {
        // A commit Git built elsewhere lands as it is; it must be a commit with the parents this request's rule gives:
        //  an amend has the replaced commit's parents, a commit on a branch that did not exist has none, and any other
        //  commit has the branch tip first (a merge conclusion has the merged commits after it).
        // The parents are read from the object store; a commit it cannot read as a plain commit is listed by Git, whose
        //  answer is the one that counts (a repository with grafts, a shallow history or replacement objects).
        let read: Option<BuiltCommit> = if settings.plain_history { read_commit(first.common_dir.as_path(), built) } else { None };
        let (named_itself, parents): (bool, Vec<String>) = the_parents_of(git, repository, built, read.as_ref())?;
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
        //  A step a stop ended is the landing's own ending, and it keeps its code so that the commit is kept beside it
        //  (`service_land.rs`, `keep_what_an_ended_step_built`).
        // A reference the write did not prove is no reason to fail the commit: real Git needs no such reference, and this commit is
        //  made as Git makes it, the branch naming it a step later. The batch lands, and the landing tells the person once what the
        //  missing reference exposed (`service_land_without_its_scratch_reference.rs`).
        record_scratch_ref(journal, seq, repository, first.git_dir.as_path());
        // The reference is the wrapper's own name, and a Git start for one rename is time the person waits on for nothing.
        //  The write is taken here only where it is provably Git's; every other repository answers `LeftToGit` and Git writes
        //  the reference exactly as it did before, its refusals in its own words (`side_reference_written_here.rs`).
        let here: Option<ASideRefWrite> = match settings.side_reference {
            SideReferenceWritePlan::LeftToGit => None,
            SideReferenceWritePlan::Here(mark) => match write_side_reference(first.git_dir.as_path(), scratch, built, mark) {
                SideReferenceWrite::Written => Some(ASideRefWrite::Written),
                SideReferenceWrite::LeftToGit => None,
            },
        };
        let written: ASideRefWrite = match here {
            Some(done) => done,
            None => keep_on_the_scratch_reference(git, repository, scratch, built),
        };
        return match written {
            ASideRefWrite::Written => Ok((vec![built.to_string()], built.to_string())),
            ASideRefWrite::Refused(words) => {
                crate::service_error!("batch {seq}: the commit Git built lands without its own reference: Git refused the write of {scratch} ({words})");
                remember_why_it_has_none(format!("Git refused the write of {scratch} ({words})"));
                Ok((vec![built.to_string()], built.to_string()))
            }
            ASideRefWrite::LeftRunning => {
                crate::service_error!("batch {seq}: the commit Git built lands without its own reference: the write of {scratch} was left running, so whether it names the commit is not known");
                remember_why_it_has_none(format!("this command stopped waiting for the write of {scratch}, so whether that reference names it is not known"));
                Ok((vec![built.to_string()], built.to_string()))
            }
            // Git could not be run, or the service's stop ended the write: the branch write that would name the commit cannot run
            //  either, so the commit is kept beside the failure (`keep_what_a_side_reference_write_left`).
            ASideRefWrite::Ended(code) => {
                crate::service_error!("batch {seq}: the commit Git built cannot be kept on {scratch}, because the write of it was ended ({code:?})");
                remember_the_tip_a_side_reference_does_not_hold(built, format!("the write of {scratch} was ended ({code:?})"));
                Err(code)
            }
        };
    } else {
        let marks: PathBuf = journal.join(format!("marks-{seq}"));
        // The marks this attempt reads are this attempt's own: a file an earlier attempt left is removed before the import
        //  starts, because `fast-import` writes its marks only when it exits and a child killed before that leaves the earlier
        //  file untouched (`fresh_marks_file`).
        let marks_are_this_attempts: bool = fresh_marks_file(marks.as_path(), seq);
        let marks_argument: String = format!("--export-marks={}", marks.display());
        // The `core.fsync` of the import is a global option of this one start; it reaches the `unpack-objects` child too, where
        //  it names the same loose-object flush the repository has.
        let fsync_setting: String = match settings.import_fsync {
            Some(value) => format!("core.fsync={value}"),
            None => String::new(),
        };
        // The import moves `refs/worktree/cli-git-service/<seq>` at its end, and a service killed between that and the intent
        //  leaves the reference with nothing that names its worktree; the record written here is what a restart finds it by
        //  (`service_scratch_ref_marker_of_a_batch.rs`).
        record_scratch_ref(journal, seq, first.repository.as_path(), first.git_dir.as_path());
        let mut import_arguments: Vec<&str> = Vec::new();
        if settings.import_fsync.is_some() {
            import_arguments.extend(["-c", fsync_setting.as_str()]);
        }
        import_arguments.extend(["fast-import", "--done", marks_argument.as_str()]);
        let imported: GitOutput = run(git, repository, import_arguments.as_slice(), Some(import_stream(scratch, old, batch, amended_parents)))?;
        // What this import made, and the cause the person is told when its own reference is what the repository refused.
        let mut refused_reference: Option<String> = None;
        let (made, last): (Vec<String>, String) = if imported.succeeded() {
            // The import carries no `--quiet`, so its `--stats` report is kept in the service's log.
            crate::inner_git_words::record_words(&["fast-import", "--done"], &imported);
            let marks_text: String = match std::fs::read_to_string(&marks) {
                Ok(read_marks) => read_marks,
                Err(_unreadable) => return Err(FailureCode::Unavailable),
            };
            let made: Vec<String> = commits_from_marks(marks_text.as_str(), batch.len()).ok_or(FailureCode::Unavailable)?;
            let last: String = made.last().cloned().ok_or(FailureCode::Unavailable)?;
            (made, last)
        } else {
            // The import makes the commits and updates the batch's reference at its end, so a repository that refuses that
            //  update leaves every commit it made named by nothing. A child that exited wrote its marks at exit, and one that
            //  read the stream to its end wrote every commit of the batch into them; a child a signal ended wrote none at all,
            //  and one that stopped inside the stream wrote fewer or none, so neither is a batch
            //  (`the_commits_a_refused_reference_left_unnamed`). What is here then lands as real Git would land it, the branch
            //  naming it a step later, and the person is told once what the missing reference exposed
            //  (`service_land_without_its_scratch_reference.rs`).
            // The whole of Git's words go to the service's log, and the person reads the line that names the refusal: the
            //  import follows it with its statistics report, which is no sentence for a person (`the_refusal_line_of`).
            let refused: String = crate::service_land_side_reference_write::refused_words_of(&imported);
            let told: String = crate::service_land_side_reference_write::the_refusal_line_of(&imported);
            let whole: Option<Vec<String>> = match imported.code.is_some() && marks_are_this_attempts {
                true => the_commits_a_refused_reference_left_unnamed(marks.as_path(), batch.len()),
                false => None,
            };
            let Some(made) = whole else { return Err(classify(&imported)) };
            let last: String = made.last().cloned().ok_or(FailureCode::Unavailable)?;
            crate::service_error!("batch {seq}: the batch without its own reference: the import could not write {scratch} ({refused})");
            refused_reference = Some(format!("the import of this batch could not write {scratch} ({told})"));
            (made, last)
        };
        // The last door of this path, where the path that takes a commit Git built has its own: a branch write asks `update-ref`
        //  only that the branch still names the old value, so a commit built on another base would move the branch past a commit
        //  another command landed between the two (`the_commit_this_attempt_built_stands_on`).
        let expected: Option<&str> = if unborn {
            None
        } else if first.amend && first.tree.is_some() {
            amended_parents.first().map(String::as_str)
        } else {
            Some(old)
        };
        if !the_commit_this_attempt_built_stands_on(git, repository, made.first().map(String::as_str), expected, first, settings)? {
            let first_made: String = made.first().cloned().unwrap_or_default();
            let why: String = format!("the import of this batch built {first_made} on another base than the one this attempt found ({})", expected.unwrap_or("no parent"));
            crate::service_error!("batch {seq}: {why}, so this batch is not landed");
            remember_the_tip_a_side_reference_does_not_hold(last.as_str(), why);
            return Err(FailureCode::Unavailable);
        }
        let _marks_removed = std::fs::remove_file(&marks);
        if let Some(why) = refused_reference {
            remember_why_it_has_none(why);
        }
        Ok((made, last))
    };
}

/// What:
///  The parents of a commit,
///  read from the object store when the history is plain and listed by Git otherwise,
///  and whether the answer names that commit at all.
/// Why:
///   Both paths of `objects_of_batch` ask the same question of a commit before it reaches the branch:
///  a commit Git built must have the parents its request's rule gives,
///  and the first commit an import built must stand on this attempt's base.
///  A commit the object store cannot read as a plain commit is listed by Git,
///  whose answer is the one that counts (a repository with grafts,
///  a shallow history or replacement objects).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { namedItself, parents } = await theParentsOf(git, repo, commit, readCommit?)
/// ```
fn the_parents_of(git: &GitContext, repository: &Path, commit: &str, read: Option<&BuiltCommit>) -> Result<(bool, Vec<String>), FailureCode> {
    let Some(commit_read) = read else {
        let listed: Vec<&str> = vec!["rev-list", "--parents", "--max-count=1", commit];
        let listing: GitOutput = run(git, repository, listed.as_slice(), None)?;
        let text: String = String::from_utf8_lossy(listing.stdout.as_slice()).into_owned();
        let mut words = text.split_whitespace();
        let named: bool = listing.succeeded() && words.next() == Some(commit);
        return Ok((named, words.map(str::to_string).collect()));
    };
    return Ok((true, commit_read.parents.clone()));
}

/// What:
///  Whether the first commit an import built has the parent this attempt's base gives,
///  which is the check the path that takes a commit Git built already makes.
/// Why:
///   `git update-ref` asks only that the branch still names the old value,
///  never that the new commit has it as a parent,
///  so a batch built on another base would move the branch past a commit another command landed in between:
///  the retry loop exists to prevent exactly that,
///  and this is the last place it can be noticed.
///  A batch on a branch that did not exist has no parent,
///  and an amend has the parents of the commit it replaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { namedItself, parents } = await theParentsOf(...); return namedItself && parents[0] === expected
/// ```
fn the_commit_this_attempt_built_stands_on(
    git: &GitContext,
    repository: &Path,
    commit: Option<&str>,
    expected: Option<&str>,
    first: &CommitRequest,
    settings: &Settings<'_>,
) -> Result<bool, FailureCode> {
    let Some(commit) = commit else { return Ok(false) };
    let read: Option<BuiltCommit> = match settings.plain_history {
        true => read_commit(first.common_dir.as_path(), commit),
        false => None,
    };
    let (named_itself, parents): (bool, Vec<String>) = the_parents_of(git, repository, commit, read.as_ref())?;
    return Ok(named_itself && parents.first().map(String::as_str) == expected);
}

/// What:
///  Whether the marks file of this batch was proven gone before its import started.
/// Why:
///  `fast-import` writes its marks when it exits,
///  so a file an earlier attempt left is still there while this attempt's import runs,
///  and a child ended by a signal before it exits leaves that earlier file untouched.
///  A batch that read it would land an earlier attempt's commits on this attempt's base.
///  A file that cannot be removed is one this attempt does not read either:
///  the answer is `false`,
///  and an import that fails is then not taken from marks at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fresh = (marks) => { try { unlinkSync(marks); return true } catch (error) { return error.code === 'ENOENT' } }
/// ```
fn fresh_marks_file(marks: &Path, seq: u64) -> bool {
    return match std::fs::remove_file(marks) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => {
            crate::service_error!("batch {seq}: the marks file {} of an earlier attempt could not be removed ({error}), so no marks are read for this attempt", marks.display());
            false
        }
    };
}

/// What:
///  The commits of a batch `fast-import` made in full,
/// when the marks it exported name every request of that batch,
/// and nothing otherwise.
/// Why:
///  `fast-import` exports its marks at the end of the stream and updates the batch's reference last,
///  so a marks file naming every commit is a batch whose objects the repository holds and whose
///  reference it refused.
///  A marks file naming fewer is an import that stopped inside the batch,
/// and the commit it names is not the commit the person's requests make together,
///  so it is not landed as one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = commitsFromMarks(readMarks(marks), requests) // nothing unless every request is there
/// ```
fn the_commits_a_refused_reference_left_unnamed(marks: &Path, requests: usize) -> Option<Vec<String>> {
    let text: String = std::fs::read_to_string(marks).ok()?;
    return commits_from_marks(text.as_str(), requests);
}

/// Controls for the objects a batch lands and the side reference that keeps them stay out of the release executable.
#[cfg(test)]
#[path = "service_land_objects_of_batch_tests.rs"]
mod tests;
