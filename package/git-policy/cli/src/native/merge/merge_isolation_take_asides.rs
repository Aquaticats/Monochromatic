//! What:
//!  The take of a worktree file a merge must replace or delete:
//! its private name,
//! the check that the file taken is the file that was judged,
//! putting back what a
//!  stopped command left.
//! Why:
//!  A judgement and a write were separate steps,
//! so a file saved between them was replaced with no notice;
//! one rename
//!  takes the file atomically,
//! so the bytes that are checked are the bytes that make way for the merge
//!  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
//! finding 4).
//!  A command that stops between the rename and the write leaves the file under a private name that names its
//!  path,
//! which the next command puts back,
//! and a file written after the take checked it is kept rather than removed.
//!  What comes of the files one take moved aside is `merge_isolation_take_asides_finish.rs`:
//!  the putting back of the ones no write reached,
//!  the proof that nothing can write into one any more,
//!  the removal of the private names once the merge's content is in place,
//!  and the restore of what Git did not write.
//!  The steps live in a file of their own so the file that installs the merge stays readable.
//!
//!  Every step that reads Git is two faces of one body:
//! the awaited one a task of the runtime calls,
//! and the synchronous one a caller outside it keeps,
//!  which enters the runtime once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = await takeAside(location, judged); ... await finishAsides(taken, stderr);
//! ```

/// The one job of the blocking pool that makes the step's file calls,
/// so a disk that makes them wait holds no worker of the runtime.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::merge_isolation_carry_parts::Capture;
use crate::merge_isolation_changes::{
    Change,
    FileIdentity,
    file_identity,
};
/// The copies a command decides all at once,
/// of which this file's entrance puts back the ones a killed take left.
use crate::merge_isolation_kept_copies_decisions_sweep::settle_the_asides_of_a_merge_awaiting;
use crate::merge_isolation_recovery_place::RecoveryPlace;
/// The private name a take leaves.
use crate::merge_isolation_take_aside_names::aside_name;
/// The proof that nothing can write into a file the take moved aside,
/// asked the way a removal asks it.
pub(crate) use crate::merge_isolation_take_asides_finish::may_remove_the_aside_awaiting;
/// What comes of the files one take moved aside,
/// named here so a caller of the take keeps one path.
pub use crate::merge_isolation_take_asides_finish::{
    TakenOnDisk,
    finish_asides,
    finish_asides_awaiting,
    put_back_awaiting,
    restore_what_git_did_not_write_awaiting,
};
use crate::merge_isolation_take_asides_proof::are_the_file_form_of_awaiting;
use crate::merge_isolation_the_merges_paths_judged::Judged;
use crate::recovery_error::{
    RecoveryError,
    io_failure,
};
/// The same rename as a job of the blocking pool,
/// and what it came to.
use crate::rename_that_never_replaces::{
    TheAnswerOfARename,
    rename_noreplace_on_the_blocking_pool,
};
use crate::transaction_git::GitContext;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Put back every file a killed or stopped take left under its private name,
///  remove the copies whose bytes are somewhere the person can reach them,
///  and say which ones stay beside a path that holds something else.
/// Why:
///   Nothing of the person's may be lost,
/// and a file a person cannot find is close to lost:
/// the path is free,
///  so the file goes back and the command says so;
/// the path holds something else,
/// so both stay,
/// the
///  private name is said,
/// and the copy is recorded so a later command ends it
///  (`merge_isolation_kept_copies.rs`).
/// This runs before the paths are judged,
/// because a path whose file waits beside it is empty
///  and would be read as someone else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = await settleTheAsidesOfAMerge({ context, root, gitDir, changes, capture, stderr });
/// ```
///  The paths a copy went back to are answered,
/// because the file that arrives there is the one the merge judged:
///  its
///  identity is unchanged by the rename and its change time is not,
/// and a caller that judges those paths against a reading
///  the command that moved the checkout made has to read them again.
pub async fn put_back_stranded_takes_awaiting<W: Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: Option<&Capture>,
    transaction_id: &str,
    out: &mut W,
) -> Vec<PathBuf> {
    return settle_the_asides_of_a_merge_awaiting(
        context,
        root,
        git_dir,
        changes,
        capture,
        transaction_id,
        out,
    )
    .await;
}

