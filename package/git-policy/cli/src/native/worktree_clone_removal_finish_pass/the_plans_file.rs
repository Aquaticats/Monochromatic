//! What:
//!  The list's step 13 for one plan:
//!  the plan's own file unlinked and proved gone,
//!  after the binding that names the same clone was proved gone.
//! Why:
//!  The plan is the last witness of a removal,
//!  so it is removed last among the records
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "When it is removed";
//!  `doc/planning/cli-git-zfs-origin-cleanup-record-design.md`,
//!  "The removal's one order of writes and removals").
//!  The empty folder goes with the last record,
//!  as `forget_destroy` removes a record and its empty folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! removeThePlan(commonDir, plan);
//! removeTheFolderWhenEmpty(plansFolder(commonDir));
//! ```

use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The plan's file removed and proved gone,
///  or the sentence for a person when it stayed.
/// Why:
///  A plan that still answers is the last witness of a removal that did not finish,
///  so the caller prints what the call answered rather than a line about a finished removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = removeThePlan(commonDir, plan);
/// ```
pub fn remove_the_plan(
    common_dir: &Path,
    plan: &DestructionPlan,
) -> Result<(), String> {
    let path: PathBuf = plans_folder(common_dir).join(plan.facts.clone_guid.file_name());
    return match std::fs::remove_file(path.as_path()) {
        Ok(()) => prove_the_plan_is_gone(path.as_path()),
        // Another command's look removed it between this pass's read and its unlink, which is the
        // removal this step wanted.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("the unlink answered `{error}`")),
    };
}

/// Whether the plan's own name still answers,
/// which a removal that finished never leaves.
/// Why:
///  The empty folder is removed only after this proof,
///  so a folder that still holds a plan is never removed from around it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveThePlanIsGone(path);
/// ```
fn prove_the_plan_is_gone(path: &Path) -> Result<(), String> {
    return match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "the plan's own path could not be read back ({error})"
        )),
        Ok(_) => Err(String::from(
            "the plan's own path still answers after its removal",
        )),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_plans_file_tests.rs"]
mod tests;
