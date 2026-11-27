//! What:
//!  The take of the entry one planned worktree write judged:
//!  the rename that moves it aside under a private name,
//!  the check of the identity and the bytes the take caught,
//!  the putting back of an entry that was not the one that was judged,
//!  and the removal of a take nothing can write into any more.
//! Why:
//!   One rename that never replaces takes the entry,
//!  so nothing standing at the path can be destroyed by it,
//!  and the check then runs on a name nobody else writes;
//!  the identity alone cannot see a write through a descriptor the file already had,
//!  so the bytes of the entry that was taken aside are read and hashed
//!  (`merge_isolation_take_asides_proof.rs`).
//!  A kill between the take and the placing leaves the judged entry under a private name that names its
//!  path (`merge_isolation_take_aside_names.rs`),
//! which the next recovery puts back.
//!  The judgement this take checks is `merge_isolation_carry_place_judgement.rs`,
//!  and the placement that calls the take is `merge_isolation_carry_place.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = await takeJudged({ context, root, path, location, judged }); // renameNoReplace, then check, then keep or put back
//! ```

/// One file call of this step handed to the blocking pool,
/// so a disk that makes it wait holds no worker of the runtime
/// (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown").
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The judgement the take checks,
/// and the answer a putting back gives.
use crate::merge_isolation_carry_place_judgement::{
    JudgedFile,
    Placed,
};
/// Why a carry stopped.
use crate::merge_isolation_carry_write_failures::CarryFailure;
/// The identity of an entry,
/// which tells the entry that was judged from one written after it.
use crate::merge_isolation_changes::{
    FileIdentity,
    file_identity,
};
/// The private name a take gives a file it moves aside.
use crate::merge_isolation_take_aside_names::aside_name;
/// The take of a worktree file a merge must replace or delete.
use crate::merge_isolation_take_asides::may_remove_the_aside_awaiting;
/// The bytes of a file as Git's object store reads them.
use crate::merge_isolation_take_asides_proof::are_the_file_form_of_awaiting;
/// One rename that never replaces,
/// and what it answered.
use crate::rename_that_never_replaces::{
    TheAnswerOfARename,
    rename_noreplace_on_the_blocking_pool,
};
/// The Git context a read and a write take.
use crate::transaction_git::GitContext;
/// A place in the worktree,
/// and the private name beside it.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The object the bytes of one entry make,
/// compared with the object the judgement recorded.
/// Why:
///   The identity alone cannot see a write through a descriptor the file already had,
/// so the bytes of the
///  entry that was taken aside are read and hashed with the path's own attributes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function areTheJudgedBytes(context, root, path, location, judged): Promise<boolean | undefined>;
/// ```
async fn are_the_judged_bytes_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    location: &Path,
    judged: &JudgedFile,
) -> Option<bool> {
    let Some(expected) = judged.content.as_deref() else {
        return Some(false);
    };
    return are_the_file_form_of_awaiting(context, root, path, location, expected).await;
}

/// What:
///  Whether the entry at `location` is the one the write judged.
/// Why:
///   An entry that differs is the person's,
///  written after the write looked,
/// so it goes back where it came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function isTheJudgedEntry(context, root, path, location, judged): Promise<boolean | undefined>;
/// ```
async fn is_the_judged_entry_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    location: &Path,
    judged: &JudgedFile,
) -> Option<bool> {
    if judged.identity.is_none() {
        return Some(false);
    }
    // The identity is a metadata look at a user's file, which can wait on a disk, so a job of the blocking pool makes it.
    let looked_at: PathBuf = location.to_path_buf();
    let identity: Option<FileIdentity> =
        match run_blocking_job_within_the_budget(move || return file_identity(looked_at.as_path()))
            .await
        {
            Ok(found) => found,
            // A look nothing made proves nothing about the entry, so the take puts it back where it was found.
            Err(refused) => {
                crate::diagnostic_log::debug(
                    "isTheJudgedEntry",
                    format!(
                        "{} could not be looked at, so the entry goes back: {refused}",
                        location.display()
                    )
                    .as_str(),
                );
                return None;
            }
        };
    if identity != judged.identity {
        return Some(false);
    }
    return are_the_judged_bytes_awaiting(context, root, path, location, judged).await;
}

