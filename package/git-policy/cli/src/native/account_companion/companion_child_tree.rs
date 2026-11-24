//! What:
//!  The scratch tree the companion's child owns inside its own tmpfs:
//!  the slot,
//!  the move target and the refusal tree,
//!  and the no-crossing walk that reaches the slot and records its identity.
//! Why:
//!  Nothing the child moves or unmounts exists before this,
//!  and every name is inside the filesystem the child owns,
//!  so the walk the product uses is judged over a tree nobody else can rename
//!  into
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The account companion",
//! "What it runs").
//!  The identity the walk answers is what the changed-identity control
//!  compares.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! mkdir(slot); mkdir(moveTarget); mkdir(refusal/bound/slot); mkdir(refusal/replaced);
//! const slotFd = openat2(root, 'slot', NO_XDEV | BENEATH); const id = statx(slotFd);
//! ```

use super::companion_buffers::CompanionBuffers;
use super::companion_calls::Failure;
use super::companion_descriptor_calls::{open_directory, open_directory_beneath};
use super::companion_file_calls::{Identity, identity_of, make_directory};
use std::os::fd::RawFd;

/// What:
///  Make every directory the companion's own tmpfs holds.
/// Why:
///  The slot and the move target must exist before a mount lands on either,
///  and the refusal tree's two folders are where the controls plant their
///  names.
///
/// # Errors
/// A directory could not be made.
pub fn make_the_tree(buffers: &CompanionBuffers) -> Result<(), Failure> {
    make_directory(buffers.slot.as_c_str())?;
    make_directory(buffers.move_target.as_c_str())?;
    make_directory(buffers.refusal.as_c_str())?;
    make_directory(buffers.bound.as_c_str())?;
    make_directory(buffers.bound_slot.as_c_str())?;
    make_directory(buffers.replaced.as_c_str())?;
    return Ok(());
}

/// What:
///  Walk from the scratch tmpfs's root to the slot and record its identity.
/// Why:
///  The walk is the product's no-crossing one over a tree the companion owns,
///  so a mount planted across a component is refused by the kernel's own bits.
///
/// # Errors
/// The root or the slot could not be opened,
///  or the identity could not be read.
pub fn walk_to_the_slot(buffers: &CompanionBuffers) -> Result<Slots, Failure> {
    let root: RawFd = open_directory(buffers.scratch.as_c_str())?;
    let slot: RawFd = open_directory_beneath(root, buffers.slot_name.as_c_str())?;
    let identity: Identity = identity_of(slot)?;
    return Ok(Slots { root, slot, identity });
}

/// What:
///  The descriptors one walk leaves open,
///  and the identity the walk answered.
/// Why:
///  The attach lands on the slot's own descriptor,
///  the controls walk from the root,
///  and the identity is what the changed-identity control compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Slots = { root: number; slot: number; identity: Identity };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Slots {
    /// The scratch tmpfs's root.
    pub root: RawFd,
    /// The slot the walk reached.
    pub slot: RawFd,
    /// The slot's identity.
    pub identity: Identity,
}
