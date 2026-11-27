//! What:
//!  The evidence one decision about a kept copy rests on:
//!  which merge's record names the copy,
//!  where else its bytes are kept,
//!  and whether the file at its path holds them again in the same form.
//! Why:
//!   A copy may be put back only for the merge that took it,
//! and removed only when its bytes are somewhere the person can reach them:
//!  the branch's own commit,
//! the merge's own commit,
//! the capture the merge keeps their work in,
//!  or the file at the path,
//! holding the same bytes and the same permissions.
//!  One run of bytes is not one file,
//! so the permissions are compared too,
//! and a symbolic link is compared as a link.
//!  The checks are a file of their own so the decision that reads them,
//! and the words a person reads about one,
//! stay inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const named = anUnfinishedMergeNamesIt(aside, live); const held = await theBytesTheMergeHolds(event);
//! const same = await theFileHoldsThem(event) && thePermissionsAreTheCopies(location, aside);
//! ```

/// The one job of the blocking pool that makes the step's file calls,
/// so a disk that makes them wait holds no worker of the runtime.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// The person's own work a merge captured,
/// which holds the bytes of a path the take of a carry moved aside.
use crate::merge_isolation_carry_parts::Capture;
/// The change one path of a merge makes,
/// whose two commits hold the bytes a copy may carry.
use crate::merge_isolation_changes::Change;
/// The private names a take leaves,
/// which say whether an unfinished merge names one.
use crate::merge_isolation_take_aside_names::Aside;
/// The bytes a copy holds,
/// and the file a path would hold for one of a merge's commits.
use crate::merge_isolation_take_asides_proof::{
    raw_bytes_of_awaiting,
    the_file_form_of_awaiting,
};
/// Real Git.
use crate::transaction_git::GitContext;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

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
pub(crate) fn an_unfinished_merge_names_it(
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
pub(crate) fn an_unfinished_merge_names_it_of(
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
pub(crate) fn the_name_carries_no_merge_key(aside: Option<&Aside>) -> bool {
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
pub(crate) enum Held {
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
pub(crate) async fn the_bytes_the_merge_holds_awaiting(
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
pub(crate) async fn the_file_holds_them_awaiting(
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
pub(crate) fn the_permissions_are_the_copies(
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
