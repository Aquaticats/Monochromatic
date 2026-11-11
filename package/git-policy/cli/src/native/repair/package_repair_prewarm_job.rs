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
//! its only effect there is the one rename that publishes a directory the worktree did not have,
//! because the clean is told to leave `Cargo.lock` alone and refuses rather than write it
//! (`package_repair_cargo_lock.rs` says how the add's own clean treats the same file).
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

use crate::diagnostic_log::debug;
use crate::package_repair_cargo::{clean_arguments, manual_command};
/// The lock file the filling's clean must not write.
use crate::package_repair_cargo_lock::LOCK_FILE;
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
///  Whether a marker is this job's:
/// the exact text its own filling wrote,
/// read only from a name that holds a regular file,
/// through an open that cannot wait.
/// Why:
///   The name alone is a convention anyone can type,
/// and only the text this job writes is proof that it made the directories the name is derived from.
/// The marker is published whole by an exclusive write,
/// so a file at that name either holds that text or is somebody else's.
/// A named pipe at that name would hold a plain read for ever,
/// because opening one for reading waits for a writer that may never come,
/// and one foreign object at the name would then stop the service's filling with no end
/// (SPEC.md,
/// "Every wait has a bound").
/// `read_regular_file` opens without waiting and reads only what the descriptor proves is a file,
/// so a pipe,
/// a socket or a link is somebody else's and answers at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markerClaims = (marker) => readRegularFile(marker, MARKER_TEXT.length + 1)?.equals(MARKER_TEXT) ?? false;
/// ```
fn marker_claims(marker: &Path) -> bool {
    let limit: u64 = (MARKER_TEXT.len() as u64).saturating_add(1);
    return matches!(crate::private_storage::read_regular_file(marker, limit), Ok(text) if text == MARKER_TEXT);
}

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
/// and names of this run's own when another file or directory holds the usual ones.
/// Why:
///   The usual name is where a killed filling's leftovers sit,
/// and only the marker this job wrote proves they are its own,
/// so nothing at those names is removed without that proof and the filling still happens.
/// Two fillings of one workspace can run at once when a worktree is removed and added again at the
/// same path while its first record is still queued,
/// and then the second one removes the first one's staging copy and the first one reports a failure;
/// the loser touches nothing but directories this job made.
/// Another command can create the marker name between this run's look at it and its own exclusive
/// write of it,
/// and that write then answers that somebody's file is there:
/// this run writes nothing there and stages under names of its own.
///
/// # Errors
///
/// Leftovers this job's marker claims that cannot be removed,
/// and a marker that cannot be written.
fn names_for(destination: &Path) -> Result<TheNames, String> {
    let usual: PathBuf = staging_of(destination);
    let usual_target: PathBuf = target_of(usual.as_path());
    let marker: PathBuf = marker_of(usual.as_path());
    if marker_claims(marker.as_path()) {
        // An earlier filling of this same workspace was interrupted, so what it left under these names is this job's.
        remove_if_any(usual.as_path())?;
        remove_if_any(usual_target.as_path())?;
        return Ok(TheNames { staging: usual, target: usual_target, marker: Some(marker) });
    }
    if usual.exists() || usual_target.exists() || marker.exists() {
        // Somebody else's file or directory holds one of the names, so this run stages under names of its own.
        return Ok(names_of_this_runs_own(usual.as_path()));
    }
    return match write_marker(marker.as_path())? {
        TheMarkersAnswer::WrittenByThisRun => Ok(TheNames { staging: usual, target: usual_target, marker: Some(marker) }),
        // Another command has the name: what is there is not this run's to remove, and this run stages
        //  under names of its own.
        TheMarkersAnswer::WrittenByAnother => Ok(names_of_this_runs_own(usual.as_path())),
    };
}

