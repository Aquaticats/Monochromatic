//! What:
//!  The retry plan on disk:
//! one small file per branch whose push waits for a new try,
//! in a directory of the service instance's own
//!  journal,
//! written when a try is planned,
//! removed when the plan ends,
//! and read when a service starts.
//! Why:
//!  A service that is stopped (a release,
//! a restart of the unit,
//! a crash) must not forget the pushes it keeps trying:
//! the defect
//!  the retry mends is a stopped push that is never tried again,
//! and a restart that forgot the plan would bring it back.
//! The
//!  person's failure record cannot carry the plan:
//! the person's next command shows it and removes it.
//! The journal is the
//!  instance's own and survives a restart (`%S/monochromatic-git-policy-cli/service/%i`),
//! and one instance is the only writer of its plan,
//! so no lock
//!  is taken.
//! A plan is replaced by publishing the new file whole before the old one is removed,
//! so a kill leaves one of them,
//!  never none;
//! when a kill leaves both,
//! the newer one is the plan.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await save(dir, plan); const plans = await load(dir); await remove(dir, gitDir, branchRef);
//! ```

/// Import what only the controls read:
/// one listing's own entries and their reading,
/// the item a plan schedules,
/// and the two times a plan is ordered by.
#[cfg(test)]
pub(super) use super::push_retry_store_loaded::{
    ListedPlan,
    read_the_plans,
};
/// Publish the plan on disk,
/// its format's version
/// and the plans a start reads back,
/// under the names a caller already reads them by.
pub use super::push_retry_store_loaded::{
    LoadedPlans,
    load,
    load_reporting,
};
/// Import the plan's record,
/// its extension,
/// the bound a plan file is held to,
/// and what reads it back.
use super::push_retry_store_plan_record::{
    PLAN_EXTENSION,
    PLAN_FILE_LIMIT,
    decode,
    encode,
};
pub use super::push_retry_store_plan_record::{
    PLAN_VERSION,
    StoredPlan,
};
/// Import the hash a plan's name is built from,
/// which the person's own notice record is named by too.
use crate::auto_push_branch_key::fnv_hex;
/// Import the blocked job a plan's step runs as,
/// and the answer it gives.
use crate::blocking_jobs_within_the_budget::{
    TheAnswerOfAJobAwaited,
    run_a_job_once_awaited_no_longer_than,
};
/// Import the reader and writer of a path's bytes in a record,
/// which is what a directory whose name is not text needs.
use crate::path_bytes_in_records as path_bytes;
/// Import the storage that makes the directory,
/// reads a record whole and writes the file whole.
use crate::private_storage::{
    create_private_directory_all,
    read_regular_file,
    write_private_file_unsynced,
};
/// Import the ceiling the after-commit work's state step is given,
/// which names the plan among what it covers.
use crate::wait_ceilings::AFTER_COMMIT_STATE_CEILING;
/// Import the push item a plan's record holds,
/// which the controls build their own plans from.
#[cfg(test)]
use crate::work_item::WorkItem;
/// Import the borrowed and owned path types a plan's directory is named with.
use std::path::{
    Path,
    PathBuf,
};
/// Import the time a step is allowed,
/// and the time a plan was written,
/// which one control hands to the reading of its listing.
use std::time::Duration;
#[cfg(test)]
use std::time::SystemTime;

/// Separates the parts of a plan file's name;
/// it is not a character a hexadecimal digit can be.
const NAME_SEPARATOR: char = '~';

/// The part of every file name that belongs to one branch of one Git directory.
/// Why:
///   The key is built from the bytes of the directory,
/// so two directories whose paths differ only in a byte that is not text get a key each;
/// their text is the same,
/// and a shared key would let the plan of the second remove the plan of the first.
fn branch_key(
    git_dir: &Path,
    branch_ref: &str,
) -> String {
    return fnv_hex(format!("{}\n{branch_ref}", path_bytes::hex_of_path(git_dir)).as_str());
}

/// The key a release before the bytes were used wrote,
/// so that its plan files are still found and removed.
/// Why:
///   A plan file an earlier release wrote carries this key,
/// and `remove` after an upgrade would otherwise leave it to be read again at every start.
fn earlier_branch_key(
    git_dir: &Path,
    branch_ref: &str,
) -> String {
    return fnv_hex(format!("{}\n{branch_ref}", git_dir.display()).as_str());
}

/// The file name of one plan:
/// the branch's key,
/// the try number and a hash of the commit.
fn plan_name(plan: &StoredPlan) -> String {
    return format!(
        "{}{NAME_SEPARATOR}{}{NAME_SEPARATOR}{}.{PLAN_EXTENSION}",
        branch_key(plan.git_dir.as_path(), plan.item.branch_ref.as_str()),
        plan.attempt,
        fnv_hex(plan.item.oid.as_str())
    );
}

