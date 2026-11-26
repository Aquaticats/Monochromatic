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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleKeptCopy({ ..., aside, change, capture, live, stderr }); await sweepKeptCopiesAwaiting(place, stderr);
//! ```

/// The one job of the blocking pool that makes a step's file calls,
/// so a disk that makes them wait holds no worker of the runtime
/// (`doc/planning/git-policy-cli-full-tokio.md`,
/// "Blocking files,
/// fsync and the runtime's own shutdown").
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The person's own work a merge captured,
/// which holds the bytes of a path the take of a carry moved aside.
use crate::merge_isolation_carry_parts::Capture;
/// Reading the bytes of a file through Git.
use crate::merge_isolation_changes::file_identity;
/// The change one path of a merge makes,
/// whose two commits hold the bytes a copy may carry.
use crate::merge_isolation_changes::{
    Change,
    worktree_path,
};
/// The records still there,
/// and whether the merge each names still runs.
use crate::merge_isolation_journal::{
    list_merge_intents,
    parse_merge_intent,
};
/// The private names a take leaves,
/// which say whether an unfinished merge names one.
use crate::merge_isolation_kept_copies::{
    KeptCopy,
    forget_the_copy,
    keep_the_copy,
    kept_copies,
};
/// The lines a copy earns,
/// which say what stands at its path and what ends it.
use crate::merge_isolation_kept_copies_words::{
    say_the_copy_stays,
    say_the_copy_stays_because,
};
/// The place a command's recovery runs in.
use crate::merge_isolation_recovery_steps::RecoveryPlace;
use crate::merge_isolation_take_aside_names::{
    Aside,
    asides_of,
    live_merge_keys,
    merge_key_of_a_record,
    stale_carry_names_of,
};
use crate::merge_isolation_take_asides::{
    CopyRemoval,
    ProofNothingCanWrite,
    may_remove_the_copy_awaiting,
    raw_bytes_of_awaiting,
    the_file_form_of_awaiting,
};
/// The step that puts one name back where it was found,
/// as a job of the blocking pool.
use crate::rename_that_never_replaces::{
    TheAnswerOfARename,
    rename_noreplace_on_the_blocking_pool,
};
/// Real Git.
use crate::transaction_git::GitContext;
use crate::transaction_owner::OwnerLiveness;
/// Writing the lines a person reads.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// A list of the live merges' keys shared with each path's job, so no path's job copies it.
use std::sync::Arc;

/// What:
///  Whether an unfinished merge names one copy.
/// Why:
///   The name of a copy carries the key of the merge that took it,
/// and only the merge whose record is still there may
///  put the copy back:
/// a record that is gone names a merge that ended,
/// and putting its copy back would
///  bring back a file the person deleted
/// (a cross-vendor review of the second round,
/// its third finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = (aside, live) => aside.merge !== undefined && live.includes(aside.merge);
/// ```
fn an_unfinished_merge_names_it(
    aside: &Aside,
    live: &[String],
) -> bool {
    let Some(named) = aside.merge.as_ref() else {
        return false;
    };
    for key in live {
        if key == named {
            return true;
        }
    }
    return false;
}

/// Whether the copy beside a path is one an unfinished merge names,
/// for a path whose copy was found by its record and not by its name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namedOf = (aside: Aside | undefined, live: string[]) => aside !== undefined && anUnfinishedMergeNamesIt(aside, live);
/// ```
fn an_unfinished_merge_names_it_of(
    aside: Option<&Aside>,
    live: &[String],
) -> bool {
    return match aside {
        Some(found) => an_unfinished_merge_names_it(found, live),
        None => false,
    };
}

/// Whether a copy's name carries no key of a merge at all,
/// which is a name an earlier release wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const keyless = (aside: Aside | undefined) => aside !== undefined && aside.merge === undefined;
/// ```
fn the_name_carries_no_merge_key(aside: Option<&Aside>) -> bool {
    return match aside {
        Some(found) => found.merge.is_none(),
        None => false,
    };
}

