//! What:
//!  One removal's entry,
//!  in the one order its proof,
//!  intent,
//!  unmount and plan take:
//!  read the add's record,
//!  read what stands on the registration's lock,
//!  prove the lineage,
//!  apply the bookmark rule,
//!  record the `unmount` intent,
//!  verify the answer,
//!  and write the destruction plan before printing the names to be lost.
//! Why:
//!  The design orders these steps and gives each of them its moment:
//!  the first write sits after the unmount and the proof,
//!  because a plan written earlier would name an unmount that never happened,
//!  and the refusal before the first delete is what keeps every snapshot and bookmark when
//!  anything is unproved
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  section 3's list and "The removal intent";
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//! "Removal and recovery",
//! step 2).
//!  Nothing here deletes:
//!  the snapshot batch and the head destroy are step 16's,
//!  and this entry stops at the plan and the list of names.
//!  A removal that finds a plan already standing for its clone stops and names it,
//!  because adopting a kept plan,
//!  re-proving it and carrying its loss forward is
//!  `removal_resume::adopt_the_removal`'s own path.
//!  A lock on the registration whose words are not this add's refuses here too,
//!  before the first write:
//!  the Git settle is the one step of the removal's order that reads `locked`,
//!  and it comes after every destroy,
//!  so a person's lock met only there would be met too late
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "How doctor and removal tell the states apart").
//!  Force changes two of those steps and nothing else:
//!  two forces delete a person's lock file and then run the removal,
//!  and a proof that cannot prove the clone is replaced by the live identities when
//!  two forces were given,
//!  so the destroy is what the owner answered on 2026-11-28 it is
//!  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`).
//!  The first write,
//!  the intent and the unmount are what the removal a person's
//!  `git worktree remove` runs:
//!  `removal_command_path::complete_the_bound_clones_removal` hands this entry the
//!  live reads,
//!  the live unmount request,
//!  the live clock and the force count.
//!  The three process facts the plan records are `removal_clock.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entered = enterTheRemoval({ entry, reads, unmount, clock, force, forcedBefore });
//! console.log(entered.lost.text());
//! ```

use crate::subvolume::fs_beneath::ObjectId;
use crate::worktree_clone_removal_proof_and_plan::bookmark_rule::{
    explanations_of,
    refuse_the_unexplained_bookmarks,
};
use crate::worktree_clone_removal_proof_and_plan::lineage_proof::{
    ProvenRemoval,
    prove_the_lineage,
    prove_the_live_lineage,
};
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanFacts,
};
use crate::worktree_clone_removal_proof_and_plan::removal_clock::RemovalClock;
use crate::worktree_clone_removal_proof_and_plan::removal_force::RemovalForce;
use crate::worktree_clone_removal_proof_and_plan::removal_forced_acts::{
    ForcedActs,
    forced_past_a_proof,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::{
    RecordedBinding,
    prove_the_adds_registration,
    read_the_origin_inventory,
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
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The removal's own context:
///  the repository's common Git directory and the slot it removes.
/// Why:
///  Every record the entry reads is found from the common directory,
///  and the slot's name is the key of the add's binding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { commonDir: string; slot: string };
/// ```
pub struct RemovalEntry<'a> {
    /// The common Git directory of the repository whose clone this is.
    pub common_dir: &'a Path,
    /// The slot the removal is about.
    pub slot: &'a str,
}

/// What:
///  Everything one entry of a removal reads and drives.
/// Why:
///  The order's first two steps ask six things of their caller:
///  the slot,
///  the pool,
///  the unmount request,
///  the clock,
///  the force count,
///  and the acts the resume already made because of a force.
///  A control answers everyone of them,
///  so no control opens this machine's device or unmounts anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EnteringTheRemoval = { entry, reads, unmount, clock, force, forcedBefore };
/// ```
pub struct EnteringTheRemoval<'a> {
    /// The repository and slot this removal is about.
    pub entry: &'a RemovalEntry<'a>,
    /// The pool reads the proof makes.
    pub reads: &'a dyn RemovalPoolReads,
    /// The request that takes the clone down.
    pub unmount: &'a dyn RemovalUnmount,
    /// The process facts the plan's header carries.
    pub clock: &'a dyn RemovalClock,
    /// How many forces this command carried.
    pub force: RemovalForce,
    /// What the resume did because of a force before this entry ran,
    /// which is the plan file a forced run went past.
    pub forced_before: ForcedActs,
}

