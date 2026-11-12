//! What:
//!  The service's own removal of one whole worktree a command handed over,
//!  with Git's own judgment of what it would keep as the verdict.
//! Why:
//!  `git worktree remove` is Git's line:
//!  it deletes what Git ignores and refuses what it would keep,
//!  so a tracked edit or an untracked file stops it and its sentence is what the person reads.
//!  The removal is the step that may not be run twice and may take minutes on a large tree,
//!  which is why the command that found the worktree handed it over instead of running it
//!  (`worktree_remove_hand_over.rs`);
//!  what is recorded before Git runs and what its walk leaves is the same record the person's own removal writes
//!  (`worktree_removal_left_behind_folder.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = removeTheWorktreeOf(item); // Git's removal, the record, and the finish
//! ```

use crate::diagnostic_log::debug;
use crate::real_git::{ResolutionInputs, ResolvedGit, process_resolution_inputs, resolve_real_git};
use crate::service_work::Verdict;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::work_item::WorkItem;
use crate::worktree_removal_left_behind_folder::{HandedToGit, Remembered, finish_a_removal_handed_to_git, remember_before_git};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemoveAWorktree";

/// What:
///  The service's own removal of one landed worktree a command handed over,
///  with Git's own words as the verdict.
/// Why:
///  `git worktree remove` is Git's line:
///  it deletes what Git ignores and refuses what it would keep,
///  so a tracked edit or an untracked file stops it and its sentence is what the person reads.
///  A path that is gone is nothing to remove:
///  either another attempt removed the worktree,
///  or the person did.
///  A Git that cannot be found or started leaves the record to be tried again,
///  because nothing about the worktree was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeTheWorktreeOf(item): Verdict
/// ```
pub(crate) fn remove_the_worktree_of(item: &WorkItem) -> Verdict {
    let target: PathBuf = PathBuf::from(item.parent.as_str());
    if !target.is_absolute() {
        return Verdict::Failed(format!("the record names no absolute path to remove: {}", item.parent));
    }
    if !target.exists() {
        debug(TAG, format!("{} is gone already; nothing to remove", target.display()).as_str());
        return Verdict::Done;
    }
    // Read before Git deletes the worktree: the registration names the pool this worktree's shadow lives in, and a path
    //  that is gone names nothing.
    let pool: Option<(PathBuf, PathBuf)> = the_pool_of(target.as_path());
    let context: GitContext = match the_service_git(item) {
        Ok(found) => found,
        Err(reason) => return Verdict::Retry(reason),
    };
    // What: The folder's entries are recorded before Git runs, so whatever its walk cannot empty is finished here rather
    //       than standing under a name no registration names (`worktree_removal_left_behind_folder.rs`).
    // Why:  Git deletes a worktree's registration whether or not its walk emptied the folder, and the relink of a fresh
    //       worktree writes into it for minutes, which is the `Directory not empty` a person's removal ends with.
    let remembered: Option<Remembered> = remember_before_git(item.git_dir.as_path(), target.as_path());
    let arguments: Vec<OsString> = vec![OsString::from("worktree"), OsString::from("remove"), target.clone().into_os_string()];
    let output: GitOutput = match run_git(&context, &GitRequest::new(item.repository.as_path(), arguments.as_slice())) {
        Ok(finished) => finished,
        Err(error) => return Verdict::Retry(format!("Git could not be started to remove {} ({error})", target.display())),
    };
    let finished: HandedToGit = finish_a_removal_handed_to_git(remembered, output.code, output.stderr.as_slice(), target.as_path());
    // What: The relink record of the worktree is settled here, because the worktree it names is gone.
    // Why:  A fresh worktree's package state is relinked by this service, and the removal may come before that work is done: a
    //       record left waiting names a directory nothing will relink (`package_repair_relink_of_a_removed_worktree.rs`).
    if !matches!(finished, HandedToGit::Left { .. }) {
        crate::package_repair_relink_of_a_removed_worktree::say_what_was_cancelled(item.git_dir.as_path(), target.as_path());
    }
    // A pool names the worktree it serves in every member's config and sits in the common Git directory, which Git's removal
    //  leaves alone, so the removal that just proved the worktree is gone takes it (`worktree_removal_clears_the_pool.rs`).
    if !matches!(finished, HandedToGit::Left { .. })
        && let Some((common, admin)) = pool
    {
        crate::worktree_removal_clears_the_pool::clear_the_pool_of_a_removed_worktree(common.as_path(), admin.as_path());
    }
    return match finished {
        HandedToGit::Removed => Verdict::Done,
        HandedToGit::FinishedAfterTheWalk { line: _why } => {
            // The merge's notice told the person where the outcome of this removal is read, and this is the one part of it
            //  nobody's command waited for: Git's walk stopped at an entry it could not delete, so this service removed the
            //  folder with whatever was still being written into it.
            crate::service_warn!(
                "Git could not empty {}, so this service removed it after Git's own walk stopped; anything still being written into it went with it, which is the risk git worktree remove states",
                target.display()
            );
            Verdict::Done
        }
        HandedToGit::Left { line } => Verdict::Failed(line),
    };
}

