//! What:
//!  The one decision every command makes about a copy a merge kept beside a path:
//!  put it back,
//!  remove it because its bytes are somewhere the person can reach them,
//!  or keep it and say where it is and what ends it.
//! Why:
//!   A take moves a file aside before a merge writes its own content there,
//! and a command that stops between the two leaves the file beside its path,
//!  so the copy is the only thing that holds those bytes.
//! Nothing of the person's may be removed,
//!  but a copy nothing looks at again is a file they cannot account for,
//!  so every command that runs the merge recovery decides every recorded copy again
//!  (`merge_isolation_kept_copies.rs`):
//! a copy whose path is empty and whose own merge is unfinished goes back,
//! a copy whose bytes the person's file holds again is removed,
//! and a copy whose bytes the branch's own commit holds is removed
//!  because Git still has them.
//!  A copy none of those explains stays,
//! is named again,
//! and ends the moment one of them does.
//!  The bytes decide a copy whose name carries no merge key at all,
//! which is a name an earlier release wrote:
//! a copy of the branch's own file,
//!  the merge's own file,
//! or the person's own captured file is the file the merge judged at that
//!  path,
//! and anything else is the person's alone,
//! so it is never put back on a guess.
//!  The evidence one decision rests on is `merge_isolation_kept_copies_decisions_evidence.rs`,
//! what it does about a copy is `merge_isolation_kept_copies_decisions_outcomes.rs`,
//! and the sweep that makes the same decision over a whole worktree
//!  is `merge_isolation_kept_copies_decisions_sweep.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleKeptCopy({ ..., aside, change, capture, live, stderr }); await sweepKeptCopiesAwaiting(place, stderr);
//! ```

