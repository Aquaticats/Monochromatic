//! What:
//!  Which slot,
//!  which clone,
//!  and which records the command proves,
//!  in the design's own order:
//!  the slot's pointer,
//!  the add's binding,
//!  the plan,
//!  the clone's own request,
//!  at least one removal record,
//!  the clone's live identity and origin,
//!  and the snapshots the plan's tuples settle against.
//! Why:
//!  The command acts only on what it can prove,
//!  and a person who typed another slot's name must not be told a removal was reversed
//!  (`doc/planning/cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md`,
//!  its "The proof" section).
//!  The design proves the records first and repeats every deciding read under the claim,
//!  so the slot that is not being removed is refused before any claim is taken,
//!  and no record that changed while the claim was taken is ever acted on.
//!  A pointer to another checkout is the caller's mistake;
//!  a damaged pointer,
//!  a missing or mismatched binding,
//!  a plan of another clone,
//!  a clone whose identity or origin differs,
//!  a request that is no local one,
//!  a plan whose owner still runs,
//!  and a snapshot list that cannot be read are states that prevent the command.
//!  Nothing here writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const guid = proveTheSlotIsARemovals(input);
//! const proved = proveTheSlotsRecords(input);
//! console.log(proved.destroyed);
//! ```

use crate::process_identity::process_birth_identity;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount::clone_slot_git_pointer::read_the_slot_git_pointer;
use crate::worktree_clone_mount::slot_name_refusal;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{
    IntentReading,
    MountIntent,
    MountOperation,
    read_intent,
};
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_cancel::cancel_refusal::{
    CancelRefusal,
    of_pointer,
};
use crate::worktree_clone_removal_cancel::the_destroy_outcomes::settle_the_destroy_outcomes;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::{
    CloneAbsence,
    the_clone_absence,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::{
    RecordedBinding,
    read_the_recorded_binding,
};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_unmount::staging_folder_name::staging_record_name;
use std::path::{
    Path,
    PathBuf,
};

/// The folder this repository's worktrees live in,
/// below its pinned parent.
pub const WORKTREES_FOLDER: &str = ".claude/worktrees";

/// What:
///  Everything the proof reads.
/// Why:
///  A control answers the pool reads and the absence probe and leaves the records real,
///  so the proof's order is the same code with a scripted pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proof = { checkout; commonDir; home; slot; reads; absence };
/// ```
pub struct TheRecordsProof<'a> {
    /// The repository's pinned parent,
    /// the checkout the slot's pointer must name a registration of.
    pub checkout: &'a Path,
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
    /// The account's home,
    /// where a staging record's presence marks a removal.
    pub home: &'a Path,
    /// The slot the command was given.
    pub slot: &'a str,
    /// The pool reads the clone's identity and snapshots.
    pub reads: &'a dyn RemovalPoolReads,
    /// The reader the clone's absence is proved through,
    /// which is the same device in the release.
    pub absence: &'a dyn DatasetReader,
}

/// What:
///  Everything one proof establishes,
///  together with the reads the act needs.
/// Why:
///  The claim is taken between the records' proof and the act,
///  and the caller repeats these reads under it,
///  so one value carries the records,
///  the plan and the settled destroyed set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Proved = { recorded; slotPath; plan?; intent; destroyed };
/// ```
#[derive(Debug)]
pub struct WhatTheRecordsProve {
    /// The identities the add recorded,
    /// which the act and the report read.
    pub recorded: RecordedBinding,
    /// The slot's own folder.
    pub slot_path: PathBuf,
    /// The plan that stands,
    /// where one does.
    pub plan: Option<DestructionPlan>,
    /// The request the clone's own property carried.
    pub intent: MountIntent,
    /// The snapshots the plan's tuples prove destroyed.
    pub destroyed: Vec<String>,
}

/// What:
///  Prove the records and the clone's own request,
///  without reading the clone's snapshot inventory.
/// Why:
///  A mistyped slot or a slot that is not being removed must be refused before a claim is
///  created,
///  because the claim is what makes the reads one moment and a caller's mistake must leave it
///  untouched.
///
/// # Errors
/// A damaged pointer,
/// a pointer to another checkout,
/// a binding that is absent or of another slot,
/// a plan that cannot be read or names another clone,
/// a clone that does not answer,
/// a request that is no local one,
/// a slot with no removal record,
/// or a plan whose owner still runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guid = proveTheSlotIsARemovals(input);
/// ```
pub fn prove_the_slot_is_a_removals(
    input: &TheRecordsProof<'_>
) -> Result<DatasetGuid, CancelRefusal> {
    let recorded: RecordedBinding = read_the_records(input)?;
    let plan: Option<DestructionPlan> = read_the_plan(input, &recorded)?;
    let _intent: MountIntent = read_the_request(input, &recorded, plan.is_some())?;
    return Ok(recorded.clone_guid);
}

