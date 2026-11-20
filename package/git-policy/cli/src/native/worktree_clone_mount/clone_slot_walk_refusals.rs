//! What:
//!  Why the walk from the pinned checkout to a clone slot stopped,
//!  why a held slot is not the one the store recorded,
//!  and the sentence each cause prints.
//! Why:
//!  Every refusal names the failing component or field
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount"),
//!  because "different" alone sends a person hunting through a whole tree.
//!  Each sentence also names every way forward this repository offers,
//!  and none of them is a refresh of the recorded identity.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = holdCloneSlot({ checkout, slot, store }); // throws with a sentence naming the field
//! ```

use super::clone_folder_identity::{CloneFolderField, CloneFolderIdentity, field_names};
use super::slot_identity_store_refusals::StoreRefusal;
use crate::fs_beneath_walk_chain::ChainRefusal;
use std::path::PathBuf;

/// What:
///  The kind of thing that stopped a walk.
/// Why:
///   The sentence depends on the kind:
///  a link planted in the way,
///  a boundary the walk never crosses,
///  a folder that is not there,
///  a component that is no folder,
///  a name that is no plain component,
///  or a read that failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkRefusalReason = 'link' | 'boundary' | 'missing' | 'not-a-directory' | 'not-a-plain-name' | 'unreadable' | 'unreadable-fields' | 'link-above';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WalkRefusalReason {
    /// A component is a symbolic link,
    /// which is never followed.
    LinkInTheWay,
    /// A component is a mount point or on another device,
    /// which the walk never crosses.
    BoundaryCrossed,
    /// A component is not there.
    Missing,
    /// A component is not a directory.
    NotADirectory,
    /// A name given as a slot is not one plain folder name.
    NotAPlainName,
    /// A component could not be opened for another reason.
    Unreadable,
    /// A folder was opened but its own facts could not be read:
    /// its identity,
    /// its mount or the path its descriptor names.
    UnreadableFields,
    /// The checkout path is reached through a link.
    LinkAmongTheAncestors,
    /// The checkout Git's path names could not be opened.
    Checkout,
}

/// What:
///  The component a walk refused,
///  the kind of refusal,
///  and the system's own words when it said something.
/// Why:
///   The component is what a person fixes;
///  the kind is what tells a planted link from a missing folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkRefusal = { component: string; reason: WalkRefusalReason; detail?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalkRefusal {
    /// The component as a person reads it,
    /// relative to the checkout,
    /// such as `.claude/worktrees/my-slot`.
    pub component: String,
    /// The kind of refusal.
    pub reason: WalkRefusalReason,
    /// The system's words,
    /// when there are any.
    pub detail: String,
}

impl WalkRefusal {
    /// The refusal in words,
    /// naming the component and every way forward.
    pub fn message(&self) -> String {
        let subject: &str = self.component.as_str();
        return match self.reason {
            WalkRefusalReason::LinkInTheWay => format!(
                "{subject} is a symbolic link, and this walk never follows a link. Ways forward: remove the link and let a real folder stand there, or name the slot where its folder really is below `.claude/worktrees`."
            ),
            WalkRefusalReason::BoundaryCrossed => format!(
                "{subject} is a mount point or lies on another filesystem than the checkout, and this walk never crosses such a boundary. Ways forward: attach the clone to a folder on the checkout's own filesystem below `.claude/worktrees`; a folder another mount covers is administratively moved, not adopted by this route."
            ),
            WalkRefusalReason::Missing => format!(
                "{subject} is not there. Ways forward: let the add create the slot below `.claude/worktrees` and run the command again; when the checkout was restored, an administrator checks the recorded `.claude/worktrees` identity before anything is attached."
            ),
            WalkRefusalReason::NotADirectory => format!(
                "{subject} is not a directory. Ways forward: remove what holds that name and let the add create the slot folder; nothing is attached onto a file."
            ),
            WalkRefusalReason::NotAPlainName => format!(
                "{subject} is no plain folder name below `.claude/worktrees`: {}. Ways forward: name the slot exactly as `git worktree list` shows it below that folder, with no slash, no `..` and no leading dot.",
                self.detail
            ),
            WalkRefusalReason::Unreadable => format!(
                "{subject} could not be opened: {}. Ways forward: give this account search permission on that folder and run the command again; the walk never widens a permission it was refused.",
                self.detail
            ),
            WalkRefusalReason::UnreadableFields => format!(
                "{subject} was opened, and its own facts could not be read: {}. Ways forward: run the command again; when the read keeps failing, check that this process can read the kernel's `/proc` entries for the folder and that its access and default ACLs fit the bound a read accepts, and report the answer the system gave.",
                self.detail
            ),
            WalkRefusalReason::LinkAmongTheAncestors => format!(
                "the checkout {subject} is reached through a link: {}. Ways forward: run the command in the checkout's own path, whose ancestors are real directories; this route never resolves a path the account can change.",
                self.detail
            ),
            WalkRefusalReason::Checkout => format!(
                "{subject} could not be opened: {}. Ways forward: run the command in the checkout Git reports (`git rev-parse --show-toplevel`) and let this account search every folder above it; this route walks nothing outside that checkout.",
                self.detail
            ),
        };
    }
}

