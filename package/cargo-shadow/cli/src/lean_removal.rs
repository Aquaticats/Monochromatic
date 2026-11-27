//! What:
//! The one way every lean-folder tool removes an entry:
//! check its identity,
//! move it into a trash directory on the same file system by an atomic rename that never replaces,
//! check the identity again,
//! and only then delete it.
//! Why:
//! Removal is one owner's action per item.
//! The rename claims the item (a second tool that races finds it gone),
//! the identity check before the rename and before the unlink (as the Git wrapper's lock removals do:
//! `package/git-policy/cli/src/native/lock/stale_ref_lock_own_ended.rs`) stops a path that was swapped for another entry from
//! being removed,
//! and a mismatch leaves what was moved in the trash for a person to look at:
//! "never discard user data".
//!
//! A claimed entry may hold a directory its owner cannot write (`0500`,
//! `0555`,
//! `0000`):
//! unlinking a file needs write and search on the directory that holds it,
//! and listing a directory needs read on it,
//! so the deletion gives the directory those three bits first when this user owns it,
//! and reports that it did.
//! A directory of another user is left as it is and named.
//! Only directories the walk enters are changed,
//! and a symbolic link is always removed as a link,
//! so nothing outside the claimed entry is touched.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function removeOne(item: Item, trash: string): Deleted  // throws Refusal
//! ```
//!
//! The protocol's parts are one file each:
//! the item as it was judged and what a deletion did (`item_and_deletion_record.rs`),
//! the refusals and the words that show them (`refusal_text.rs`),
//! the claim into the trash (`claim_into_trash.rs`),
//! the deletion of the claim (`deletion_walk.rs`),
//! and the finish of an earlier run's claims (`trash_sweep.rs`).

/// The name of the trash directory each tool keeps beside what it removes from.
pub const TRASH_DIRECTORY_NAME: &str = ".cargo-shadow-trash";

/// The claim of one item into the trash.
mod claim_into_trash;
/// The deletion of a claimed entry.
mod deletion_walk;
/// The item as it was judged and what a completed deletion did.
mod item_and_deletion_record;
/// The refusals and the words that show them.
mod refusal_text;
/// The finish of an earlier run's claims.
mod trash_sweep;

/// Re-exports the claim and the read of an item's identity.
pub use self::claim_into_trash::{
    claim_into_trash,
    item_of,
};
/// Re-exports the walk's byte reading and the deletion that ends a claim.
pub use self::deletion_walk::{
    allocated_bytes_of,
    delete_claimed,
};
/// Re-exports the deletion a test drives as another user and the mode widening it applies,
/// which nothing outside the unit tests of this module names.
#[cfg(test)]
pub(crate) use self::deletion_walk::{
    delete_claimed_as,
    widened_mode,
};
/// Re-exports the item as it was judged and what a deletion did.
pub use self::item_and_deletion_record::{
    Deleted,
    Item,
};
/// Re-exports why an item was not removed and the clause that names changed modes.
pub use self::refusal_text::{
    Refusal,
    modes_named,
};
/// Re-exports what finishing an earlier run's claims did and the sweep that does it.
pub use self::trash_sweep::{
    TrashSweep,
    empty_trash,
};

/// The tests of the removal primitive.
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "lean_removal_tests.rs"]
mod lean_removal_tests;
