//! What:
//!  The finish pass itself:
//!  one look at the destruction plan folder,
//!  every plan in it taken as far as its own state allows,
//!  and the folder removed when it is empty.
//! Why:
//!  The pass runs before every forwarded command,
//!  in the place the wrapper already finishes an interrupted destroy,
//!  and its subject is a plan whose head is `gone` and whose binding's files still stand
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "The finish pass that settles a registration";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 16).
//!  Nothing here creates the folder,
//!  moves a record aside or writes into one:
//!  a plan that does not read is reported and left,
//!  and a foreign entry is never read.
//!  It is built ahead of the plan's order and no command calls it:
//!  the step that switches the removal's command path on calls it,
//!  and that step is where its wiring lives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines: string[] = finishTheRemovalPlans(pass);
//! ```

use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_removal_finish_pass::pass_on_one_plan::finish_one_plan;
use crate::worktree_clone_removal_finish_pass::pass_refusal_lines::{the_entry_is_no_plan_name, the_folder_could_not_be_listed, the_plan_was_not_read};
use crate::worktree_clone_removal_finish_pass::the_pass_seams::FinishPass;
use crate::worktree_clone_removal_finish_pass::the_plans_folder::{FolderLook, remove_the_folder_when_empty};
use crate::worktree_clone_removal_proof_and_plan::plan_reading::{is_a_plan_entry, read_the_plan};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub use crate::worktree_clone_removal_finish_pass::the_pass_seams::FinishPass as TheFinishPass;

/// What:
///  Every line this pass prints for one repository,
///  after it has taken every plan as far as its state allows.
/// Why:
///  The pass is a look before a person's own command,
///  so its lines are returned to that command rather than printed here,
///  and a repository with no plan folder costs one look and prints nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = finishTheRemovalPlans(pass);
/// ```
pub fn finish_the_removal_plans(pass: &FinishPass<'_>) -> Vec<String> {
    let folder: PathBuf = plans_folder(pass.common_dir);
    let mut lines: Vec<String> = Vec::new();
    let names: Vec<std::ffi::OsString> = match pass.folder.look(folder.as_path()) {
        FolderLook::NoFolder => return lines,
        FolderLook::NotAFolder { said } => {
            lines.push(the_folder_could_not_be_listed(folder.as_path(), said.as_str()));
            return lines;
        }
        FolderLook::Entries(found) => found,
    };
    for name in &names {
        finish_one_entry(pass, folder.as_path(), name.as_os_str(), &mut lines);
    }
    // A folder a kill left empty between a plan's unlink and the folder's removal goes here, and an
    // empty folder is the whole of what this call can remove: a folder that still holds a record
    // answers `ENOTEMPTY` and stays.
    remove_the_folder_when_empty(folder.as_path());
    return lines;
}

/// One entry of the folder:
/// a plan read under its own name,
/// or the line for an entry that is not one.
/// Why:
///  A reader that guessed at a foreign entry would read a person's file as a removal's record,
///  and a plan that cannot be read is never moved or removed,
///  so each case stops here with its own sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// finishOneEntry(pass, folder, name, lines);
/// ```
fn finish_one_entry(pass: &FinishPass<'_>, folder: &Path, name: &OsStr, lines: &mut Vec<String>) {
    let path: PathBuf = folder.join(name);
    let Some(spelled) = name.to_str() else {
        lines.push(the_entry_is_no_plan_name(path.as_path()));
        return;
    };
    if !is_a_plan_entry(spelled) {
        lines.push(the_entry_is_no_plan_name(path.as_path()));
        return;
    }
    let guid: DatasetGuid = match DatasetGuid::parse(spelled) {
        Ok(found) => found,
        Err(refusal) => {
            lines.push(the_plan_was_not_read(path.as_path(), refusal.message().as_str()));
            return;
        }
    };
    let plan: DestructionPlan = match read_the_plan(pass.common_dir, guid) {
        Ok(found) => found,
        Err(not_read) => {
            lines.push(the_plan_was_not_read(path.as_path(), not_read.message(path.as_path()).as_str()));
            return;
        }
    };
    finish_one_plan(pass, &plan, path.as_path(), lines);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "pass_over_the_plans_tests.rs"]
mod tests;
