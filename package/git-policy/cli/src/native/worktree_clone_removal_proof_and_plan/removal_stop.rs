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
            BookmarkRefusalReason::Unexplained => "no live snapshot of this clone and no recorded plan line carries its tuple, so this clone did not make it",
            BookmarkRefusalReason::OriginSide => "the origin-side inventory the add recorded carries its tuple, so it came from the history this clone inherited",
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
    /// The destruction plan could not be read.
    PlanUnreadable {
        /// The plan's file.
        path: PathBuf,
        /// Why it was not read.
        refusal: PlanRefusal,
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
}

impl RemovalStop {
    /// What:
    ///  The sentence a person reads about one stop.
    /// Why:
    ///  Every one of them says what refused,
    ///  that nothing was destroyed,
    ///  and what changes before a retry,
    ///  because the removal keeps the registration and the records and leaves the clone alone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// stop.message();
    /// ```
    pub fn message(&self) -> String {
        return match self {
            RemovalStop::NoBinding { slot, reason } => format!(
                "the binding the add wrote for the slot `{slot}` could not be read ({reason}), so nothing proves which clone, origin or registration this removal is about. Nothing was destroyed, and the registration stands. Ways forward: read the binding record filed under the slot's name in the wrapper's record folder, and run the removal again once it reads; a removal never repairs or removes one."
            ),
            RemovalStop::BindingIncomplete { slot, field } => format!(
                "the binding the add wrote for the slot `{slot}` carries no `{field}`, so the removal cannot prove the clone's lineage. Nothing was destroyed, and the registration stands. Ways forward: read the binding record under the slot's name and run the removal again once it carries the field; a slot whose add never reached the clone has nothing for a removal to destroy."
            ),
            RemovalStop::DoesNotAnswer { object, reason } => format!(
                "the pool does not answer for `{object}` ({reason}), so the removal cannot prove its lineage and stops before its first delete. Nothing was destroyed, and the registration stands. Ways forward: run the removal again once the pool answers for that name."
            ),
            RemovalStop::IdentityDiffers { object, field, recorded, live } => format!(
                "`{object}` answers with {field} `{live}` and the add recorded `{recorded}`, so this object is not the one the add made and the removal stops before its first delete. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; the wrapper never changes a recorded identity to match the pool."
            ),
            RemovalStop::OriginIsNotTheRecordedOne { clone, recorded, live } => format!(
                "the clone `{clone}` answers with origin `{}`, and the add recorded `{recorded}`, so the clone inherited another history than the one this removal proved and stops before its first delete. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; the wrapper never updates the recorded lineage to match an external change.",
                live.as_deref().unwrap_or("none")
            ),
            RemovalStop::TransferredHistory { clone, snapshot, createtxg, clone_createtxg } => format!(
                "the live snapshot `{snapshot}` of `{clone}` was created in transaction group {createtxg}, which is not above the clone's own {clone_createtxg}, so it is history this clone inherited rather than made. Destroying it would delete a snapshot the main checkout still depends on, which item 11 measured as a loss no promotion restores. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; the removal refuses a promoted or otherwise mismatched clone before its first delete."
            ),
            RemovalStop::OriginSideHit { kind, name, guid } => format!(
                "the live {} `{name}` carries GUID `{}`, which the origin-side inventory the add recorded also holds, so it belongs to the history this clone inherited. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; a removal destroys only a clone's own snapshots.",
                match kind {
                    EntryKind::Snapshot => "snapshot",
                    EntryKind::Bookmark => "bookmark",
                },
                guid.file_name()
            ),
            RemovalStop::OriginInventoryUnreadable { slot, reason } => format!(
                "the origin-side inventory the add recorded for the slot `{slot}` could not be read ({reason}), so nothing proves which history this clone inherited. Nothing was destroyed, and the registration stands. Ways forward: read the inventory beside the binding record and run the removal again; the removal never rebuilds or updates one."
            ),
            RemovalStop::OriginInventoryOfAnotherDataset { inventory_dataset, recorded_dataset } => format!(
                "the origin-side inventory names the dataset `{inventory_dataset}` and the binding records `{recorded_dataset}` as the origin dataset, so the two records disagree about what this clone was cloned from. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; the removal changes neither record."
            ),
            RemovalStop::InventoryUnreadable { clone, reason } => format!(
                "the snapshots and bookmarks of `{clone}` could not be listed completely ({reason}), and a list this route could not read is a refusal rather than a short history. Nothing was destroyed, and the registration stands. Ways forward: run the removal again once the pool answers."
            ),
            RemovalStop::BookmarksRefused { clone, refused } => {
                let mut lines: Vec<String> = Vec::with_capacity(refused.len());
                for bookmark in refused {
                    lines.push(format!(
                        "- `{}` (GUID `{}`, creation transaction group {}): {}",
                        bookmark.name,
                        bookmark.guid.file_name(),
                        bookmark.createtxg,
                        bookmark.reason.clause()
                    ));
                }
                format!(
                    "the bookmark rule refused {} bookmark(s) of `{clone}`, and the removal stops before its first delete with every snapshot and bookmark still there. Nothing was destroyed, and the registration stands. The refused bookmarks are:\n{}\nWays forward: a person destroys a refused bookmark with `zfs destroy <clone>#<bookmark>` and runs the removal again; the removal never destroys a bookmark and never tries a subset.",
                    refused.len(),
                    lines.join("\n")
                )
            }
            RemovalStop::OriginCheckRefused { refusal, history } => refusal.message(history),
            RemovalStop::TheSnapshotSetIsNotSendable { said, sent } => format!(
                "this removal cannot submit its verified snapshots as one request: {said} No snapshot request was sent for that set, and {sent} request(s) this run already sent are recorded in the plan. The removal stops with the clone unmounted and the intent and the plan standing. Ways forward: read the plan's file in the wrapper's plan folder; the set is rebuilt from a fresh inventory before the next attempt, and a byte budget sized to the set is what removes this stop."
            ),
            RemovalStop::ThePlanCouldNotHoldTheAnswer { path, said } => format!(
                "the destruction plan at {} could not hold an answer the kernel gave ({said}), so the plan now says less about this clone's own snapshots than the pool does. No further request was sent, the clone stands, and the intent and the plan are kept. Ways forward: read the plan's file in the wrapper's plan folder and the clone's own snapshots; a name the plan calls `planned` may already be gone, and a later attempt settles such a name by a fresh read before anything is sent for it.",
                path.display()
            ),
            RemovalStop::APlanStands { slot, path } => format!(
                "a removal's destruction plan for the slot `{slot}` stands at {}, and this entry writes a plan only when none stands. Nothing was destroyed, every snapshot and bookmark stands, and the registration stands. Ways forward: the removal that wrote this plan continues from it, and a resumed removal reads its lines so a snapshot it already recorded is proved rather than requested again.",
                path.display()
            ),
            RemovalStop::PlanUnreadable { path, refusal } => refusal.message(path.as_path()),
            RemovalStop::NoRunNonce { reason } => format!(
                "this removal cannot make the nonce that ties its record to its own invocation ({reason}). Nothing was destroyed, every snapshot and bookmark stands, and the registration stands. Ways forward: run the removal again; if the machine's random source keeps failing, an administrator has to repair it before a removal can record anything."
            ),
            RemovalStop::NoOwnerIdentity { reason } => format!(
                "this process's own birth identity could not be read ({reason}), so a record it wrote could not name the removal that wrote it. Nothing was destroyed, and the registration stands. Ways forward: run the removal again; a machine whose process table cannot be read cannot tell a running removal from an abandoned one."
            ),
            RemovalStop::ASnapshotIsNotTheRecordedOne { snapshot, recorded_guid, recorded_txg, live_txg, live_name } => format!(
                "the plan recorded the snapshot `{snapshot}` with GUID `{recorded_guid}` in creation transaction group {recorded_txg}, and the live object carrying that GUID is `{live_name}` in group {live_txg}, so it is not the object this removal planned. Nothing was destroyed for that name, and the plan keeps every line it had. Ways forward: compare the two identities with `zfs get -H -p -o value guid,createtxg <name>`; no wrapper command settles this, because an outcome is never replaced with a guess."
            ),
            RemovalStop::TheCloneNameWasReused { name, recorded_guid, live_guid } => format!(
                "the name `{name}` now holds a dataset with GUID `{live_guid}` and the plan recorded `{recorded_guid}`, so the recorded clone's absence is unproved and nothing is settled. Ways forward: `git monochromatic-git-policy-cli settle-worktree-clone-record` is the command for a slot whose clone cannot be proved gone; no removal settles a reused name."
            ),
            RemovalStop::HoldersSeeTheClone { holders, incomplete } => format!(
                "a fresh all-namespace scan still sees this clone held, in {} namespace(s):\n{}\n{incomplete} further read(s) could not be completed. Nothing was destroyed for those holders, the plan keeps every line it had, and the clone stays unmounted. Ways forward: release the holders, or run a forced removal once it is measured; a scan that sees no holder never proves there is none.",
                holders.len(),
                holders.join("\n")
            ),
            RemovalStop::TheSnapshotsRemain { names } => format!(
                "this run left snapshots of the clone that the plan does not record as destroyed, so the head destroy was not sent. The clone stands and the plan keeps every line and its answers. The names and their states are:\n{}\nWays forward: run the removal again, which proves each name by its GUID before it submits anything, and release any hold or dependent clone the pool named.",
                names.join("\n")
            ),
            RemovalStop::TheOriginCleanupCouldNotBeRecorded { path, said } => format!(
                "the origin cleanup record at {} could not be written ({said}), so this removal refused before its first destroy. Nothing was destroyed and the clone stands unmounted with the plan and the intent kept. Ways forward: fix what refused the write and run the removal again; a removal that destroyed without this record would leave the checkout's origin snapshot with nothing that carries its cleanup.",
                path.display()
            ),
            RemovalStop::TheCloneIsTheSettlingCommands { slot, path, head } => format!(
                "the clone of the slot `{slot}` is proved gone, and the plan at {} says `head={head}`, so this run cannot have destroyed it. Nothing was changed. Ways forward: `git monochromatic-git-policy-cli settle-worktree-clone-record {slot} --confirm-removal-of {slot} --confirm-clone-guid <guid>` is the command for a clone destroyed outside its removal; a removal never settles a head it did not request.",
                path.display()
            ),
            RemovalStop::NoLiveIntent { clone } => format!(
                "a destruction plan stands for `{clone}`, and the clone's own `unmount` property names no request this run may go on from, so nothing proves the unmount the plan claims. Nothing was destroyed. Ways forward: run `git worktree add` at the slot again if the worktree must come back, or read the plan's file in the wrapper's plan folder; a removal never removes the property a person wrote."
            ),
            RemovalStop::ThePlanContradictsThePool { head, clone } => format!(
                "the plan says `head={head}` and the clone `{clone}` still answers on the pool, so the record and the pool disagree about whether a destroy happened. Nothing was destroyed. Ways forward: read the plan's file in the wrapper's plan folder and compare it with the pool; a removal never rewrites a recorded outcome to match what it finds."
            ),
            RemovalStop::TheSettleRefused { step, said } => format!(
                "the clone is proved destroyed, and the settle stopped at `{step}` ({said}). The plan keeps the record of what was destroyed, and the origin cleanup record stands. Ways forward: read the plan's file in the wrapper's plan folder, fix what the step names, and run the removal again; it adopts the plan and finishes the settle without destroying anything twice."
            ),
            RemovalStop::TheRecordsCouldNotGo { path, said } => format!(
                "the clone is proved destroyed and the settle finished, and the removal's own records could not go: {} ({said}). Nothing is destroyed again by a later run. Ways forward: remove what the message names once you have read it; the plan is the last witness and is removed only after the binding's files are proved gone.",
                path.display()
            ),
            RemovalStop::UnmountFailed { step, said } => format!(
                "the unmount step `{step}` did not prove the clone absent ({said}), so the removal stops before it records a destruction plan. Nothing was destroyed, every snapshot and bookmark stands, and the clone either stays where the step left it or stands at its slot again. Ways forward: find what holds the clone, release it, and run the removal again; with `--force` the wrapper ends the holders other than the caller and its session."
            ),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_stop_tests.rs"]
mod tests;
