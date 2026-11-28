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
//!  and refuses,
//!  naming the field,
//!  when anything moved
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Descriptor mount and unmount";
//!  the build order is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 4).
//!  This folder is the part of that step that needs no mount authority:
//!  the real `fsmount` with `nosuid,nodev`,
//!  the `move_mount` onto the held target,
//!  the post-check against a real mount and the rollback of a misplaced attach
//!  run in the throwaway guest,
//!  because they need a mount namespace this host's tests do not have.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `attach_the_clones_words.rs`:
//!   the filesystem type,
//!   the option that names a dataset,
//!   the attributes and the call names,
//!   the refusal one failed call answers,
//!   and the mount an attach made.
//! - `attach_the_clones_calls.rs`:
//!   the four kernel calls one attach is made of.
//! - `attach_the_clone.rs`:
//!   the attach's steps,
//!   and the descriptor-bound unmount a rollback makes.
//! - `attached_mount_identity.rs`:
//!   the attach's flags and the mount facts recorded for one folder.
//! - `clone_folder_identity_acl.rs`:
//!   the two ACLs of a folder,
//!   read through its descriptor and kept as the kernel's own bytes.
//! - `clone_folder_identity.rs`:
//!   the six recorded fields,
//!   the field-by-field compare,
//!   and the store token.
//! - `clone_slot_git_pointer.rs`:
//!   the first observation's bounded,
//!   regular,
//!   no-follow read of the slot's Git pointer.
//! - `parent_adoption_facts.rs`:
//!   the policy a replacement must hold and the state of each binding and slot.
//! - `slot_identity_store_refusals.rs`:
//!   why the store could not be read,
//!   added to or rewritten,
//!   and each cause's sentence.
//! - `clone_slot_walk_refusals.rs`:
//!   why a walk or an identity compare refused,
//!   naming the component or field.
//! - `parent_adoption_decision.rs`:
//!   the table that decides whether a recreated parent folder may be recorded.
//! - `slot_identity_store_control.rs`:
//!   whose the folder and its records must be,
//!   and the refusal naming an account that could write them.
//! - `slot_name.rs`:
//!   the one rule a slot name is held to,
//!   shared by the store,
//!   the walk and the per-slot claim.
//! - `slot_identity_store_records.rs`:
//!   one record file's grammar,
//!   its kinds and its parsing.
//! - `slot_identity_store_reading.rs`:
//!   one record read through the held folder,
//!   the folder's listing,
//!   and the staging names a killed writer left.
//! - `slot_identity_store.rs`:
//!   the store folder's path,
//!   the folder held open,
//!   and the lookups and the listing a reader makes through it.
//! - `clone_slot_walk.rs`:
//!   the shapes one walk to a slot takes and hands back,
//!   and whether the walk compares the slot against its record.
//! - `clone_slot_walk_names.rs`:
//!   the names a walk's sentences use and the refusal each step answers.
//! - `clone_slot_walk_to_the_parent.rs`:
//!   the walk that holds the checkout and the `.claude/worktrees` folder,
//!   and the adoption's own walk of that folder.
//! - `clone_slot_walk_to_the_slot.rs`:
//!   the walk that holds one slot and compares both identities,
//!   and the first observation of a slot the store holds nothing for.
//! - `clone_slot_walk_after_the_attach.rs`:
//!   the mount the held target's name resolves to,
//!   and the walk repeated at the canonical slot.
//! - `attach_post_check.rs`:
//!   whether the mount the attach made is the mount the canonical slot shows.
//! - `misplaced_attach_rollback.rs`:
//!   the plan that takes a misplaced mount down and the report it leaves.
//! - `slot_identity_store_publication_staging_file.rs`:
//!   the record's bytes and their bound,
//!   the two staging names,
//!   the staging file and the two ways it is taken away again.
//! - `slot_identity_store_publication_one_winner.rs`:
//!   the one call that gives a staging file its record's name,
//!   the loser's answer when two recorders meet,
//!   and the fallback for a folder that makes no hard links.
//! - `slot_identity_store_publication.rs`:
//!   the three writes:
//!   a slot's first record,
//!   the parent's first record,
//!   and the adoption's replacement.
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

/// Why the slot identity store could not be read,
/// added to or rewritten.
pub mod slot_identity_store_refusals;

/// Whose the store folder and its records must be,
/// and the permission bits that say another account could write them.
mod slot_identity_store_control;

/// What makes a name no plain slot name,
/// shared by the store,
/// the walk and the per-slot claim.
mod slot_name;

/// One record file's grammar:
/// its header,
/// its kinds and its own encoding.
mod slot_identity_store_records;

/// One record read,
/// the folder's listing,
/// and the staging names a killed writer left.
mod slot_identity_store_reading;

/// The store folder of recorded folder identities,
/// held open by one descriptor.
pub mod slot_identity_store;

/// The hook a control sets to catch any listing of the store folder.
#[cfg(test)]
pub(crate) use slot_identity_store_reading::WHEN_THE_STORE_IS_LISTED;

/// The file a publication fills before its record has a name.
pub mod slot_identity_store_publication_staging_file;

/// The one call that gives a staging file its record's name.
pub mod slot_identity_store_publication_one_winner;

/// The three writes the store takes:
/// a slot's first record,
/// the parent's first record,
/// and the adoption's replacement.
pub mod slot_identity_store_publication;

/// The attach's flags and the mount facts one folder's descriptor reports.
pub mod attached_mount_identity;

/// The words and numbers one attach speaks in.
pub mod attach_the_clones_words;

/// The four kernel calls one attach is made of.
pub mod attach_the_clones_calls;

