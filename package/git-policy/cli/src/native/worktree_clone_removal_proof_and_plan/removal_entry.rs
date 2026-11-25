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
//!  re-proving it and carrying its loss forward is the resume the later steps build.
//!  The first write,
//!  the intent and the unmount are switched on by plan step 16's wiring:
//!  this release's removal command runs `removal_reading::read_the_removal` and writes nothing,
//!  and step 16 replaces that call with this entry once the snapshot batch,
//!  the head destroy,
//!  the origin cleanup and the Git settle exist to finish or cancel what it starts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entered = enterTheRemoval(context, reads, unmount, clock);
//! console.log(entered.lost.text());
//! ```

use crate::subvolume::fs_beneath::ObjectId;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::bookmark_rule::{explanations_of, refuse_the_unexplained_bookmarks};
use crate::worktree_clone_removal_proof_and_plan::lineage_proof::{ProvenRemoval, prove_the_lineage};
use crate::worktree_clone_removal_proof_and_plan::loss_names::NamesToBeLost;
use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::{DestructionPlan, PlanFacts, owner_text};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::{RecordedBinding, prove_the_adds_registration, read_the_origin_inventory, read_the_recorded_binding};
use crate::worktree_clone_removal_proof_and_plan::removal_reads::RemovalPoolReads;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::started_moment::now_with_offset;
use crate::worktree_clone_removal_proof_and_plan::unmount_request::{ExpectedUnmount, RemovalIntent, RemovalUnmount, UnmountProof, verify_the_answer};
use crate::worktree_removal_deferred_origin_check::removal_history::RemovalHistory;
use std::path::{Path, PathBuf};

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
///  The three facts a plan's header takes from the process rather than from a record:
///  the run's nonce,
///  the writing process and the moment.
/// Why:
///  A control that planted these would be reading its own constants back,
///  and the field's meaning is the process that wrote it,
///  so the seam is this trait with one live answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemovalClock { run(); owner(); started(); }
/// ```
pub trait RemovalClock {
    /// The removal run's own 32-digit nonce.
    ///
    /// # Throws
    ///
    /// The random source failed.
    fn run(&self) -> Result<RequestNonce, RemovalStop>;

    /// The process that writes the plan,
    /// spelled `<pid>:linux:<start time>`.
    ///
    /// # Throws
    ///
    /// The process's own birth identity cannot be read.
    fn owner(&self) -> Result<String, RemovalStop>;

    /// The moment the removal started.
    fn started(&self) -> String;
}

/// What:
///  The release answer:
///  the operating system's clock,
///  this process and its birth identity.
/// Why:
///  The fields are facts a person reads,
///  and no reader decides anything by them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clock = TheLiveClock;
/// ```
pub struct TheLiveClock;

impl RemovalClock for TheLiveClock {
    fn run(&self) -> Result<RequestNonce, RemovalStop> {
        return RequestNonce::fresh().map_err(|refusal| return RemovalStop::NoRunNonce { reason: refusal.message() });
    }

    fn owner(&self) -> Result<String, RemovalStop> {
        let identity: String = crate::process::process_identity::current_birth_identity().map_err(|error| return RemovalStop::NoOwnerIdentity { reason: format!("{error:?}") })?;
        return Ok(owner_text(std::process::id(), identity.as_str()));
    }

    fn started(&self) -> String {
        return now_with_offset();
    }
}

