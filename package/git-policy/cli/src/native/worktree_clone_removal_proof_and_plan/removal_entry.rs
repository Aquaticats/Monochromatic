//! What:
//!  One removal's entry,
//!  in the one order its proof,
//!  intent,
//!  unmount and plan take:
//!  read the add's record and the origin-side inventory,
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
//!  The first write,
//!  the intent and the unmount are what the removal a person's
//!  `git worktree remove` runs:
//!  `removal_command_path::complete_the_bound_clones_removal` hands this entry the
//!  live reads,
//!  the live unmount request and the live clock.
//!  The three process facts the plan records are `removal_clock.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entered = enterTheRemoval(context, reads, unmount, clock);
//! console.log(entered.lost.text());
//! ```

use crate::subvolume::fs_beneath::ObjectId;
use crate::worktree_clone_binding_record::lock_words::{
    LockStanding,
    lock_standing_of,
};
use crate::worktree_clone_managed_lock::lock_and_pointer_readings::read_the_lock;
use crate::worktree_clone_managed_lock::reading_a_bounded_file::Reading;
use crate::worktree_clone_managed_lock::recorded_lock_facts::read_the_recorded_facts;
use crate::worktree_clone_removal_proof_and_plan::bookmark_rule::{
    explanations_of,
    refuse_the_unexplained_bookmarks,
};
use crate::worktree_clone_removal_proof_and_plan::lineage_proof::{
    ProvenRemoval,
    prove_the_lineage,
};
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{
    DestructionPlan,
    PlanFacts,
};
use crate::worktree_clone_removal_proof_and_plan::removal_clock::RemovalClock;
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
/// type Entered = { pool; plan; planPath; lost };
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
    let prepared: PreparedRemoval = prepare_the_removal(entry, reads)?;
    let intent: RemovalIntent = RemovalIntent::fresh()?;
    let answer = unmount.request_the_unmount(&intent.mount_intent())?;
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
    prove_the_adds_registration(entry.common_dir, &prepared.recorded)?;
    let facts: PlanFacts = facts_of(
        &prepared.recorded,
        &prepared.proven,
        &proof,
        prepared.registration,
        clock,
    )?;
    let plan: DestructionPlan =
        DestructionPlan::first_write(facts, prepared.proven.own_snapshots.as_slice());
    let folder: PlanFolder = PlanFolder::open_or_create(entry.common_dir).map_err(|refusal| {
        return RemovalStop::PlanUnreadable {
            path: crate::worktree_clone_binding_record::registration_home::plans_folder(
                entry.common_dir,
            ),
            refusal,
        };
    })?;
    let plan_path: PathBuf = folder.publish(&plan).map_err(|refusal| {
        return RemovalStop::PlanUnreadable {
            path: folder.path().join(prepared.recorded.clone_guid.file_name()),
            refusal,
        };
    })?;
    return Ok(EnteredRemoval {
        pool: prepared.recorded.pool,
        plan,
        plan_path,
        lost: prepared.lost,
    });
}

/// What:
///  Every step of one removal's order that changes nothing:
///  the add's record,
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
        prepare_the_removal_over_a_plan(entry, reads, true)?;
    return Ok(prepared);
}

/// What:
///  Every step of one removal's order that changes nothing,
///  with the plan that already stands for this clone,
///  which a resumed removal adopts and a fresh entry refuses.
/// Why:
///  The fresh entry refuses a standing plan and a resumed removal adopts one,
///  and both take the same reads in the same order,
///  so the refusal cannot name one check here and another there.
///
/// # Throws
/// The check that refused,
/// with the object it named and the sentence a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [prepared, standing] = prepareTheRemovalOverAPlan(entry, reads);
/// ```
pub(super) fn prepare_the_removal_over_a_plan(
    entry: &RemovalEntry<'_>,
    reads: &dyn RemovalPoolReads,
    refuse_a_standing_plan: bool,
) -> Result<(PreparedRemoval, Option<DestructionPlan>), RemovalStop> {
    let recorded: RecordedBinding = read_the_recorded_binding(entry.common_dir, entry.slot)?;
    refuse_a_lock_this_add_did_not_write(entry)?;
    let registration: ObjectId = prove_the_adds_registration(entry.common_dir, &recorded)?;
    let origin_inventory = read_the_origin_inventory(entry.common_dir, &recorded)?;
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
    let proven: ProvenRemoval = prove_the_lineage(reads, &recorded, &origin_inventory, &history)?;
    let explanations = explanations_of(&proven.inventory, standing.as_ref());
    let refused = refuse_the_unexplained_bookmarks(
        &explanations,
        proven.inventory.bookmarks().as_slice(),
        &origin_inventory,
    );
    if !refused.is_empty() {
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
        },
        standing,
    ));
}

/// What:
///  The refusal a removal owes a lock whose words are not its add's,
///  read before anything is written.
/// Why:
///  Git refuses a locked registration to its own removal and refuses it to one force,
///  and its own message therefore steers a person to `-f -f`
///  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
///  "What Git 2.56.1 does"),
///  which here runs this wrapper's removal instead of Git's.
///  The wrapper's order reads the lock at its Git settle alone,
///  which comes after the snapshot batch,
///  the head destroy and the origin attempt,
///  and the settle refuses words it cannot recompose from the add's nonce
///  (the same design's S3);
///  so a person's lock met there would be met after the clone's history went.
///  The lock is therefore read here,
///  under the slot's claim,
///  and words this add did not write refuse the removal as the add's own resume refuses
///  them.
///
/// # Throws
/// The lock's own words,
/// or the read that refused them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseALockThisAddDidNotWrite(entry);
/// ```
fn refuse_a_lock_this_add_did_not_write(entry: &RemovalEntry<'_>) -> Result<(), RemovalStop> {
    let home: PathBuf =
        crate::worktree_clone_binding_record::registration_home::home_folder(entry.common_dir);
    let facts = read_the_recorded_facts(entry.common_dir, entry.slot).map_err(|refusal| {
        return RemovalStop::NoBinding {
            slot: String::from(entry.slot),
            reason: refusal.words(entry.slot, home.as_path()),
        };
    })?;
    let Some(registration) = facts.registration.as_ref() else {
        return Ok(());
    };
    let cause: String = match read_the_lock(registration) {
        Reading::Absent => return Ok(()),
        Reading::Refused(refused) => refused.words(),
        Reading::Bytes(bytes) => match lock_standing_of(bytes.as_slice(), &facts.nonce) {
            LockStanding::Verified
            | LockStanding::CutShort {
                ..
            } => return Ok(()),
            LockStanding::Foreign => {
                let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
                let line: &str = text.lines().next().unwrap_or("");
                if line.is_empty() {
                    String::from(
                        "its file is empty, which is also what a person's `git worktree lock` with no reason leaves",
                    )
                } else {
                    format!("its words are `{line}`")
                }
            }
        },
    };
    return Err(RemovalStop::TheLocksWordsAreNotThisAdds {
        slot: String::from(entry.slot),
        cause,
    });
}

/// What:
///  One removal's proof and loss,
///  with the identities that stage proved.
/// Why:
///  The entry writes its plan from exactly these values and the reading prints them,
///  so neither can hold a clone of one reading and an origin of another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prepared = { recorded; registration; proven; lost };
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
