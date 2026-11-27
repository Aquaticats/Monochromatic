//! What:
//!  Put one planned worktree write in place,
//!  or leave the path alone because what stands there is not the entry that was judged:
//!  the placing of the new entry,
//!  the removal of an entry the write judged,
//!  and keeping the planned bytes beside a path whose bytes the write cannot prove.
//!  The judgement such a write stands on is `merge_isolation_carry_place_judgement.rs`,
//!  the private names it makes are `merge_isolation_carry_place_private_names.rs`,
//!  and the take of the judged entry is `merge_isolation_carry_place_take.rs`.
//! Why:
//!  A judgement and a write are separate steps,
//!  so a person who saves the file between them had their bytes replaced with no
//!  notice;
//! one rename takes the entry atomically,
//!  so the bytes that are checked are exactly the bytes that make way for the write
//!  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
//! finding 4,
//!  which is how the second pass's install already writes a merged file).
//!  The identity is the cheap check and the bytes are the authority:
//! a program that keeps its descriptor
//!  across the rename can write the same length into the file and put the modification time back,
//! which leaves every
//!  part of the identity as it was.
//!  A kill between the take and the placing leaves the judged entry under a private name that names its
//!  path (`merge_isolation_take_asides.rs`),
//! which the next recovery puts back;
//!  the new bytes under this module's own private name are made again by the
//!  next command,
//! so a kill that leaves one loses nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const placed = await placeFile({ context, root, path, judged, write }); // renameNoReplace take, check, place or put back
//! ```

/// One file call of this step handed to the blocking pool,
/// so a disk that makes it wait holds no worker of the runtime
/// (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown").
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The entry such a write makes.
use crate::merge_isolation_carry_place_write::write_entry;
/// Why a carry stopped.
use crate::merge_isolation_carry_write_failures::CarryFailure;
/// What its worktree file becomes.
use crate::merge_isolation_carry_write_forms::FileWrite;
/// The worktree path of one changed path.
use crate::merge_isolation_changes::worktree_path;
/// The private name a take and a kept entry are given.
use crate::merge_isolation_take_aside_names::aside_name;
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

/// The judgement a write stands on and the answer it gives,
/// named here so a caller of the placement keeps one path.
pub(crate) use crate::merge_isolation_carry_place_judgement::{
    JudgedFile,
    Placed,
    explained_entries,
    is_still_the_judged_entry,
    judged_of,
};
/// The private name the new bytes are written under,
/// and the removal of a name the write turned out not to need.
use crate::merge_isolation_carry_place_private_names::{
    remove_a_private_name_awaiting,
    write_private_awaiting,
};
/// The take of the judged entry,
/// what it caught,
/// and the putting back of an entry that was not the one that was judged.
use crate::merge_isolation_carry_place_take::{
    Taken,
    drop_the_aside_awaiting,
    take_judged_awaiting,
};

/// What:
///  The private names one write leaves beside a path,
///  and what ends each of them.
/// Why:
///  A write either lands on the path or keeps what it would have written beside it,
/// and the second is
///  what a path its caller cannot prove holds:
/// a path a save may stand at is left as it is,
/// and the bytes the
///  merge carries stay beside it,
/// recorded with the copies of a merge and named,
/// so they are not a file no
///  command ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SetAsides = { heldOpen: Path[]; unproven: Path[]; keepThePlannedBytes: boolean };
/// ```
pub(crate) struct SetAsides<'a> {
    /// The names a program could still write into,
    /// which stay and are named.
    pub(crate) held_open: &'a mut Vec<PathBuf>,
    /// The names holding bytes this command did not write at the path,
    /// which stay beside it and are named.
    pub(crate) unproven: &'a mut Vec<PathBuf>,
    /// Whether the bytes planned for this path stay beside it instead of taking it.
    pub(crate) keep_the_planned_bytes: bool,
}