/// The file name the plan of a branch is kept under,
/// for a caller that must know which files a round is about to write.
pub fn name_of(plan: &StoredPlan) -> String {
    return plan_name(plan);
}

/// What:
///  Writes the plan of a branch and removes the files of its earlier plans.
/// Why:
///   The new file is published whole before the old ones go,
/// so a kill between the two leaves a plan,
/// not nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeExclusive(join(dir, name(plan)), encode(plan)); await removeOthersOfBranch(dir, plan);
/// ```
pub fn save(
    directory: &Path,
    plan: &StoredPlan,
) -> Result<(), String> {
    if let Err(error) = create_private_directory_all(directory) {
        return Err(format!("creating the plan directory failed: {error}"));
    }
    let name: String = plan_name(plan);
    match write_private_file_unsynced(
        directory.join(name.as_str()).as_path(),
        encode(plan).as_bytes(),
    ) {
        Ok(()) => {}
        // The same try of the same commit is already there.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(format!("writing the plan failed: {error}")),
    }
    remove_files_of(
        directory,
        plan.git_dir.as_path(),
        plan.item.branch_ref.as_str(),
        &[
            branch_key(plan.git_dir.as_path(), plan.item.branch_ref.as_str()),
            earlier_branch_key(plan.git_dir.as_path(), plan.item.branch_ref.as_str()),
        ],
        std::slice::from_ref(&name),
    );
    return Ok(());
}

/// What:
///  Removes the files of the plan of a branch;
/// one that cannot be removed stays and is read again by the next start.
/// Why:
///   The worker that decided the branch needs no new try removes the file itself,
/// so a forget travels to the journal in the round that decided it
/// and no later round replays it onto a plan that has been planned again since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeFilesOf(gitDir, branchRef, keys, [])
/// ```
pub fn remove(
    directory: &Path,
    git_dir: &Path,
    branch_ref: &str,
) {
    remove_files_of(
        directory,
        git_dir,
        branch_ref,
        &[
            branch_key(git_dir, branch_ref),
            earlier_branch_key(git_dir, branch_ref),
        ],
        &[],
    );
}

/// Removes the plan files of one branch of `git_dir`,
/// whose keys are `keys`,
/// except the ones named in `keep`,
/// and removes a file only when its record was read and names this directory and this branch.
/// Why:
///   "never discard user data."
/// The key an earlier release wrote was built from the text of the path,
/// and two directories whose names differ only in a byte that is not text have one text,
/// so that key alone can name both;
/// only the record inside the file says which plan it holds.
/// A file that cannot be read,
/// or that holds no plan,
/// says nothing about the push it schedules,
/// which may be neither done nor cancelled,
/// so it stays and `load` judges it and says so,
/// since deleting it needs permission on its directory alone.
///  The names in `keep` are the plans a round is about to write,
/// which a forget of the same round must not take.
fn remove_files_of(
    directory: &Path,
    git_dir: &Path,
    branch_ref: &str,
    keys: &[String],
    keep: &[String],
) {
    let listing = match std::fs::read_dir(directory) {
        Ok(listing) => listing,
        // A journal that never held a plan.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            crate::service_warn!(
                "the plan files of {branch_ref} in {} could not be listed, so the next start reads them again: {error}",
                directory.display()
            );
            return;
        }
    };
    for listed in listing {
        let entry = match listed {
            Ok(entry) => entry,
            // An entry the listing could not read may be a plan of this branch, and it stays to be read again.
            Err(error) => {
                crate::service_warn!(
                    "the plan files of {branch_ref} in {} could not all be listed, so the next start reads them again: {error}",
                    directory.display()
                );
                continue;
            }
        };
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if keep.contains(&name) || !names_the_branch(keys, name.as_str()) {
            continue;
        }
        let path: PathBuf = entry.path();
        if !records_this_branch(path.as_path(), git_dir, branch_ref) {
            continue;
        }
        // A plan that stays says so, so a try the person was told about is not lost in silence.
        if let Err(error) = std::fs::remove_file(path.as_path()) {
            crate::service_warn!(
                "the retry plan file {} could not be removed, so the next start reads it again: {error}",
                path.display()
            );
        }
    }
}