/// What:
///  Why a clone slot could not be held:
///  the walk stopped,
///  the store could not be read,
///  the parent folder or the slot is not the recorded one,
///  or the slot's two descriptors name different folders.
/// Why:
///   The attach compares against the recorded identity and refuses otherwise,
/// so each cause needs its own sentence and none of them refreshes the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CloneSlotRefusal = { kind: 'walk'; refusal } | { kind: 'store'; refusal } | { kind: 'identity'; recorded; found; fields } | { kind: 'target' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloneSlotRefusal {
    /// The walk from the checkout stopped before the slot.
    Walk(WalkRefusal),
    /// The recorded identities could not be read or do not hold what the walk needs.
    Store(StoreRefusal),
    /// The `.claude/worktrees` folder is not the one whose identity was recorded.
    ParentIdentity {
        /// The checkout the store was compared with,
        /// because one store records one checkout's folders.
        checkout: PathBuf,
        /// The identity the store recorded.
        recorded: Box<CloneFolderIdentity>,
        /// The identity the walk read from the held folder.
        found: Box<CloneFolderIdentity>,
        /// The fields in which they differ.
        fields: Vec<CloneFolderField>,
    },
    /// The slot folder is not the one whose identity was recorded.
    SlotIdentity {
        /// The slot's name.
        slot: String,
        /// The identity the store recorded.
        recorded: Box<CloneFolderIdentity>,
        /// The identity the walk read from the held folder.
        found: Box<CloneFolderIdentity>,
        /// The fields in which they differ.
        fields: Vec<CloneFolderField>,
    },
    /// The slot's reading descriptor and its mount target named different folders.
    TargetAnotherObject {
        /// The slot's name.
        slot: String,
        /// The identity of the folder opened for reading.
        first: String,
        /// The identity of the folder opened as the mount target.
        second: String,
    },
}

impl CloneSlotRefusal {
    /// The refusal in words,
    /// naming the failing component or field and every way forward.
    pub fn message(&self) -> String {
        return match self {
            CloneSlotRefusal::Walk(refusal) => refusal.message(),
            CloneSlotRefusal::Store(refusal) => refusal.message(),
            CloneSlotRefusal::ParentIdentity { checkout, recorded, found, fields } => format!(
                "the `.claude/worktrees` parent folder below {} is not the folder whose identity was recorded: the {} differs. Recorded: {}. Found: {}. A changed identity is not permission to adopt the replacement, so no clone is mounted, and one store records one checkout: a folder of a second checkout never matches this record. Ways forward: restore the folder whose identity is recorded; or, after a checkout restore or a controlled recreation, an administrator runs `mount-git-worktree-clone-datasets adopt-worktree-parent` with the intended replacement's device and inode and runs the command again.",
                checkout.display(),
                field_names(fields),
                recorded.describe(),
                found.describe()
            ),
            CloneSlotRefusal::SlotIdentity { slot, recorded, found, fields } => format!(
                "the clone slot `{slot}` is not the folder whose identity was recorded: the {} differs. Recorded: {}. Found: {}. The record is never refreshed because a new inode or mode appeared. Ways forward: restore the folder whose identity is recorded, or settle the binding with the removal workflow; a slot with no managed binding takes ordinary reflink creation or Git's own checkout instead, and only an administrator's `adopt-worktree-parent` may replace the parent folder's record.",
                field_names(fields),
                recorded.describe(),
                found.describe()
            ),
            CloneSlotRefusal::TargetAnotherObject { slot, first, second } => format!(
                "the clone slot `{slot}` changed between the two descriptors this walk opened: one names {first} and the mount target names {second}. Nothing is mounted onto a folder that moved between two looks. Ways forward: run the command again once the account stops renaming the slot; anything already attached stays where it is."
            ),
        };
    }
}

