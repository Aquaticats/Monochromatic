//! What:
//!  The sentence each way of stopping prints to the person who ran the removal:
//!  what refused,
//!  that nothing was destroyed,
//!  and what changes before a retry.
//! Why:
//!  Every stop keeps the registration,
//!  the records and every snapshot,
//!  so one sentence says so,
//!  and a person reads the same words whether the stop came before the first delete
//!  or after one of the batch's requests,
//!  whose own sentence says which of the two states it left.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! stop.message();
//! ```

use crate::worktree_clone_binding_record::binding_inventory::EntryKind;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;

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
            RemovalStop::NoBinding {
                slot,
                reason,
            } => format!(
                "the binding the add wrote for the slot `{slot}` could not be read ({reason}), so nothing proves which clone, origin or registration this removal is about. Nothing was destroyed, and the registration stands. Ways forward: read the binding record filed under the slot's name in the wrapper's record folder, and run the removal again once it reads; a removal never repairs or removes one."
            ),
            RemovalStop::BindingIncomplete {
                slot,
                field,
            } => format!(
                "the binding the add wrote for the slot `{slot}` carries no `{field}`, so the removal cannot prove the clone's lineage. Nothing was destroyed, and the registration stands. Ways forward: read the binding record under the slot's name and run the removal again once it carries the field; a slot whose add never reached the clone has nothing for a removal to destroy."
            ),
            RemovalStop::TheSlotIsClaimed {
                slot,
                pid,
                path,
            } => format!(
                "another command holds the claim on the slot `{slot}` (process {pid}, claim file {}), so this removal took no claim, read no record and destroyed nothing. Nothing was destroyed by this run. Ways forward: wait for that command to end, or run the removal again once it has; a held slot is refused rather than queued, and a removal that was killed leaves a claim the next command takes over under the claim's own rule.",
                path.display()
            ),
            RemovalStop::TheClaimCannotBeTaken {
                slot,
                said,
            } => format!(
                "the claim on the slot `{slot}` cannot be taken ({said}), so this removal took no claim, read no record and destroyed nothing. Nothing was destroyed by this run. Ways forward: read what the message names, check whether a command of the wrapper is working on this slot, and run the removal again; a claim file is the object that decides who may work on the slot, so no automatic step removes one."
            ),
            RemovalStop::DoesNotAnswer {
                object,
                reason,
            } => format!(
                "the pool does not answer for `{object}` ({reason}), so the removal cannot prove its lineage and stops before its first delete. Nothing was destroyed, and the registration stands. Ways forward: run the removal again once the pool answers for that name."
            ),
            RemovalStop::IdentityDiffers {
                object,
                field,
                recorded,
                live,
            } => format!(
                "`{object}` answers with {field} `{live}` and the add recorded `{recorded}`, so this object is not the one the add made and the removal stops before its first delete. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; the wrapper never changes a recorded identity to match the pool."
            ),
            RemovalStop::OriginIsNotTheRecordedOne {
                clone,
                recorded,
                live,
            } => format!(
                "the clone `{clone}` answers with origin `{}`, and the add recorded `{recorded}`, so the clone inherited another history than the one this removal proved and stops before its first delete. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; the wrapper never updates the recorded lineage to match an external change.",
                live.as_deref().unwrap_or("none")
            ),
            RemovalStop::TransferredHistory {
                clone,
                snapshot,
                createtxg,
                clone_createtxg,
            } => format!(
                "the live snapshot `{snapshot}` of `{clone}` was created in transaction group {createtxg}, which is not above the clone's own {clone_createtxg}, so it is history this clone inherited rather than made. Destroying it would delete a snapshot the main checkout still depends on, which item 11 measured as a loss no promotion restores. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; the removal refuses a promoted or otherwise mismatched clone before its first delete."
            ),
            RemovalStop::OriginSideHit {
                kind,
                name,
                guid,
            } => format!(
                "the live {} `{name}` carries GUID `{}`, which the origin-side inventory the add recorded also holds, so it belongs to the history this clone inherited. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; a removal destroys only a clone's own snapshots.",
                match kind {
                    EntryKind::Snapshot => "snapshot",
                    EntryKind::Bookmark => "bookmark",
                },
                guid.file_name()
            ),
            RemovalStop::OriginInventoryUnreadable {
                slot,
                reason,
            } => format!(
                "the origin-side inventory the add recorded for the slot `{slot}` could not be read ({reason}), so nothing proves which history this clone inherited. Nothing was destroyed, and the registration stands. Ways forward: read the inventory beside the binding record and run the removal again; the removal never rebuilds or updates one."
            ),
            RemovalStop::OriginInventoryOfAnotherDataset {
                inventory_dataset,
                recorded_dataset,
            } => format!(
                "the origin-side inventory names the dataset `{inventory_dataset}` and the binding records `{recorded_dataset}` as the origin dataset, so the two records disagree about what this clone was cloned from. Nothing was destroyed, and the registration stands. Ways forward: an administrator has to recover the lineage; the removal changes neither record."
            ),
            RemovalStop::InventoryUnreadable {
                clone,
                reason,
            } => format!(
                "the snapshots and bookmarks of `{clone}` could not be listed completely ({reason}), and a list this route could not read is a refusal rather than a short history. Nothing was destroyed, and the registration stands. Ways forward: run the removal again once the pool answers."
            ),
            RemovalStop::BookmarksRefused {
                clone,
                refused,
            } => {
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
            RemovalStop::OriginCheckRefused {
                refusal,
                history,
            } => refusal.message(history),
            RemovalStop::TheSnapshotSetIsNotSendable {
                said,
                sent,
            } => format!(
                "this removal cannot submit its verified snapshots as one request: {said} No snapshot request was sent for that set, and {sent} request(s) this run already sent are recorded in the plan. The removal stops with the clone unmounted and the intent and the plan standing. Ways forward: read the plan's file in the wrapper's plan folder; the set is rebuilt from a fresh inventory before the next attempt, and a byte budget sized to the set is what removes this stop."
            ),
            RemovalStop::ThePlanCouldNotHoldTheAnswer {
                path,
                said,
            } => format!(
                "the destruction plan at {} could not hold an answer the kernel gave ({said}), so the plan now says less about this clone's own snapshots than the pool does. No further request was sent, the clone stands, and the intent and the plan are kept. Ways forward: read the plan's file in the wrapper's plan folder and the clone's own snapshots; a name the plan calls `planned` may already be gone, and a later attempt settles such a name by a fresh read before anything is sent for it.",
                path.display()
            ),
            RemovalStop::APlanStands {
                slot,
                path,
            } => format!(
                "a removal's destruction plan for the slot `{slot}` stands at {}, and this entry writes a plan only when none stands. Nothing was destroyed, every snapshot and bookmark stands, and the registration stands. Ways forward: the removal that wrote this plan continues from it, and a resumed removal reads its lines so a snapshot it already recorded is proved rather than requested again.",
                path.display()
            ),
            RemovalStop::PlanUnreadable {
                path,
                refusal,
            } => refusal.message(path.as_path()),
            RemovalStop::NoRunNonce {
                reason,
            } => format!(
                "this removal cannot make the nonce that ties its record to its own invocation ({reason}). Nothing was destroyed, every snapshot and bookmark stands, and the registration stands. Ways forward: run the removal again; if the machine's random source keeps failing, an administrator has to repair it before a removal can record anything."
            ),
            RemovalStop::NoOwnerIdentity {
                reason,
            } => format!(
                "this process's own birth identity could not be read ({reason}), so a record it wrote could not name the removal that wrote it. Nothing was destroyed, and the registration stands. Ways forward: run the removal again; a machine whose process table cannot be read cannot tell a running removal from an abandoned one."
            ),
            RemovalStop::ASnapshotIsNotTheRecordedOne {
                snapshot,
                recorded_guid,
                recorded_txg,
                live_txg,
                live_name,
            } => format!(
                "the plan recorded the snapshot `{snapshot}` with GUID `{recorded_guid}` in creation transaction group {recorded_txg}, and the live object carrying that GUID is `{live_name}` in group {live_txg}, so it is not the object this removal planned. Nothing was destroyed for that name, and the plan keeps every line it had. Ways forward: compare the two identities with `zfs get -H -p -o value guid,createtxg <name>`; no wrapper command settles this, because an outcome is never replaced with a guess."
            ),
            RemovalStop::TheCloneNameWasReused {
                name,
                recorded_guid,
                live_guid,
            } => format!(
                "the name `{name}` now holds a dataset with GUID `{live_guid}` and the plan recorded `{recorded_guid}`, so the recorded clone's absence is unproved and nothing is settled. Ways forward: `git monochromatic-git-policy-cli settle-worktree-clone-record` is the command for a slot whose clone cannot be proved gone; no removal settles a reused name."
            ),
            RemovalStop::HoldersSeeTheClone {
                holders,
                incomplete,
            } => format!(
                "a fresh all-namespace scan still sees this clone held, in {} namespace(s):\n{}\n{incomplete} further read(s) could not be completed. Nothing was destroyed for those holders, the plan keeps every line it had, and the clone stays unmounted. Ways forward: release the holders and run the removal again; a scan that sees no holder never proves there is none, and this release's removal ends no holder.",
                holders.len(),
                holders.join("\n")
            ),
            RemovalStop::TheSnapshotsRemain {
                names,
            } => format!(
                "this run left snapshots of the clone that the plan does not record as destroyed, so the head destroy was not sent. The clone stands and the plan keeps every line and its answers. The names and their states are:\n{}\nWays forward: run the removal again, which proves each name by its GUID before it submits anything, and release any hold or dependent clone the pool named.",
                names.join("\n")
            ),
            RemovalStop::TheOriginCleanupCouldNotBeRecorded {
                path,
                said,
            } => format!(
                "the origin cleanup record at {} could not be written ({said}), so this removal refused before its first destroy. Nothing was destroyed and the clone stands unmounted with the plan and the intent kept. Ways forward: fix what refused the write and run the removal again; a removal that destroyed without this record would leave the checkout's origin snapshot with nothing that carries its cleanup.",
                path.display()
            ),
            RemovalStop::TheCloneIsTheSettlingCommands {
                slot,
                path,
                head,
            } => format!(
                "the clone of the slot `{slot}` is proved gone, and the plan at {} says `head={head}`, so this run cannot have destroyed it. Nothing was changed. Ways forward: `git monochromatic-git-policy-cli settle-worktree-clone-record {slot} --confirm-removal-of {slot} --confirm-clone-guid <guid>` is the command for a clone destroyed outside its removal; a removal never settles a head it did not request.",
                path.display()
            ),
            RemovalStop::NoLiveIntent {
                clone,
            } => format!(
                "a destruction plan stands for `{clone}`, and the clone's own `unmount` property names no request this run may go on from, so nothing proves the unmount the plan claims. Nothing was destroyed. Ways forward: run `git worktree add` at the slot again if the worktree must come back, or read the plan's file in the wrapper's plan folder; a removal never removes the property a person wrote."
            ),
            RemovalStop::ThePlanContradictsThePool {
                head,
                clone,
            } => format!(
                "the plan says `head={head}` and the clone `{clone}` still answers on the pool, so the record and the pool disagree about whether a destroy happened. Nothing was destroyed. Ways forward: read the plan's file in the wrapper's plan folder and compare it with the pool; a removal never rewrites a recorded outcome to match what it finds."
            ),
            RemovalStop::TheSettleRefused {
                step,
                said,
            } => format!(
                "the clone is proved destroyed, and the settle stopped at `{step}` ({said}). The plan keeps the record of what was destroyed, and the origin cleanup record stands. Ways forward: read the plan's file in the wrapper's plan folder, fix what the step names, and run the removal again; it adopts the plan and finishes the settle without destroying anything twice."
            ),
            RemovalStop::TheRecordsCouldNotGo {
                path,
                said,
            } => format!(
                "the clone is proved destroyed and the settle finished, and the removal's own records could not go: {} ({said}). Nothing is destroyed again by a later run. Ways forward: remove what the message names once you have read it; the plan is the last witness and is removed only after the binding's files are proved gone.",
                path.display()
            ),
            RemovalStop::UnmountFailed {
                step,
                said,
            } => format!(
                "the unmount step `{step}` did not prove the clone absent ({said}), so the removal stops before it records a destruction plan. Nothing was destroyed, every snapshot and bookmark stands, and the clone either stays where the step left it or stands at its slot again. Ways forward: find what holds the clone, release it, and run the removal again; this release's removal ends no holder, and `git worktree remove` offers no way to ask it to."
            ),
            RemovalStop::TheRemovalCancelledItself {
                said,
            } => String::from(said.as_str()),
            RemovalStop::TheHeadDestroyWasRefused {
                errno,
                names,
            } => format!(
                "the head destroy of this clone did not take it (errno {}), and every name the pool still answers is:\n{}\nThe clone stands with what this run proved about it, and the plan records the request its head word names. Ways forward: release what holds the clone and run the removal again, which adopts the plan and destroys nothing twice.",
                match errno {
                    Some(held) => held.to_string(),
                    None => String::from("none"),
                },
                if names.is_empty() {
                    String::from("  no snapshot of this clone is left")
                } else {
                    names.join("\n")
                }
            ),
            RemovalStop::TheRemovalCouldNotCancelItself {
                stop,
                said,
            } => format!(
                "{stop}, so the removal asked to put the clone back at its slot and could not: {said}. Nothing of the clone's own history was destroyed by a stop that came before a delete, and the plan keeps every line it had. Ways forward: read what refused, then run `git monochromatic-git-policy-cli cancel-worktree-removal <slot>` to finish putting the clone back, or run the removal again once the request can be answered."
            ),
        };
    }
}