/// Whether the plan in `path` was read and names `git_dir` and `branch_ref`,
/// which is the only case a cleanup may remove it.
/// Why:
///   A read or decode that fails is no proof about the plan:
/// an unreadable file and a file that holds no plan both stay,
/// and `load` says what it found and removes what is no plan.
/// The record is read the way every record of this wrapper is read
/// (`private_storage::read_regular_file`):
/// never through a link,
/// and never waiting inside `open(2)` for a writer that a name holding a pipe waits for.
fn records_this_branch(
    path: &Path,
    git_dir: &Path,
    branch_ref: &str,
) -> bool {
    let Ok(bytes) = read_regular_file(path, PLAN_FILE_LIMIT) else {
        return false;
    };
    let Ok(text) = std::str::from_utf8(bytes.as_slice()) else {
        return false;
    };
    let Ok(found) = decode(text) else {
        return false;
    };
    return found.git_dir == git_dir && found.item.branch_ref == branch_ref;
}

/// Whether a plan file's name begins with the key of one of the branch's plans.
fn names_the_branch(
    keys: &[String],
    name: &str,
) -> bool {
    for key in keys {
        if name.starts_with(format!("{key}{NAME_SEPARATOR}").as_str()) {
            return true;
        }
    }
    return false;
}

/// What:
///  Runs one step over the plans' files as a job of the process's blocking pool,
/// awaiting it no longer than `ceiling`,
/// and gives its answer or the words it is refused with.
/// Why:
///   Such a step lists a directory,
/// reads records and writes or removes files,
/// and every one of those can wait in the kernel on a disk,
/// a lock or a mount that stopped answering.
/// The caller is a task of the runtime,
/// and a task that made the call itself would hold the worker the runtime needs,
/// where no timer can end its wait
/// (`blocking_jobs_within_the_budget.rs`).
/// The ceiling is the after-commit work's state step,
/// which names the plan among what it covers (`wait_ceilings.rs`).
/// What the answer says is what happened:
/// no place within the ceiling proves the step did not run,
/// while a step that outlasted its own ceiling may still be finishing what it began.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await aPlanFileStep(AFTER_COMMIT_STATE_CEILING, () => save(dir, plan))
/// ```
async fn a_plan_file_step_awaited_no_longer_than(
    ceiling: Duration,
    step: &str,
    job: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Result<(), String> {
    return match run_a_job_once_awaited_no_longer_than(ceiling, job).await {
        TheAnswerOfAJobAwaited::Ran(answer) => answer,
        TheAnswerOfAJobAwaited::NotRun => Err(format!(
            "{step} did not run, because no place of the process's bound and no thread came within {ceiling:?}"
        )),
        TheAnswerOfAJobAwaited::NoAnswer => Err(format!(
            "{step} gave no answer within {ceiling:?}, so what it began may still be being written"
        )),
    };
}

/// The journal a step of the plans' files works over.
/// Why:
///   A job of the blocking pool takes its state by value and is `'static`,
/// so the paths it works over travel with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanJournal = { directory: string; gitDir: string; branchRef: string };
/// ```
struct PlanJournal {
    /// The directory the plan's files are kept in.
    directory: PathBuf,
    /// The Git directory the plans belong to.
    git_dir: PathBuf,
    /// The branch whose files the step touches.
    branch_ref: String,
}

/// What:
///  Writes the plan of a branch and removes the files of its earlier plans,
/// as a job of the blocking pool.
/// Why:
///   `save` is storage,
/// and the worker that decided a retry keeps the plan on it,
/// which is a task of the runtime.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await saveAwaiting(dir, plan)
/// ```
pub async fn save_awaiting(
    directory: &Path,
    plan: &StoredPlan,
) -> Result<(), String> {
    let state: (PathBuf, StoredPlan) = (directory.to_path_buf(), plan.clone());
    return a_plan_file_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the write of a retry plan",
        move || {
            return save(state.0.as_path(), &state.1);
        },
    )
    .await;
}

/// What:
///  Removes the files of the plan of a branch,
/// as a job of the blocking pool.
/// Why:
///   `remove` is storage,
/// and the worker that decided the branch needs no new try runs it,
/// which is a task of the runtime.
/// A removal that gave no answer is said by the caller,
/// and the file stays for the next start either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeAwaiting(dir, gitDir, branchRef)
/// ```
pub async fn remove_awaiting(
    directory: &Path,
    git_dir: &Path,
    branch_ref: &str,
) -> Result<(), String> {
    let state: PlanJournal = PlanJournal {
        directory: directory.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(branch_ref),
    };
    return a_plan_file_step_awaited_no_longer_than(
        AFTER_COMMIT_STATE_CEILING,
        "the removal of a branch's retry plan",
        move || {
            remove(
                state.directory.as_path(),
                state.git_dir.as_path(),
                state.branch_ref.as_str(),
            );
            return Ok(());
        },
    )
    .await;
}

/// Plan file controls stay out of the release executable.
#[cfg(test)]
#[path = "push_retry_store_tests.rs"]
mod tests;