/// What:
///  Where the bytes one copy holds are also kept,
///  when the merge itself holds them.
/// Why:
///   A copy of the branch's own file can go:
/// `git restore --source=<the branch's commit>` gives those bytes back,
/// and that commit is the one the branch grew from,
///  not a merge commit only a record keeps.
/// A copy of the merge's own file cannot:
/// the merge's commit is kept by
///  the record and by a side ref,
/// so its bytes are reachable only while the merge is unfinished.
/// A copy of
///  the person's captured work cannot either:
/// the capture is a ref a finished merge drops,
/// and its bytes are the
///  ones the carry is about to write into the file anyway.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Held = { kind: 'branch'; oid: string } | { kind: 'merge' } | { kind: 'capture' };
/// ```
enum Held {
    /// The commit the branch held before the merge does.
    BranchCommit(String),
    /// The merge's own commit does.
    MergeCommit,
    /// The merge's capture does.
    Capture,
}

/// What:
///  The place the merge itself holds the bytes of one path,
///  when the copy beside that path carries exactly those bytes.
/// Why:
///   An earlier release's copy names no merge at all,
/// so no record can say whether an unfinished merge took it;
///  the bytes can.
/// A copy holding the branch's file,
/// the merge's file,
/// or the person's own captured
///  file is the file the merge judged at that path,
/// so it belongs at the path,
///  and the merge judges it there as it did.
///  A copy holding anything else is a person's newer bytes that nothing else
///  holds,
/// and nothing puts those back on a guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function theBytesTheMergeHolds(context, root, change, capture, aside): Promise<string | undefined>;
/// ```
async fn the_bytes_the_merge_holds_awaiting(
    context: &GitContext,
    root: &Path,
    change: &Change,
    capture: Option<&Capture>,
    aside: &Path,
) -> Option<Held> {
    let held: String = raw_bytes_of_awaiting(context, root, aside).await?;
    if let Some(captured) = capture {
        for entry in &captured.paths {
            if entry.path != change.path {
                continue;
            }
            if let Some(file) = entry.worktree.as_ref()
                && file.raw == held
            {
                return Some(Held::Capture);
            }
        }
    }
    // The file form is what the worktree would hold for one of the merge's commits, so a copy of either side's file is the
    //  file the merge judged.
    if the_file_form_of_awaiting(
        context,
        root,
        change.path.as_slice(),
        change.old_oid.as_str(),
    )
    .await
    .as_deref()
        == Some(held.as_str())
    {
        return Some(Held::BranchCommit(String::from(change.old_oid.as_str())));
    }
    if the_file_form_of_awaiting(
        context,
        root,
        change.path.as_slice(),
        change.new_oid.as_str(),
    )
    .await
    .as_deref()
        == Some(held.as_str())
    {
        return Some(Held::MergeCommit);
    }
    return None;
}

/// What:
///  Whether the file at `location` holds exactly the bytes of the copy.
/// Why:
///   A copy whose bytes the person's file holds again is the second one,
/// and removing it loses nothing.
///  The two entries are compared by kind first,
/// because Git opens the file a link points at instead of
///  hashing the link's own target,
/// which would call two unlike entries equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function theFileHoldsThem(context, root, location, aside): Promise<boolean>;
/// ```
async fn the_file_holds_them_awaiting(
    context: &GitContext,
    root: &Path,
    location: &Path,
    aside: &Path,
) -> bool {
    // The two looks are file calls that can wait on a disk, so one job of the blocking pool makes both.
    let (path_at, copy_at): (PathBuf, PathBuf) = (location.to_path_buf(), aside.to_path_buf());
    let looked = run_blocking_job_within_the_budget(move || {
        let Ok(here) = std::fs::symlink_metadata(path_at.as_path()) else {
            return None;
        };
        let Ok(there) = std::fs::symlink_metadata(copy_at.as_path()) else {
            return None;
        };
        return Some((here, there));
    })
    .await;
    // An entry that is not there, and a look the pool never made, both leave the bytes unproven, so the copy is kept.
    let Ok(Some((here, there))) = looked else {
        return false;
    };
    if here.file_type().is_symlink() != there.file_type().is_symlink() {
        return false;
    }
    let Some(held) = raw_bytes_of_awaiting(context, root, aside).await else {
        return false;
    };
    return raw_bytes_of_awaiting(context, root, location)
        .await
        .as_deref()
        == Some(held.as_str());
}