/// What:
///  The three names a filling of one workspace uses when another file or directory holds the usual
/// ones:
/// a staging name of this run's own,
/// its target beside it,
/// and no marker,
/// because this run wrote none.
/// Why:
///   A run that removes only names it chose itself cannot delete somebody else's work,
/// and a marker this run did not write is never its to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesOfThisRunsOwn = (usual) => { const staging = anOwnName(usual); return { staging, target: targetOf(staging) }; };
/// ```
fn names_of_this_runs_own(usual: &Path) -> TheNames {
    let own: PathBuf = a_name_of_this_runs_own(usual);
    let own_target: PathBuf = target_of(own.as_path());
    return TheNames { staging: own, target: own_target, marker: None };
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

/// What a run's exclusive write of one marker came to.
/// Why:
///   The marker is removed when the filling ends,
///   and removing the name is this run's only when the write that made it was this run's:
/// a name somebody else took is theirs,
/// whatever it holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TheMarkersAnswer {
    /// This run created the marker,
    /// so it is this run's to remove.
    WrittenByThisRun,
    /// Another writer had the name already,
    /// so this run wrote nothing there.
    WrittenByAnother,
}

/// What:
///  Write the marker that claims the two names of one workspace,
/// and say whether it is this run's.
/// Why:
///   An exclusive write makes two fillings of one workspace agree on one marker instead of replacing
/// each other's,
/// and the answer says which of them made it:
/// `AlreadyExists` means another command's file is at that name,
/// which may be a person's own note,
/// so this run neither treats it as its claim nor removes it later.
///
/// # Errors
///
/// A marker that cannot be written,
/// because what the names might hold can then no longer be told from somebody's work.
fn write_marker(marker: &Path) -> Result<TheMarkersAnswer, String> {
    return match crate::private_storage::write_private_file_unsynced(marker, MARKER_TEXT) {
        Ok(()) => Ok(TheMarkersAnswer::WrittenByThisRun),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(TheMarkersAnswer::WrittenByAnother),
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
    if let Err(reason) = remove_if_any(names.target.as_path()) {
        // The filling itself is done and published, and this directory is this job's own: the next filling of the same
        //  workspace clears it.
        debug(TAG, format!("{reason}; the filling is done and the directory stays for the next one").as_str());
    }
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

/// What the clean is told about the worktree's lock,
/// so the report of a clean it stopped names the flag to look up.
const LOCK_FLAG: &str = "--locked";

/// What:
///  The arguments the filling's clean runs with:
/// the add's own clean,
/// with `--locked` added.
/// Why:
///   The clean resolves the worktree's workspace,
/// and Cargo rewrites a `Cargo.lock` that no longer matches its manifest and creates one that is
/// missing;
/// the filling runs after the add has returned,
/// while the person works in the worktree,
/// so the file it would write is the person's.
/// Reading the lock before the clean and putting it back after cannot tell Cargo's write from the
/// person's own,
/// and the bytes a kill leaves between the two are rewritten ones,
/// so the filling asks Cargo to leave the lock alone instead:
/// a lock Cargo would have to change stops the clean,
/// and the worktree keeps every byte it had.
/// The add's own clean does not pass this flag,
/// because `SPEC.md` records why its put-back stands where the worktree was just made and nobody has
/// it yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleanArgumentsOfTheCopy = (members) => [...cleanArguments(members), '--locked'];
/// ```
fn clean_arguments_of_the_copy(members: &[String]) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = clean_arguments(members);
    arguments.push(OsString::from(LOCK_FLAG));
    return arguments;
}

/// What:
///  Why a tool ended as it did,
/// in the words a person reads.
/// Why:
///   A report names what stopped the clean,
/// and the tool's own last words name the manifest or lock it refused,
/// so a variant's name alone is not the answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endReason = (end) => end.kind === 'failed' ? end.reason : end.kind === 'timedOut' ? 'it showed no sign of work for its quiet window and was stopped' : ceilingReason(Tool.cargo, end.ceiling);
/// ```
fn end_reason(end: &ToolEnd) -> String {
    return match end {
        ToolEnd::Succeeded => String::from("it succeeded"),
        ToolEnd::Failed(reason) => reason.clone(),
        ToolEnd::TimedOut => String::from("it showed no sign of work for its quiet window and was stopped"),
        ToolEnd::CeilingReached(ceiling) => crate::package_repair_report::ceiling_reason(crate::package_repair_report::Tool::Cargo, *ceiling),
    };
}

/// What:
///  What a person does next when the filling's clean stopped,
/// in the words they read.
/// Why:
///   The one failure this job causes by itself is a lock Cargo would have had to change,
/// and its remedy is the lock's own command or the clean the add would have run;
/// any other reason is the tool's,
/// and the tool's own words name it.
/// A person is told where their bytes went either way,
/// so a stopped filling never leaves them guessing whether the worktree was written.
fn what_to_do_next(entry: &PrewarmEntry, reason: &str) -> String {
    if !reason.contains(LOCK_FILE) {
        return String::from("The reason above names what to fix, and the next add fills the copy.");
    }
    return format!(
        "Update the lock in {} (for example `cargo update --offline --workspace` there) and the next add fills the copy, or clean the worktree's \
         own crates with `{}`, which may update the lock as your own clean would",
        entry.root.display(),
        manual_command(entry.root.as_path(), entry.members.as_slice()),
    );
}

/// What:
///  Clean the worktree's own crates out of the staging copy.
/// Why:
///   The source's own artifacts name the source's paths,
/// so they must not reach the worktree,
/// while the dependency builds hold nothing path-specific and are the reason the copy exists.
/// Both directories are this job's own,
/// which is what keeps the clean away from the workspace's live state,
/// and the clean is told to leave `Cargo.lock` alone,
/// which is what keeps it away from the worktree's files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cleanTheCopy({ fill, entry, names }): ToolRun
/// ```
async fn clean_the_copy(fill: &Fill, entry: &PrewarmEntry, names: &TheNames) -> ToolRun {
    let command: ToolCommand = ToolCommand {
        arguments: clean_arguments_of_the_copy(entry.members.as_slice()),
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
    make_the_destinations_own_directory(entry)?;
    let names: TheNames = names_for(entry.to.as_path())?;
    let filled: Result<Filled, String> = fill_the_staging(fill, entry, &names).await;
    give_back(&names);
    return filled;
}

/// What:
///  Make the directory the destination and its two staging names live in.
/// Why:
///   The names are taken before the clone,
/// and the marker that claims them sits beside the destination,
/// so that directory must exist before the marker is written.
/// Cargo keys an external build directory by the workspace's path,
/// and the name it keys it by is two levels below the root a person's configuration names
/// (`build-dir = "~/build/{workspace-path-hash}"` resolves to `~/build/8d/0e503e8832e433`),
/// so neither level exists before the first build of a new worktree,
/// and Cargo makes them itself before a build.
/// The clone makes the same directory before it publishes anything,
/// so making it here changes nothing the clone would leave.
///
/// # Errors
///
/// A directory that cannot be made,
/// which fills nothing and says why.
fn make_the_destinations_own_directory(entry: &PrewarmEntry) -> Result<(), String> {
    let Some(parent) = entry.to.parent() else { return Ok(()) };
    return match std::fs::create_dir_all(parent) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} could not be made, so nothing of this workspace was filled ({error})", parent.display())),
    };
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
    //  own, and a lock that no longer matches its manifest would be rewritten where it lies. The clean is told to leave the
    //  lock alone, so nothing here writes the worktree's files: a lock it would have to change stops the clean, and the
    //  worktree keeps every byte it had.
    let run: ToolRun = clean_the_copy(fill, entry, names).await;
    if !matches!(run.end, ToolEnd::Succeeded) {
        let _removed = remove_if_any(names.staging.as_path());
        let reason: String = end_reason(&run.end);
        return Err(format!(
            "the copy of {} could not be cleaned ({reason}), so nothing was published and everything staged for it was removed. The worktree \
             is exactly as it was: the clean is asked not to change {LOCK_FILE} (`{LOCK_FLAG}`), and nothing here writes it. {}",
            entry.from.display(),
            what_to_do_next(entry, reason.as_str()),
        ));
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
