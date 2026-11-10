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
//! and `Cargo.lock` put back as checked out,
//! because Cargo resolves the worktree's workspace for the clean.
//! A destination that exists means a build got there first,
//! so the staging copy is dropped and the build keeps everything it made.
//! The staging directory,
//! the clean's target directory and the marker beside them are names of this job's own,
//! and the marker is what proves a directory at one of them is its leftover:
//! a directory no marker claims is somebody's work and is never removed,
//! so a workspace whose usual names are taken is filled under names of this run's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await prewarmAwaiting(item); // for each entry: clone to the staging name, clean it, publish it unless the destination exists
//! ```

use crate::diagnostic_log::debug;
use crate::package_repair_cargo::clean_arguments;
use crate::package_repair_cargo_lock::{read_lock, restore_lock};
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolLimits, ToolRun, find_program, run_tool_awaiting};
use crate::package_repair_prewarm::{BUILD_DIRECTORY_VARIABLE, PrewarmEntry, STAGING_SUFFIX, TARGET_DIRECTORY_VARIABLE, decode};
use crate::package_repair_request::RepairEnvironment;
use crate::package_repair_tree_clone::{Cloned, clone_tree_awaiting};
use crate::rename_that_never_replaces::{rename_noreplace, rename_unsupported};
use crate::service_work_handlers_replanning_and_acceptance::Verdict;
use crate::work_item::WorkItem;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairPrewarmJob";

/// The program the filling resolves,
/// as the repair does.
const PROGRAM: &str = "cargo";

/// Where a worktree keeps its own link to the toolchain's Cargo,
/// relative to the worktree root,
/// as the toolchain repair resolves `mpm-shadow`.
const OWN_CARGO_LINK: &str = ".cache/toolchain-bin/cargo";

/// What:
///  What every workspace of one record needs before it copies anything.
/// Why:
///  The program lookup and the bounds are the same for every workspace of one record,
/// so they are read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fill = { cargo: string; limits: ToolLimits };
/// ```
pub(crate) struct Fill {
    /// The Cargo that cleans the staging copy.
    pub(crate) cargo: PathBuf,
    /// The bounds it runs under.
    pub(crate) limits: ToolLimits,
}

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
    if let Some(found) = find_program(PROGRAM, environment.path()) {
        return Some(found);
    }
    let own: PathBuf = root.join(OWN_CARGO_LINK);
    return if own.is_file() { Some(own) } else { None };
}

/// What:
///  The staging directory a workspace's build directory is cloned into.
/// Why:
///   It sits beside the destination,
/// so publishing it is one rename on one filesystem,
/// and its name is the destination's,
/// so no workspace can fill another's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stagingOf = (destination) => `${destination}${STAGING_SUFFIX}`;
/// ```
pub fn staging_of(destination: &Path) -> PathBuf {
    let mut named: OsString = destination.as_os_str().to_os_string();
    named.push(STAGING_SUFFIX);
    return PathBuf::from(named);
}

/// What:
///  The target directory the clean is given,
/// so it deletes nothing from the worktree's own.
/// Why:
///  `cargo clean --package` removes a package's final artifacts from the target directory as well as
///  its intermediates from the build directory,
/// and the worktree's target directory may hold what the copy carried and what a build made.
/// Naming a directory of this job's own keeps the filling's deletions inside directories it made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const targetOf = (staging) => `${staging}${TARGET_SUFFIX}`;
/// ```
fn target_of(staging: &Path) -> PathBuf {
    let mut named: OsString = staging.as_os_str().to_os_string();
    named.push(TARGET_SUFFIX);
    return PathBuf::from(named);
}

/// What the target directory of a clean is suffixed with,
/// beside the staging directory.
const TARGET_SUFFIX: &str = ".target";

/// What:
///  The marker that says the staging copy and the clean's target beside it are this job's own.
/// Why:
///   The two names are this job's by convention,
/// and a directory at one of them is this job's only when a filling of it said so:
/// the marker is written before anything is cloned or cleaned,
/// so a filling that was killed leaves the proof its successor needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markerOf = (staging) => `${staging}${MARKER_SUFFIX}`;
/// ```
fn marker_of(staging: &Path) -> PathBuf {
    let mut named: OsString = staging.as_os_str().to_os_string();
    named.push(MARKER_SUFFIX);
    return PathBuf::from(named);
}

/// What the marker of the staging name is suffixed with,
/// beside the staging directory.
const MARKER_SUFFIX: &str = ".owner";

/// The text a marker carries,
/// so a person who finds one knows which program wrote it and what it claims.
const MARKER_TEXT: &[u8] = b"monochromatic-git-policy-cli's Cargo prewarm job wrote this file. \
The directory whose name is this file's name without '.owner' is its staging copy, \
and the '.target' directory beside it is the target its clean deletes from. \
Both are the wrapper's own and are removed or published when the filling ends.\n";