/// What:
///  Whether the two entries carry the same permissions,
/// so the file at the path is the copy in everything a copy holds.
/// Why:
///   One run of bytes is not one file:
/// a copy that is executable where its path is not is the only executable
///  copy of those bytes,
/// and removing it loses the person's mode
/// (`SPEC.md`,
/// the copy whose bytes the file at its path holds again).
///  A symbolic link's permissions are the platform's and say nothing about the link,
/// so two links holding one target are one entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thePermissionsAreTheCopies = (location, aside) => isLink(location) ? isLink(aside) : mode(location) === mode(aside);
/// ```
fn the_permissions_are_the_copies(
    location: &Path,
    aside: &Path,
) -> bool {
    let Ok(here) = std::fs::symlink_metadata(location) else {
        return false;
    };
    let Ok(there) = std::fs::symlink_metadata(aside) else {
        return false;
    };
    if here.file_type().is_symlink() && there.file_type().is_symlink() {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return here.permissions().mode() & PERMISSION_BITS
            == there.permissions().mode() & PERMISSION_BITS;
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (here, there);
        return true;
    }
}

/// The permission bits a copy carries,
/// which are every bit a person can see and set on an entry:
/// the read,
/// write and execute bits of each of the owner,
/// the group and everyone,
/// and the setuid,
/// setgid and sticky bits above them.
/// A copy that carries setuid where its path does not is the only entry with that bit,
/// and Git records none of these but the execute bits,
/// so removing it would lose a mode the person set (a Kimi K3.1 Pro review of `1a98764d1f`,
/// its sixth finding).
#[cfg(unix)]
const PERMISSION_BITS: u32 = 0o7777;

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
async fn settle_kept_copy_awaiting<W: Write + ?Sized>(
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
                            CopyRemoval::HeldOpen => {
                                Fate::KeptBecause("a program still holds the copy open")
                            }
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
        Fate::Kept | Fate::KeptBecause(_) => {
            // The copy is recorded, so the next command that runs the recovery decides it again even though no later merge
            //  touches its path.
            keep_the_copy(git_dir, copy.location.as_path(), copy.aside.as_path());
            match fate {
                Fate::KeptBecause(why) => say_the_copy_stays_because(out, copy, why),
                _ => say_the_copy_stays(out, copy, named),
            }
        }
    }
    return false;
}

/// What:
///  Put one copy back where it belongs,
///  and say so.
/// Why:
///   The copy is the file the merge judged at that path,
/// so the path holds it again and the merge goes on
///  from the checkout it expected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = async (aside, location) => { const moved = await renameNoReplaceOnThePool(aside, location); moved.kind === 'renamed' ? forget(gitDir, aside) : sayTheCopyStays(...); };
/// ```
async fn put_the_copy_back_awaiting<W: Write + ?Sized>(
    git_dir: &Path,
    copy: &KeptCopy,
    named: bool,
    out: &mut W,
) -> bool {
    match rename_noreplace_on_the_blocking_pool(copy.aside.as_path(), copy.location.as_path()).await
    {
        TheAnswerOfARename::Renamed => {
            forget_the_copy(git_dir, copy.aside.as_path());
            let _said = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: put back {}, which a command that stopped while finishing a merge had set aside; its \
                     bytes are the ones it had.\n",
                    copy.location.display()
                )
                .as_bytes(),
            );
            return true;
        }
        // The path was taken between the reading and the rename, so both stay and the copy is named; a rename the pool did not
        //  make leaves both as they are, and the copy is named the same way.
        TheAnswerOfARename::NameTaken(_)
        | TheAnswerOfARename::Failed(_)
        | TheAnswerOfARename::NoThreadToRunIt(_) => {
            say_the_copy_stays(out, copy, named);
            return false;
        }
    }
}

