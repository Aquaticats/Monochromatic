//! What:
//!  The host half of the descriptor attach a ZFS worktree clone's mount goes through:
//!  the no-crossing walk from the pinned checkout,
//!  the folder identities the store records,
//!  the field-by-field compare,
//!  the post-attach check,
//!  the rollback plan of a misplaced attach,
//!  and the administrator's parent-adoption decision.
//! Why:
//!  The design attaches a clone to the folder it opened rather than to a path,
//!  compares the recorded identity of that folder rather than its kind,
//!  and refuses, naming the field, when anything moved
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  the build order is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 4).
//!  This folder is the part of that step that needs no mount authority:
//!  the real `fsmount` with `nosuid,nodev`,
//!  the `move_mount` onto the held target,
//!  the post-check against a real mount and the rollback of a misplaced attach
//!  run in the throwaway guest, because they need a mount namespace this host's tests do not have.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `clone_folder_identity_acl.rs`:
//!   the two ACLs of a folder,
//!   read through its descriptor and kept as the kernel's own bytes.
//! - `clone_folder_identity.rs`:
//!   the six recorded fields,
//!   the field-by-field compare,
//!   and the store token.
//! - `slot_identity_store_refusals.rs`:
//!   why the store could not be read, added to or rewritten,
//!   and each cause's sentence.
//! - `slot_identity_store_records.rs`:
//!   one store line,
//!   its kinds and its parsing.
//! - `slot_identity_store.rs`:
//!   the store's path,
//!   its reader,
//!   the one append a slot gets and the parent write an adoption makes.
//! - `attached_mount_identity.rs`:
//!   the attach's flags and the mount facts recorded for one folder.
//! - `clone_slot_walk_refusals.rs`:
//!   why a walk or an identity compare refused,
//!   naming the component or field.
//! - `clone_slot_walk.rs`:
//!   the walk that holds the checkout, the `.claude/worktrees` folder and one slot,
//!   and the second walk that reads the canonical slot.
//! - `attach_post_check.rs`:
//!   whether the mount the attach made is the mount the canonical slot shows.
//! - `misplaced_attach_rollback.rs`:
//!   the plan that takes a misplaced mount down and the report it leaves.
//! - `parent_adoption_facts.rs`:
//!   the policy a replacement must hold and the state of each binding and slot.
//! - `parent_adoption_decision.rs`:
//!   the table that decides whether a recreated parent folder may be recorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = holdCloneSlot({ checkout, slot, store });
//! const outcome = postAttachCheck({ held, attachedMountId: 41, canonicalSlot }, lookAgainAtTheSlot({ checkout, slot }));
//! ```

/// The ACLs of one folder,
/// read as the kernel's own bytes.
mod clone_folder_identity_acl;

/// The six fields the route records for one folder,
/// their compare and their store token.
pub mod clone_folder_identity;

/// Why the slot identity store could not be read, added to or rewritten.
pub mod slot_identity_store_refusals;

/// One line of the slot identity store.
mod slot_identity_store_records;

/// The store of recorded folder identities,
/// its path and its two writes.
pub mod slot_identity_store;

/// The attach's flags and the mount facts one folder's descriptor reports.
pub mod attached_mount_identity;

/// Why the walk or an identity compare refused.
pub mod clone_slot_walk_refusals;

/// The walk from the checkout to one slot,
/// and the walk that reads the canonical slot again.
pub mod clone_slot_walk;

/// The check that the attach landed on the slot it holds.
pub mod attach_post_check;

/// The plan that takes a misplaced mount down,
/// and its report.
pub mod misplaced_attach_rollback;

/// The shapes an adoption compares against.
pub mod parent_adoption_facts;

/// The administrator's decision to record a recreated parent folder.
pub mod parent_adoption_decision;

/// The identity fields and their compare.
pub use clone_folder_identity::{CloneFolderField, CloneFolderIdentity};
/// The bound one ACL read accepts.
pub use clone_folder_identity_acl::MAXIMUM_ACL_BYTES;
/// The store's path and its records.
pub use slot_identity_store::{SLOT_IDENTITY_STORE_PATH, SlotIdentityStore, append_slot_identity, record_parent_identity};
/// The store's record type and the key its parent record carries.
pub use slot_identity_store_records::{PARENT_KEY, StoredIdentity};
/// Why the store refused.
pub use slot_identity_store_refusals::StoreRefusal;
/// The mount facts and the enforced flags.
pub use attached_mount_identity::{ATTACH_MOUNT_FLAGS, MountIdentity, MountIdentityRefusal, mount_facts_of, mount_identity_of};
/// Why a walk or an identity compare refused.
pub use clone_slot_walk_refusals::{CloneSlotRefusal, WalkRefusal, WalkRefusalReason};
/// The walks and what they hold.
pub use clone_slot_walk::{FreshSlotLookup, HeldCloneSlot, HoldCloneSlotRequest, LookAgainAtSlotRequest, WORKTREES_PARENT_NAMES, hold_clone_slot, look_again_at_the_slot};
/// The post-attach check.
pub use attach_post_check::{AttachOutcome, AttachPostCheck, MisplacedAttach, MisplacedReason, post_attach_check};
/// The rollback plan and its report.
pub use misplaced_attach_rollback::{RollbackOutcome, RollbackPlan, RollbackRefusal, plan_rollback};
/// The shapes an adoption compares against.
pub use parent_adoption_facts::{BindingState, ParentPolicy, SlotState};
/// The adoption decision.
pub use parent_adoption_decision::{Adoption, AdoptionFacts, AdoptionRefusal, decide_adoption};