/// What:
///  Remove the entry at `location`
///  if it is the one the write judged.
/// Why:
///   A removal is a write of "nothing" and is judged the same way:
/// an entry that appeared after the judgement is the
///  person's and is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeJudged({ context, root, path, location, judged }): Promise<Placed>;
/// ```
async fn remove_judged_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    location: &Path,
    judged: &JudgedFile,
    transaction_id: &str,
    kept: &mut Vec<PathBuf>,
) -> Result<Placed, CarryFailure> {
    if judged.content.is_none() {
        // Nothing stood there when the write looked, so an entry that is there now came after it.
        let looked_at: PathBuf = location.to_path_buf();
        // The look at the entry is a call that can wait on a disk, so a job of the blocking pool makes it.
        let stands: Result<bool, crate::blocking_job::BlockingJobError> =
            run_blocking_job_within_the_budget(move || {
                return match std::fs::symlink_metadata(looked_at.as_path()) {
                    // The path is free, so there is nothing for this write to do.
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                    // An entry stands there, and a look that failed for any other reason is read as one: neither is this write's to toss.
                    _ => true,
                };
            })
            .await;
        return match stands {
            Ok(false) => Ok(Placed::Nothing),
            Ok(true) => Ok(Placed::Changed {
                kept: None,
            }),
            Err(refused) => Err(CarryFailure::Failed(format!(
                "{} could not be looked at: {refused}",
                location.display()
            ))),
        };
    }
    return match take_judged_awaiting(context, root, path, location, judged, transaction_id).await?
    {
        Taken::Aside(aside) => {
            // The entry that was judged is the one this write removes, and its bytes are in the merge's capture.
            drop_the_aside_awaiting(context, root, path, aside.as_path(), judged, kept).await?;
            Ok(Placed::Written)
        }
        Taken::Changed {
            kept: stranded,
        } => Ok(Placed::Changed {
            kept: stranded,
        }),
    };
}

/// What:
///  Write one entry's bytes at its path,
///  or leave the path alone when it holds something the write did not judge.
/// Why:
///   The take is one rename that never replaces,
/// so the bytes that are checked are the bytes that make way for the
///  new entry.
/// A kill between the take and the placing leaves the judged entry under a private name beside the path,
///  which the next command puts back.
///  The entry the take moved aside is removed once the new entry stands there,
/// because its bytes are the merge's own,
///  which the capture keeps;
/// the one exception is a take nothing proves:
/// the file may be a save the person made after a killed
///  merge,
/// so it is kept and named instead of removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function placeBytes({ context, root, path, judged, write, setAsides }): Promise<Placed>;
/// ```
async fn place_bytes_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    judged: &JudgedFile,
    write: &FileWrite,
    transaction_id: &str,
    set_asides: &mut SetAsides<'_>,
) -> Result<Placed, CarryFailure> {
    let FileWrite::Bytes {
        bytes,
        mode,
        the_persons_bits,
    } = write
    else {
        return Err(CarryFailure::Failed(format!(
            "{} was planned as bytes and is not",
            crate::path_bytes::path_message_text(path)
        )));
    };
    let location: PathBuf = worktree_path(root, path);
    if set_asides.keep_the_planned_bytes {
        // Nothing proves the entry standing at the path is the one Git's own update wrote, and its bytes are exactly the merge's
        //  own, which is also what a save made after a killed merge holds. The merge finishes and the file stays as it is: the
        //  bytes this carry would have written are kept beside it under a name that names the merge, and the caller records and
        //  names them, so the person's bytes exist whichever of the two the file was.
        let aside: PathBuf = aside_name(location.as_path(), transaction_id);
        // Keeping the bytes beside the path is a write that can wait on a disk, so a job of the blocking pool makes it.
        let kept_at: PathBuf = aside.clone();
        let kept_bytes: Vec<u8> = bytes.clone();
        let kept_mode: String = String::from(mode.as_str());
        let bits: Option<u32> = *the_persons_bits;
        let kept: Result<(), String> = match run_blocking_job_within_the_budget(move || {
            return write_entry(
                kept_at.as_path(),
                kept_bytes.as_slice(),
                kept_mode.as_str(),
                bits,
            );
        })
        .await
        {
            Ok(written) => written,
            // The system gave the pool no thread, so the bytes were not written beside the path.
            Err(refused) => Err(refused.to_string()),
        };
        return match kept {
            Ok(()) => {
                set_asides.unproven.push(aside);
                Ok(Placed::Nothing)
            }
            Err(error) => Err(CarryFailure::Failed(format!(
                "{} could not be kept beside {}, which was left as it is: {error}",
                location.display(),
                location.display()
            ))),
        };
    }
    let private: PathBuf = match write_private_awaiting(
        location.as_path(),
        bytes,
        mode,
        *the_persons_bits,
        transaction_id,
    )
    .await
    {
        Ok(made) => made,
        Err(error) => {
            return Err(CarryFailure::Failed(format!(
                "{} could not be written: {error}",
                location.display()
            )));
        }
    };
    if judged.content.is_none() {
        // Nothing stood there when the write looked, so the path is made and never replaced.
        return match rename_noreplace_on_the_blocking_pool(private.as_path(), location.as_path())
            .await
        {
            TheAnswerOfARename::Renamed => Ok(Placed::Written),
            TheAnswerOfARename::NameTaken(_taken) => {
                remove_a_private_name_awaiting(private.as_path()).await;
                Ok(Placed::Changed {
                    kept: None,
                })
            }
            TheAnswerOfARename::Failed(error) => Err(CarryFailure::Failed(format!(
                "{} could not be made: {error}",
                location.display()
            ))),
            TheAnswerOfARename::NoThreadToRunIt(refused) => Err(CarryFailure::Failed(format!(
                "{} could not be made: {refused}",
                location.display()
            ))),
        };
    }
    let aside: PathBuf = match take_judged_awaiting(
        context,
        root,
        path,
        location.as_path(),
        judged,
        transaction_id,
    )
    .await?
    {
        Taken::Aside(waits) => waits,
        Taken::Changed {
            kept: stranded,
        } => {
            remove_a_private_name_awaiting(private.as_path()).await;
            return Ok(Placed::Changed {
                kept: stranded,
            });
        }
    };
    let placed: Placed =
        match rename_noreplace_on_the_blocking_pool(private.as_path(), location.as_path()).await {
            TheAnswerOfARename::Renamed => Placed::Written,
            // Another write took the path in the gap; the entry put aside is kept and named, and the bytes this write
            //  would have placed are made again by the next command.
            TheAnswerOfARename::NameTaken(_taken) => Placed::Changed {
                kept: Some(aside.clone()),
            },
            // The entry put aside stays where it is, so the message names it rather than leaving a person to find it.
            TheAnswerOfARename::Failed(error) => {
                return Err(CarryFailure::Failed(format!(
                    "{} could not be written: {error}; the file it held before is at {}",
                    location.display(),
                    aside.display()
                )));
            }
            // The rename did not run, or may have: the entry put aside still names where the file before it is.
            TheAnswerOfARename::NoThreadToRunIt(refused) => {
                return Err(CarryFailure::Failed(format!(
                    "{} could not be written: {refused}; the file it held before is at {}",
                    location.display(),
                    aside.display()
                )));
            }
        };
    if matches!(placed, Placed::Written) {
        // What the take holds is the entry the write replaces, whose bytes the merge's capture keeps.
        drop_the_aside_awaiting(
            context,
            root,
            path,
            aside.as_path(),
            judged,
            set_asides.held_open,
        )
        .await?;
    } else {
        remove_a_private_name_awaiting(private.as_path()).await;
    }
    return Ok(placed);
}