/// What:
///  Remove one copy whose bytes the person's file holds again,
///  and say so.
/// Why:
///   Two files holding one run of bytes are one too many,
/// and the copy beside the path is the one to go:
///  the person's file is where they look for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm(aside); forget(gitDir, aside); say(`${aside} was removed: ${location} holds those same bytes`);
/// ```
async fn remove_the_copy_because_the_file_holds_it_awaiting<W: Write + ?Sized>(
    git_dir: &Path,
    copy: &KeptCopy,
    proof: ProofNothingCanWrite,
    out: &mut W,
) {
    match proof
        .remove_on_the_blocking_pool(copy.aside.as_path())
        .await
    {
        Ok(()) => {
            forget_the_copy(git_dir, copy.aside.as_path());
            let _said = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: removed {}, a copy of {} that a merge set aside: {} holds those same bytes again, so \
                     the copy was the second one and nothing of yours was lost.\n",
                    copy.aside.display(),
                    copy.location.display(),
                    copy.location.display()
                )
                .as_bytes(),
            );
        }
        Err(error) => {
            let _said = out.write_all(
                format!("monochromatic-git-policy-cli: {} could not be removed ({error}); it still holds a copy of {}.\n", copy.aside.display(), copy.location.display())
                    .as_bytes(),
            );
        }
    }
}

/// What:
///  Remove one copy whose bytes one of the merge's commits holds,
///  and say where they are.
/// Why:
///   The bytes are in the repository,
/// so the copy beside the path is the second one,
/// and the line names the
///  commit and the command that puts those bytes back in the checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rm(aside); forget(gitDir, aside); say(`${aside} was removed: ${oid} holds its bytes`);
/// ```
async fn remove_the_copy_because_git_holds_it_awaiting<W: Write + ?Sized>(
    git_dir: &Path,
    copy: &KeptCopy,
    object: &str,
    out: &mut W,
) {
    // The removal is a file call that can wait on a disk, so a job of the blocking pool makes it.
    let copy_at: PathBuf = copy.aside.clone();
    let removed: std::io::Result<()> = match run_blocking_job_within_the_budget(move || {
        return std::fs::remove_file(copy_at.as_path());
    })
    .await
    {
        Ok(outcome) => outcome,
        // The system gave the pool no thread, so the name may still be there and the copy is said to stay as it is.
        Err(refused) => Err(std::io::Error::other(refused.to_string())),
    };
    match removed {
        Ok(()) => {
            forget_the_copy(git_dir, copy.aside.as_path());
            let _said = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: removed {}, a copy of {} that a merge set aside: {} holds other bytes, and {} holds \
                     the bytes of the copy, so nothing of yours was lost; `git restore --no-enforce-worktree --source={} -- {}` puts \
                     them in the checkout if you want them.\n",
                    copy.aside.display(),
                    copy.location.display(),
                    copy.location.display(),
                    object,
                    object,
                    copy.location.display()
                )
                .as_bytes(),
            );
        }
        Err(error) => {
            let _said = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: {} could not be removed ({error}); it still holds a copy of {}, whose bytes {} \
                     holds.\n",
                    copy.aside.display(),
                    copy.location.display(),
                    object
                )
                .as_bytes(),
            );
        }
    }
}