/// What:
///  The three names one filling of one workspace works in.
/// Why:
///   The staging copy is what the source's build state is cloned into and published from,
/// the target directory is what the clean deletes from,
/// and the marker is the proof a later filling needs before it removes either.
/// A workspace whose usual names another directory holds is filled under names of this run's own,
/// so filling it never means deleting somebody's work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheNames = { staging: string; target: string; marker?: string };
/// ```
struct TheNames {
    /// The directory the source's build state is cloned into,
    /// and published from.
    staging: PathBuf,
    /// The target directory the clean deletes from.
    target: PathBuf,
    /// The marker that says the two names are this job's,
    /// when this run claimed the usual ones.
    marker: Option<PathBuf>,
}

/// What:
///  Take the names one workspace is filled under:
/// the usual ones when nothing is there or when this job's own marker claims what is,
/// and names of this run's own when another directory holds the usual ones.
/// Why:
///   The usual name is where a killed filling's leftovers sit,
/// and only the marker proves they are this job's,
/// so a directory no marker claims is never removed and the filling still happens.
///
/// # Errors
///
/// Leftovers this job's marker claims that cannot be removed,
/// and a marker that cannot be written.
fn names_for(destination: &Path) -> Result<TheNames, String> {
    let usual: PathBuf = staging_of(destination);
    let usual_target: PathBuf = target_of(usual.as_path());
    let marker: PathBuf = marker_of(usual.as_path());
    if marker.exists() {
        // An earlier filling of this same workspace was interrupted, so what it left under these names is this job's.
        remove_if_any(usual.as_path())?;
        remove_if_any(usual_target.as_path())?;
        return Ok(TheNames { staging: usual, target: usual_target, marker: Some(marker) });
    }
    if usual.exists() || usual_target.exists() {
        // Another directory holds the usual name: it is somebody's work, so this run stages under a name of its own.
        let own: PathBuf = a_name_of_this_runs_own(usual.as_path());
        let own_target: PathBuf = target_of(own.as_path());
        return Ok(TheNames { staging: own, target: own_target, marker: None });
    }
    write_marker(marker.as_path())?;
    return Ok(TheNames { staging: usual, target: usual_target, marker: Some(marker) });
}

/// What:
///  A staging name for a workspace whose usual name another directory holds:
/// the usual name,
/// this process and the moment it asked.
/// Why:
///   Nothing may be removed that this job cannot prove it made,
/// and a name no other run has taken yet is proof enough:
/// the clone refuses a destination that exists,
/// and this run removes only the names it chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anOwnName = (usual) => `${usual}-${process.pid}-${nanos.toString(16)}`;
/// ```
fn a_name_of_this_runs_own(usual: &Path) -> PathBuf {
    let since_the_epoch: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap_or_default();
    let nanos: u128 = since_the_epoch.as_nanos();
    let mut named: OsString = usual.as_os_str().to_os_string();
    named.push(format!("-{}-{nanos:x}", std::process::id()));
    return PathBuf::from(named);
}

/// What:
///  Write the marker that claims the two names of one workspace.
/// Why:
///   An exclusive write makes two fillings of one workspace agree on one marker instead of replacing
/// each other's,
/// and a marker that is already there is this job's own either way.
///
/// # Errors
///
/// A marker that cannot be written,
/// because what the names might hold can then no longer be told from somebody's work.
fn write_marker(marker: &Path) -> Result<(), String> {
    return match crate::private_storage::write_private_file_unsynced(marker, MARKER_TEXT) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(format!("{} could not be written, so nothing of this workspace was filled ({error})", marker.display())),
    };
}

/// What:
///  Give the two names back when the filling of one workspace ends,
/// whatever it came to.
/// Why:
///   The target directory is the clean's and the marker is the claim's,
/// and both are removed here rather than on the failing paths alone,
/// so a successful filling leaves no directory of this job's behind for every build directory it fills.
/// The marker goes after the target,
/// so a filling that is killed in between leaves the proof its successor needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeIfAny(target); removeFile(marker, { force: true });
/// ```
fn give_back(names: &TheNames) {
    let _removed_target = remove_if_any(names.target.as_path());
    let Some(marker) = names.marker.as_ref() else { return };
    match std::fs::remove_file(marker.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => debug(TAG, format!("{} could not be removed after the filling ({error})", marker.display()).as_str()),
    }
}

/// What:
///  Remove a directory this job made when a step failed or a build got there first.
/// Why:
///   The names belong to this job alone,
/// and the caller removes only the names this run chose or its own marker claims,
/// so what they hold was made by an earlier filling of the same workspace and nothing a person or a
/// build made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeIfAny(path); // absent is fine
/// ```
fn remove_if_any(path: &Path) -> Result<(), String> {
    return match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{} could not be removed ({error})", path.display())),
    };
}