/// What:
///  Prove the records and the live clone together,
///  and settle what the plan's tuples prove destroyed.
/// Why:
///  The act may follow only this proof:
///  the records say which clone,
///  the pool says the clone is the recorded one and names its own origin,
///  and the inventory settles every tuple,
///  so a clone of another identity and an outcome nobody can prove are both refused before
///  anything moves.
///
/// # Errors
/// Every refusal the records' proof makes,
/// an origin that is not the recorded one,
/// an inventory that cannot be read,
/// or a tuple whose outcome cannot be settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveTheSlotsRecords(input);
/// ```
pub fn prove_the_slots_records(
    input: &TheRecordsProof<'_>
) -> Result<WhatTheRecordsProve, CancelRefusal> {
    let recorded: RecordedBinding = read_the_records(input)?;
    let plan: Option<DestructionPlan> = read_the_plan(input, &recorded)?;
    let intent: MountIntent = read_the_request(input, &recorded, plan.is_some())?;
    let identity = live_identity(input, &recorded)?;
    let live_origin: Option<String> =
        Some(identity.stats.origin()).filter(|origin| return !origin.is_empty());
    if live_origin.as_deref() != Some(recorded.origin_snapshot.as_str()) {
        return Err(CancelRefusal::TheCloneIsNotTheRecordedOne {
            field: "originSnapshot",
            recorded: recorded.origin_snapshot.clone(),
            live: live_origin.unwrap_or_else(|| return String::from("none")),
        });
    }
    let live: Vec<crate::worktree_clone_removal_proof_and_plan::removal_inventory::LiveObject> =
        input
            .reads
            .snapshots_of(recorded.clone_dataset.as_str(), recorded.clone_guid)
            .map_err(|reason| {
                return CancelRefusal::TheInventoryIsUnreadable {
                    clone: recorded.clone_dataset.clone(),
                    reason,
                };
            })?;
    let destroyed: Vec<String> = match &plan {
        Some(standing) => settle_the_destroy_outcomes(standing, live.as_slice())?,
        None => Vec::new(),
    };
    let slot_path: PathBuf = input.checkout.join(WORKTREES_FOLDER).join(input.slot);
    return Ok(WhatTheRecordsProve {
        recorded,
        slot_path,
        plan,
        intent,
        destroyed,
    });
}

/// What:
///  The records that name one slot's clone:
///  its pointer,
///  its binding,
///  and the plan beside them.
/// Why:
///  These are the reads a caller's mistake must be refused on,
///  and the plan's owner speaks for the removal where a claim cannot.
///
/// # Errors
/// The pointer,
/// binding or plan that cannot be read or that names another object,
/// or a plan whose owner still runs.
fn read_the_records(input: &TheRecordsProof<'_>) -> Result<RecordedBinding, CancelRefusal> {
    if let Some(reason) = slot_name_refusal(input.slot) {
        return Err(CancelRefusal::NotBeingRemoved {
            slot: String::from(input.slot),
            found: format!("`{}` is not a slot name ({reason}).", input.slot),
        });
    }
    let slot_path: PathBuf = input.checkout.join(WORKTREES_FOLDER).join(input.slot);
    let folder: std::fs::File = std::fs::File::open(slot_path.as_path()).map_err(|error| {
        return CancelRefusal::ThePointerIsNotOneGitdirLine {
            slot: String::from(input.slot),
            said: format!(
                "the slot folder {} could not be read ({error})",
                slot_path.display()
            ),
        };
    })?;
    read_the_slot_git_pointer(&folder, input.checkout, input.slot)
        .map_err(|refusal| return of_pointer(input.slot, &refusal))?;
    let recorded: RecordedBinding = read_the_recorded_binding(input.common_dir, input.slot)
        .map_err(|stop| return of_a_removal_stop(input.slot, &stop))?;
    if recorded.slot != input.slot {
        return Err(CancelRefusal::TheBindingIsNotThisSlots {
            slot: String::from(input.slot),
            said: format!("is filed as the slot `{}`", recorded.slot),
        });
    }
    let plan: Option<DestructionPlan> = read_the_plan(input, &recorded)?;
    if let Some(standing_plan) = &plan
        && let Some(owner) = live_plan_owner(standing_plan)
    {
        return Err(CancelRefusal::ALiveRemoval {
            owner,
        });
    }
    return Ok(recorded);
}