/// What:
///  What one entered removal holds:
///  the plan it wrote,
///  the file it published at,
///  the names it printed as lost,
///  and the pool its requests name.
/// Why:
///  The step after this one reads the plan's tuples and submits them,
///  and the person reads the list,
///  so both leave this call with their own value.
///  The pool comes with them because the plan records dataset names and no pool,
///  and a snapshot request names one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entered = { pool; plan; planPath; lost; forced };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnteredRemoval {
    /// The pool the plan's datasets stand in,
    /// as the add's record named it.
    pub pool: String,
    /// The plan this removal wrote.
    pub plan: DestructionPlan,
    /// The file the plan was published at.
    pub plan_path: PathBuf,
    /// The names this removal printed as lost.
    pub lost: NamesToBeLost,
    /// What this run did only because of a force.
    pub forced: ForcedActs,
}

/// What:
///  Enter one removal:
///  prove,
///  refuse or go on,
///  request the unmount,
///  verify it,
///  write the plan,
///  and print the loss.
/// Why:
///  Every refusal returns before the plan is written and before anything is deleted,
///  and the order is the design's own.
///  A plan that already stands is named and left,
///  because the removal that wrote it continues from it.
///
/// # Throws
/// The check that refused,
/// with the object it named and the sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entered = enterTheRemoval(entry, reads, unmount, clock);
/// ```
pub fn enter_the_removal(
    entry: &RemovalEntry<'_>,
    reads: &dyn RemovalPoolReads,
    unmount: &dyn RemovalUnmount,
    clock: &dyn RemovalClock,
) -> Result<EnteredRemoval, RemovalStop> {
    return enter_the_removal_with_force(&EnteringTheRemoval {
        entry,
        reads,
        unmount,
        clock,
        force: RemovalForce::none(),
        forced_before: ForcedActs::nothing(),
    });
}

/// What:
///  The same entry over the force a person's command carried,
///  and the acts its resume already made because of one.
/// Why:
///  Two forces delete a person's lock and pass a clone the wrapper cannot
///  prove,
///  so the entry has to know the count and the acts before it began,
///  and a control hands both in.
///
/// # Throws
/// The check that refused,
/// with the object it named and the sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entered = enterTheRemovalWithForce({ entry, reads, unmount, clock, force, forcedBefore });
/// ```
pub fn enter_the_removal_with_force(
    request: &EnteringTheRemoval<'_>
) -> Result<EnteredRemoval, RemovalStop> {
    let prepared: PreparedRemoval = prepare_the_removal_over_a_plan(
        &ThePreparedInputs {
            entry: request.entry,
            reads: request.reads,
            force: request.force,
            forced_before: request.forced_before.clone(),
        },
        true,
    )
    .map(|(prepared, _standing)| return prepared)?;
    let intent: RemovalIntent = RemovalIntent::fresh()?;
    let answer = request
        .unmount
        .request_the_unmount(&intent.mount_intent())?;
    let proof: UnmountProof = verify_the_answer(
        &answer,
        &ExpectedUnmount {
            clone_guid: prepared.recorded.clone_guid,
            nonce: intent.nonce.clone(),
        },
    )?;
    // The design verifies the clone GUID, the underlay identity and the Git registration again after the unmount, so a
    // registration swapped during the request stops the plan rather than the destroy. The proof is the same identity
    // comparison the entry made before the request, so a registration whose inode and birth time changed refuses here.
    prove_the_adds_registration(request.entry.common_dir, &prepared.recorded)?;
    let facts: PlanFacts = facts_of(
        &prepared.recorded,
        &prepared.proven,
        &proof,
        prepared.registration,
        request.clock,
    )?;
    let plan: DestructionPlan =
        DestructionPlan::first_write(facts, prepared.proven.own_snapshots.as_slice());
    let folder: PlanFolder =
        PlanFolder::open_or_create(request.entry.common_dir).map_err(|refusal| {
            return RemovalStop::PlanUnreadable {
                slot: String::from(request.entry.slot),
                path: crate::worktree_clone_binding_record::registration_home::plans_folder(
                    request.entry.common_dir,
                ),
                refusal,
            };
        })?;
    let plan_path: PathBuf = folder.publish(&plan).map_err(|refusal| {
        return RemovalStop::PlanUnreadable {
            slot: String::from(request.entry.slot),
            path: folder.path().join(prepared.recorded.clone_guid.file_name()),
            refusal,
        };
    })?;
    return Ok(EnteredRemoval {
        pool: prepared.recorded.pool,
        plan,
        plan_path,
        lost: prepared.lost,
        forced: prepared.forced,
    });
}