/// What:
///  Decide every copy a merge kept in this worktree,
///  for a command that runs the recovery,
///  and leave alone every copy a merge that still runs owns.
/// Why:
///   A copy is left beside a path one merge touched,
/// and a later merge that touches another path says nothing
///  about it,
/// so without this step the copy is a file no command ends
///  (`merge-carry-review-round-2-findings-1`,
/// its first finding).
///  The copies of a merge whose process still runs are not this command's to decide:
/// that merge kept them because
///  a path it is still writing was in the way,
/// and putting one back under it makes Git refuse the checkout the merge was about to
///  make,
/// so a command that has nothing to do with that merge would fail it (the owner's "Agents should never
///  wait on each other on commits or merges",
/// and `SPEC.md`,
/// the record of a merge whose owner still runs).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await sweepKeptCopiesAwaiting(place, runningKeys, stderr);
/// ```
pub(crate) async fn sweep_kept_copies_awaiting<W: Write + ?Sized>(
    place: &RecoveryPlace,
    running: &[String],
    out: &mut W,
) {
    // The copies this worktree's records name and the keys of the merges that are live are read from a folder of the Git
    //  directory, so one job of the blocking pool makes both.
    let records_of: PathBuf = place.git_dir.clone();
    let listed = run_blocking_job_within_the_budget(move || {
        let copies: Vec<KeptCopy> = kept_copies(records_of.as_path());
        if copies.is_empty() {
            return (copies, Vec::new());
        }
        return (copies, live_merge_keys(records_of.as_path()));
    })
    .await;
    let (copies, live): (Vec<KeptCopy>, Vec<String>) = match listed {
        Ok(found) => found,
        // A records folder the pool never read names no copy this command could decide, so it decides none and the next command
        //  meets them.
        Err(refused) => {
            crate::diagnostic_log::debug(
                "sweepKeptCopies",
                format!(
                    "the copies kept beside this worktree's paths could not be read: {refused}"
                )
                .as_str(),
            );
            return;
        }
    };
    for copy in copies {
        // The copy's own name says which merge took it, and the record cannot: a name is the only thing that survives a release
        //  that changed the record's shape.
        let names_beside: PathBuf = copy.location.clone();
        let asides: Vec<Aside> = match run_blocking_job_within_the_budget(move || {
            return asides_of(names_beside.as_path());
        })
        .await
        {
            Ok(found) => found,
            // A folder the pool never read says nothing about this path's copies, so they wait for the next command.
            Err(_refused) => continue,
        };
        let mut named: Option<Aside> = None;
        for aside in asides {
            if aside.path == copy.aside {
                named = Some(aside);
            }
        }
        // A merge that still runs decides its own copies when it finishes them; a command beside it only reads.
        if an_unfinished_merge_names_it_of(named.as_ref(), running) {
            continue;
        }
        // A merge that wrote its record after the caller read the owners is not in `running`, and its copies are as much its
        //  own as a running merge's: putting one back under the path it cleared makes its own checkout fail. The names are read
        //  again here, beside the copy they decide, so a merge that started in between keeps what it took.
        if named.is_some() {
            let records_again: PathBuf = place.git_dir.clone();
            let owners_that_run: Vec<String> = match run_blocking_job_within_the_budget(move || {
                return merges_whose_owner_runs(records_again.as_path());
            })
            .await
            {
                Ok(found) => found,
                // A reading the pool never made proves nothing about this copy's merge, so the copy is left for that merge.
                Err(_refused) => continue,
            };
            if an_unfinished_merge_names_it_of(named.as_ref(), owners_that_run.as_slice()) {
                continue;
            }
        }
        settle_kept_copy_awaiting(
            &place.git,
            place.cwd.as_path(),
            place.git_dir.as_path(),
            &copy,
            named.as_ref(),
            None,
            None,
            live.as_slice(),
            out,
        )
        .await;
    }
    return;
}