/// What:
///  Clean the worktree's own crates out of the staging copy.
/// Why:
///   The source's own artifacts name the source's paths,
/// so they must not reach the worktree,
/// while the dependency builds hold nothing path-specific and are the reason the copy exists.
/// Both directories are this job's own,
/// which is what keeps the clean away from the workspace's live state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cleanTheCopy({ fill, entry, names }): ToolRun
/// ```
async fn clean_the_copy(fill: &Fill, entry: &PrewarmEntry, names: &TheNames) -> ToolRun {
    let command: ToolCommand = ToolCommand {
        arguments: clean_arguments(entry.members.as_slice()),
        directory: entry.root.clone(),
        set: vec![
            (OsString::from(BUILD_DIRECTORY_VARIABLE), names.staging.as_os_str().to_os_string()),
            (OsString::from(TARGET_DIRECTORY_VARIABLE), names.target.as_os_str().to_os_string()),
        ],
    };
    return run_tool_awaiting(fill.cargo.as_path(), &command, fill.limits).await;
}

/// What:
///  Publish the staging copy under the workspace's build directory name,
/// or say that something else is already there.
/// Why:
///   The rename never replaces,
/// so a build that started while the copy was being made keeps every artifact it wrote and the
/// staging copy is the only thing removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const publish = (staging, destination) => renameNoReplace(staging, destination);
/// ```
fn publish(staging: &Path, destination: &Path) -> Result<bool, String> {
    return match rename_noreplace(staging, destination) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) if rename_unsupported(&error) => Err(format!("{} cannot be published, because this filesystem cannot rename without replacing ({error})", destination.display())),
        Err(error) => Err(format!("{} could not be published ({error})", destination.display())),
    };
}

/// What a workspace's filling came to.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Filled {
    /// The copy was published under the workspace's build directory name.
    Published,
    /// The worktree already had a build directory,
    /// so nothing was published.
    Kept,
}

/// What:
///  Fill one workspace's build directory:
/// take the names this job fills under,
/// clone the source's,
/// clean the copy,
/// publish it,
/// and give the names back.
/// Why:
///   Each workspace's state is in a directory of its own,
/// so one workspace's failure is reported and the next one still runs.
/// The names are taken before anything is written,
/// so nothing this job did not make is removed,
/// and given back whatever the filling came to,
/// so a successful one leaves no directory behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fillOne({ fill, entry }): Promise<Filled>;
/// ```
async fn fill_one(fill: &Fill, entry: &PrewarmEntry) -> Result<Filled, String> {
    // A destination that exists is a build that got there first: the worktree keeps everything it made.
    if entry.to.exists() {
        return Ok(Filled::Kept);
    }
    let names: TheNames = names_for(entry.to.as_path())?;
    let filled: Result<Filled, String> = fill_the_staging(fill, entry, &names).await;
    give_back(&names);
    return filled;
}

/// What:
///  The three steps of one workspace's filling:
/// clone the source's build directory into the staging copy,
/// clean the worktree's own crates out of the copy,
/// and publish it under the worktree's name.
/// Why:
///   A copy that could not be finished is this job's own,
/// and nothing else reads it,
/// so each step that fails removes it and says which step it was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function fillTheStaging({ fill, entry, names }): Promise<Filled>;
/// ```
async fn fill_the_staging(fill: &Fill, entry: &PrewarmEntry, names: &TheNames) -> Result<Filled, String> {
    let made: Result<Cloned, String> = clone_tree_awaiting(entry.from.as_path(), names.staging.as_path(), 1).await;
    if let Err(reason) = made {
        let _removed = remove_if_any(names.staging.as_path());
        return Err(format!("the build state of {} could not be cloned ({reason})", entry.from.display()));
    }
    // A build that started during the clone owns the destination now, and this job touches nothing of it.
    if entry.to.exists() {
        let _removed = remove_if_any(names.staging.as_path());
        return Ok(Filled::Kept);
    }
    // Cargo resolves the worktree's workspace for this clean even though it deletes only from directories of this job's
    //  own, and that rewrites a `Cargo.lock` that no longer matches its manifest or creates one that is missing. The
    //  worktree must stay as Git wrote it, so the lock is put back exactly as the add's own clean puts it back.
    let lock_before: Option<Vec<u8>> = read_lock(entry.root.as_path());
    let run: ToolRun = clean_the_copy(fill, entry, names).await;
    restore_lock(entry.root.as_path(), lock_before.as_deref());
    if !matches!(run.end, ToolEnd::Succeeded) {
        let _removed = remove_if_any(names.staging.as_path());
        return Err(format!("the copy of {} could not be cleaned ({:?})", entry.from.display(), run.end));
    }
    return match publish(names.staging.as_path(), entry.to.as_path()) {
        Ok(true) => Ok(Filled::Published),
        Ok(false) => {
            let _removed = remove_if_any(names.staging.as_path());
            Ok(Filled::Kept)
        }
        Err(reason) => {
            let _removed = remove_if_any(names.staging.as_path());
            Err(reason)
        }
    };
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
    let fill: Fill = Fill { cargo, limits: environment.limits() };
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

/// The filling's controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_prewarm_job_tests.rs"]
mod tests;