/// What:
///  Put an entry that turned out not to be the one that was judged back where it was found,
///  and say that the path was left alone.
/// Why:
///   The bytes are the person's,
/// and a path freed by the take is the one place they belong;
/// a path another
///  write already took keeps both entries,
/// and the one that was put aside is named rather than dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = async (location, aside) => { const moved = await renameNoReplaceOnThePool(aside, location); return moved.kind === 'name-taken' ? { kind: 'changed', kept: aside } : { kind: 'changed' }; };
/// ```
async fn put_back_awaiting(
    location: &Path,
    aside: &Path,
) -> Result<Placed, CarryFailure> {
    return match rename_noreplace_on_the_blocking_pool(aside, location).await {
        TheAnswerOfARename::Renamed => Ok(Placed::Changed {
            kept: None,
        }),
        // Another write took the path, so both entries stay and the one put aside is named.
        TheAnswerOfARename::NameTaken(_taken) => Ok(Placed::Changed {
            kept: Some(aside.to_path_buf()),
        }),
        TheAnswerOfARename::Failed(error) => Err(CarryFailure::Failed(format!(
            "{} could not be put back: {error}",
            aside.display()
        ))),
        TheAnswerOfARename::NoThreadToRunIt(refused) => Err(CarryFailure::Failed(format!(
            "{} could not be put back: {refused}",
            aside.display()
        ))),
    };
}

/// What:
///  Whether the entry a take put aside may lose its private name,
///  and the private name that stays when it may not.
/// Why:
///   The bytes the take holds are the merge's own,
///  which the capture keeps,
///  but a program that already had the file open
///  can still write into it after the take checked it,
///  so the private name goes only on the proof nothing can write any
///  more,
///  and a file the proof does not cover is kept and named
///  (`merge_isolation_take_asides.rs`).
///  A take that holds no identity cannot prove anything about the file,
/// so its private name stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropTheAside({ context, root, path, aside, judged, kept }): Promise<void>; // proof ? unlink : kept.push(aside)
/// ```
pub(super) async fn drop_the_aside_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    aside: &Path,
    judged: &JudgedFile,
    kept: &mut Vec<PathBuf>,
) -> Result<(), CarryFailure> {
    let (Some(taken), Some(expected)) = (judged.identity, judged.content.as_deref()) else {
        kept.push(aside.to_path_buf());
        return Ok(());
    };
    let Some(proof) =
        may_remove_the_aside_awaiting(context, root, path, aside, taken, expected).await
    else {
        kept.push(aside.to_path_buf());
        return Ok(());
    };
    return match proof.remove_on_the_blocking_pool(aside).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CarryFailure::Failed(format!(
            "{} could not be removed: {error}",
            aside.display()
        ))),
    };
}

/// What a take of one worktree entry found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Taken = { kind: 'aside'; aside: string } | { kind: 'changed'; kept?: string };
/// ```
pub(super) enum Taken {
    /// The entry that was judged waits at this private name.
    Aside(PathBuf),
    /// The path holds bytes the write did not judge,
    /// and nothing was taken.
    Changed {
        /// Where an entry of the person's waits,
        /// when it could not go back.
        kept: Option<PathBuf>,
    },
}

/// What:
///  Take the entry at `location` aside under a private name,
///  and check that it is the one the write judged.
/// Why:
///   The take is one rename that never replaces,
/// so nothing standing at the path can be destroyed by it;
///  the check then runs on a name nobody else writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function takeJudged({ context, root, path, location, judged }): Promise<Taken>;
/// ```
pub(super) async fn take_judged_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    location: &Path,
    judged: &JudgedFile,
    transaction_id: &str,
) -> Result<Taken, CarryFailure> {
    let aside: PathBuf = aside_name(location, transaction_id);
    // The take is one rename, and a rename writes the directory it lands in, so a job of the blocking pool makes it.
    match rename_noreplace_on_the_blocking_pool(location, aside.as_path()).await {
        TheAnswerOfARename::Renamed => {}
        // The path was freed between the judgement and the take, which is not this write to make.
        TheAnswerOfARename::Failed(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Taken::Changed {
                kept: None,
            });
        }
        // A private name another entry holds is a rename that failed, and both names stay: the path and that entry.
        TheAnswerOfARename::Failed(error) | TheAnswerOfARename::NameTaken(error) => {
            return Err(CarryFailure::Failed(format!(
                "{} could not be set aside: {error}",
                location.display()
            )));
        }
        TheAnswerOfARename::NoThreadToRunIt(refused) => {
            return Err(CarryFailure::Failed(format!(
                "{} could not be set aside: {refused}",
                location.display()
            )));
        }
    }
    match is_the_judged_entry_awaiting(context, root, path, aside.as_path(), judged).await {
        Some(true) => return Ok(Taken::Aside(aside)),
        // A read that fails cannot prove the bytes, so the entry goes back rather than be replaced.
        Some(false) | None => {
            return match put_back_awaiting(location, aside.as_path()).await? {
                Placed::Changed {
                    kept,
                } => Ok(Taken::Changed {
                    kept,
                }),
                Placed::Written | Placed::Nothing => Ok(Taken::Changed {
                    kept: None,
                }),
            };
        }
    }
}