/// What:
///  Why a take stopped,
///  and the file it left under its private name when it could not put it back.
/// Why:
///   The rename happens before the bytes are read,
/// so a read that fails has already changed the worktree;
/// the
///  caller must add that file to its rollback and say the merge began when the file cannot go back
///  (a cross-vendor review of the fourth round,
/// the brief's item 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TakeFailure = { error: RecoveryError; stranded?: Path };
/// ```
pub struct TakeFailure {
    /// What stopped the take.
    pub error: RecoveryError,
    /// The file that stayed at its private name,
    /// when it could not be put back.
    pub stranded: Option<PathBuf>,
}

/// What a take of one worktree path found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Take = { kind: 'nothing' } | { kind: 'aside'; aside: string } | { kind: 'changed' };
/// ```
pub enum Take {
    /// Nothing stands there,
    /// as it was judged.
    Nothing,
    /// The file that was judged waits at this private name,
    /// with the identity the take read after the rename.
    Aside {
        /// Where the file that was judged waits.
        aside: PathBuf,
        /// What the file system says of it there.
        taken: FileIdentity,
        /// The object the file held when it was judged.
        expected: String,
    },
    /// Something else stands there now,
    /// or the judged file was written after it was judged;
    /// it is not the merge's to replace,
    /// and it is back where it was found.
    Changed,
}

/// What:
///  Take the file at `location` aside by one rename that never replaces,
/// and check that what was taken is the file that was judged.
/// Why:
///   A judgement and the write of a merged file were separate steps,
/// so a file saved between them was
///  overwritten with no notice
/// (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 4).
/// One `rename` takes
///  the file atomically,
/// so the bytes that are checked are exactly the bytes that make way for the merge;
/// a
///  file that differs from the judgement is put back and never replaced.
/// The check is the file's identity,
/// which is
///  what `stale_ref_lock_retire.rs` compares for a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeAside(location: string, judged?: FileIdentity): Take; // renameNoReplace; identityOf(aside) === judged ? aside : putBack
/// ```
pub async fn take_aside_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    path: &[u8],
    location: &Path,
    judged: &Judged,
    transaction_id: &str,
) -> Result<Take, TakeFailure> {
    let aside: PathBuf = aside_name(location, transaction_id);
    // The take is one rename that writes the directory it lands in, so a job of the blocking pool makes it.
    match rename_noreplace_on_the_blocking_pool(location, aside.as_path()).await {
        TheAnswerOfARename::Renamed => {}
        TheAnswerOfARename::Failed(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // The path was empty when it was judged; a file that is there now came after the judgement and is not the
            //  merge's to replace.
            return Ok(if judged.content.is_some() {
                Take::Changed
            } else {
                Take::Nothing
            });
        }
        // A private name another entry holds is a rename that failed, and both names stay: the path and that entry.
        TheAnswerOfARename::Failed(error) | TheAnswerOfARename::NameTaken(error) => {
            return Err(TakeFailure {
                error: io_failure("setting aside", location, &error),
                stranded: None,
            });
        }
        // The rename did not run, or may have: nothing here may say the file was taken, so the take stops naming the path.
        TheAnswerOfARename::NoThreadToRunIt(refused) => {
            return Err(TakeFailure {
                error: RecoveryError(format!(
                    "{} could not be set aside: {refused}",
                    location.display()
                )),
                stranded: None,
            });
        }
    }
    // The identity of the file that was taken is a look at a user's file, which can wait on a disk, so a job of the blocking pool
    //  makes it.
    let looked_at: PathBuf = aside.clone();
    let taken: Option<FileIdentity> =
        match run_blocking_job_within_the_budget(move || return file_identity(looked_at.as_path()))
            .await
        {
            Ok(found) => found,
            // A look the pool never made cannot prove the file that was taken, so it goes back rather than be replaced.
            Err(_refused) => return put_back_and_report_awaiting(location, aside).await,
        };
    let Some(expected) = judged.content.as_deref() else {
        // Nothing stood there when it was judged, so a file that is there now is someone else's.
        return put_back_and_report_awaiting(location, aside).await;
    };
    // The identity is the cheap check and the bytes are the authority:
    // a program that keeps its descriptor across the rename can write the same
    //  length into the file and put the modification time back, which leaves every part of the identity as it was.
    if judged.identity.is_none() || taken != judged.identity {
        return put_back_and_report_awaiting(location, aside).await;
    }
    match are_the_file_form_of_awaiting(&place.git, root, path, aside.as_path(), expected).await {
        Some(true) => {}
        Some(false) => return put_back_and_report_awaiting(location, aside).await,
        // A read that fails cannot prove the bytes, so the file goes back rather than be replaced.
        None => {
            return Err(put_back_after_a_failed_read_awaiting(
                location,
                aside,
                RecoveryError(String::from(
                    "the bytes of the file the merge replaces could not be read",
                )),
            )
            .await);
        }
    }
    return match taken {
        Some(identity) => Ok(Take::Aside {
            aside,
            taken: identity,
            expected: String::from(expected),
        }),
        None => put_back_and_report_awaiting(location, aside).await,
    };
}

