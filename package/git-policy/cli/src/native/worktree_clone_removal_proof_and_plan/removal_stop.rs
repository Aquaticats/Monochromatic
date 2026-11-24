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
//!  "Name the mismatched dataset, snapshot or dependency, keep the registration and removal
//!  record, and report the need for administrator lineage recovery before retry. Do not try a
//!  subset, promote back automatically, or invoke ordinary Git removal to clear the slot"
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`, "Removal and recovery", step 2).
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
///  Every way a removal stops before its first delete.
/// Why:
///  A stop keeps the registration,
///  the records and every snapshot,
///  and each arm names what a person has to change before a retry.
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
    /// The origin is marked for deferred destroy.
    OriginMarked {
        /// Step 15's own refusal,
        /// with its counts and its message.
        refusal: Box<OriginRefusal>,
        /// What the plan already records as lost,
        /// which its sentence says,
        /// and nothing on a first removal.
        history: RemovalHistory,
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
                "the binding the add wrote for the slot `{slot}` could not be read ({reason}), so nothing proves which clone, origin or registration this removal is about. Nothing was destroyed, and the registration stands. Ways forward: run `monochromatic-git-policy-cli doctor`, which reports the binding record; a removal never repairs or removes one."
            ),
            RemovalStop::BindingIncomplete { slot, field } => format!(
                "the binding the add wrote for the slot `{slot}` carries no `{field}`, so the removal cannot prove the clone's lineage. Nothing was destroyed, and the registration stands. Ways forward: run `monochromatic-git-policy-cli doctor` and read the binding record; a slot whose add never reached the clone has nothing for a removal to destroy."
            ),
            RemovalStop::DoesNotAnswer { object, reason } => format!(
                "the pool does not answer for `{object}` ({reason}), so the removal cannot prove its lineage and stops before its first delete. Nothing was destroyed, and the registration stands. Ways forward: run `monochromatic-git-policy-cli doctor`, and run the removal again once the pool answers."
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
            RemovalStop::OriginMarked { refusal, history } => refusal.message(history),
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