/// What:
///  Every step of one removal's order that changes nothing:
///  the add's record,
///  the lock standing on its registration,
///  the registration's own identity,
///  the origin-side inventory,
///  a plan that already stands,
///  the lineage proof,
///  the bookmark rule,
///  and the names a removal would lose.
/// Why:
///  The read-only reading and the writing entry take these steps in one order,
///  so a refusal cannot name one check here and another there,
///  and everything a removal decides before its first write is decided in one place.
///
/// # Throws
/// The check that refused,
/// with the object it named and the sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const prepared = prepareTheRemoval(entry, reads);
/// ```
pub(super) fn prepare_the_removal(
    entry: &RemovalEntry<'_>,
    reads: &dyn RemovalPoolReads,
) -> Result<PreparedRemoval, RemovalStop> {
    let (prepared, _standing): (PreparedRemoval, Option<DestructionPlan>) =
        prepare_the_removal_over_a_plan(
            &ThePreparedInputs {
                entry,
                reads,
                force: RemovalForce::none(),
                forced_before: ForcedActs::nothing(),
            },
            true,
        )?;
    return Ok(prepared);
}

/// What:
///  What the steps before a removal's first write are made over.
/// Why:
///  The add's record,
///  the pool,
///  the force count and the acts a force already made are read by every one of
///  them,
///  and one value keeps a caller from pairing a scripted pool with a live force.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ThePreparedInputs = { entry, reads, force, forcedBefore };
/// ```
pub(super) struct ThePreparedInputs<'a> {
    /// The repository and slot this removal is about.
    pub(super) entry: &'a RemovalEntry<'a>,
    /// The pool reads the proof makes.
    pub(super) reads: &'a dyn RemovalPoolReads,
    /// How many forces this command carried.
    pub(super) force: RemovalForce,
    /// What the resume did because of a force before this entry ran.
    pub(super) forced_before: ForcedActs,
}

