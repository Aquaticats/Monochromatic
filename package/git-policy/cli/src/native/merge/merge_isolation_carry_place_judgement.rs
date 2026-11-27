//! What:
//!  The judgement one planned worktree write stands on,
//!  and the answer that write gives:
//!  what stood at the path when the write was judged,
//!  the index entries a check accepts,
//!  whether the path still holds the entry that was judged,
//!  and what the write came to.
//! Why:
//!   A judgement and a write are separate steps,
//!  so a person who saves the file between them had their bytes replaced with no
//!  notice;
//! the identity of the entry is the cheap check and the object its bytes make is the
//!  authority,
//!  so both are recorded before a byte is written
//! (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
//! finding 4).
//!  Taking the judged entry aside on a private name and checking what the take caught
//!  are `merge_isolation_carry_place_take.rs`,
//!  and the placement itself is `merge_isolation_carry_place.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type JudgedFile = { identity?: FileIdentity; content?: string };
//! type Placed = { kind: 'written' } | { kind: 'nothing' } | { kind: 'changed'; kept?: string };
//! ```

/// The index entry one path's write is.
use crate::merge_isolation_carry_write_forms::IndexWrite;
/// The identity of the entry and the object its bytes make,
/// which say whether what stands at a path is the entry a write judged.
use crate::merge_isolation_changes::{
    FileIdentity,
    WorktreeState,
    file_identity,
    worktree_path,
};
/// A place in the worktree,
/// and where an entry of the person's waits.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  What stood at a worktree path when the write was judged.
/// Why:
///   The take compares both parts of it:
/// the identity of the entry the file system hands out,
///  and the object its bytes make,
/// so an entry written after the judgement is never replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JudgedFile = { identity?: FileIdentity; content?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct JudgedFile {
    /// The identity of the entry that stood there,
    /// or nothing when nothing did.
    pub(crate) identity: Option<FileIdentity>,
    /// The object the bytes of that entry make,
    /// or nothing when nothing stood there.
    pub(crate) content: Option<String>,
}

/// What one planned write came to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placed = { kind: 'written' } | { kind: 'nothing' } | { kind: 'changed'; kept?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Placed {
    /// The path holds what the write asked for.
    Written,
    /// There was nothing for the write to do.
    Nothing,
    /// The path holds bytes the write did not judge,
    /// so nothing was written to it;
    /// `kept` names the entry that was taken aside and could not go back.
    Changed {
        /// Where an entry of the person's waits,
        /// when the path was taken by another write before it could go back.
        kept: Option<PathBuf>,
    },
}

/// What:
///  What stood at a worktree path when the step was judged,
/// for the placement that refuses to replace anything else.
/// Why:
///   The state names the object the bytes make and the identity names the entry itself,
/// and a write needs both to
///  prove that what it is about to replace is still the thing it judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judgedOf = (state, identity) => ({ identity, content: state.kind === 'content' ? state.object : undefined });
/// ```
pub(crate) fn judged_of(
    state: &WorktreeState,
    identity: Option<FileIdentity>,
) -> JudgedFile {
    return match state {
        WorktreeState::Content {
            object,
            ..
        } => JudgedFile {
            identity,
            content: Some(object.clone()),
        },
        WorktreeState::Missing | WorktreeState::Other => JudgedFile {
            identity,
            content: None,
        },
    };
}

/// What:
///  The entries an index check accepts,
///  which are the ones already there and the one this step is about to write.
/// Why:
///   A step that was killed after its index records were written finds its own work in the index;
/// reading that
///  as someone else's dropped the layers it had not written yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const allowed = (known, plan) => plan.kind === 'entry' ? [...known, [plan.mode, plan.oid]] : plan.kind === 'remove' ? [...known, null] : known;
/// ```
pub(crate) fn explained_entries<'a>(
    known: &[Option<(&'a str, &'a str)>],
    plan: &'a IndexWrite,
) -> Vec<Option<(&'a str, &'a str)>> {
    let mut allowed: Vec<Option<(&str, &str)>> = known.to_vec();
    match plan {
        IndexWrite::Leave => {}
        IndexWrite::Entry(mode, oid) => allowed.push(Some((mode.as_str(), oid.as_str()))),
        IndexWrite::Remove => allowed.push(None),
    }
    return allowed;
}

/// What:
///  Whether the entry at one path is still the one a write judged,
///  read without taking anything.
/// Why:
///   A step judges every path before it writes the first byte of its index records,
/// so a path written
///  before that point stops the whole step with nothing written;
/// the take then covers the window that follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isStillTheJudgedEntry(root, path, judged): boolean; // identityOf(location) === judged.identity
/// ```
pub(crate) fn is_still_the_judged_entry(
    root: &Path,
    path: &[u8],
    judged: &JudgedFile,
) -> bool {
    return file_identity(worktree_path(root, path).as_path()) == judged.identity;
}
