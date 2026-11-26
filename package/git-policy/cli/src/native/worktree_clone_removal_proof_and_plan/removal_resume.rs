//! What:
//!  Where a removal starts:
//!  a fresh entry when no plan stands,
//!  an adopted kept plan when one does and the clone still stands,
//!  and the head-gone path when the clone is proved gone.
//! Why:
//!  A removal that was killed,
//! refused or cancelled leaves its plan,
//!  and the next removal of that clone reads it,
//!  proves it,
//!  adopts it under its own nonce,
//!  keeps every line and carries the loss forward
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "How long the plan a cancellation keeps stays";
//!  `doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "Which resume path a removal takes").
//!  A plan with no live intent while the head still stands is refused,
//!  because nothing then proves the unmount it claims,
//!  and a plan whose head is gone is never refused for that reason:
//!  the intent was a property of the clone and went with it.
//!  A clone destroyed outside the wrapper while the plan says `present` or `refused` is the
//!  settling command's case,
//! and a removal refuses it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const started = startTheRemoval({ entry, reads, unmount, clock, probe });
//! ```

use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_mount_protocol::clone_mount_intent::{
    IntentReading,
    read_intent,
};
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_entry::{
    EnteredRemoval,
    PreparedRemoval,
    RemovalClock,
    RemovalEntry,
    enter_the_removal,
    facts_of,
    prepare_the_removal_over_a_plan,
    refuse_a_plan_of_another_clone,
};
use crate::worktree_clone_removal_proof_and_plan::removal_head_destroy::{
    CloneAbsence,
    settle_the_recorded_names,
    the_clone_absence,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::{
    prove_the_adds_registration,
    read_the_recorded_binding,
};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{
    ExpectedUnmount,
    RemovalIntent,
    RemovalUnmount,
    UnmountProof,
    verify_the_answer,
};
use std::path::PathBuf;

/// What:
///  Where one removal starts,
///  and what the rest of it needs from that start.
/// Why:
///  The two paths take different steps:
///  the head-present one unmounts,
/// proves and submits,
///  and the head-gone one settles and finishes without a clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Start = { kind: 'present'; entered } | { kind: 'gone'; plan; planPath; lost; pool };
/// ```
pub enum TheStart {
    /// The clone stands,
    /// so the removal may destroy what its plan holds.
    Present {
        /// The plan this run writes or adopted,
        /// with the names it printed.
        entered: EnteredRemoval,
    },
    /// The clone is proved gone and its plan's head word says a destroy may have been sent.
    Gone {
        /// The plan,
        /// which every one of its names still records.
        plan: DestructionPlan,
        /// The plan's file.
        plan_path: PathBuf,
        /// The names a removal would lose.
        lost: NamesToBeLost,
        /// The pool its dataset names stand in.
        pool: String,
    },
}

/// What:
///  Where one removal starts,
///  by reading the plan that stands and the clone it names.
/// Why:
///  One read order serves both paths:
///  the binding,
///  the plan,
///  then the clone's own absence,
///  and no other fact decides the path.
///
/// # Throws
/// A binding or plan that cannot be read,
/// a plan of another clone,
/// a pool that cannot prove the clone's absence,
/// or the state a removal refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = startTheRemoval({ entry, reads, unmount, clock, probe });
/// ```
pub fn start_the_removal(
    entry: &RemovalEntry<'_>,
    reads: &dyn RemovalPoolReads,
    unmount: &dyn RemovalUnmount,
    clock: &dyn RemovalClock,
    probe: &dyn DatasetReader,
) -> Result<TheStart, RemovalStop> {
    let recorded = read_the_recorded_binding(entry.common_dir, entry.slot)?;
    let standing: Option<DestructionPlan> =
        read_an_existing_plan(entry.common_dir, recorded.clone_guid).map_err(|refusal| {
            return RemovalStop::PlanUnreadable {
                path: crate::worktree_clone_binding_record::registration_home::plans_folder(
                    entry.common_dir,
                )
                .join(recorded.clone_guid.file_name()),
                refusal,
            };
        })?;
    let Some(plan) = standing else {
        return Ok(TheStart::Present {
            entered: enter_the_removal(entry, reads, unmount, clock)?,
        });
    };
    refuse_a_plan_of_another_clone(&plan, &recorded)?;
    let plan_path: PathBuf =
        crate::worktree_clone_binding_record::registration_home::plans_folder(entry.common_dir)
            .join(recorded.clone_guid.file_name());
    let head: HeadWord = plan.head;
    return match the_clone_absence(probe, recorded.clone_dataset.as_str(), recorded.clone_guid)? {
        CloneAbsence::There {
            ..
        } => Ok(TheStart::Present {
            entered: adopt_the_removal(entry, reads, unmount, clock, probe)?,
        }),
        CloneAbsence::Reused {
            guid,
        } => Err(RemovalStop::TheCloneNameWasReused {
            name: recorded.clone_dataset,
            recorded_guid: recorded.clone_guid.file_name(),
            live_guid: guid.file_name(),
        }),
        CloneAbsence::Gone => match head {
            HeadWord::Requested | HeadWord::Unknown | HeadWord::Gone => {
                let lost: NamesToBeLost = NamesToBeLost {
                    slot: String::from(entry.slot),
                    dataset: recorded.clone_dataset.clone(),
                    snapshots: plan
                        .named_snapshots()
                        .iter()
                        .map(|snapshot| return snapshot.name.clone())
                        .collect(),
                    bookmarks: reads
                        .bookmarks_of(recorded.clone_dataset.as_str(), recorded.clone_guid)
                        .map(|objects| {
                            return objects
                                .iter()
                                .map(|object| return object.full_name())
                                .collect();
                        })
                        .unwrap_or_default(),
                };
                Ok(TheStart::Gone {
                    plan,
                    plan_path,
                    lost,
                    pool: recorded.pool,
                })
            }
            HeadWord::Present | HeadWord::Refused => {
                Err(RemovalStop::TheCloneIsTheSettlingCommands {
                    slot: String::from(entry.slot),
                    path: plan_path,
                    head: String::from(head.word()),
                })
            }
        },
    };
}

/// What:
///  The entry's own order over a plan that already stands:
///  prove,
///  request the unmount again,
///  verify it,
///  and rewrite the plan under this run's nonce with every earlier line kept.
/// Why:
///  A retry after a cancel presents the kept plan,
///  adopts it under its own nonce and proceeds to the first deferred check,
///  so the unmount is requested again and the plan's `unmountNonce` names the new receipt.
///  The design's rule that an `unknown` name is settled by the head-present arm runs here,
///  before the rewrite,
///  so the plan never holds a name whose state the live inventory contradicts.
///
/// # Throws
/// A plan with no live intent,
/// a check that refused,
/// or a plan write that refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entered = adoptTheRemoval(entry, reads, unmount, clock, probe);
/// ```
pub fn adopt_the_removal(
    entry: &RemovalEntry<'_>,
    reads: &dyn RemovalPoolReads,
    unmount: &dyn RemovalUnmount,
    clock: &dyn RemovalClock,
    probe: &dyn DatasetReader,
) -> Result<EnteredRemoval, RemovalStop> {
    let (prepared, standing): (PreparedRemoval, Option<DestructionPlan>) =
        prepare_the_removal_over_a_plan(entry, reads, false)?;
    let Some(mut plan) = standing else {
        return Err(RemovalStop::APlanStands {
            slot: String::from(entry.slot),
            path: PathBuf::new(),
        });
    };
    refuse_the_live_intent(probe, plan.facts.clone_dataset.as_str())?;
    let intent: RemovalIntent = RemovalIntent::fresh()?;
    let answer = unmount.request_the_unmount(&intent.mount_intent())?;
    let proof: UnmountProof = verify_the_answer(
        &answer,
        &ExpectedUnmount {
            clone_guid: prepared.recorded.clone_guid,
            nonce: intent.nonce.clone(),
        },
    )?;
    prove_the_adds_registration(entry.common_dir, &prepared.recorded)?;
    plan.facts = facts_of(
        &prepared.recorded,
        &prepared.proven,
        &proof,
        prepared.registration,
        clock,
    )?;
    settle_the_recorded_names(&mut plan, prepared.proven.inventory.objects.as_slice())?;
    let folder: PlanFolder = PlanFolder::open_or_create(entry.common_dir).map_err(|refusal| {
        return RemovalStop::PlanUnreadable {
            path: crate::worktree_clone_binding_record::registration_home::plans_folder(
                entry.common_dir,
            ),
            refusal,
        };
    })?;
    let plan_path: PathBuf = folder.publish(&plan).map_err(|refusal| {
        return RemovalStop::ThePlanCouldNotHoldTheAnswer {
            path: folder.path().join(prepared.recorded.clone_guid.file_name()),
            said: refusal.message(folder.path()),
        };
    })?;
    let lost: NamesToBeLost = NamesToBeLost {
        slot: String::from(entry.slot),
        dataset: prepared.recorded.clone_dataset.clone(),
        snapshots: plan
            .named_snapshots()
            .iter()
            .map(|snapshot| return snapshot.name.clone())
            .collect(),
        bookmarks: prepared.lost.bookmarks.clone(),
    };
    return Ok(EnteredRemoval {
        pool: prepared.recorded.pool,
        plan,
        plan_path,
        lost,
    });
}

/// What:
///  Whether the clone's own desired-state property still asks for something a removal may go on
///  from.
/// Why:
///  A plan with no live intent while the head still stands is refused,
///  because nothing then proves the unmount it claims;
///  a live `mount` intent is the reattach of a removal that cancelled itself,
///  so a plan beside it is adopted.
///  A value this route cannot read authorizes nothing.
///
/// # Throws
/// No intent,
/// an intent that cannot be read,
/// or a pool that does not answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseTheLiveIntent(probe, clone);
/// ```
pub fn refuse_the_live_intent(
    probe: &dyn DatasetReader,
    clone: &str,
) -> Result<(), RemovalStop> {
    let identity = probe.identity(clone).map_err(|failure| {
        return RemovalStop::DoesNotAnswer {
            object: String::from(clone),
            reason: failure.message(),
        };
    })?;
    return match read_intent(Some(&identity.properties), clone) {
        IntentReading::Request(_intent) => Ok(()),
        IntentReading::Absent => Err(RemovalStop::NoLiveIntent {
            clone: String::from(clone),
        }),
        IntentReading::Inherited {
            from,
        } => Err(RemovalStop::NoLiveIntent {
            clone: format!("{clone} (inherited from {from})"),
        }),
        IntentReading::Refused(refusal) => Err(RemovalStop::NoLiveIntent {
            clone: format!("{clone} ({})", refusal.message()),
        }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_resume_tests.rs"]
mod tests;
