//! What:
//!  The plain facts one add's claim of a snapshot ahead is written from,
//! none of which reads the pool or the commit service:
//! whether a name below the pool folder is a directory,
//! which object a name leads to now,
//! what the wait for a copy came to,
//! whether the commit service gave up this add's request,
//! and the ceiling in the words a person reads.
//! Why:
//!  The claim,
//! the wait and the hand-over are one subject,
//! in `worktree_snapshot_ahead_claim.rs`,
//! which reads the pool and makes the rename;
//! these answers need no pool entry and no service,
//! so they sit beside it and that file stays one thing a reader can hold.
//! The pool they are about is named in `worktree_snapshot_ahead.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Waited = { kind: 'claimed' } | { kind: 'givenUp' } | { kind: 'ceiling' };
//! ```

/// Import the handles and the reads of one name below an open folder.
use crate::fs_beneath::{
    ObjectId,
    entry_metadata_at,
    identity_at,
    open_root,
};
/// Import the work item states that say a request was refused for good.
use crate::work_item::{
    FAILED,
    state_directory,
};
/// Import the proof one copy equal to the source's index carries.
use crate::worktree_snapshot_index_step::Proof;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
/// What:
///  The identity of what a path's name leads to now,
/// or nothing when its parent cannot be opened or the name cannot be looked up.
/// Why:
///   The claimed snapshot's identity is what the add records for a later removal to check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityAt(openRoot(dirname(path)), basename(path)) ?? undefined
/// ```
pub(crate) fn identity_of_name(path: &Path) -> Option<ObjectId> {
    let parent: File = open_root(path.parent().unwrap_or(path)).ok()?;
    return identity_at(&parent, path.file_name()?).ok();
}

/// What:
///  Whether the entry `name` below `folder` is a directory,
/// which is the only thing that can hold a copy of a source.
/// Why:
///   The proof opens the entry as a tree,
/// and an entry that is a link is refused rather than renamed,
/// because the rename
///  would move whatever stands at the link's target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isADirectory = (dir, name) => lstatSync(join(handlePath(dir), name)).isDirectory();
/// ```
pub(crate) fn is_a_directory(
    folder: &File,
    name: &OsStr,
) -> bool {
    return match entry_metadata_at(folder, name) {
        Ok(found) => found.is_dir(),
        Err(_unreadable) => false,
    };
}

/// What:
///  A wait in the unit a person reads it in:
/// whole seconds from one second up,
/// milliseconds below that.
/// Why:
///   A control shortens the ceiling to milliseconds,
/// and a person's sentence must state the bound it actually
///  reached rather than rounding it to "0 s".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inWords = (d: number) => d >= 1000 ? `${Math.round(d / 1000)} s` : `${d} ms`;
/// ```
pub(crate) fn in_words(ceiling: Duration) -> String {
    return if ceiling.as_secs() >= 1 {
        format!("{} s", ceiling.as_secs())
    } else {
        format!("{} ms", ceiling.as_millis())
    };
}

/// What:
///  What one add's wait for a snapshot ahead came to.
/// Why:
///   The three answers decide three different things:
/// a claimed copy travels on to the index step,
///  a request the service gave up ends the rung for a copy that can never come,
/// and a ceiling ends it with
///  the reason the person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Waited = { kind: 'claimed'; made: ObjectId; proof: Proof; copiedAtMillis: number } | { kind: 'givenUp' } | { kind: 'ceiling' };
/// ```
pub(crate) enum Waited {
    /// A copy proved equal to the source's index,
    /// the object now at the stage name,
    /// and the millisecond its name says it was made in.
    Claimed(ObjectId, Proof, u128),
    /// The service gave this add's own request up for good.
    GivenUp,
    /// Nothing came within the ceiling.
    Ceiling,
}

/// What:
///  Whether the service gave this add's own request for a copy up for good:
/// its record stands in `failed`,
///  which is the state nothing scans.
/// Why:
///   A request the service refused for a condition nothing can change is answered once,
/// and
///  nothing later makes the copy it asked for;
/// a wait that could only end at its ceiling would cost the add that
///  ceiling for work that cannot come,
/// where taking its own snapshot serves it.
/// The id is the one the ask
///  answered,
/// so another add's refused request ends no wait.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const givenUp = (workDir, asked) => asked !== undefined && existsSync(join(workDir, 'failed', asked));
/// ```
pub(crate) fn given_up(
    work_dir: &Path,
    asked: Option<&str>,
) -> bool {
    let Some(id) = asked else { return false };
    return state_directory(work_dir, FAILED).join(id).exists();
}
