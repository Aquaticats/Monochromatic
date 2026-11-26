//! What:
//!  The identities a removal planned for its clone and its origin,
//!  and the answer a plan that does not carry one of them gives.
//! Why:
//!  The check compares the live objects with what this removal recorded,
//!  never with what the live pool happens to say first:
//!  a check that re-derived its own expectation from the live objects would agree with any
//!  substitution (`doc/planning/cli-git-zfs-deferred-destroy-checks-design.md`,
//!  "Where the planned identities come from").
//!  A plan that lacks one of these fields is a refusal rather than a check with a missing
//!  expectation,
//!  and the refusal names the field and the plan's own file, so a person knows which record
//!  to look at.
//!  This folder reads no plan itself:
//!  the plan's grammar and reader belong to the removal
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`),
//!  so a caller hands the fields it read and this type decides whether they are enough.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const planned = PlannedOrigin.require('fix-flaky-test', fields); // or PlanIncomplete
//! ```

use super::origin_refusal::OriginRefusal;
use super::origin_refusals::PlanField;
use crate::worktree_clone_mount_protocol::DatasetGuid;
use std::path::{
    Path,
    PathBuf,
};

//region The planned identities

/// What:
///  Everything the check compares a live object with.
/// Why:
///  The tuple is what step 1 proved before the removal unmounted anything,
///  and every message names the field of it that disagreed.
///  The slot is the one field the plan need not carry:
///  the removal was asked for a slot,
///  and the command a refusal names is spelled with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlannedOrigin = { slot; cloneName; cloneGuid; originDatasetName; originDatasetGuid; originSnapshotName; originSnapshotGuid; originSnapshotCreatetxg };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedOrigin {
    /// The slot the removal was asked for, which names it in every command a refusal prints.
    pub slot: String,
    /// The clone's full dataset name.
    pub clone_name: String,
    /// The clone's dataset GUID, as this removal proved it.
    pub clone_guid: DatasetGuid,
    /// The origin dataset's full name.
    pub origin_dataset_name: String,
    /// The origin dataset's GUID.
    pub origin_dataset_guid: DatasetGuid,
    /// The origin snapshot's full name,
    /// which is the origin dataset's name, an `@`, and the snapshot's.
    pub origin_snapshot_name: String,
    /// The origin snapshot's GUID.
    pub origin_snapshot_guid: DatasetGuid,
    /// The transaction group the origin snapshot was created in.
    pub origin_snapshot_createtxg: u64,
}

//endregion

//region What a plan must carry

/// What:
///  The fields a caller read from a removal's plan,
///  each still optional until the whole tuple answers.
/// Why:
///  One shape lets a caller hand what it read without deciding which absences matter,
///  and the plan's own file travels with them so a refusal can name it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fields: PlannedOriginFields = { planHome, cloneName, cloneGuid, ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedOriginFields {
    /// The plan file the fields were read from,
    /// which a refusal names.
    pub plan_home: PathBuf,
    /// The clone's full dataset name.
    pub clone_name: Option<String>,
    /// The clone's dataset GUID.
    pub clone_guid: Option<DatasetGuid>,
    /// The origin dataset's full name.
    pub origin_dataset_name: Option<String>,
    /// The origin dataset's GUID.
    pub origin_dataset_guid: Option<DatasetGuid>,
    /// The origin snapshot's full name.
    pub origin_snapshot_name: Option<String>,
    /// The origin snapshot's GUID.
    pub origin_snapshot_guid: Option<DatasetGuid>,
    /// The transaction group the origin snapshot was created in.
    pub origin_snapshot_createtxg: Option<u64>,
}

/// One field the check needs,
/// or the field the plan lacks.
fn carried<T>(
    field: PlanField,
    slot: &str,
    plan_home: &Path,
    value: Option<T>,
) -> Result<T, MissingField> {
    let Some(found) = value else {
        return Err(MissingField {
            slot: String::from(slot),
            field,
            plan_home: plan_home.to_path_buf(),
        });
    };
    return Ok(found);
}

impl PlannedOrigin {
    /// What:
    ///  The planned identities a plan's fields give,
    ///  or the refusal of the first field the plan does not carry.
    /// Why:
    ///  The order is the plan's own field order,
    ///  so a person reading a refusal reads the field whose line the plan lost first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const planned = PlannedOrigin.require('fix-flaky-test', fields);
    /// ```
    ///
    /// # Errors
    /// The plan carries no value for one of the fields the check compares.
    pub fn require(
        slot: &str,
        fields: PlannedOriginFields,
    ) -> Result<PlannedOrigin, MissingField> {
        let plan_home: PathBuf = fields.plan_home;
        let home: &Path = plan_home.as_path();
        return Ok(PlannedOrigin {
            slot: String::from(slot),
            clone_name: carried(PlanField::CloneName, slot, home, fields.clone_name)?,
            clone_guid: carried(PlanField::CloneGuid, slot, home, fields.clone_guid)?,
            origin_dataset_name: carried(
                PlanField::OriginDatasetName,
                slot,
                home,
                fields.origin_dataset_name,
            )?,
            origin_dataset_guid: carried(
                PlanField::OriginDatasetGuid,
                slot,
                home,
                fields.origin_dataset_guid,
            )?,
            origin_snapshot_name: carried(
                PlanField::OriginSnapshotName,
                slot,
                home,
                fields.origin_snapshot_name,
            )?,
            origin_snapshot_guid: carried(
                PlanField::OriginSnapshotGuid,
                slot,
                home,
                fields.origin_snapshot_guid,
            )?,
            origin_snapshot_createtxg: carried(
                PlanField::OriginSnapshotCreatetxg,
                slot,
                home,
                fields.origin_snapshot_createtxg,
            )?,
        });
    }
}

/// What:
///  The field a plan does not carry,
///  and the plan it was read from.
/// Why:
///  The answer stays small so a caller's `Result` carries no large error,
///  and one conversion gives the refusal a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MissingField = { slot: string; field: PlanField; planHome: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingField {
    /// The slot the removal was asked for.
    pub slot: String,
    /// The field the plan does not carry.
    pub field: PlanField,
    /// The plan file the fields were read from.
    pub plan_home: PathBuf,
}

impl From<MissingField> for OriginRefusal {
    /// The refusal a person reads for this missing field.
    fn from(missing: MissingField) -> Self {
        return OriginRefusal::PlanIncomplete {
            slot: missing.slot,
            field: missing.field,
            plan_home: missing.plan_home,
        };
    }
}

//endregion

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "planned_origin_tests.rs"]
mod tests;