/// What:
///  What one entered removal holds:
///  the plan it wrote,
///  the file it published at,
///  and the names it printed as lost.
/// Why:
///  The step after this one reads the plan's tuples and submits them,
///  and the person reads the list,
///  so both leave this call with their own value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entered = { plan; planPath; lost };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnteredRemoval {
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
pub fn enter_the_removal(entry: &RemovalEntry<'_>, reads: &dyn RemovalPoolReads, unmount: &dyn RemovalUnmount, clock: &dyn RemovalClock) -> Result<EnteredRemoval, RemovalStop> {
    let prepared: PreparedRemoval = prepare_the_removal(entry, reads)?;
    let intent: RemovalIntent = RemovalIntent::fresh()?;
    let answer = unmount.request_the_unmount(&intent.mount_intent())?;
    let proof: UnmountProof = verify_the_answer(&answer, &ExpectedUnmount { clone_guid: prepared.recorded.clone_guid, nonce: intent.nonce.clone() })?;
    // The design verifies the clone GUID, the underlay identity and the Git registration again after the unmount, so a
    // registration swapped during the request stops the plan rather than the destroy. The proof is the same identity
    // comparison the entry made before the request, so a registration whose inode and birth time changed refuses here.
    prove_the_adds_registration(entry.common_dir, &prepared.recorded)?;
    let facts: PlanFacts = facts_of(&prepared.recorded, &prepared.proven, &proof, prepared.registration, clock)?;
    let plan: DestructionPlan = DestructionPlan::first_write(facts, prepared.proven.own_snapshots.as_slice());
    let folder: PlanFolder = PlanFolder::open_or_create(entry.common_dir).map_err(|refusal| {
        return RemovalStop::PlanUnreadable { path: crate::worktree_clone_binding_record::registration_home::plans_folder(entry.common_dir), refusal };
    })?;
    let plan_path: PathBuf = folder.publish(&plan).map_err(|refusal| return RemovalStop::PlanUnreadable { path: folder.path().join(prepared.recorded.clone_guid.file_name()), refusal })?;
    return Ok(EnteredRemoval { plan, plan_path, lost: prepared.lost });
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
pub(super) fn prepare_the_removal(entry: &RemovalEntry<'_>, reads: &dyn RemovalPoolReads) -> Result<PreparedRemoval, RemovalStop> {
    let recorded: RecordedBinding = read_the_recorded_binding(entry.common_dir, entry.slot)?;
    let registration: ObjectId = prove_the_adds_registration(entry.common_dir, &recorded)?;
    let origin_inventory = read_the_origin_inventory(entry.common_dir, &recorded)?;
    let standing: Option<DestructionPlan> = read_an_existing_plan(entry.common_dir, recorded.clone_guid).map_err(|refusal| {
        return RemovalStop::PlanUnreadable { path: crate::worktree_clone_binding_record::registration_home::plans_folder(entry.common_dir).join(recorded.clone_guid.file_name()), refusal };
    })?;
    let history: RemovalHistory = history_of(standing.as_ref());
    if let Some(plan) = standing {
        // The plan's own identities must be this clone's before its tuples explain a bookmark.
        refuse_a_plan_of_another_clone(&plan, &recorded)?;
        let path: PathBuf = crate::worktree_clone_binding_record::registration_home::plans_folder(entry.common_dir).join(recorded.clone_guid.file_name());
        return Err(RemovalStop::APlanStands { slot: String::from(entry.slot), path });
    }
    let proven: ProvenRemoval = prove_the_lineage(reads, &recorded, &origin_inventory, &history)?;
    let explanations = explanations_of(&proven.inventory, None);
    let refused = refuse_the_unexplained_bookmarks(&explanations, proven.inventory.bookmarks().as_slice(), &origin_inventory);
    if !refused.is_empty() {
        return Err(RemovalStop::BookmarksRefused { clone: recorded.clone_dataset.clone(), refused });
    }
    let lost: NamesToBeLost = NamesToBeLost {
        slot: String::from(entry.slot),
        dataset: recorded.clone_dataset.clone(),
        snapshots: proven.own_snapshots.iter().map(|snapshot| return snapshot.name.clone()).collect(),
        bookmarks: proven.inventory.bookmarks().iter().map(|bookmark| return bookmark.full_name()).collect(),
    };
    return Ok(PreparedRemoval { recorded, registration, proven, lost });
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
fn history_of(plan: Option<&DestructionPlan>) -> RemovalHistory {
    let Some(found) = plan else {
        return RemovalHistory::NothingDestroyed;
    };
    let mut names: Vec<String> = Vec::new();
    for line in &found.attempts {
        if line.outcome == crate::worktree_clone_removal_proof_and_plan::plan_words::SnapshotOutcome::Destroyed && !names.contains(&line.snapshot.name) {
            names.push(line.snapshot.name.clone());
        }
    }
    return RemovalHistory::of_destroyed(names);
}

/// Whether one standing plan is about this clone and origin.
///
/// # Throws
/// The field in which the plan and the add's record disagree.
fn refuse_a_plan_of_another_clone(plan: &DestructionPlan, recorded: &RecordedBinding) -> Result<(), RemovalStop> {
    let pairs: [(&'static str, String, String); 6] = [
        ("cloneDataset", plan.facts.clone_dataset.clone(), recorded.clone_dataset.clone()),
        ("cloneGuid", plan.facts.clone_guid.file_name(), recorded.clone_guid.file_name()),
        ("originDataset", plan.facts.origin_dataset.clone(), recorded.origin_dataset.clone()),
        ("originDatasetGuid", plan.facts.origin_dataset_guid.file_name(), recorded.origin_dataset_guid.file_name()),
        ("originSnapshot", plan.facts.origin_snapshot.clone(), recorded.origin_snapshot.clone()),
        ("originSnapshotGuid", plan.facts.origin_snapshot_guid.file_name(), recorded.origin_snapshot_guid.file_name()),
    ];
    for (field, plan_value, recorded_value) in pairs {
        if plan_value != recorded_value {
            return Err(RemovalStop::IdentityDiffers { object: String::from("the standing destruction plan"), field, recorded: recorded_value, live: plan_value });
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
fn facts_of(recorded: &RecordedBinding, proven: &ProvenRemoval, proof: &UnmountProof, registration: ObjectId, clock: &dyn RemovalClock) -> Result<PlanFacts, RemovalStop> {
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