/// What:
///  Every step of one removal's order that changes nothing,
///  with the plan that already stands for this clone,
///  which a resumed removal adopts and a fresh entry refuses.
/// Why:
///  The fresh entry refuses a standing plan and a resumed removal adopts one,
///  and both take the same reads in the same order,
///  so the refusal cannot name one check here and another there.
///  Force joins these steps in two places:
///  a lock two forces delete, whose line the report prints,
///  and a proof that cannot prove the clone, which two forces replace with the
///  pool's own answer and one force or none refuses.
///
/// # Throws
/// The check that refused,
/// with the object it named and the sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [prepared, standing] = prepareTheRemovalOverAPlan(request, true);
/// ```
pub(super) fn prepare_the_removal_over_a_plan(
    request: &ThePreparedInputs<'_>,
    refuse_a_standing_plan: bool,
) -> Result<(PreparedRemoval, Option<DestructionPlan>), RemovalStop> {
    let entry: &RemovalEntry<'_> = request.entry;
    let recorded: RecordedBinding = read_the_recorded_binding(entry.common_dir, entry.slot)?;
    let mut forced: ForcedActs = request.forced_before.clone();
    if let Some(line) = super::the_lock_a_removal_refuses::the_locks_answer(
        entry.common_dir,
        entry.slot,
        request.force,
    )? {
        forced.lines.push(line);
    }
    let registration: ObjectId = prove_the_adds_registration(entry.common_dir, &recorded)?;
    let origin_inventory = read_the_origin_inventory(entry.common_dir, &recorded)?;
    let standing: Option<DestructionPlan> =
        read_an_existing_plan(entry.common_dir, recorded.clone_guid).map_err(|refusal| {
            return RemovalStop::PlanUnreadable {
                slot: String::from(entry.slot),
                path: crate::worktree_clone_binding_record::registration_home::plans_folder(
                    entry.common_dir,
                )
                .join(recorded.clone_guid.file_name()),
                refusal,
            };
        })?;
    let history: RemovalHistory = history_of(standing.as_ref());
    if let Some(plan) = &standing {
        // The plan's own identities must be this clone's before its tuples explain a bookmark.
        refuse_a_plan_of_another_clone(plan, &recorded)?;
        if refuse_a_standing_plan {
            let path: PathBuf =
                crate::worktree_clone_binding_record::registration_home::plans_folder(
                    entry.common_dir,
                )
                .join(recorded.clone_guid.file_name());
            return Err(RemovalStop::APlanStands {
                slot: String::from(entry.slot),
                path,
            });
        }
    }
    let proven: ProvenRemoval =
        match prove_the_lineage(request.reads, &recorded, &origin_inventory, &history) {
            Ok(proven) => proven,
            Err(stop) => the_proof_with_force(request, &recorded, stop, &mut forced)?,
        };
    let explanations = explanations_of(&proven.inventory, standing.as_ref());
    let refused = refuse_the_unexplained_bookmarks(
        &explanations,
        proven.inventory.bookmarks().as_slice(),
        &origin_inventory,
    );
    // Force replaces the proof, so no plan line explains a bookmark, and a forced removal destroys what the pool lists
    //  for the clone; the report names every bookmark that goes with the head (`loss_names`).
    if !refused.is_empty() && !request.force.is_given() {
        return Err(RemovalStop::BookmarksRefused {
            clone: recorded.clone_dataset.clone(),
            refused,
        });
    }
    let lost: NamesToBeLost = NamesToBeLost {
        slot: String::from(entry.slot),
        dataset: recorded.clone_dataset.clone(),
        snapshots: proven
            .own_snapshots
            .iter()
            .map(|snapshot| return snapshot.name.clone())
            .collect(),
        bookmarks: proven
            .inventory
            .bookmarks()
            .iter()
            .map(|bookmark| return bookmark.full_name())
            .collect(),
    };
    return Ok((
        PreparedRemoval {
            recorded,
            registration,
            proven,
            lost,
            forced,
        },
        standing,
    ));
}