/// The one job of the blocking pool that makes a step's file calls,
/// so a disk that makes them wait holds no worker of the runtime.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The person's own work a merge captured,
/// which holds the bytes of a path the take of a carry moved aside.
use crate::merge_isolation_carry_parts::Capture;
/// The change one path of a merge makes,
/// and the reading of the bytes of a file through Git.
use crate::merge_isolation_changes::{
    Change,
    file_identity,
};
/// The copies a merge kept,
/// and the record of each.
use crate::merge_isolation_kept_copies::{
    KeptCopy,
    forget_the_copy,
    keep_the_copy,
};
/// Whether an unfinished merge names one copy,
/// where else the bytes of a copy are kept,
/// and whether the file at its path holds them again in the same form.
use crate::merge_isolation_kept_copies_decisions_evidence::{
    Held,
    an_unfinished_merge_names_it_of,
    the_bytes_the_merge_holds_awaiting,
    the_file_holds_them_awaiting,
    the_name_carries_no_merge_key,
    the_permissions_are_the_copies,
};
/// What one decision does about a copy,
/// and the line each of the three prints.
use crate::merge_isolation_kept_copies_decisions_outcomes::{
    put_the_copy_back_awaiting,
    remove_the_copy_because_git_holds_it_awaiting,
    remove_the_copy_because_the_file_holds_it_awaiting,
};
/// The lines a copy earns,
/// which say what stands at its path and what ends it.
use crate::merge_isolation_kept_copies_words::{
    say_the_copy_stays,
    say_the_copy_stays_because,
    say_the_copy_stays_held_open,
};
/// The private names a take leaves,
/// which say whether an unfinished merge names one.
use crate::merge_isolation_take_aside_names::Aside;
/// The proof that no program can write into a file whose last name is about to go,
/// and the last look a kept copy's removal is decided by.
use crate::merge_isolation_take_asides_proof::{
    CopyRemoval,
    ProofNothingCanWrite,
    may_remove_the_copy_awaiting,
};
/// Real Git.
use crate::transaction_git::GitContext;
/// Writing the lines a person reads.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What one decision about a kept copy came to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fate = 'put-back' | 'the-file-holds-them' | 'git-holds-them' | 'kept';
/// ```
enum Fate {
    /// The path is free,
    /// and the copy is the file the merge itself holds there.
    PutBack,
    /// The path holds a file whose bytes are the copy's own,
    /// with the proof that nothing can write into the copy held until its name goes.
    TheFileHoldsThem(ProofNothingCanWrite),
    /// The copy's bytes are the ones one of the merge's commits holds for the path,
    /// so Git still has them.
    GitHoldsThem(String),
    /// Nothing proves an end,
    /// so the copy stays and is named.
    Kept,
    /// The bytes at the path are the copy's own,
    /// and something else the copy holds is not at the path,
    /// so the copy stays and is named with the reason.
    KeptBecause(&'static str),
    /// The bytes at the path are the copy's own,
    /// and programs a scan this command made can read hold the copy open,
    /// so the copy stays and is named with each of them.
    HeldOpen(Vec<crate::index_lock_holders::Holder>),
}

/// What:
///  Decide one copy,
///  and carry the decision out:
///  put it back,
///  remove it,
///  or keep it and say where it is.
/// Why:
///   Every command makes this one decision,
/// so a copy ends the same way whichever command meets it,
/// and the line it prints says what is true of the copy now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await settleKeptCopy({ context, root, gitDir, copy, aside, change, capture, live, stderr });
/// ```
pub(crate) async fn settle_kept_copy_awaiting<W: Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    copy: &KeptCopy,
    aside: Option<&Aside>,
    change: Option<&Change>,
    capture: Option<&Capture>,
    live: &[String],
    out: &mut W,
) -> bool {
    let named: bool = an_unfinished_merge_names_it_of(aside, live);
    // The look at the copy and the one at the path it belongs to are file calls that can wait on a disk, so one job of the
    //  blocking pool makes both.
    let (copy_at, path_at): (PathBuf, PathBuf) = (copy.aside.clone(), copy.location.clone());
    let looked = run_blocking_job_within_the_budget(move || {
        return (
            std::fs::symlink_metadata(copy_at.as_path()).is_ok(),
            std::fs::symlink_metadata(path_at.as_path()).is_ok(),
        );
    })
    .await;
    let (stands, path_stands): (bool, bool) = match looked {
        Ok(answers) => answers,
        // A look the pool never made cannot say the copy is gone, so it is kept and named instead of forgotten.
        Err(refused) => {
            crate::diagnostic_log::debug(
                "settleKeptCopy",
                format!(
                    "{} could not be looked at, so it stays where it is: {refused}",
                    copy.aside.display()
                )
                .as_str(),
            );
            say_the_copy_stays(out, copy, named);
            return false;
        }
    };
    if !stands {
        // The copy is gone, so nothing names it any more.
        forget_the_copy(git_dir, copy.aside.as_path());
        return false;
    }
    let free: bool = !path_stands;
    // A copy with no merge key at all is a name an earlier release wrote, so no record can say whether an unfinished merge took
    //  it; its bytes can, and only while a merge is unfinished here: a finished merge's copy is the older file the person may
    //  have deleted on purpose, and the merge that would judge it is gone.
    let keyless: bool = the_name_carries_no_merge_key(aside);
    let held: Option<Held> = match change {
        Some(found) if !live.is_empty() => {
            the_bytes_the_merge_holds_awaiting(context, root, found, capture, copy.aside.as_path())
                .await
        }
        _ => None,
    };
    let fate: Fate = if free && (named || (keyless && held.is_some())) {
        Fate::PutBack
    } else if !free
        && the_file_holds_them_awaiting(
            context,
            root,
            copy.location.as_path(),
            copy.aside.as_path(),
        )
        .await
    {
        // One run of bytes is not one file: the entries differ in their permissions, which Git does not record outside the
        //  executable bit, or a program still holds the copy open, and a write through that descriptor lands in the file an
        //  unlink would leave with no name at all. Both are the person's, so the copy goes only when neither holds.
        // The entry at the path is read before the comparison ends, and the removal asks for it again as its last look: a save
        //  or a chmod that arrives after the comparison leaves the copy the only name of the older bytes or the older mode.
        // The identity of the entry at the path and the permissions of both entries are file looks that can wait on a disk,
        //  so one job of the blocking pool makes them.
        let (the_path, the_copy): (PathBuf, PathBuf) = (copy.location.clone(), copy.aside.clone());
        let compared = run_blocking_job_within_the_budget(move || {
            return (
                file_identity(the_path.as_path()),
                the_permissions_are_the_copies(the_path.as_path(), the_copy.as_path()),
            );
        })
        .await;
        match compared {
            Ok((the_file, same_permissions)) => {
                if !same_permissions {
                    Fate::KeptBecause("the two entries do not have the same permissions")
                } else {
                    match the_file {
                        Some(standing) => match may_remove_the_copy_awaiting(
                            context,
                            root,
                            copy.aside.as_path(),
                            copy.location.as_path(),
                            standing,
                        )
                        .await
                        {
                            CopyRemoval::Proven(proof) => Fate::TheFileHoldsThem(proof),
                            CopyRemoval::HeldBy(holders) => Fate::HeldOpen(holders),
                            CopyRemoval::Unproven => Fate::KeptBecause(
                                "nothing could prove that no program holds the copy open",
                            ),
                            CopyRemoval::Changed => Fate::KeptBecause(
                                "the file at its path or the copy changed after they were compared",
                            ),
                        },
                        None => Fate::KeptBecause(
                            "the file at its path changed after they were compared",
                        ),
                    }
                }
            }
            // A comparison the pool never made proves nothing about the two entries, so the copy stays and is named.
            Err(_refused) => {
                Fate::KeptBecause("this command could not read both entries to compare them")
            }
        }
    } else if let Some(Held::BranchCommit(object)) = held {
        Fate::GitHoldsThem(object)
    } else {
        Fate::Kept
    };
    match fate {
        Fate::PutBack => return put_the_copy_back_awaiting(git_dir, copy, named, out).await,
        Fate::TheFileHoldsThem(proof) => {
            remove_the_copy_because_the_file_holds_it_awaiting(git_dir, copy, proof, out).await
        }
        Fate::GitHoldsThem(object) => {
            remove_the_copy_because_git_holds_it_awaiting(git_dir, copy, object.as_str(), out).await
        }
        Fate::Kept | Fate::KeptBecause(_) | Fate::HeldOpen(_) => {
            // The copy is recorded, so the next command that runs the recovery decides it again even though no later merge
            //  touches its path.
            keep_the_copy(git_dir, copy.location.as_path(), copy.aside.as_path());
            match fate {
                Fate::KeptBecause(why) => say_the_copy_stays_because(out, copy, why),
                Fate::HeldOpen(holders) => {
                    say_the_copy_stays_held_open(out, copy, holders.as_slice())
                }
                _ => say_the_copy_stays(out, copy, named),
            }
        }
    }
    return false;
}
