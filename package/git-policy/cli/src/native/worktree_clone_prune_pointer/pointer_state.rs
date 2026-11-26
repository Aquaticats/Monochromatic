//! What:
//!  What one slot's `.git` pointer says against the bytes the add recorded:
//!  intact,
//!  different,
//!  absent,
//!  or unprovable,
//!  and which copy of the pointer the slot's path shows.
//! Why:
//!  The design keeps the underlay pointer for `git worktree prune`,
//!  independent of the clone's own pointer,
//!  because a registration whose pointer is gone is pruned by an explicit
//!  `git worktree prune` at once
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Unmounted,
//! pruned or moved worktrees";
//!  plan step 12).
//!  The record holds the exact bytes Git wrote,
//!  so damage is a comparison rather than a guess,
//!  and a pointer the wrapper never recorded is its own answer rather than
//!  either neighbour.
//!  Nothing here writes a file or reads a mount:
//!  the callers hand in the reading and act on the answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const state = pointerStateOf(record.pointer, readTheSlotPointer(slot)); // 'intact'
//! ```

use crate::worktree_clone_managed_lock::lock_and_pointer_readings::read_the_slot_pointer;
use crate::worktree_clone_managed_lock::reading_a_bounded_file::Reading;
use crate::worktree_clone_slot_state::slot_state_mounts::CloneMount;
use std::path::Path;

/// What:
///  What the slot's `.git` holds against the bytes the add recorded.
/// Why:
///  Each answer has one next safe action,
///  and a pointer the read refused is not the same state as an absent one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PointerState = { kind: 'intact' } | { kind: 'differs'; found } | { kind: 'present'; found } | { kind: 'missing' } | { kind: 'unprovable'; reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PointerState {
    /// The bytes are the ones the add recorded.
    Intact,
    /// A regular file stands there with other bytes.
    Differs {
        /// The bytes found.
        found: u64,
    },
    /// A regular file stands there,
    /// and the record keeps no bytes to compare it with.
    Present {
        /// The bytes found.
        found: u64,
    },
    /// Nothing stands there.
    Missing,
    /// The bytes cannot be compared,
    /// because the record holds none or the read refused.
    Unprovable {
        /// Why no comparison was made.
        reason: String,
    },
}

impl PointerState {
    /// The state in the words a report prints.
    pub fn word(&self) -> &'static str {
        return match self {
            PointerState::Intact => "intact",
            PointerState::Differs {
                ..
            } => "different",
            PointerState::Present {
                ..
            } => "present, and the record keeps no bytes to compare it with",
            PointerState::Missing => "absent",
            PointerState::Unprovable {
                ..
            } => "cannot be proved",
        };
    }
}

/// What:
///  Which copy of the pointer the slot's path shows.
/// Why:
///  While the clone is mounted the slot's `.git` is the clone's own copy,
///  and while it is unmounted it is the underlay's copy the add kept for
///  prune,
///  so a report that named no copy would leave a person unable to tell which
///  file it read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PointerSite = { kind: 'underlay' } | { kind: 'clone-root' } | { kind: 'another-mount'; described } | { kind: 'unprovable'; reason };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PointerSite {
    /// The slot shows the checkout dataset's own folder,
    /// which holds the pointer the add kept for prune.
    Underlay,
    /// The slot shows the clone's own root,
    /// whose `.git` is the copy the add installed.
    AtTheCloneRoot,
    /// Something that is not the clone's own root stands at the slot,
    /// so the `.git` there is neither copy this route knows.
    AnotherMount {
        /// The mount in words.
        described: String,
    },
    /// The mount table could not be read,
    /// so which copy the slot shows is unproved.
    Unprovable {
        /// Why the site was not read.
        reason: String,
    },
}

impl PointerSite {
    /// The sentence a report prints for the copy it read.
    pub fn sentence(&self) -> String {
        return match self {
            PointerSite::Underlay => String::from(
                "the check reads the underlay pointer the add kept for `git worktree prune`, which is the copy the slot shows while the clone is unmounted",
            ),
            PointerSite::AtTheCloneRoot => String::from(
                "the check reads the clone's own pointer, which is the copy the slot shows while the clone is mounted; the underlay's copy is covered by the mount",
            ),
            PointerSite::AnotherMount {
                described,
            } => format!(
                "{described} stands at the slot, so the `.git` there is neither copy this route wrote"
            ),
            PointerSite::Unprovable {
                reason,
            } => format!("which copy of the pointer the slot shows cannot be read: {reason}"),
        };
    }
}

/// What:
///  Where the slot's `.git` comes from,
///  read from the clone's mount.
/// Why:
///  The classification already decides whether the clone's own root is at the
///  slot,
///  mounted elsewhere,
///  or nowhere,
///  and this reading reuses that answer rather than comparing the table again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const site = pointerSiteOf(cloneMountOf(rows, slotMount, clone, slot));
/// ```
pub fn pointer_site_of(mount: &CloneMount) -> PointerSite {
    return match mount {
        CloneMount::AtItsSlot => PointerSite::AtTheCloneRoot,
        CloneMount::Nowhere => PointerSite::Underlay,
        // A clone mounted elsewhere leaves the slot's own folder showing, which is the underlay this route made.
        CloneMount::Elsewhere {
            ..
        } => PointerSite::Underlay,
        CloneMount::SomethingElseAtTheSlot {
            described,
        } => PointerSite::AnotherMount {
            described: described.clone(),
        },
        CloneMount::Unreadable {
            reason,
        } => PointerSite::Unprovable {
            reason: reason.clone(),
        },
    };
}

/// What:
///  What one reading of the slot's `.git` says against the recorded bytes.
/// Why:
///  The comparison is byte for byte,
///  because the record holds the pointer Git was given,
///  and a file with other bytes names another registration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = pointerStateOf(recorded, reading);
/// ```
pub fn pointer_state_of(
    recorded: Option<&[u8]>,
    reading: &Reading,
) -> PointerState {
    return match reading {
        Reading::Absent => PointerState::Missing,
        Reading::Bytes(found) => match recorded {
            Some(wanted) if found.as_slice() == wanted => PointerState::Intact,
            Some(_other) => PointerState::Differs {
                found: found.len() as u64,
            },
            None => PointerState::Present {
                found: found.len() as u64,
            },
        },
        Reading::Refused(cause) => PointerState::Unprovable {
            reason: cause.words(),
        },
    };
}

/// What:
///  The state of the pointer at one slot's path,
///  read with the bound the clone mount route gives that file.
/// Why:
///  Every reader of this file asks the same question,
///  so the read and the comparison are one call and no caller can compare
///  bytes it read with another bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = readPointerStateAt(slotPath, record.pointer);
/// ```
pub fn read_pointer_state_at(
    slot_path: &Path,
    recorded: Option<&[u8]>,
) -> PointerState {
    let reading: Reading = read_the_slot_pointer(slot_path);
    return pointer_state_of(recorded, &reading);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "pointer_state_tests.rs"]
mod tests;