/// What:
///  The proof a removal goes on with when the pool did not prove the clone.
/// Why:
///  Two forces pass a clone the wrapper cannot prove,
///  and the owner answered on 2026-11-28 that such a removal
///  "destroys the dataset the slot's record names even when the pool's answer
///  does not match the record"
///  (`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`).
///  One force or none refuses,
///  and the sentence names two forces as the way through.
///  A refusal that is not an identity a force passes is unchanged:
///  a pool that does not answer,
///  a listing that could not be read,
///  and the transferred history or the origin-side object a proof of a clone
///  that stands does refuse.
///
/// # Throws
/// The stop the proof made,
/// or the stop that names the two forces a person would need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proven = theProofWithForce(request, recorded, stop);
/// ```
fn the_proof_with_force(
    request: &ThePreparedInputs<'_>,
    recorded: &RecordedBinding,
    stop: RemovalStop,
    forced: &mut ForcedActs,
) -> Result<ProvenRemoval, RemovalStop> {
    if !a_forced_removal_passes(&stop) {
        return Err(stop);
    }
    if !request.force.is_twice() {
        return Err(RemovalStop::TheCloneCannotBeProved {
            slot: String::from(request.entry.slot),
            cause: the_disagreement_in(&stop),
            recorded_guid: recorded.clone_guid.file_name(),
            answered_guid: the_pools_answer_in(&stop),
        });
    }
    let proven: ProvenRemoval = prove_the_live_lineage(request.reads, recorded)?;
    // Both GUIDs are kept, because the report is the only trace of a destroy nothing proved.
    let acts: ForcedActs = forced_past_a_proof(
        recorded.clone_dataset.as_str(),
        recorded.clone_guid.file_name().as_str(),
        proven.identities.clone.guid.file_name().as_str(),
    );
    forced.lines.extend(acts.lines);
    forced.recorded_guid = acts.recorded_guid;
    forced.answered_guid = acts.answered_guid;
    return Ok(proven);
}

/// What:
///  Whether a force passes the refusal a proof made.
/// Why:
///  Only a proof that found another object than the record names is what two
///  forces go past,
///  and the record's risk is exactly that case:
///  a GUID that differs or a name that was reused.
///  A pool that cannot answer is no such state,
///  and nothing about this route's other refusals is decided here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const passes = (stop) => stop.kind === 'identity-differs' || stop.kind === 'origin-is-not-the-recorded-one';
/// ```
fn a_forced_removal_passes(stop: &RemovalStop) -> bool {
    return matches!(
        stop,
        RemovalStop::IdentityDiffers { .. } | RemovalStop::OriginIsNotTheRecordedOne { .. }
    );
}

/// What:
///  The disagreement one proof made,
///  as the clause the refusal that names two forces prints.
/// Why:
///  The refusal's own sentence is long and promises what nothing was done to,
///  and a clause beside "the clone cannot be proved" has to say which two values
///  disagreed rather than repeat that promise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cause = theDisagreementIn(stop);
/// ```
fn the_disagreement_in(stop: &RemovalStop) -> String {
    return match stop {
        RemovalStop::IdentityDiffers {
            object,
            field,
            recorded,
            live,
        } => format!(
            "the pool answers `{live}` for `{object}`'s `{field}`, where the add recorded `{recorded}`"
        ),
        RemovalStop::OriginIsNotTheRecordedOne {
            clone,
            recorded,
            live,
        } => format!(
            "the clone `{clone}` has origin `{}`, where the add recorded `{recorded}`",
            live.as_deref().unwrap_or("none")
        ),
        other => String::from(other.message()),
    };
}

/// The GUID the pool answered in an identity refusal,
/// when the refusal compared the clone's own GUID.
fn the_pools_answer_in(stop: &RemovalStop) -> Option<String> {
    return match stop {
        RemovalStop::IdentityDiffers {
            field: "cloneGuid",
            live,
            ..
        } => Some(live.clone()),
        _ => None,
    };
}

/// What:
///  One removal's proof and loss,
///  with the identities that stage proved and the acts force made.
/// Why:
///  The entry writes its plan from exactly these values and the reading prints them,
///  so neither can hold a clone of one reading and an origin of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prepared = { recorded; registration; proven; lost; forced };
/// ```
pub(super) struct PreparedRemoval {
    /// The add's record as the removal read it.
    pub(super) recorded: RecordedBinding,
    /// The registration's live identity,
    /// which the plan's `admin` field records.
    pub(super) registration: ObjectId,
    /// The proof's identities,
    /// inventory and own snapshots.
    pub(super) proven: ProvenRemoval,
    /// The names a removal would lose.
    pub(super) lost: NamesToBeLost,
    /// What this run did only because of a force.
    pub(super) forced: ForcedActs,
}