/// What:
///  The keys of the merges whose records are there at this instant and whose owner still runs.
/// Why:
///  A copy names the merge that took it,
/// and only that merge may decide it while it runs:
/// a command beside
///  a live merge that put the copy back under the path the merge cleared would make its own checkout fail
/// (`SPEC.md`,
///  "a copy whose merge still runs is left for that merge to finish").
/// The list a caller built is a reading of the
///  records,
/// and a merge that wrote its record after it is not in it,
/// so the copy's own name is looked up against a reading taken here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mergesWhoseOwnerRuns(gitDir: string): string[]; // listMergeIntents().filter(isAlive).map(nameKey)
/// ```
fn merges_whose_owner_runs(git_dir: &Path) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    let Ok(records) = list_merge_intents(git_dir) else {
        return keys;
    };
    for record in records {
        let Ok(bytes) = std::fs::read(record.as_path()) else {
            continue;
        };
        let Some(intent) = parse_merge_intent(bytes.as_slice()) else {
            continue;
        };
        let liveness =
            crate::merge_isolation_recovery_records::owner_liveness(&intent, record.as_path());
        if matches!(liveness, Ok(OwnerLiveness::Alive)) {
            keys.push(merge_key_of_a_record(record.as_path()));
        }
    }
    return keys;
}

/// What:
///  Decide every copy left beside one path of a merge that is running,
///  with the change and the capture that say which bytes are the merge's own.
/// Why:
///   The merge's own record and capture are the only evidence that says
/// whether a copy an earlier release left is the file this merge judged at that
///  path,
/// and the path must hold that file before the paths are judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const change of changes) for (const aside of asidesOf(worktreePath(root, change.path))) await settleKeptCopy({ ..., aside, change, capture, live, stderr });
/// ```
pub(crate) async fn settle_the_asides_of_a_merge_awaiting<W: Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: Option<&Capture>,
    transaction_id: &str,
    out: &mut W,
) -> Vec<PathBuf> {
    // The keys of the live merges are read from a folder of the Git directory, so a job of the blocking pool reads them.
    let records_of: PathBuf = git_dir.to_path_buf();
    let live: Arc<Vec<String>> = match run_blocking_job_within_the_budget(move || {
        return live_merge_keys(records_of.as_path());
    })
    .await
    {
        Ok(found) => Arc::new(found),
        // A folder the pool never read names no merge whose copies are its own, so the caller is answered none and the next
        //  command decides them.
        Err(refused) => {
            crate::diagnostic_log::debug(
                "settleTheAsidesOfAMerge",
                format!("the merges whose copies live beside this worktree's paths could not be read: {refused}").as_str(),
            );
            return Vec::new();
        }
    };
    // The paths a copy went back to, which now hold the file the merge judged there: a rename leaves that file's identity as it
    //  was and changes only its change time, so a caller that judges those paths against an older reading must read them again.
    let mut put_back: Vec<PathBuf> = Vec::new();
    for change in changes {
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        // The bytes a killed carry had written under their own name are a temporary of the placing, made again by the next
        //  command and held in the object store either way, so a name nothing is waiting on goes before the copy beside it is
        //  decided: otherwise a finished merge leaves an untracked file in the worktree for good (`SPEC.md`, no half-applied
        //  result). Reading those names, removing them and reading the asides are file calls that can wait on a disk, so one job
        //  of the blocking pool makes all three.
        let (path_beside, keys, finished): (PathBuf, Arc<Vec<String>>, String) = (
            location.clone(),
            Arc::clone(&live),
            String::from(transaction_id),
        );
        let looked = run_blocking_job_within_the_budget(move || {
            for stale in
                stale_carry_names_of(path_beside.as_path(), keys.as_slice(), finished.as_str())
            {
                let _removed = std::fs::remove_file(stale.as_path());
            }
            return asides_of(path_beside.as_path());
        })
        .await;
        let Ok(asides) = looked else {
            // A path the pool never read says nothing about the copies beside it, so they wait for the next command.
            continue;
        };
        for aside in asides {
            let copy: KeptCopy = KeptCopy {
                location: location.clone(),
                aside: aside.path.clone(),
            };
            if settle_kept_copy_awaiting(
                context,
                root,
                git_dir,
                &copy,
                Some(&aside),
                Some(change),
                capture,
                live.as_slice(),
                out,
            )
            .await
            {
                put_back.push(location.clone());
            }
        }
    }
    return put_back;
}