/// What:
///  The walk's refusal of one component of a chain below the checkout.
/// Why:
///  The chain reports the component's position and the open's error;
///  a person reads the folder's name and a sentence that fits the cause.
pub fn chain_walk_refusal(names: &[&str], refusal: &ChainRefusal) -> WalkRefusal {
    let step: usize = match refusal {
        ChainRefusal::Link { step } | ChainRefusal::Boundary { step } | ChainRefusal::Unreadable { step, .. } => *step,
    };
    let reason: WalkRefusalReason = match refusal {
        ChainRefusal::Link { .. } => WalkRefusalReason::LinkInTheWay,
        ChainRefusal::Boundary { .. } => WalkRefusalReason::BoundaryCrossed,
        ChainRefusal::Unreadable { error, .. } => reason_of(error),
    };
    let detail: String = match refusal {
        ChainRefusal::Unreadable { error, .. } if reason == WalkRefusalReason::Unreadable => error.to_string(),
        _ => String::new(),
    };
    return WalkRefusal { component: component_of(names, step), reason, detail };
}

/// What:
///  The refusal of one directory opened by name below the worktrees folder,
///  such as the slot itself.
/// Why:
///  The slot's own open fails outside the chain,
///  and its sentence names the slot as a person reads it.
pub fn one_component_refusal(component: &str, error: &std::io::Error) -> WalkRefusal {
    let reason: WalkRefusalReason = match error.raw_os_error() {
        Some(libc::ELOOP) => WalkRefusalReason::LinkInTheWay,
        Some(libc::EXDEV) => WalkRefusalReason::BoundaryCrossed,
        Some(libc::ENOENT) => WalkRefusalReason::Missing,
        Some(libc::ENOTDIR) => WalkRefusalReason::NotADirectory,
        None if error.kind() == std::io::ErrorKind::InvalidInput => WalkRefusalReason::NotAPlainName,
        _ => WalkRefusalReason::Unreadable,
    };
    let detail: String = if reason == WalkRefusalReason::Unreadable { error.to_string() } else { String::new() };
    return WalkRefusal { component: component.to_string(), reason, detail };
}

/// What:
///  The refusal of one folder that was opened and whose own facts could not be read,
///  such as its identity,
///  its mount or the path its descriptor names.
/// Why:
///  An over-large ACL or a missing `/proc` line is not an open failure,
///  and a person told the folder could not be opened looks in the wrong place.
pub fn fields_refusal(component: &str, error: &std::io::Error) -> CloneSlotRefusal {
    return CloneSlotRefusal::Walk(WalkRefusal { component: component.to_string(), reason: WalkRefusalReason::UnreadableFields, detail: error.to_string() });
}

/// The reason for an open failure inside a chain,
/// where a name that is no plain component cannot happen.
fn reason_of(error: &std::io::Error) -> WalkRefusalReason {
    return match error.raw_os_error() {
        Some(libc::ENOENT) => WalkRefusalReason::Missing,
        Some(libc::ENOTDIR) => WalkRefusalReason::NotADirectory,
        _ => WalkRefusalReason::Unreadable,
    };
}

/// The component at `step` as a person reads it,
/// below the checkout.
fn component_of(names: &[&str], step: usize) -> String {
    let Some(part) = names.get(..=step) else { return String::from("a folder below the checkout") };
    return part.join("/");
}
