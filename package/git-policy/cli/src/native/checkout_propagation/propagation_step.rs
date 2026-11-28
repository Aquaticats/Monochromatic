//! What:
//!  What the step does,
//!  in the accepted design's order:
//!  resolve,
//!  open,
//!  read,
//!  check,
//!  read the table and what stands beneath,
//!  set,
//!  prove,
//!  and file the answer.
//! Why:
//!  The order is the design's,
//!  the same descriptor carries the read,
//!  the set and the read after it so the act names one instance,
//!  and each refusal files the fields the run reached,
//!  so nothing is changed and nothing is left unexplained
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "Where it runs,
//!  as whom,
//!  and in what order").
//!  This file is the one piece nothing reaches in this release:
//!  its call at the head of `boot-reconcile` and of the default sweep is plan
//!  step 19's wiring,
//!  which is not built.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = runTheStep(places, resolution, calls); // ok: { outcome, line } | refusal: { step, errno, message }
//! ```

use crate::checkout_propagation::checkout_mount_checks::{
    check_the_facts,
    check_the_row,
};
use crate::checkout_propagation::mount_descriptor_read::DescriptorMount;
use crate::checkout_propagation::propagation_calls::PropagationCalls;
use crate::checkout_propagation::propagation_record::PropagationRecord;
use crate::checkout_propagation::propagation_step_names::{
    CheckoutResolution,
    PropagationDone,
    PropagationPlaces,
    PropagationRefusal,
};
use crate::checkout_propagation::propagation_step_records::a_refusal_with_facts;
use crate::checkout_propagation::propagation_step_records::{
    a_refusal,
    an_answer,
    filed,
    finished,
    refused_call,
    refused_check,
    refused_with_facts,
};
use crate::checkout_propagation::propagation_words::{
    PropagationOutcome,
    PropagationStep,
};
use crate::checkout_propagation::shared_mounts_beneath::{
    SharedMountsBeneath,
    shared_mounts_beneath,
};
use crate::subvolume::fs_mount_options::MountFacts;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Run the step:
///  resolve,
///  open,
///  read,
///  check,
///  read the table and what stands beneath,
///  set,
///  prove,
///  and file the answer.
/// Why:
///  The order is the design's,
///  the same descriptor carries the read,
///  the set and the read after it so the act names one instance,
///  and each refusal files the fields the run reached,
///  so nothing is changed and nothing is left unexplained
///  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
///  "Where it runs,
///  as whom,
///  and in what order").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = runTheStep(places, resolution, calls);
/// ```
///
/// # Errors
/// The checkout was not resolved,
/// a kernel act refused,
/// the mount was not the checkout's own instance,
/// the table held no row for it,
/// the read after the set did not prove it,
/// or the answer could not be filed.
pub fn run_the_step(
    places: &PropagationPlaces,
    resolution: &CheckoutResolution,
    calls: &dyn PropagationCalls,
) -> Result<PropagationDone, PropagationRefusal> {
    let (checkout, dataset): (PathBuf, String) = match resolution {
        CheckoutResolution::At {
            checkout,
            dataset,
        } => (checkout.clone(), dataset.clone()),
        CheckoutResolution::Refused {
            why,
        } => {
            let record: PropagationRecord = a_refusal(
                places,
                PropagationStep::Resolve,
                None,
                SharedMountsBeneath::Unread,
            );
            return Err(filed(places, record, why.clone()));
        }
    };
    let opened: std::fs::File = calls
        .open_the_checkout(checkout.as_path())
        .map_err(|failure| {
            return refused_call(places, PropagationStep::Open, &failure);
        })?;
    let before: DescriptorMount = calls.mount_of(&opened).map_err(|failure| {
        return refused_call(places, PropagationStep::Statmount, &failure);
    })?;
    let checked: Result<(), PropagationRefusal> =
        match check_the_facts(&before, checkout.as_path(), dataset.as_str()) {
            Some(refusal) => Err(refused_check(places, checkout.as_path(), &before, &refusal)),
            None => Ok(()),
        };
    checked?;
    let table: Vec<MountFacts> = calls.read_the_table().map_err(|failure| {
        return refused_call(places, PropagationStep::BeforeTable, &failure);
    })?;
    let row: &MountFacts = match check_the_row(&before, table.as_slice(), dataset.as_str()) {
        Ok(row) => row,
        Err(refusal) => {
            let record: PropagationRecord = a_refusal_with_facts(
                places,
                &before,
                checkout.as_path(),
                refusal.word,
                None,
                SharedMountsBeneath::Unread,
            );
            return Err(filed(places, record, refusal.why));
        }
    };
    let beneath: SharedMountsBeneath = shared_mounts_beneath(table.as_slice(), row.mount_id);
    if !before.propagation.carries_shared() {
        let outcome: PropagationOutcome = match before.propagation.is_private() {
            true => PropagationOutcome::AlreadyPrivate,
            false => PropagationOutcome::Left,
        };
        let record: PropagationRecord =
            an_answer(places, checkout.as_path(), &before, beneath, outcome);
        return finished(places, &record);
    }
    return set_then_prove(places, checkout.as_path(), &before, beneath, &opened, calls);
}

/// What:
///  Set the mount private through the descriptor the read verified,
///  prove the set on the same instance,
///  and file the answer.
/// Why:
///  The set is the one act that changes a mount,
///  and the read after it is what tells a set that worked from one that
///  answered success and left the set in place,
///  so both are one function and no caller reports a set it never proved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = setThenProve(places, checkout, before, beneath, opened, calls);
/// ```
fn set_then_prove(
    places: &PropagationPlaces,
    checkout: &Path,
    before: &DescriptorMount,
    beneath: SharedMountsBeneath,
    opened: &std::fs::File,
    calls: &dyn PropagationCalls,
) -> Result<PropagationDone, PropagationRefusal> {
    calls.make_private(opened).map_err(|failure| {
        return refused_with_facts(
            places,
            checkout,
            before,
            beneath.clone(),
            PropagationStep::Set,
            &failure,
        );
    })?;
    let after: DescriptorMount = calls.mount_of(opened).map_err(|failure| {
        return refused_with_facts(
            places,
            checkout,
            before,
            beneath.clone(),
            PropagationStep::VerifyAfter,
            &failure,
        );
    })?;
    let proved: bool = after.mount_id == before.mount_id
        && after.propagation.is_private()
        && after.peer_group.is_none()
        && after.master.is_none();
    if !proved {
        let record: PropagationRecord = PropagationRecord {
            after: Some(after.propagation),
            step: PropagationStep::VerifyAfter,
            ..an_answer(
                places,
                checkout,
                before,
                beneath,
                PropagationOutcome::Refused,
            )
        };
        return Err(filed(
            places,
            record,
            format!(
                "the set answered success and the mount reads {} with peer group {} and master {}, so the change was not proved",
                after.propagation.word(),
                after
                    .peer_group
                    .map_or(String::from("none"), |number| return number.to_string()),
                after
                    .master
                    .map_or(String::from("none"), |number| return number.to_string())
            ),
        ));
    }
    let record: PropagationRecord = PropagationRecord {
        after: Some(after.propagation),
        ..an_answer(
            places,
            checkout,
            before,
            beneath,
            PropagationOutcome::Applied,
        )
    };
    return finished(places, &record);
}
/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "propagation_step_tests.rs"]
mod tests;
