//! What:
//!  Fill the Cargo build directories a new worktree's record asks for,
//! inside the commit service,
//! after the add that queued it has returned.
//! Why:
//!  The clone of a source checkout's build directory is the heaviest step the package repair
//!  does,
//! and it only ever fills a directory the new worktree does not have yet,
//! so nothing about the worktree's correctness depends on it finishing inside the add
//! (`package_repair_prewarm.rs`).
//! The filling is three steps per workspace:
//! clone the source's build directory into a staging directory beside the destination,
//! clean the worktree's own crates out of the staging copy,
//! and publish the staging copy under the destination's name by a rename that never replaces.
//! Both directories the clean reads and writes are named here,
//! so the filling deletes nothing from the worktree itself:
//! its only effects there are the one rename that publishes a directory the worktree did not have
//! and the `Cargo.lock` the checked-out one goes back over,
//! because Cargo resolves the worktree's workspace for the clean.
//! That lock is never written by the clean:
//! it runs with `--locked`,
//! a lock such a clean refuses is resolved by Cargo first,
//! and the checked-out bytes go back only when the file still holds exactly what that resolution wrote,
//! so a write of the person's lands in their own file and stays there.
//! A destination that exists means a build got there first,
//! so the staging copy is dropped and the build keeps everything it made.
//! The staging directory,
//! the clean's target directory and the marker beside them are names of this job's own,
//! and the marker holding this job's own text is what proves a directory at one of them is its leftover:
//! anything at those names without that proof is somebody's work and is never removed,
//! so a workspace whose usual names are taken is filled under names of this run's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await prewarmAwaiting(item); // for each entry: clone to the staging name, clean it, publish it unless the destination exists
//! ```
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_prewarm_job_names.rs`:
//!   the staging name,
//!   the target directory,
//!   the marker that claims them,
//!   and taking and giving the names back.
//! - `package_repair_prewarm_job_staged_state.rs`:
//!   the add's move of the worktree's copied build state,
//!   under that marker.
//! - `package_repair_prewarm_job_filling.rs`:
//!   the clone,
//!   the clean and the publish of one workspace's build directory.

/// Import the filling work one entry is handed and the one workspace it fills.
use self::filling::{Fill, Filled, fill_one};
/// Import the tag every line of this module's diagnostic log carries.
use self::names::TAG;
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the lookup of the program a filling runs.
use crate::package_repair_command::find_program;
/// Import the record's entries and the reading of them.
use crate::package_repair_prewarm::{PrewarmEntry, decode};
/// Import what the process offers a filling.
use crate::package_repair_request::RepairEnvironment;
/// Import what one work item came to.
use crate::service_work_handlers_replanning_and_acceptance::Verdict;
/// Import the one item a record of the service holds.
use crate::work_item::WorkItem;
/// Import native paths.
use std::path::{Path, PathBuf};

/// The program the filling resolves,
/// as the repair does.
const PROGRAM: &str = "cargo";

/// Where a worktree keeps its own link to the toolchain's Cargo,
/// relative to the worktree root,
/// as the toolchain repair resolves `mpm-shadow`.
const OWN_CARGO_LINK: &str = ".cache/toolchain-bin/cargo";

/// What:
///  The program that cleans a staging copy:
/// the search path's `cargo`,
/// else the worktree's own link to the toolchain's.
/// Why:
///   The service is a user unit whose search path need not hold the toolchain the worktree's
/// launchers name,
/// and the worktree's own link is the one its commands use.
/// The link sits at the worktree's root,
/// so the caller passes the worktree root and not the workspace below it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cargoFor = (worktreeRoot, environment) => findProgram('cargo', environment.path()) ?? (exists(join(worktreeRoot, OWN_CARGO_LINK)) ? join(worktreeRoot, OWN_CARGO_LINK) : undefined);
/// ```
fn cargo_for(root: &Path, environment: &RepairEnvironment) -> Option<PathBuf> {
    if let Some(found) = find_program(PROGRAM, environment.path(), &environment.start_directory) {
        return Some(found);
    }
    let own: PathBuf = root.join(OWN_CARGO_LINK);
    return if own.is_file() { Some(own) } else { None };
}

