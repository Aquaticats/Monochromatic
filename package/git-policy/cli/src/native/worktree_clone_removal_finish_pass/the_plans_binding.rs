//! What:
//!  The list's step 12 for one plan:
//!  the binding that names this plan's clone read and compared,
//!  its two inventories removed first and the binding last,
//!  and every file proved gone before the plan may go.
//! Why:
//!  The plan is the last witness,
//!  so it goes only after the binding that names the same clone is proved gone,
//!  and the binding's own end puts the inventories first so a kill inside the step can never leave
//!  an inventory without its binding
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The record's end";
//!  `doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals").
//!  The condition is the binding design's "The condition a finisher meets":
//!  the plan's five origin fields equal the binding's,
//!  its `slot=` is the path `targetHex` holds,
//!  and its `cloneGuid=` is the plan's.
//!  A binding that names another clone is a new slot's work and is left,
//!  and a binding that cannot be read,
//!  or whose fields disagree,
//!  leaves both records for a person.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const end = finishThePlansBinding(commonDir, slot, plan); // removed | missing | anothers | unreadable | differs
//! ```

use crate::worktree_clone_binding_record::binding_encoding::path_as_hexadecimal;
use crate::worktree_clone_binding_record::record_publication::{
    RecordRefusalAtTheFile,
    read_binding,
};
use crate::worktree_clone_binding_record::registration_home::{
    binding_name,
    clone_inventory_name,
    home_folder,
    origin_inventory_name,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  What step 12 left for one plan.
/// Why:
///  The caller decides what a person reads from the same four answers the step decides on,
///  so the records' names travel with the outcome and no caller reads the folder again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheBindingEnd = { kind: 'removed'; files } | { kind: 'missing'; files } | { kind: 'anothers' } | { kind: 'unreadable'; path; said } | { kind: 'differs'; path; field; binding; plan };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TheBindingEnd {
    /// This plan's binding's files are removed and proved gone,
    /// named in the order they went.
    Removed {
        /// The files removed,
        /// inventories first and the binding last.
        files: Vec<String>,
    },
    /// No binding stood,
    /// so step 12 was already done,
    /// and any inventory the step's kill left was removed here.
    Missing {
        /// The inventory files removed,
        /// if any stood.
        files: Vec<String>,
    },
    /// The binding at the slot names another clone,
    /// which is a new slot's record and not this plan's.
    AnotherClones,
    /// The binding could not be read.
    Unreadable {
        /// The binding's path.
        path: PathBuf,
        /// The sentence the reader gave.
        said: String,
    },
    /// The binding names this plan's clone and disagrees in one field.
    Differs {
        /// The binding's path.
        path: PathBuf,
        /// The field the two records disagree in.
        field: &'static str,
        /// What the binding holds.
        binding: String,
        /// What the plan holds.
        plan: String,
    },
}