/// What:
///  The plan that stands for one clone,
///  proved to name that clone.
/// Why:
///  A plan of another clone is a disagreement between two records,
///  and a plan this build cannot read may describe a destroy nobody can settle.
///
/// # Errors
/// The plan that cannot be read,
/// or one that names another clone than the binding does.
fn read_the_plan(
    input: &TheRecordsProof<'_>,
    recorded: &RecordedBinding,
) -> Result<Option<DestructionPlan>, CancelRefusal> {
    let plan_path: PathBuf = plans_folder(input.common_dir).join(recorded.clone_guid.file_name());
    let plan: Option<DestructionPlan> =
        read_an_existing_plan(input.common_dir, recorded.clone_guid).map_err(|refusal| {
            return CancelRefusal::ThePlanIsUnreadable {
                path: plan_path.clone(),
                said: refusal.message(plan_path.as_path()),
            };
        })?;
    let Some(standing_plan) = &plan else {
        return Ok(None);
    };
    if standing_plan.facts.clone_guid != recorded.clone_guid {
        return Err(CancelRefusal::ThePlanIsOfAnotherClone {
            path: plan_path,
            plan_guid: standing_plan.facts.clone_guid.file_name(),
            recorded_guid: recorded.clone_guid.file_name(),
        });
    }
    if standing_plan.facts.clone_dataset != recorded.clone_dataset {
        return Err(CancelRefusal::TheRecordsDisagree {
            what: String::from("the plan and the binding name different clone datasets"),
            first: standing_plan.facts.clone_dataset.clone(),
            second: recorded.clone_dataset.clone(),
        });
    }
    return Ok(plan);
}

/// What:
///  The clone's own request and the at-least-one-removal-record rule.
/// Why:
///  A local `mount` request beside no plan and no staged record is a pending add and not a
///  removal,
///  and a slot with no removal record at all is a mistyped name rather than a state.
///
/// # Errors
/// A clone that does not answer,
/// a request that is no local one,
/// or a slot with no removal record.
fn read_the_request(
    input: &TheRecordsProof<'_>,
    recorded: &RecordedBinding,
    has_plan: bool,
) -> Result<MountIntent, CancelRefusal> {
    let identity = live_identity(input, recorded)?;
    let intent: MountIntent =
        match read_intent(Some(&identity.properties), recorded.clone_dataset.as_str()) {
            IntentReading::Request(request) => request,
            IntentReading::Absent => {
                return Err(CancelRefusal::TheIntentIsNotALocalRequest {
                    clone: recorded.clone_dataset.clone(),
                    said: String::from("no local value"),
                });
            }
            IntentReading::Inherited {
                from,
            } => {
                return Err(CancelRefusal::TheIntentIsNotALocalRequest {
                    clone: recorded.clone_dataset.clone(),
                    said: format!("an inherited value from `{from}`"),
                });
            }
            IntentReading::Refused(refusal) => {
                return Err(CancelRefusal::TheIntentIsNotALocalRequest {
                    clone: recorded.clone_dataset.clone(),
                    said: refusal.message(),
                });
            }
        };
    let staged: bool = staged_record_stands(input.home, recorded.clone_guid);
    if !has_plan && !staged && intent.operation != MountOperation::Unmount {
        return Err(CancelRefusal::NotBeingRemoved {
            slot: String::from(input.slot),
            found: format!(
                "Its destruction plan is not there, no staged-folder record stands in the account's cache, and the clone's own property asks for `{}`.",
                intent.operation.word()
            ),
        });
    }
    return Ok(intent);
}

/// What:
///  The clone's live identity,
///  proved to be the recorded GUID or refused with the reason.
/// Why:
///  A name the pool answers with another GUID is another object,
///  and a name the pool does not answer is the head-gone state rather than a clone.
///
/// # Errors
/// The identity that differs,
/// or the clone that is gone,
/// renamed or unreadable.
fn live_identity(
    input: &TheRecordsProof<'_>,
    recorded: &RecordedBinding,
) -> Result<monochromatic_zfs_request_to_openzfs_kernel_device::DatasetIdentity, CancelRefusal> {
    let identity = input
        .reads
        .identity(recorded.clone_dataset.as_str())
        .map_err(|reason| return the_clone_absence_arm(input, recorded, &reason))?;
    let live_clone_guid: DatasetGuid =
        dataset_guid_of(identity.stats.dds_guid).map_err(|said| {
            return CancelRefusal::TheCloneDoesNotAnswer {
                dataset: recorded.clone_dataset.clone(),
                reason: said,
            };
        })?;
    if live_clone_guid != recorded.clone_guid {
        return Err(CancelRefusal::TheCloneIsNotTheRecordedOne {
            field: "cloneGuid",
            recorded: recorded.clone_guid.file_name(),
            live: live_clone_guid.file_name(),
        });
    }
    return Ok(identity);
}