/// What:
///  The common Git directory of this repository and the registration directory of `target`,
///  as a removal reads them before Git deletes the worktree.
/// Why:
///   The pool a worktree's commits fill is named from its registration directory and lives in the common Git directory,
///  and both are named by the worktree's own `.git` pointer,
///  which the removal deletes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const poolOf = (target) => { const admin = gitDirOf(target); return [commonDirectoryOf(admin), admin]; };
/// ```
fn the_pool_of(target: &Path) -> Option<(PathBuf, PathBuf)> {
    let pointer: Vec<u8> = std::fs::read(target.join(".git")).ok()?;
    let text: String = String::from_utf8_lossy(pointer.as_slice()).into_owned();
    let named: PathBuf = PathBuf::from(text.trim_end().strip_prefix("gitdir: ")?.trim());
    let admin: PathBuf = if named.is_absolute() { named } else { target.join(named) };
    let common: PathBuf = crate::common_directory_of_a_git_directory::common_directory_of(admin.as_path());
    return Some((common, admin));
}

/// What:
///  How the commit service starts real Git for a removal it was handed a record for.
/// Why:
///  The service's handler of a record is handed the record alone,
///  so the Git it starts is the one this process resolves,
///  as the wrapper's own start does:
///  the same search,
///  the same refusal of a wrapper,
///  and the same record of a Git found from the unit.
///  The overlay of a command does not apply here:
///  the hooks are off for this whole process (`hooks_off::activate`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const serviceGit = (item) => ({ realGit: resolveRealGit(processResolutionInputs(process.env)), overlay: [], commonDir: item.gitDir });
/// ```
/// What:
///  The words for process facts a Git search needs and this machine would not give.
/// Why:
///  The reason names what could not be read,
///  so a person reads why no Git was started rather than a sentence about a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noProcessFacts = (error) => `the process facts a Git search needs cannot be read (${error})`;
/// ```
fn no_process_facts(error: std::io::Error) -> String {
    return format!("the process facts a Git search needs cannot be read ({error})");
}

/// What:
///  The words for a removal that found no Git to run.
/// Why:
///  Nothing about the worktree was changed,
///  so the record is kept for another try,
///  and the reason says which record and why nothing ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noGitForARemoval = (failure) => `no Git can be started to remove the worktree a record names: ${failure}`;
/// ```
fn no_git_for_a_removal(failure: crate::real_git::RealGitNotFound) -> String {
    return format!("no Git can be started to remove the worktree a record names: {failure}");
}

fn the_service_git(item: &WorkItem) -> Result<GitContext, String> {
    let environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let inputs: ResolutionInputs = process_resolution_inputs(environment.as_slice()).map_err(no_process_facts)?;
    let resolved: ResolvedGit = resolve_real_git(&inputs).map_err(no_git_for_a_removal)?;
    return Ok(GitContext { real_git: resolved.path, overlay: Vec::new(), global_prefix: Vec::new(), common_dir: Some(item.git_dir.clone()) });
}