/// What:
///  Step 12 for one plan,
///  under the binding design's finisher condition.
/// Why:
///  The pass removes a binding only when it names the plan's own clone,
///  and it removes the plan only after this answer proves the files gone,
///  so one function holds both the order and its proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const end = finishThePlansBinding(commonDir, slot, plan);
/// ```
pub fn finish_the_plans_binding(
    common_dir: &Path,
    slot: &str,
    plan: &DestructionPlan,
) -> TheBindingEnd {
    let home: PathBuf = home_folder(common_dir);
    let binding_path: PathBuf = home.join(binding_name(slot));
    let record = match read_binding(home.as_path(), slot) {
        Ok(found) => found,
        Err(RecordRefusalAtTheFile::Missing) => {
            let files: Vec<String> = remove_the_inventories(home.as_path(), slot);
            return match the_files_are_gone(home.as_path(), files.as_slice()) {
                true => TheBindingEnd::Missing {
                    files,
                },
                false => TheBindingEnd::Unreadable {
                    path: binding_path,
                    said: String::from("an inventory still answers after its removal"),
                },
            };
        }
        Err(refusal) => {
            return TheBindingEnd::Unreadable {
                path: binding_path.clone(),
                said: refusal.message(binding_path.as_path()),
            };
        }
    };
    let pairs: [(&'static str, String, String); 1] = [(
        "slot",
        path_as_hexadecimal(record.slot_path().unwrap_or_default().as_path()),
        path_as_hexadecimal(plan.facts.target.as_path()),
    )];
    for (field, binding, plan_value) in pairs {
        if binding != plan_value {
            return TheBindingEnd::Differs {
                path: binding_path,
                field,
                binding,
                plan: plan_value,
            };
        }
    }
    let binding_clone: String = record
        .clone_guid
        .map(|guid| return guid.file_name())
        .unwrap_or_default();
    if binding_clone != plan.facts.clone_guid.file_name() {
        return TheBindingEnd::AnotherClones;
    }
    let origins: [(&'static str, Option<String>, String); 5] = [
        (
            "originDataset",
            record.origin_dataset.clone(),
            plan.facts.origin_dataset.clone(),
        ),
        (
            "originDatasetGuid",
            record
                .origin_dataset_guid
                .map(|guid| return guid.file_name()),
            plan.facts.origin_dataset_guid.file_name(),
        ),
        (
            "originSnapshot",
            Some(record.origin_snapshot.clone()),
            plan.facts.origin_snapshot.clone(),
        ),
        (
            "originSnapshotGuid",
            record
                .origin_snapshot_guid
                .map(|guid| return guid.file_name()),
            plan.facts.origin_snapshot_guid.file_name(),
        ),
        (
            "originSnapshotCreatetxg",
            record
                .origin_snapshot_createtxg
                .map(|txg| return txg.to_string()),
            plan.facts.origin_snapshot_createtxg.to_string(),
        ),
    ];
    for (field, binding, plan_value) in origins {
        if binding.as_deref() != Some(plan_value.as_str()) {
            return TheBindingEnd::Differs {
                path: binding_path,
                field,
                binding: binding.unwrap_or_default(),
                plan: plan_value,
            };
        }
    }
    let mut files: Vec<String> = Vec::new();
    for name in [
        origin_inventory_name(slot),
        clone_inventory_name(slot),
        binding_name(slot),
    ] {
        let path: PathBuf = home.join(name.as_str());
        match remove_one_file(path.as_path()) {
            OneFile::Removed | OneFile::AlreadyGone => files.push(name),
            OneFile::Refused => {
                return TheBindingEnd::Unreadable {
                    path,
                    said: String::from("the file could not be removed, so nothing proves it gone"),
                };
            }
        }
    }
    return match the_files_are_gone(home.as_path(), files.as_slice()) {
        true => TheBindingEnd::Removed {
            files,
        },
        false => TheBindingEnd::Unreadable {
            path: binding_path,
            said: String::from("a file still answers after its removal"),
        },
    };
}

/// What:
///  The two inventories a kill inside step 12 can leave without their binding,
///  removed in the order the step gives.
/// Why:
///  An inventory standing with no binding is the same step's leftover,
///  and it refuses the next add at the slot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const files = removeTheInventories(home, slot);
/// ```
fn remove_the_inventories(
    home: &Path,
    slot: &str,
) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    for name in [origin_inventory_name(slot), clone_inventory_name(slot)] {
        let path: PathBuf = home.join(name.as_str());
        if let OneFile::Removed = remove_one_file(path.as_path()) {
            files.push(name);
        }
    }
    return files;
}

/// What:
///  What one record file's unlink answered.
/// Why:
///  A kill between two of the three files leaves the rest,
///  so a second run finds one already gone and goes on,
///  and a line that called a name it never removed would overstate what the pass did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OneFile = 'removed' | 'already-gone' | 'refused';
/// ```
enum OneFile {
    /// The name stood and the unlink took it.
    Removed,
    /// The name was not there,
    /// which is the same step done.
    AlreadyGone,
    /// The unlink refused,
    /// so nothing proves the name gone.
    Refused,
}

/// One record file removed,
/// with a name already gone read as the same removal.
/// Why:
///  A kill between two of the three files leaves the rest,
///  so a second run finds one already gone and goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = removeOneFile(path);
/// ```
fn remove_one_file(path: &Path) -> OneFile {
    return match std::fs::remove_file(path) {
        Ok(()) => OneFile::Removed,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => OneFile::AlreadyGone,
        Err(_error) => OneFile::Refused,
    };
}

/// Whether every name of `files` is gone from the wrapper's home.
/// Why:
///  The plan goes only after the binding's files are proved gone,
///  and a name that still answers is a removal that did not happen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = theFilesAreGone(home, files);
/// ```
fn the_files_are_gone(
    home: &Path,
    files: &[String],
) -> bool {
    for name in files {
        if std::fs::symlink_metadata(home.join(name.as_str()).as_path()).is_ok() {
            return false;
        }
    }
    return true;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_plans_binding_tests.rs"]
mod tests;
