//! What:
//!  The removal's own records, removed after the settle proved the clone gone:
//!  the two inventories first,
//!  the binding last,
//!  and the plan only once the binding's files are proved gone.
//! Why:
//!  The plan is the last witness of a removal,
//!  so it is removed only after the binding that names the same clone is proved gone,
//!  and the binding's own end puts its inventories first so a kill inside that step can never
//!  leave an inventory without the binding, which would refuse the next add at the slot
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "When it is removed";
//!  `doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The record's end").
//!  A binding that names another clone is a new slot's and is left,
//!  and a binding whose fields disagree with the plan is refused rather than removed:
//!  the plan's own condition is that the two records name one clone.
//!  Nothing here removes a registration, a pointer or a directory a person filled;
//!  those belong to the settle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const gone = removeTheRemovalRecords(commonDir, slot, plan);
//! ```

use crate::worktree_clone_binding_record::binding_encoding::path_as_hexadecimal;
use crate::worktree_clone_binding_record::record_publication::read_binding;
use crate::worktree_clone_binding_record::registration_home::{binding_name, clone_inventory_name, home_folder, origin_inventory_name, plans_folder};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use std::path::{Path, PathBuf};

/// What:
///  What one removal's records looked like before they went.
/// Why:
///  A person reads which records the removal removed,
///  and a control asserts the order without reading three directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordsGone = { files: string[]; planRemoved: boolean; folderRemoved: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordsGone {
    /// The record files this run removed,
    /// in the order it removed them.
    pub files: Vec<String>,
    /// Whether the plan's file went.
    pub plan_removed: bool,
    /// Whether the empty plan folder went with it.
    pub folder_removed: bool,
}

/// What:
///  The removal's records removed under the plan's own condition:
///  the binding names this clone and this slot,
///  and its five origin fields equal the plan's.
/// Why:
///  The plan is the last witness,
///  and a binding that names another clone is a new slot's work rather than this removal's
///  leftover.
///
/// # Throws
/// A binding that cannot be read,
/// one whose fields disagree with the plan,
/// or a file that could not be proved gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = removeTheRemovalRecords(commonDir, slot, plan);
/// ```
pub fn remove_the_removal_records(common_dir: &Path, slot: &str, plan: &DestructionPlan) -> Result<RecordsGone, RemovalStop> {
    let home: PathBuf = home_folder(common_dir);
    let binding_path: PathBuf = home.join(binding_name(slot));
    let record = read_binding(home.as_path(), slot).map_err(|refusal| {
        return RemovalStop::TheRecordsCouldNotGo { path: binding_path.clone(), said: format!("{refusal:?}") };
    })?;
    refuse_a_binding_of_another_clone(&record, plan, binding_path.as_path())?;
    let mut gone: RecordsGone = RecordsGone { files: Vec::new(), plan_removed: false, folder_removed: false };
    for name in [origin_inventory_name(slot), clone_inventory_name(slot), binding_name(slot)] {
        let path: PathBuf = home.join(name.as_str());
        remove_one_file(path.as_path())?;
        gone.files.push(String::from(name));
    }
    for name in [origin_inventory_name(slot), clone_inventory_name(slot), binding_name(slot)] {
        let path: PathBuf = home.join(name.as_str());
        if std::fs::symlink_metadata(path.as_path()).is_ok() {
            return Err(RemovalStop::TheRecordsCouldNotGo { path, said: String::from("the file still answers after its removal") });
        }
    }
    let plan_path: PathBuf = plans_folder(common_dir).join(plan.facts.clone_guid.file_name());
    remove_one_file(plan_path.as_path())?;
    gone.plan_removed = true;
    let folder: PathBuf = plans_folder(common_dir);
    if std::fs::remove_dir(folder.as_path()).is_ok() {
        gone.folder_removed = true;
    }
    return Ok(gone);
}

/// What:
///  Whether the binding that stands names this plan's clone and slot,
///  with the plan's five origin fields.
/// Why:
///  A binding that names another clone is a new slot's,
///  and the plan's own condition refuses to remove it or itself in that state.
///
/// # Throws
/// The field in which the two records disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refuseABindingOfAnotherClone(read, plan, path);
/// ```
fn refuse_a_binding_of_another_clone(record: &crate::worktree_clone_binding_record::binding_record::BindingRecord, plan: &DestructionPlan, path: &Path) -> Result<(), RemovalStop> {
    let pairs: [(&'static str, String, String); 2] = [
        ("cloneGuid", record.clone_guid.map(|guid| return guid.file_name()).unwrap_or_default(), plan.facts.clone_guid.file_name()),
        ("slot", path_as_hexadecimal(record.slot_path().unwrap_or_default().as_path()), path_as_hexadecimal(plan.facts.target.as_path())),
    ];
    for (field, binding_value, plan_value) in pairs {
        if binding_value != plan_value {
            return Err(RemovalStop::TheRecordsCouldNotGo { path: path.to_path_buf(), said: format!("the binding's `{field}` is `{binding_value}` and the plan's is `{plan_value}`") });
        }
    }
    let origins: [(&'static str, Option<String>, String); 5] = [
        ("originDataset", record.origin_dataset.clone(), plan.facts.origin_dataset.clone()),
        ("originDatasetGuid", record.origin_dataset_guid.map(|guid| return guid.file_name()), plan.facts.origin_dataset_guid.file_name()),
        ("originSnapshot", Some(record.origin_snapshot.clone()), plan.facts.origin_snapshot.clone()),
        ("originSnapshotGuid", record.origin_snapshot_guid.map(|guid| return guid.file_name()), plan.facts.origin_snapshot_guid.file_name()),
        ("originSnapshotCreatetxg", record.origin_snapshot_createtxg.map(|txg| return txg.to_string()), plan.facts.origin_snapshot_createtxg.to_string()),
    ];
    for (field, binding_value, plan_value) in origins {
        if binding_value.as_deref() != Some(plan_value.as_str()) {
            return Err(RemovalStop::TheRecordsCouldNotGo {
                path: path.to_path_buf(),
                said: format!("the binding's `{field}` is `{}` and the plan's is `{plan_value}`", binding_value.unwrap_or_default()),
            });
        }
    }
    return Ok(());
}

/// One record file removed,
/// with a name already gone read as the same removal.
///
/// # Throws
/// The unlink that failed.
fn remove_one_file(path: &Path) -> Result<(), RemovalStop> {
    return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(RemovalStop::TheRecordsCouldNotGo { path: path.to_path_buf(), said: error.to_string() }),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "removal_records_end_tests.rs"]
mod tests;
