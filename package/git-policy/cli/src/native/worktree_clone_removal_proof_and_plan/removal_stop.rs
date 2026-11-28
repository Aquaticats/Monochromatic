//! What:
//!  Why a removal stopped before its first delete:
//!  each check that refused,
//!  the objects it named,
//!  and the sentence a person reads.
//! Why:
//!  The design requires a refusal to name the mismatched dataset,
//!  snapshot or dependency,
//!  to keep the registration and the records,
//!  and to report the need for administrator lineage recovery before a retry:
//!  "Name the mismatched dataset,
//! snapshot or dependency,
//! keep the registration and removal
//!  record,
//! and report the need for administrator lineage recovery before retry.
//! Do not try a
//!  subset,
//! promote back automatically,
//! or invoke ordinary Git removal to clear the slot"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//! "Removal and recovery",
//! step 2).
//!  One type carries every stop so the removal has one place to print from and one place to keep its
//!  promise that nothing was deleted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Stop = { kind: 'transferred-history'; snapshot; guid; createtxg } | ...;
//! throw new Error(stop.message());
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_proof_and_plan::plan_refusal::PlanRefusal;
use crate::worktree_removal_deferred_origin_check::origin_refusal::OriginRefusal;
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use std::path::PathBuf;

/// What:
///  One refused bookmark:
///  the object the rule refused and the reason its tuple is not explained.
/// Why:
///  The decided rule refuses the whole removal and names each refused bookmark,
///  so a person can destroy the one the design leaves to them and retry
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Bookmarks on a clone being removed").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RefusedBookmark = { name; guid; createtxg; reason: 'no-live-snapshot' | 'origin-side' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefusedBookmark {
    /// The bookmark's full name,
    /// `<clone>#<bookmark>`.
    pub name: String,
    /// The GUID it carries from the snapshot it was made from.
    pub guid: DatasetGuid,
    /// That snapshot's creation transaction group.
    pub createtxg: u64,
    /// Why the rule refuses it.
    pub reason: BookmarkRefusalReason,
}

/// What:
///  Why one bookmark's tuple is not explained.
/// Why:
///  The two reasons have different ways forward:
///  a stamp from a snapshot that is already gone is a person's `zfs destroy`,
///  and one that matches the origin side is history this clone inherited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reason = 'no-live-or-planned-snapshot' | 'matches-the-origin-side';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BookmarkRefusalReason {
    /// Neither a live passing clone snapshot nor a recorded plan line carries its tuple.
    Unexplained,
    /// The origin-side inventory the add recorded carries its tuple.
    OriginSide,
}

impl BookmarkRefusalReason {
    /// The clause a message prints.
    pub fn clause(self) -> &'static str {
        return match self {
            BookmarkRefusalReason::Unexplained => {
                "no live snapshot of this clone and no recorded plan line carries its tuple, so this clone did not make it"
            }
            BookmarkRefusalReason::OriginSide => {
                "the origin-side inventory the add recorded carries its tuple, so it came from the history this clone inherited"
            }
        };
    }
}