/// What:
///  Put back a file that turned out not to be the one that was judged,
///  and say what the take made of it.
/// Why:
///  A file that changed after the judgement belongs to someone else,
/// so it goes back where it was found and the
///  merge stops for it rather than replacing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = async (location, aside) => { const moved = await renameNoReplaceOnThePool(aside, location); if (moved.kind === 'renamed') return 'changed'; throw ...; };
/// ```
async fn put_back_and_report_awaiting(
    location: &Path,
    aside: PathBuf,
) -> Result<Take, TakeFailure> {
    let waiting_at: PathBuf = aside.clone();
    return match rename_noreplace_on_the_blocking_pool(waiting_at.as_path(), location).await {
        TheAnswerOfARename::Renamed => Ok(Take::Changed),
        // A rename that failed says why, and both names stay: the path and the private one the file is at.
        TheAnswerOfARename::Failed(error) | TheAnswerOfARename::NameTaken(error) => {
            Err(TakeFailure {
                error: RecoveryError(format!(
                    "{} changed after this command judged it and it could not be put back ({error}); it is at {} and nothing was \
                 replaced by the merge.",
                    location.display(),
                    aside.display()
                )),
                stranded: Some(aside),
            })
        }
        // The system gave the pool no thread, so the file may still be under its private name; it is named rather than lost.
        TheAnswerOfARename::NoThreadToRunIt(refused) => Err(TakeFailure {
            error: RecoveryError(format!(
                "{} changed after this command judged it and it could not be put back ({refused}); it is at {} and nothing was \
                 replaced by the merge.",
                location.display(),
                aside.display()
            )),
            stranded: Some(aside),
        }),
    };
}

/// What:
///  Put back a file the take moved aside when the read of its bytes failed,
///  and say whether it went back.
/// Why:
///   A read that fails after the rename still changed the worktree,
/// so the caller's rollback must know about this
///  file,
/// and a file that cannot go back is named and counted as installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBackAfterAFailedRead = async (location, aside, error) => { const moved = await renameNoReplaceOnThePool(aside, location); return moved.kind === 'renamed' ? { error } : { error, stranded: aside }; };
/// ```
async fn put_back_after_a_failed_read_awaiting(
    location: &Path,
    aside: PathBuf,
    error: RecoveryError,
) -> TakeFailure {
    // The putting back is a rename, so a job of the blocking pool makes it.
    return match rename_noreplace_on_the_blocking_pool(aside.as_path(), location).await {
        TheAnswerOfARename::Renamed => TakeFailure {
            error,
            stranded: None,
        },
        TheAnswerOfARename::Failed(back) => TakeFailure {
            error: RecoveryError(format!(
                "{} could not be read for the merge ({}) and could not be put back either ({back}); it is at {} and nothing was \
                 replaced by the merge.",
                location.display(),
                error.0,
                aside.display()
            )),
            stranded: Some(aside),
        },
        TheAnswerOfARename::NameTaken(back) => TakeFailure {
            error: RecoveryError(format!(
                "{} could not be read for the merge ({}) and could not be put back either ({back}); it is at {} and nothing was \
                 replaced by the merge.",
                location.display(),
                error.0,
                aside.display()
            )),
            stranded: Some(aside),
        },
        TheAnswerOfARename::NoThreadToRunIt(refused) => TakeFailure {
            error: RecoveryError(format!(
                "{} could not be read for the merge ({}) and could not be put back either ({refused}); it is at {} and nothing \
                 was replaced by the merge.",
                location.display(),
                error.0,
                aside.display()
            )),
            stranded: Some(aside),
        },
    };
}