/// The attach's steps,
/// the mount it made and the descriptor-bound unmount.
pub mod attach_the_clone;

/// Why the walk or an identity compare refused.
pub mod clone_slot_walk_refusals;

/// The shapes one walk to a clone slot takes and hands back.
pub mod clone_slot_walk;

/// The names a walk's sentences use and the refusal each step answers.
pub mod clone_slot_walk_names;

/// The walk that holds the checkout and its worktrees folder.
pub mod clone_slot_walk_to_the_parent;

/// The walk that holds one slot and compares both identities.
pub mod clone_slot_walk_to_the_slot;

/// The reads at the slot after an attach.
pub mod clone_slot_walk_after_the_attach;

/// The slot's Git pointer,
/// read as one bounded regular file and resolved nowhere.
pub mod clone_slot_git_pointer;

/// The check that the attach landed on the slot it holds.
pub mod attach_post_check;

/// The plan that takes a misplaced mount down,
/// and its report.
pub mod misplaced_attach_rollback;

/// The shapes an adoption compares against.
pub mod parent_adoption_facts;

/// The administrator's decision to record a recreated parent folder.
pub mod parent_adoption_decision;

/// Step 4's guest arms:
/// the real mount at a held slot,
/// the enforced flags,
/// item 2's refusals and its rename trial,
/// and the decoy control,
/// all run in the throwaway guest the recipe makes.
#[cfg(all(test, target_os = "linux"))]
#[path = "attach_arm_tests.rs"]
mod attach_arm_tests;

/// The post-attach check.
pub use attach_post_check::{
    AttachOutcome,
    AttachPostCheck,
    MisplacedAttach,
    MisplacedReason,
    post_attach_check,
};
/// The attach's steps and the descriptor-bound unmount.
pub use attach_the_clone::{
    attach_the_clone,
    unmount_through_descriptor,
};
/// The words one attach speaks in,
/// and the mount it made.
pub use attach_the_clones_words::{
    AttachCall,
    AttachRefusal,
    AttachedMount,
    Placement,
};
/// The mount facts and the enforced flags.
pub use attached_mount_identity::{
    ATTACH_MOUNT_FLAGS,
    MountIdentity,
    MountIdentityRefusal,
    mount_facts_of,
    mount_identity_of,
};
/// The identity fields and their compare.
pub use clone_folder_identity::{
    CloneFolderField,
    CloneFolderIdentity,
};
/// The bound one ACL read accepts.
pub use clone_folder_identity_acl::MAXIMUM_ACL_BYTES;
/// The slot's Git pointer and the registration it must name.
pub use clone_slot_git_pointer::{
    GitPointerRefusal,
    read_the_slot_git_pointer,
    the_registration_of,
};
/// The shapes one walk takes and hands back.
pub use clone_slot_walk::{
    FreshSlotLookup,
    HeldCloneSlot,
    HeldWorktreesParent,
    HoldCloneSlotRequest,
    LookAgainAtSlotRequest,
    SlotRecord,
};
/// The reads at the slot after an attach.
pub use clone_slot_walk_after_the_attach::{
    look_again_at_the_slot,
    mount_id_at_the_held_target,
};
/// The names a walk's sentences use.
pub use clone_slot_walk_names::WORKTREES_PARENT_NAMES;
/// Why a walk or an identity compare refused.
pub use clone_slot_walk_refusals::{
    CloneSlotRefusal,
    WalkRefusal,
    WalkRefusalReason,
};
/// The walk that holds the checkout and its worktrees folder.
pub use clone_slot_walk_to_the_parent::hold_the_worktrees_parent;
/// The walk that holds one slot.
pub use clone_slot_walk_to_the_slot::{
    hold_clone_slot,
    hold_clone_slot_under_root_control,
    observe_the_clone_slot,
    observe_the_clone_slot_under_root_control,
};
/// The rollback plan and its report.
pub use misplaced_attach_rollback::{
    RollbackOutcome,
    RollbackPlan,
    RollbackRefusal,
    plan_rollback,
    plan_rollback_of,
};
/// The adoption decision.
pub use parent_adoption_decision::{
    Adoption,
    AdoptionFacts,
    AdoptionRefusal,
    decide_adoption,
};
/// The shapes an adoption compares against.
pub use parent_adoption_facts::{
    BindingState,
    ParentPolicy,
    SlotState,
};
/// The store folder's path,
/// its open and its lookups and listing.
pub use slot_identity_store::{
    SLOT_IDENTITY_STORE_PATH,
    SlotIdentityStore,
};
/// Whose the store folder and its records must be,
/// and the modes they are made with.
pub use slot_identity_store_control::{
    RECORD_MODE,
    STORE_FOLDER_MODE,
    STORE_OWNER,
    store_control_refusal,
};
/// The store's three writes.
pub use slot_identity_store_publication::{
    publish_parent_identity_under_root_control,
    publish_slot_identity_under_root_control,
    replace_parent_identity_under_root_control,
};
/// The per-file bound,
/// derived from the kernel's own limits.
pub use slot_identity_store_reading::RECORD_MAXIMUM_BYTES;
/// The store's header line,
/// its record type and the key its parent record carries.
pub use slot_identity_store_records::{
    PARENT_KEY,
    PARENT_RECORD_NAME,
    SLOT_NAME_MAXIMUM_BYTES,
    STORE_HEADER,
    StoredIdentity,
    is_known_staging_name,
};
/// Why the store refused.
pub use slot_identity_store_refusals::StoreRefusal;
pub(crate) use slot_name::slot_name_refusal;