/// What:
///  Fill every build directory one prewarm record names,
/// awaited by the service's round.
/// Why:
///   The add no longer does this work (`package_repair_prewarm.rs`);
/// the record says which workspaces,
/// which source checkout,
/// and which members must not be carried,
/// so no metadata read is repeated here.
/// A record whose payload cannot be read is a failure that says why,
/// and one workspace that fails does not stop the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prewarmAwaiting(item): Promise<Verdict>;
/// ```
pub async fn prewarm_awaiting(item: &WorkItem) -> Verdict {
    return prewarm_awaiting_with(item, &RepairEnvironment::from_process()).await;
}

/// What:
///  `prewarm_awaiting` with the environment handed over instead of read from the process.
/// Why:
///   The program lookup and the two bounds come from the process,
/// which a control cannot change,
/// so a control that proves where the worktree's own Cargo is found hands its own environment in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// prewarmAwaitingWith(item, environment);
/// ```
async fn prewarm_awaiting_with(item: &WorkItem, environment: &RepairEnvironment) -> Verdict {
    let entries: Vec<PrewarmEntry> = match decode(item.parent.as_str()) {
        Ok(decoded) => decoded,
        Err(reason) => return Verdict::Failed(reason),
    };
    if entries.is_empty() {
        return Verdict::Skipped(String::from("the record names no workspaces"));
    }
    // The worktree's own link to the toolchain's Cargo sits at the worktree's root, whatever workspace the first entry
    //  names, so the record's repository is what the lookup starts from.
    let Some(cargo) = cargo_for(item.repository.as_path(), environment) else {
        return Verdict::Failed(format!("no `{PROGRAM}` could be found to clean the copies with"));
    };
    let fill: Fill = Fill { cargo, limits: environment.limits(), kept_in: Some(item.git_dir.clone()) };
    let mut published: usize = 0;
    let mut kept: usize = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in &entries {
        match fill_one(&fill, entry).await {
            Ok(Filled::Published) => published += 1,
            Ok(Filled::Kept) => kept += 1,
            Err(reason) => failures.push(reason),
        }
    }
    debug(TAG, format!("{}: {published} published, {kept} kept, {} failed", item.repository.display(), failures.len()).as_str());
    return match failures.first() {
        None => Verdict::Done,
        Some(first_failure) => Verdict::Failed(first_failure.clone()),
    };
}

/// What:
///  `fill_one` for a control,
/// which may not be a task of the runtime.
/// Why:
///   The filling awaits a tree clone and a Cargo child,
/// so a control enters the process's runtime once and waits there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fillOneInAControl = (fill, entry) => runOnRuntime(fillOne(fill, entry));
/// ```
#[cfg(test)]
pub(crate) fn fill_one_in_a_control(fill: &Fill, entry: &PrewarmEntry) -> Result<Filled, String> {
    return match crate::lazy_runtime::run_on_runtime(fill_one(fill, entry)) {
        Ok(filled) => filled,
        Err(refusal) => Err(format!("the filling could not run ({refusal})")),
    };
}

/// The names one filling works in and the marker that proves they are this job's.
#[path = "package_repair_prewarm_job_names.rs"]
mod names;
/// The add's own move of the worktree's copied build state.
#[path = "package_repair_prewarm_job_staged_state.rs"]
mod staged_state;
/// The clone,
/// the clean and the publish of one workspace's build directory.
#[path = "package_repair_prewarm_job_filling.rs"]
mod filling;

/// The staging name of a workspace's build directory,
/// which the add also names.
pub use self::names::staging_of;
/// What the add's move of the copied build state came to,
/// and the move itself.
pub(crate) use self::staged_state::{Staged, stage_the_worktrees_own_state};

// The names this module's controls reach through `super::`,
// re-exported so a control sees the whole module.
#[cfg(test)]
pub(crate) use self::filling::{publish, what_to_do_next};
#[cfg(test)]
pub(crate) use self::names::{marker_of, target_of, write_marker};
#[cfg(test)]
pub(crate) use self::staged_state::{EXCLUDED_STAGING_PATTERNS, exclude_the_staging_names};

/// The filling's controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_prewarm_job_tests.rs"]
mod tests;
