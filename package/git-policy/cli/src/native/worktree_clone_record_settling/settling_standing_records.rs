//! What:
//!  The two records that may stand beside a slot's binding:
//!  an origin cleanup record and a destruction plan.
//! Why:
//!  The command's fifth read compares a standing origin cleanup record with the
//!  binding's five origin fields and never removes or rewrites it:
//!  the origin's cleanup stays the origin look's business
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The settling command",
//!  read 5).
//!  Its sixth read takes a plan whose `cloneGuid=` is this clone's:
//!  a plan whose head is `gone` is the finish pass's and refuses,
//!  any other plan is printed and goes with the records,
//!  and a plan that cannot be read or whose identities disagree refuses.
//!  A plan beside a settling binding whose clone a person destroyed outside the
//!  wrapper is otherwise a ring of commands each naming the other
//!  (K7 of the tenth removal-side read).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cleanup = aStandingOriginCleanup(commonDir, recorded);
//! const plan = aStandingPlan(commonDir, recorded);
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_reading::{
    an_existing_cleanup_record,
    origin_cleanups_folder,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_record::OriginCleanupRecord;
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_removal_proof_and_plan::plan_reading::read_an_existing_plan;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    HeadWord,
    SnapshotOutcome,
};
use crate::worktree_clone_removal_proof_and_plan::removal_identities::RecordedBinding;
use std::path::{
    Path,
    PathBuf,
};

/// The five origin-side facts both a plan and a cleanup record carry,
/// named as the records name them.
///
/// The binding's `origin_snapshot=` holds the full `dataset@name`,
/// where both other records hold the snapshot's own name,
/// so this is the one form the three are compared in.
pub fn the_five_origin_fields(recorded: &RecordedBinding) -> [(String, String); 5] {
    let own_name: String = the_own_snapshot_name(recorded.origin_snapshot.as_str());
    return [
        (
            String::from("originDataset"),
            recorded.origin_dataset.clone(),
        ),
        (
            String::from("originDatasetGuid"),
            recorded.origin_dataset_guid.file_name(),
        ),
        (String::from("originSnapshot"), own_name),
        (
            String::from("originSnapshotGuid"),
            recorded.origin_snapshot_guid.file_name(),
        ),
        (
            String::from("originSnapshotCreatetxg"),
            recorded.origin_snapshot_createtxg.to_string(),
        ),
    ];
}

/// The snapshot's own name from a full `dataset@name`,
/// and the whole text when it carries no `@`.
fn the_own_snapshot_name(full: &str) -> String {
    return match full.rsplit_once('@') {
        Some((_dataset, name)) => String::from(name),
        None => String::from(full),
    };
}

/// What:
///  The origin cleanup record that stands for this binding's origin snapshot,
///  compared with the binding's five origin fields.
/// Why:
///  A standing record that names the same objects is printed and left,
///  and one that names others refuses:
///  the command removes evidence,
///  so two records that disagree stop it before anything goes.
///
/// # Throws
/// A record that cannot be read,
/// or one whose fields disagree with the binding's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleanup = aStandingOriginCleanup(commonDir, recorded); // undefined when none stands
/// ```
pub fn a_standing_origin_cleanup(
    common_dir: &Path,
    recorded: &RecordedBinding,
) -> Result<Option<OriginCleanupRecord>, SettlingStop> {
    let own_name: String = the_own_snapshot_name(recorded.origin_snapshot.as_str());
    let folder: PathBuf = origin_cleanups_folder(common_dir);
    let path: PathBuf = folder.join(own_name.as_str());
    let record: OriginCleanupRecord =
        match an_existing_cleanup_record(common_dir, own_name.as_str()) {
            Ok(Some(found)) => found,
            Ok(None) => return Ok(None),
            Err(refusal) => {
                let said: String = refusal.message(path.as_path());
                return Err(SettlingStop::TheOriginCleanupCouldNotBeRead {
                    path,
                    said,
                });
            }
        };
    let standing: [(String, String); 5] = [
        (String::from("originDataset"), record.origin_dataset.clone()),
        (
            String::from("originDatasetGuid"),
            record.origin_dataset_guid.file_name(),
        ),
        (
            String::from("originSnapshot"),
            record.origin_snapshot.clone(),
        ),
        (
            String::from("originSnapshotGuid"),
            record.origin_snapshot_guid.file_name(),
        ),
        (
            String::from("originSnapshotCreatetxg"),
            record.origin_snapshot_createtxg.to_string(),
        ),
    ];
    refuse_a_disagreement(
        the_five_origin_fields(recorded),
        standing,
        |field, binding_value, standing_value| {
            return SettlingStop::TheOriginCleanupDisagrees {
                field,
                recorded: binding_value,
                standing: standing_value,
            };
        },
    )?;
    return Ok(Some(record));
}