/// What:
///  Every way a removal stops.
/// Why:
///  A stop before the first delete keeps the registration,
///  the records and every snapshot;
///  a stop after one of the batch's requests keeps the plan's own record of what that request
///  did,
///  and its message says which of the two states it left.
///  Each arm names what a person has to change before a retry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Stop = { kind: ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemovalStop {
    /// The add's binding record for this slot could not be read.
    NoBinding {
        /// The slot the record is filed under.
        slot: String,
        /// What the read answered.
        reason: String,
    },
    /// Another command holds the slot's per-slot claim.
    TheSlotIsClaimed {
        /// The slot whose claim another command holds.
        slot: String,
        /// The owning process's id.
        pid: i64,
        /// The claim file that names the owner.
        path: PathBuf,
    },
    /// The per-slot claim could not be taken.
    TheClaimCannotBeTaken {
        /// The slot whose claim could not be taken.
        slot: String,
        /// What refused,
        /// in its own words.
        said: String,
    },
    /// The binding record lacks a field a removal needs.
    BindingIncomplete {
        /// The slot the record is filed under.
        slot: String,
        /// The field the removal needs.
        field: &'static str,
    },
    /// A dataset or snapshot the proof reads does not answer the pool.
    DoesNotAnswer {
        /// The object the read named.
        object: String,
        /// What the read answered.
        reason: String,
    },
    /// A recorded identity disagrees with the live one.
    IdentityDiffers {
        /// The object the comparison is about.
        object: String,
        /// The field that differs.
        field: &'static str,
        /// What the add recorded.
        recorded: String,
        /// What the pool answers now.
        live: String,
    },
    /// The clone's live origin is not the origin snapshot the add recorded.
    OriginIsNotTheRecordedOne {
        /// The clone.
        clone: String,
        /// The origin snapshot the add recorded.
        recorded: String,
        /// The clone's live origin,
        /// or nothing when it has none.
        live: Option<String>,
    },
    /// One live snapshot of the clone is not the clone's own:
    /// it was made at or before the clone itself.
    TransferredHistory {
        /// The clone.
        clone: String,
        /// The snapshot.
        snapshot: String,
        /// Its creation transaction group.
        createtxg: u64,
        /// The clone's own.
        clone_createtxg: u64,
    },
    /// One live object of the clone carries a tuple the origin-side inventory recorded.
    OriginSideHit {
        /// Whether the object is a snapshot or a bookmark.
        kind: EntryKind,
        /// Its full name.
        name: String,
        /// The GUID it carries.
        guid: DatasetGuid,
    },
    /// The origin-side inventory the add recorded could not be read.
    OriginInventoryUnreadable {
        /// The slot the inventory is filed under.
        slot: String,
        /// What the read answered.
        reason: String,
    },
    /// The origin-side inventory names another dataset than the binding does.
    OriginInventoryOfAnotherDataset {
        /// The dataset the inventory inventoried.
        inventory_dataset: String,
        /// The origin dataset the binding records.
        recorded_dataset: String,
    },
    /// The clone's snapshots and bookmarks could not be listed completely.
    InventoryUnreadable {
        /// The clone.
        clone: String,
        /// What the read answered.
        reason: String,
    },
    /// The decided bookmark rule refused one or more bookmarks.
    BookmarksRefused {
        /// The clone.
        clone: String,
        /// Every bookmark the rule refused,
        /// each named.
        refused: Vec<RefusedBookmark>,
    },
    /// The origin is marked for deferred destroy,
    /// or a check could not judge the clone or its origin,
    /// at the look or at one of the two checks.
    OriginCheckRefused {
        /// Step 15's own refusal,
        /// with its cause,
        /// its counts and its message.
        refusal: Box<OriginRefusal>,
        /// What the plan already records as lost,
        /// which its sentence says,
        /// and nothing on a first removal.
        history: RemovalHistory,
    },
    /// The verified snapshot set is not a request this route will send.
    TheSnapshotSetIsNotSendable {
        /// What the split or the request builder refused,
        /// in its own words.
        said: String,
        /// How many requests this run had already sent,
        /// whose answers the plan records.
        sent: usize,
    },
    /// The plan could not hold an answer the kernel gave,
    /// so it now says less than the pool does.
    ThePlanCouldNotHoldTheAnswer {
        /// The plan's file.
        path: PathBuf,
        /// Why the line could not be written,
        /// in the record's own words.
        said: String,
    },
    /// A destruction plan for this clone already stands,
    /// and a resumed removal is what continues it.
    APlanStands {
        /// The slot the plan belongs to.
        slot: String,
        /// The plan's file.
        path: PathBuf,
    },
    /// The destruction plan could not be read,
    /// and this removal carried no force to go past it.
    PlanUnreadable {
        /// The slot the plan belongs to.
        slot: String,
        /// The plan's file.
        path: PathBuf,
        /// Why it was not read.
        refusal: PlanRefusal,
    },
    /// The clone the add recorded cannot be proved,
    /// and this removal carried less than two forces.
    TheCloneCannotBeProved {
        /// The slot whose clone it is.
        slot: String,
        /// What the proof refused,
        /// in its own words with the identities it compared.
        cause: String,
        /// The GUID the add's record held.
        recorded_guid: String,
        /// The GUID the pool answered,
        /// when it answered one.
        answered_guid: Option<String>,
    },
    /// A forced removal went past a plan it could not read,
    /// and the clone that plan's record names does not answer the pool.
    ThePlanIsUnreadableAndTheCloneIsGone {
        /// The slot whose plan it is.
        slot: String,
        /// The plan's file.
        path: PathBuf,
        /// What the pool answered for the record's clone dataset.
        reason: String,
    },
    /// The lock this add did not write could not be deleted,
    /// which two forces ask for and the removal stops on when it fails.
    TheLockCouldNotGo {
        /// The lock's file.
        path: PathBuf,
        /// Why the unlink failed.
        said: String,
    },
    /// The removal's own run nonce could not be made.
    NoRunNonce {
        /// What the random source answered.
        reason: String,
    },
    /// This process's own birth identity could not be read,
    /// so the plan cannot name its writer.
    NoOwnerIdentity {
        /// What the read answered.
        reason: String,
    },
    /// A live snapshot carries a recorded GUID in another creation transaction group,
    /// so it is not the object this plan recorded.
    ASnapshotIsNotTheRecordedOne {
        /// The snapshot's recorded name.
        snapshot: String,
        /// The recorded GUID.
        recorded_guid: String,
        /// The recorded creation transaction group.
        recorded_txg: u64,
        /// The live one.
        live_txg: u64,
        /// The live snapshot's full name,
        /// which differs from the recorded one when a snapshot was renamed.
        live_name: String,
    },
    /// The recorded clone's name now holds another object.
    TheCloneNameWasReused {
        /// The recorded name.
        name: String,
        /// The recorded GUID.
        recorded_guid: String,
        /// The live one.
        live_guid: String,
    },
    /// All-namespace holders the fresh scan named stand between the removal and its destroy.
    HoldersSeeTheClone {
        /// Each namespace and the mounts it holds,
        /// in the scan's own words.
        holders: Vec<String>,
        /// How many reads the scan could not complete.
        incomplete: usize,
    },
    /// This run's requests left snapshots the plan does not record as destroyed.
    TheSnapshotsRemain {
        /// Every name whose current state is not `destroyed`,
        /// with that state.
        names: Vec<String>,
    },
    /// The origin cleanup record could not be written,
    /// so the removal refused before its first destroy.
    TheOriginCleanupCouldNotBeRecorded {
        /// The record's file.
        path: PathBuf,
        /// Why it could not be written,
        /// in the record's own words.
        said: String,
    },
    /// The clone is proved gone,
    /// and its plan's head word says no destroy of this run's may have sent it,
    /// so the settling command is the route.
    TheCloneIsTheSettlingCommands {
        /// The slot whose clone is gone.
        slot: String,
        /// The plan that stands.
        path: PathBuf,
        /// The plan's head word.
        head: String,
    },
    /// A standing plan has no live unmount intent,
    /// so nothing proves the unmount it claims.
    NoLiveIntent {
        /// The clone,
        /// with what the property held instead.
        clone: String,
    },
    /// A plan's head word contradicts the clone the pool answers.
    ThePlanContradictsThePool {
        /// The plan's head word.
        head: String,
        /// The clone that answered.
        clone: String,
    },
    /// The registration's lock holds words this add cannot attribute to itself,
    /// so a person's lock may stand on it and the removal refuses before its first
    /// write.
    TheLocksWordsAreNotThisAdds {
        /// The slot the lock stands on.
        slot: String,
        /// What the lock holds,
        /// or why its read refused it.
        cause: String,
    },
    /// The pool could not prove whether the clone a standing plan names is
    /// gone,
    /// so the plan's own resume path cannot be chosen.
    TheClonesAbsenceCouldNotBeProved {
        /// The slot the plan belongs to.
        slot: String,
        /// The plan's file.
        path: PathBuf,
        /// The plan's head word.
        head: String,
        /// What the read answered.
        reason: String,
    },
    /// The registration settle refused.
    TheSettleRefused {
        /// The step that refused.
        step: String,
        /// What it answered.
        said: String,
    },
    /// The removal's records could not be removed after a proved destruction.
    TheRecordsCouldNotGo {
        /// The file that refused.
        path: PathBuf,
        /// Why.
        said: String,
    },
    /// The unmount request was refused,
    /// retained a mount,
    /// or did not answer.
    UnmountFailed {
        /// The step that refused.
        step: String,
        /// What it answered.
        said: String,
    },
    /// A holder stopped the removal,
    /// its answer put the clone back at its slot,
    /// and the removal's report is the answer's own.
    TheRemovalCancelledItself {
        /// The report the answer wrote,
        /// one line per name.
        said: String,
    },
    /// The head destroy did not take the clone,
    /// and the answer was not a holder the removal cancels itself for.
    TheHeadDestroyWasRefused {
        /// The kernel's errno,
        /// when it kept one.
        errno: Option<u32>,
        /// Every name the pool still answers,
        /// with its state in the plan.
        names: Vec<String>,
    },
    /// A holder stopped the removal and the answer could not put the clone back.
    TheRemovalCouldNotCancelItself {
        /// Where the removal stopped.
        stop: &'static str,
        /// What the answer said.
        said: String,
    },
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_stop_tests.rs"]
mod tests;