/// What:
///  Whether the plan's owner still runs,
///  as `<pid>:<birth identity>`.
/// Why:
///  A plan whose owner runs names a removal that is running,
///  and the command refuses rather than reversing under it,
///  which is what the design's agreement of the plan's owner with the claim's means.
///  An owner this machine cannot probe is neither proved alive nor proved dead,
///  so it is reported and the command refuses.
fn live_plan_owner(plan: &DestructionPlan) -> Option<String> {
    let owner: &str = plan.facts.owner.as_str();
    let (pid_text, birth) = owner.split_once(':')?;
    let pid: i64 = pid_text.parse().ok()?;
    return match process_birth_identity(pid) {
        Ok(None) => None,
        Ok(Some(current)) => (current == birth).then(|| return owner.to_string()),
        Err(error) => Some(format!(
            "{owner} (whether it still runs cannot be decided: {error})"
        )),
    };
}

/// The head-gone arm:
/// the clone's name does not answer,
/// so the absence proof decides whether the head is gone,
/// whether the name holds another object,
/// or whether the pool could not be read.
fn the_clone_absence_arm(
    input: &TheRecordsProof<'_>,
    recorded: &RecordedBinding,
    reason: &str,
) -> CancelRefusal {
    return match the_clone_absence(
        input.absence,
        recorded.clone_dataset.as_str(),
        recorded.clone_guid,
    ) {
        Ok(CloneAbsence::Gone) => CancelRefusal::TheHeadIsGone {
            slot: String::from(input.slot),
            dataset: recorded.clone_dataset.clone(),
            guid: recorded.clone_guid.file_name(),
            said: format!(
                "no child of its pinned parent carries GUID `{}` either",
                recorded.clone_guid.file_name()
            ),
        },
        Ok(CloneAbsence::There {
            name,
            guid,
        }) => CancelRefusal::TheCloneIsNotTheRecordedOne {
            field: "cloneGuid",
            recorded: recorded.clone_guid.file_name(),
            live: format!("{} under `{name}`", guid.file_name()),
        },
        Ok(CloneAbsence::Reused {
            guid,
        }) => CancelRefusal::TheCloneIsNotTheRecordedOne {
            field: "cloneGuid",
            recorded: recorded.clone_guid.file_name(),
            live: guid.file_name(),
        },
        Err(stop) => CancelRefusal::TheCloneDoesNotAnswer {
            dataset: recorded.clone_dataset.clone(),
            reason: format!("{reason}; the absence proof answered {}", stop.message()),
        },
    };
}

/// One dataset GUID the pool reported,
/// or the refusal a GUID 0 owes.
fn dataset_guid_of(guid: u64) -> Result<DatasetGuid, String> {
    if guid == 0 {
        return Err(String::from(
            "the pool reported GUID 0, and no live object has that identity",
        ));
    }
    return DatasetGuid::parse(guid.to_string().as_str())
        .map_err(|_| return String::from("the pool's GUID is not canonical"));
}

/// Whether a staging record for this clone stands in the account's cache,
/// which is one of the three records that make a slot's state a removal's.
fn staged_record_stands(
    home: &Path,
    clone_guid: DatasetGuid,
) -> bool {
    let path: PathBuf = home.join(".cache").join(staging_record_name(clone_guid));
    return std::fs::symlink_metadata(path.as_path()).is_ok();
}

/// One removal's stop as this command's refusal,
/// naming the slot the record belongs to.
fn of_a_removal_stop(
    slot: &str,
    stop: &RemovalStop,
) -> CancelRefusal {
    return match stop {
        RemovalStop::NoBinding {
            reason,
            ..
        } => CancelRefusal::NoBinding {
            slot: String::from(slot),
            reason: reason.clone(),
        },
        other => CancelRefusal::TheBindingIsNotThisSlots {
            slot: String::from(slot),
            said: format!("cannot be read as this removal's ({})", other.message()),
        },
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_proof_tests.rs"]
mod tests;