/// What:
///  The loss a standing plan already records.
/// Why:
///  A refusal's sentence has to say what this clone has already lost,
///  and only the plan knows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const history = RemovalHistory.ofDestroyed(plan.destroyedNames());
/// ```
pub(super) fn history_of(plan: Option<&DestructionPlan>) -> RemovalHistory {
    let Some(found) = plan else {
        return RemovalHistory::NothingDestroyed;
    };
    let mut names: Vec<String> = Vec::new();
    for line in &found.attempts {
        if line.outcome
            == crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome::Destroyed
            && !names.contains(&line.snapshot.name)
        {
            names.push(line.snapshot.name.clone());
        }
    }
    return RemovalHistory::of_destroyed(names);
}

/// Whether one standing plan is about this clone and origin.
///
/// # Throws
/// The field in which the plan and the add's record disagree.
pub(super) fn refuse_a_plan_of_another_clone(
    plan: &DestructionPlan,
    recorded: &RecordedBinding,
) -> Result<(), RemovalStop> {
    let pairs: [(&'static str, String, String); 6] = [
        (
            "cloneDataset",
            plan.facts.clone_dataset.clone(),
            recorded.clone_dataset.clone(),
        ),
        (
            "cloneGuid",
            plan.facts.clone_guid.file_name(),
            recorded.clone_guid.file_name(),
        ),
        (
            "originDataset",
            plan.facts.origin_dataset.clone(),
            recorded.origin_dataset.clone(),
        ),
        (
            "originDatasetGuid",
            plan.facts.origin_dataset_guid.file_name(),
            recorded.origin_dataset_guid.file_name(),
        ),
        (
            "originSnapshot",
            plan.facts.origin_snapshot.clone(),
            recorded.origin_snapshot.clone(),
        ),
        (
            "originSnapshotGuid",
            plan.facts.origin_snapshot_guid.file_name(),
            recorded.origin_snapshot_guid.file_name(),
        ),
    ];
    for (field, plan_value, recorded_value) in pairs {
        if plan_value != recorded_value {
            return Err(RemovalStop::IdentityDiffers {
                object: String::from("the standing destruction plan"),
                field,
                recorded: recorded_value,
                live: plan_value,
            });
        }
    }
    return Ok(());
}

/// What:
///  The plan's own fields,
///  from the identities the proof matched,
///  the answer it verified and the add's record.
/// Why:
///  The plan copies the identities rather than referring to them,
///  because it must stand alone for everything it destroys.
///
/// # Throws
/// A clock fact this process cannot give,
/// which is the only field that is not a record's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = factsOf(recorded, proven, proof, registration, clock);
/// ```
pub(super) fn facts_of(
    recorded: &RecordedBinding,
    proven: &ProvenRemoval,
    proof: &UnmountProof,
    registration: ObjectId,
    clock: &dyn RemovalClock,
) -> Result<PlanFacts, RemovalStop> {
    return Ok(PlanFacts {
        run: clock.run()?,
        owner: clock.owner()?,
        started: clock.started(),
        clone_dataset: recorded.clone_dataset.clone(),
        clone_guid: proven.identities.clone.guid,
        clone_createtxg: proven.identities.clone.createtxg,
        origin_dataset: recorded.origin_dataset.clone(),
        origin_dataset_guid: proven.identities.origin_dataset.guid,
        origin_snapshot: recorded.origin_snapshot.clone(),
        origin_snapshot_guid: proven.identities.origin_snapshot.guid,
        origin_snapshot_createtxg: proven.identities.origin_snapshot.createtxg,
        registration_id: recorded.registration_id.clone(),
        admin: registration.encode(),
        target: recorded.slot_path.clone(),
        unmount_nonce: proof.nonce.clone(),
        unmount_mount_id: proof.mount_id,
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_entry_tests.rs"]
mod tests;
