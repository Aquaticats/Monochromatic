//! What:
//!  The add's durable record:
//! the generation its claim took,
//! a standing record
//!  proved to be this add's continuation,
//! the record's first version,
//! and the
//!  slot identity every later step relies on.
//! Why:
//!  A failed or killed add keeps the claim,
//! the slot and the locked registration,
//!  and the next add at the slot reads the record's phase and continues from it
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//! A0 to A12 and
//!  W5 and W6 of the fourth check),
//!  so the record is written before any mutation beyond the claim and every
//!  field a resume depends on is proved here rather than read again later.
//!  `add_route_run.rs` runs the phases and would go over the file-size limit
//!  carrying the record's own writes and checks,
//!  and `add_route_run_end.rs` holds the end one add reaches after them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const record = resumed ?? freshTheRecord(...); // both proved against this add
//! verifyTheSlotIdentity(steps, record); // the slot is the folder the record names
//! ```

use super::add_capture_store::clear_other_adds_records;
use super::add_plan_stop::refuse_while_a_plan_stands;
use super::add_preflight::Preflight;
use super::add_route_gate::RouteFacts;
use super::add_route_plan::{
    AddNames,
    name_the_add,
    request_of,
};
use super::add_route_steps::{
    StepRefusal,
    Steps,
};
use crate::worktree_clone_binding_record::binding_encoding::FolderIdentity;
use crate::worktree_clone_binding_record::binding_record::BindingRecord;
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::registration_home::{
    binding_path,
    claim_generation_of,
    home_folder,
};
use std::ffi::OsString;

/// The generation the claim this process holds took.
pub(super) fn the_claims_generation(
    home: &std::path::Path,
    slot: &str,
) -> Result<u64, StepRefusal> {
    let listing: std::fs::ReadDir = std::fs::read_dir(home).map_err(|error| {
        return StepRefusal::refused(format!(
            "the record home {} cannot be listed ({error})",
            home.display()
        ));
    })?;
    let mut newest: u64 = 0;
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| {
            return StepRefusal::refused(format!(
                "the record home {} cannot be listed ({error})",
                home.display()
            ));
        })?;
        let Some(name) = entry.file_name().to_str().map(String::from) else {
            continue;
        };
        if let Some(generation) = claim_generation_of(name.as_str(), slot) {
            newest = newest.max(generation);
        }
    }
    if newest == 0 {
        return Err(StepRefusal::refused(format!(
            "no claim generation for {slot} stands after this command took one"
        )));
    }
    return Ok(newest);
}

/// Read a standing record and prove it is this add's continuation.
pub(super) fn resume_the_record(
    route: &RouteFacts,
    plan: &crate::worktree_command_facts::AddPlan,
    arguments: &[OsString],
    mut record: BindingRecord,
    generation: u64,
) -> Result<BindingRecord, StepRefusal> {
    if record.repository_path().as_deref() != Some(route.common_dir.as_path())
        || record.slot_path().as_deref() != Some(route.destination.as_path())
    {
        let shown: String = record.repository_path().map_or_else(
            || return String::from("none"),
            |path| return path.display().to_string(),
        );
        return Err(StepRefusal::refused(format!(
            "the record at the slot names repository {shown} and another slot's path, so it is not this repository's record for {}. Ways forward: inspect it and remove it if it is not the add's",
            route.destination.display()
        )));
    }
    if record.checkout_dataset != route.checkout_dataset {
        return Err(StepRefusal::refused(format!(
            "the record names the checkout dataset {} and this checkout is {}, so nothing in it can be trusted; Ways forward: inspect the record and remove it, then run the add again",
            record.checkout_dataset, route.checkout_dataset
        )));
    }
    let request = request_of(plan, arguments).map_err(StepRefusal::refused)?;
    if record.request != request {
        return Err(StepRefusal::refused(format!(
            "the record was written by an add given `{}` and this add was given `{}`; Ways forward: run the same add again, which continues that record, or remove the worktree with `git worktree remove {}` and then delete the binding record at {} yourself, because Git's removal leaves that record and no command of this release removes it",
            record.request.encode(),
            request.encode(),
            route.slot,
            binding_path(route.common_dir.as_path(), route.slot.as_str()).display()
        )));
    }
    record.claim = generation;
    return Ok(record);
}

/// Write the record's first version before any mutation beyond the claim.
pub(super) fn fresh_the_record(
    route: &RouteFacts,
    plan: &crate::worktree_command_facts::AddPlan,
    arguments: &[OsString],
    preflight: &Preflight,
    home: &std::fs::File,
    generation: u64,
) -> Result<BindingRecord, StepRefusal> {
    // A plan that names the clone comes from a removal's record design, and the
    // plan's head word and the clone the pool can find decide the one command the
    // person reads beside it (plan step 10's choice 2C). The same call stands in the
    // preflight, so one state prints one sentence.
    refuse_while_a_plan_stands(route, &preflight.device)?;
    let names: AddNames = name_the_add(route, plan, arguments).map_err(StepRefusal::refused)?;
    clear_other_adds_records(
        home_folder(route.common_dir.as_path()).as_path(),
        route.slot.as_str(),
        &names.nonce,
    )
    .map_err(StepRefusal::refused)?;
    let repository_identity: FolderIdentity = FolderIdentity::of_folder(route.common_dir.as_path())
        .map_err(|error| {
            return StepRefusal::refused(format!(
                "the common Git directory cannot be identified ({error})"
            ));
        })?;
    let record: BindingRecord = BindingRecord {
        claim: generation,
        nonce: names.nonce,
        request: names.request,
        phase: Phase::Claimed,
        settle_reason: None,
        checkout_dataset: route.checkout_dataset.clone(),
        checkout_dataset_guid: preflight.checkout_guid,
        repository: route.common_dir.as_os_str().as_encoded_bytes().to_vec(),
        repository_identity,
        slot: route.destination.as_os_str().as_encoded_bytes().to_vec(),
        slot_identity: None,
        registration: None,
        registration_identity: None,
        pool: route.pool.clone(),
        pool_guid: preflight.pool_guid,
        origin_snapshot: names.origin_snapshot,
        origin_snapshot_guid: None,
        origin_snapshot_createtxg: None,
        origin_dataset: None,
        origin_dataset_guid: None,
        clone_dataset: names.clone_dataset,
        clone_guid: None,
        clone_createtxg: None,
        pointer: None,
    };
    crate::worktree_clone_binding_record::record_publication::write_binding(home, route.slot.as_str(), &record).map_err(|error| return StepRefusal::refused(format!("the add's first record write was refused ({error}), so nothing beyond the claim was changed")))?;
    return Ok(record);
}

/// Verify the unmounted slot's identity before a step that relies on it.
pub(super) fn verify_the_slot_identity(
    steps: &Steps<'_>,
    record: &BindingRecord,
) -> Result<(), StepRefusal> {
    let Some(recorded) = record.slot_identity else {
        return Err(StepRefusal::refused(String::from(
            "the record names no slot identity, so nothing proves the slot Git registered is this add's",
        )));
    };
    let live: FolderIdentity = FolderIdentity::of_folder(steps.route.destination.as_path())
        .map_err(|error| {
            return StepRefusal::refused(format!(
                "the slot {} cannot be identified ({error})",
                steps.route.destination.display()
            ));
        })?;
    if live != recorded {
        return Err(StepRefusal::refused(format!(
            "the slot {} is another folder than the one the registration was made in ({} instead of {}); Ways forward: inspect the slot, then remove the worktree and run the add again",
            steps.route.destination.display(),
            live.encode(),
            recorded.encode()
        )));
    }
    return Ok(());
}