/// What:
///  Write one planned worktree write,
/// which is bytes,
///  a removal,
/// or nothing at all.
/// Why:
///   Every write of a carry,
/// a clear and a putting back goes through this one step,
/// so an entry changed
///  after it was judged is never replaced by any of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function placeFile({ context, root, path, judged, write, setAsides }): Promise<Placed>;
/// ```
pub(crate) async fn place_file_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    judged: &JudgedFile,
    write: &FileWrite,
    transaction_id: &str,
    set_asides: &mut SetAsides<'_>,
) -> Result<Placed, CarryFailure> {
    // The comparisons run from the top of the worktree, where a relative directory option of the invocation does not reach:
    //  a `-C repo` applied to a request that already runs in the worktree would name the worktree twice, and every read of
    //  `--path=<path>` would fail, which reads as a file that nobody judged.
    let rooted: GitContext = GitContext {
        global_prefix: Vec::new(),
        ..context.clone()
    };
    let location: PathBuf = worktree_path(root, path);
    return match write {
        FileWrite::Leave => Ok(Placed::Nothing),
        FileWrite::Remove => {
            remove_judged_awaiting(
                &rooted,
                root,
                path,
                location.as_path(),
                judged,
                transaction_id,
                set_asides.held_open,
            )
            .await
        }
        FileWrite::Bytes {
            ..
        } => {
            place_bytes_awaiting(
                &rooted,
                root,
                path,
                judged,
                write,
                transaction_id,
                set_asides,
            )
            .await
        }
    };
}

/// The controls of the placement stay out of the release executable.
#[cfg(test)]
#[path = "merge_isolation_carry_place_tests.rs"]
mod tests;