/// What:
///  The destruction plan filed under this clone's GUID,
///  compared with the binding it stands beside.
/// Why:
///  The plan's own condition is that the two records name one clone,
///  and a plan the command prints and removes must be this clone's:
///  its GUID,
///  the slot it names,
///  the registration's name,
///  and the five origin fields.
///  A plan whose head is `gone` refuses,
///  because the finish pass owns it.
///
/// # Throws
/// A plan that cannot be read,
/// one whose head is `gone`,
/// or one whose fields disagree with the binding's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = aStandingPlan(commonDir, recorded); // undefined when none stands
/// ```
pub fn a_standing_plan(
    common_dir: &Path,
    recorded: &RecordedBinding,
) -> Result<Option<DestructionPlan>, SettlingStop> {
    let path: PathBuf =
        crate::worktree_clone_binding_record::registration_home::plans_folder(common_dir)
            .join(recorded.clone_guid.file_name());
    let plan: DestructionPlan = match read_an_existing_plan(common_dir, recorded.clone_guid) {
        Ok(Some(found)) => found,
        Ok(None) => return Ok(None),
        Err(refusal) => {
            return Err(SettlingStop::ThePlanCouldNotBeRead {
                path: path.clone(),
                said: refusal.message(path.as_path()),
            });
        }
    };
    if plan.head == HeadWord::Gone {
        return Err(SettlingStop::ThePlanIsTheFinishPasss {
            path,
        });
    }
    refuse_the_plans_identities(recorded, &plan)?;
    return Ok(Some(plan));
}

/// What:
///  The plan's clone,
///  slot and registration compared with the binding's.
/// Why:
///  The plan is printed and removed under its own condition,
///  so a plan naming another clone,
///  another slot or another registration is refused rather than carried out.
///
/// # Throws
/// The field in which the two records disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseThePlansIdentities(recorded, plan);
/// ```
fn refuse_the_plans_identities(
    recorded: &RecordedBinding,
    plan: &DestructionPlan,
) -> Result<(), SettlingStop> {
    let pairs: [(String, String, String); 3] = [
        (
            String::from("cloneGuid"),
            recorded.clone_guid.file_name(),
            plan.facts.clone_guid.file_name(),
        ),
        (
            String::from("target"),
            recorded.slot_path.display().to_string(),
            plan.facts.target.display().to_string(),
        ),
        (
            String::from("registrationId"),
            String::from_utf8_lossy(recorded.registration_id.as_slice()).into_owned(),
            String::from_utf8_lossy(plan.facts.registration_id.as_slice()).into_owned(),
        ),
    ];
    for (field, binding, plan_value) in pairs {
        if binding != plan_value {
            return Err(SettlingStop::ThePlanDisagrees {
                field,
                binding,
                plan: plan_value,
            });
        }
    }
    let standing: [(String, String); 5] = [
        (
            String::from("originDataset"),
            plan.facts.origin_dataset.clone(),
        ),
        (
            String::from("originDatasetGuid"),
            plan.facts.origin_dataset_guid.file_name(),
        ),
        (
            String::from("originSnapshot"),
            the_own_snapshot_name(plan.facts.origin_snapshot.as_str()),
        ),
        (
            String::from("originSnapshotGuid"),
            plan.facts.origin_snapshot_guid.file_name(),
        ),
        (
            String::from("originSnapshotCreatetxg"),
            plan.facts.origin_snapshot_createtxg.to_string(),
        ),
    ];
    return refuse_a_disagreement(
        the_five_origin_fields(recorded),
        standing,
        |field, binding_value, plan_value| {
            return SettlingStop::ThePlanDisagrees {
                field,
                binding: binding_value,
                plan: plan_value,
            };
        },
    );
}

/// The first of five fields in which a standing record and the binding disagree.
fn refuse_a_disagreement(
    recorded: [(String, String); 5],
    standing: [(String, String); 5],
    stop: fn(String, String, String) -> SettlingStop,
) -> Result<(), SettlingStop> {
    for ((field, binding), (_same, other)) in recorded.into_iter().zip(standing) {
        if binding != other {
            return Err(stop(field, binding, other));
        }
    }
    return Ok(());
}

/// What:
///  The names a plan recorded as destroyed,
///  in the order it recorded them.
/// Why:
///  The command prints these before it removes the plan,
///  because they are the only record of what a cancelled removal lost,
///  and the project chose that the plan goes with them printed
///  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
///  "A further answer:
/// the settling command and a plan a cancellation kept").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lost = namesThePlanRecordedAsDestroyed(plan);
/// ```
pub fn the_names_the_plan_lost(plan: &DestructionPlan) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for attempt in &plan.attempts {
        if attempt.outcome == SnapshotOutcome::Destroyed && !names.contains(&attempt.snapshot.name)
        {
            names.push(attempt.snapshot.name.clone());
        }
    }
    return names;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "settling_standing_records_tests.rs"]
mod tests;
