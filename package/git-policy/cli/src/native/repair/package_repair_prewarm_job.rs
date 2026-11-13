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
//! and a marker this job wrote for that very build directory is what proves a directory at one of
//! them is its leftover:
//! anything at those names without that proof is somebody's work and is never removed,
//! so a workspace whose usual names are taken is filled under names of this run's own.
//! A marker a copy carried from another worktree names another build directory,
//! so no add believes it and the repair it would have skipped runs again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await prewarmAwaiting(item); // for each entry: clone to the staging name, clean it, publish it unless the destination exists
//! ```

use crate::diagnostic_log::debug;
use crate::package_repair_cargo::{locked_clean_arguments, manual_command};
/// The lock file the filling's report names,
/// and the faces that read it and put the checked-out bytes back over a resolution's.
use crate::package_repair_cargo_lock::{LOCK_FILE, Placed, place_the_resolution, read_lock, restore_only_the_cleans_own};
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolLimits, ToolRun, find_program, run_tool_awaiting};
use crate::package_repair_prewarm::{BUILD_DIRECTORY_VARIABLE, PrewarmEntry, STAGING_SUFFIX, TARGET_DIRECTORY_VARIABLE, decode};
use crate::package_repair_prewarm_claim::{TheClaim, TheLease, claim_the_names};
use crate::package_repair_request::RepairEnvironment;
// The marker's own vocabulary, reading and writing live beside this job
// (`package_repair_prewarm_marker.rs`), and this module names what it uses so its controls reach them too.
pub(crate) use crate::package_repair_prewarm_marker::{
    TheMarkerThere, TheMarkersAnswer, marker_claims, marker_of, take_the_marker_back, the_marker_there, write_marker,
};
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
/// type Fill = { cargo: string; limits: ToolLimits; keptIn?: string; worktree: string };
/// ```
pub(crate) struct Fill {
    /// The Cargo that cleans the staging copy.
    pub(crate) cargo: PathBuf,
    /// The bounds it runs under.
    pub(crate) limits: ToolLimits,
    /// The Git directory of the worktree whose lock the clean may displace,
    /// or nothing when no Git directory is known,
    /// in which case a lock the clean changed is left as it is found.
    pub(crate) kept_in: Option<PathBuf>,
    /// The worktree every workspace of one record sits in,
    /// which is the root a resolution's mirror of a workspace is laid out below.
    pub(crate) worktree: PathBuf,
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
/// type TheNames = { staging: string; target: string; marker?: string; lease?: TheLease };
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
    /// The lease that proves this filling is the one at work on these names,
    /// when this run took the usual ones.
    lease: Option<TheLease>,
}

/// What:
///  Take the names one workspace is filled under:
/// the usual ones when nothing is there or when this job's own marker claims what is,
/// and names of this run's own when another file or directory holds the usual ones.
/// Why:
///   The usual name is where a killed filling's leftovers sit,
/// and only a marker this job wrote for this very build directory proves they are its own,
/// so nothing at those names is removed without that proof and the filling still happens.
/// A marker a copy carried from another worktree names a build directory that is not this one,
/// so it is taken back and the usual names are claimed for this fill.
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
    // The lease is taken before the names are read, and it is what tells a filling that is still at work from one that was
    //  killed: a marker says a filling wrote these names, and only the kernel can say whether it is still writing them.
    let lease: TheLease = match claim_the_names(usual.as_path()) {
        TheClaim::Held(lease) => lease,
        // A filling of this very build directory holds the names and has not finished: this run touches none of them and
        //  stages under names of its own, so it can never delete a live filling's copy.
        TheClaim::Taken => return Ok(names_of_this_runs_own(usual.as_path())),
        // Nothing proves who holds the names, so nothing here may be removed for a dead filling.
        TheClaim::Refused(reason) => {
            debug(TAG, format!("{reason}; this run stages under names of its own rather than remove what it cannot prove is a dead filling's").as_str());
            return Ok(names_of_this_runs_own(usual.as_path()));
        }
    };
    let standing: TheMarkerThere = the_marker_there(marker.as_path());
    if matches!(&standing, TheMarkerThere::ThisJobs(claim) if claim.to == destination) {
        // An earlier filling of this same workspace was interrupted, so what it left under these names is this job's.
        remove_if_any(usual.as_path())?;
        remove_if_any(usual_target.as_path())?;
        return Ok(TheNames { staging: usual, target: usual_target, marker: Some(marker), lease: Some(lease) });
    }
    if matches!(&standing, TheMarkerThere::ThisJobs(claim) if claim.to != destination) {
        // A copy carried another worktree's claim here, and it says nothing about this one's names, so it goes and this
        //  fill claims them.
        take_the_marker_back(marker.as_path())?;
    }
    if usual.exists() || usual_target.exists() || marker.exists() {
        // Somebody else's file or directory holds one of the names, so this run stages under names of its own.
        return Ok(names_of_this_runs_own(usual.as_path()));
    }
    return match write_marker(marker.as_path(), destination)? {
        TheMarkersAnswer::WrittenByThisRun => Ok(TheNames { staging: usual, target: usual_target, marker: Some(marker), lease: Some(lease) }),
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
    return TheNames { staging: own, target: own_target, marker: None, lease: None };
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
    if let Some(marker) = names.marker.as_ref() {
        match std::fs::remove_file(marker.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => debug(TAG, format!("{} could not be removed after the filling ({error})", marker.display()).as_str()),
        }
    }
    // The lease goes last, while this filling still holds it: a filling that takes the names after this point makes its own
    //  lease file, so the name it finds gone is nobody's. A lease file that cannot be removed is a debug line only: the next
    //  filling creates the name again and locks it.
    if let Some(lease) = names.lease.as_ref()
        && let Err(error) = std::fs::remove_file(lease.path())
        && error.kind() != std::io::ErrorKind::NotFound
    {
        debug(TAG, format!("{} could not be removed after the filling ({error})", lease.path().display()).as_str());
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
/// Why:
///   The switch itself is one place,
/// in `package_repair_cargo::locked_clean_arguments`,
/// and the words a person reads are another,
/// so the report spells the same flag these words name.
const LOCK_FLAG: &str = "--locked";

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
/// and the clean itself is told to leave `Cargo.lock` alone,
/// which is what keeps the clean out of the worktree's files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cleanTheCopy({ fill, entry, names }): ToolRun
/// ```
async fn clean_the_copy(fill: &Fill, entry: &PrewarmEntry, names: &TheNames) -> ToolRun {
    let command: ToolCommand = ToolCommand {
        arguments: locked_clean_arguments(entry.members.as_slice()),
        directory: entry.root.clone(),
        set: vec![
            (OsString::from(BUILD_DIRECTORY_VARIABLE), names.staging.as_os_str().to_os_string()),
            (OsString::from(TARGET_DIRECTORY_VARIABLE), names.target.as_os_str().to_os_string()),
        ],
    };
    return run_tool_awaiting(fill.cargo.as_path(), &command, fill.limits).await;
}

/// The manifest a workspace is read from,
/// which the mirror of a workspace copies rather than links.
const MANIFEST_FILE: &str = "Cargo.toml";

/// The directory under the Git directory that holds one resolution's mirror of one workspace.
const RESOLUTIONS_DIRECTORY: &str = "monochromatic-git-policy-cli-lock-resolutions";

/// What:
///  The bytes Cargo resolves for a workspace whose `Cargo.lock` a `--locked` clean refused,
///  made in a mirror of that workspace instead of in the worktree.
/// Why:
///   A `--locked` clean refuses a lock that no longer matches its manifest,
/// and the resolution that makes it match is a write to `Cargo.lock`,
/// which is the person's file:
/// a write of theirs made at any moment before Cargo's own would be truncated by it,
/// and no step taken from outside the writer can bring those bytes back.
///  So the resolution is made where the person's file is not:
/// a mirror of the workspace below the worktree root,
/// whose directories on the way are this job's and whose every other entry is a link to the real one,
/// with the workspace's manifest and lock copied into it.
/// Cargo resolves the same manifests with the same lock bytes and writes `Cargo.lock` in the mirror,
/// whether or not there is any to read,
/// and the bytes placed in the worktree afterwards are the ones the workspace itself would have
/// resolved (`place_the_resolution`).
/// A mirror that cannot be made,
/// a resolution that fails,
/// and a resolution that writes no lock all answer an error,
/// and the caller gives that workspace up rather than touch the person's file.
///
/// # Errors
///
/// No Git directory to keep a mirror under,
/// a workspace outside the worktree,
/// a mirror that cannot be made or read,
/// and a Cargo that failed or wrote no lock.
async fn resolve_the_workspace_elsewhere(fill: &Fill, entry: &PrewarmEntry) -> Result<Vec<u8>, String> {
    let Some(kept_in) = fill.kept_in.as_ref() else {
        return Err(format!(
            "the {LOCK_FILE} of {} would have to be resolved, and no Git directory was named to resolve it outside the worktree",
            entry.root.display()
        ));
    };
    let root: PathBuf = the_resolutions_mirror_root(kept_in, entry.root.as_path());
    // A mirror a killed resolution left is this job's own, and it is rebuilt rather than trusted.
    let _cleared = std::fs::remove_dir_all(root.as_path());
    let mirrored: PathBuf = match mirror_the_workspace(fill.worktree.as_path(), entry.root.as_path(), root.as_path()) {
        Ok(mirrored) => mirrored,
        Err(reason) => {
            let _abandoned = std::fs::remove_dir_all(root.as_path());
            return Err(reason);
        }
    };
    let arguments: Vec<OsString> = ["metadata", "--offline", "--format-version", "1"].iter().map(OsString::from).collect();
    let command: ToolCommand = ToolCommand { arguments, directory: mirrored.clone(), set: Vec::new() };
    let run: ToolRun = run_tool_awaiting(fill.cargo.as_path(), &command, fill.limits).await;
    let resolved: Option<Vec<u8>> = read_lock(mirrored.as_path());
    let _mirror_cleared = std::fs::remove_dir_all(root.as_path());
    return match (run.end, resolved) {
        (ToolEnd::Succeeded, Some(bytes)) => Ok(bytes),
        (ToolEnd::Succeeded, None) => Err(format!("Cargo resolved the workspace of {} and wrote no {LOCK_FILE} in the mirror it was given", entry.root.display())),
        (end, _) => Err(format!("the workspace of {} could not be resolved outside the worktree ({})", entry.root.display(), end_reason(&end))),
    };
}

/// The most bytes of one workspace path's hexadecimal form that go into one directory name.
/// Why:
///   A file system refuses a name longer than 255 bytes,
/// and a workspace path of ordinary depth is longer than that once it is written in hexadecimal,
///  so the text is split into names that each fit and that read back as one path when they are joined.
const NAME_BYTES: usize = 200;

/// What:
///  The directory under the Git directory that holds one workspace's resolution mirror.
/// Why:
///   The workspace's own path decides it,
/// so two workspaces of one worktree never write over each other,
/// and the path survives as bytes whatever the text encoding of a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = chunk(hexOfPath(workspace), 200).reduce((path, piece) => join(path, piece), join(keptIn, RESOLUTIONS_DIRECTORY));
/// ```
fn the_resolutions_mirror_root(kept_in: &Path, workspace: &Path) -> PathBuf {
    let hex: String = crate::path_bytes_in_records::hex_of_path(workspace);
    let mut directory: PathBuf = kept_in.join(RESOLUTIONS_DIRECTORY);
    for piece in hex.as_bytes().chunks(NAME_BYTES) {
        directory = directory.join(String::from_utf8_lossy(piece).into_owned());
    }
    return directory;
}

/// What:
///  Lay out a mirror of one workspace below a root of this job's own,
///  and answer the mirror's own directory.
/// Why:
///   A path dependency is written relative to the manifest that names it,
/// and workspaces of this repository climb out of their own root,
/// so the mirror keeps the same place below the worktree root:
/// every directory on the way is this job's and every other entry is a link to the real one.
/// The workspace's own manifest and lock are copies,
/// which is what makes Cargo write its resolution into the mirror.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mirrored = mirrorTheWorkspace(worktree, workspace, root); // links for every sibling, copies at the leaf
/// ```
fn mirror_the_workspace(worktree: &Path, workspace: &Path, root: &Path) -> Result<PathBuf, String> {
    let Some(relative) = workspace.strip_prefix(worktree).ok() else {
        return Err(format!("{} is not inside the worktree {}", workspace.display(), worktree.display()));
    };
    let steps: Vec<std::ffi::OsString> = relative.iter().map(std::ffi::OsStr::to_os_string).collect();
    let mut real: PathBuf = worktree.to_path_buf();
    let mut mirrored: PathBuf = root.to_path_buf();
    for depth in 0..=steps.len() {
        let leaf: bool = depth == steps.len();
        if let Err(error) = std::fs::create_dir_all(mirrored.as_path()) {
            return Err(format!("{} could not be made ({error})", mirrored.display()));
        }
        let listing = match std::fs::read_dir(real.as_path()) {
            Ok(listing) => listing,
            Err(error) => return Err(format!("{} could not be read ({error})", real.display())),
        };
        for entry in listing.flatten() {
            let name: std::ffi::OsString = entry.file_name();
            // The entry the mirror walks into is the directory this step made, and the leaf's manifest and lock are copies.
            if !leaf && Some(name.as_os_str()) == Some(steps[depth].as_os_str()) {
                continue;
            }
            if name == std::ffi::OsStr::new(".git") {
                continue;
            }
            if leaf && (name == std::ffi::OsStr::new(MANIFEST_FILE) || name == std::ffi::OsStr::new(LOCK_FILE)) {
                continue;
            }
            let named: PathBuf = mirrored.join(name.as_os_str());
            if let Err(error) = std::os::unix::fs::symlink(entry.path(), named.as_path()) {
                return Err(format!("{} could not be linked into the mirror ({error})", named.display()));
            }
        }
        if leaf {
            for name in [MANIFEST_FILE, LOCK_FILE] {
                let from: PathBuf = real.join(name);
                if !from.is_file() {
                    continue;
                }
                let to: PathBuf = mirrored.join(name);
                if let Err(error) = std::fs::copy(from.as_path(), to.as_path()) {
                    return Err(format!("{} could not be copied into the mirror ({error})", to.display()));
                }
            }
            break;
        }
        real = real.join(steps[depth].as_os_str());
        mirrored = mirrored.join(steps[depth].as_os_str());
    }
    return Ok(mirrored);
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
    // A destination that exists is a build that got there first: the worktree keeps everything it made. A filling of this
    //  workspace that was killed after it published left its marker and an empty staging copy beside that directory, and
    //  this job's marker is the proof those names are its own, so the next filling gives them back.
    if entry.to.exists() {
        give_back_what_a_killed_filling_left(entry);
        return Ok(Filled::Kept);
    }
    make_the_destinations_own_directory(entry)?;
    let names: TheNames = match entry.staged_in.as_ref() {
        // The add moved the worktree's own copied state aside before it answered, so there is nothing to clone: the state to
        //  clean is already there.
        Some(staging) => match staged_names(staging, entry.to.as_path())? {
            Some(names) => names,
            None => return Ok(Filled::Kept),
        },
        None => names_for(entry.to.as_path())?,
    };
    let filled: Result<Filled, String> = fill_the_staging(fill, entry, &names).await;
    give_back(&names);
    return filled;
}

/// What:
///  The names a filling of the worktree's own staged state works in,
/// or nothing when the add staged none.
/// Why:
///   The add moves the copied build state to the staging name in one rename before the add answers,
/// so a build that starts at once finds no build directory and compiles everything itself.
/// This run cleans that state out of the way it always does,
/// and publishes it back only where the worktree still has no build directory of its own.
/// The marker the add wrote is the proof the directory is the wrapper's own,
/// so anything without it is somebody's work and is never removed.
///
/// # Errors
///
/// A staging name this job's own marker for this very build directory does not claim.
fn staged_names(staging: &Path, destination: &Path) -> Result<Option<TheNames>, String> {
    if !staging.exists() {
        return Ok(None);
    }
    // The add staged this state and no filling of this worktree is at work on it yet; a second record naming the same
    //  staging is the case where one is, and then this run leaves it to that filling.
    let lease: Option<TheLease> = match claim_the_names(staging) {
        TheClaim::Held(lease) => Some(lease),
        TheClaim::Taken => return Ok(None),
        TheClaim::Refused(reason) => {
            debug(TAG, format!("{reason}; this run leaves the staged state to whichever filling holds it").as_str());
            return Ok(None);
        }
    };
    let marker: PathBuf = marker_of(staging);
    if !marker_claims(marker.as_path(), destination) {
        return Err(format!("{} holds what no filling of this job staged, so it is left exactly as it is found", staging.display()));
    }
    let target: PathBuf = target_of(staging);
    return Ok(Some(TheNames { staging: staging.to_path_buf(), target, marker: Some(marker), lease }));
}

/// What:
///  Move the worktree's own copied build state into the filling's staging name,
/// under this job's marker,
/// in one rename;
/// or say that there was nothing to move.
/// Why:
///   Cargo treats the copied artifacts of the worktree's own crates as fresh,
/// so a build that started before a clean would reuse them at the wrong paths.
/// The add cannot wait for the clean (`REPAIR_TOOL_CEILING` is eighteen minutes and the quiet window is six hundred
///  seconds), and it must not leave that state where a build can trust it,
/// so it moves the state aside in one rename before it answers:
/// a build that starts at once finds no build directory and compiles everything itself,
/// which is correct and only slower,
/// and this job cleans the state and publishes it back where nothing has rebuilt yet.
/// The marker is written first,
/// so a kill between the two leaves a name this job proves is its own.
/// A marker already standing at the name is believed only when this add can place it and the state
/// it claims is already out of the worktree:
/// one written for another build directory came here inside a copy and claims nothing here,
/// and one written for this build directory whose staging name is empty is a marker whose rename
/// never happened,
/// so the repair runs again instead of being skipped.
///
/// # Errors
///
/// A staging name somebody else's file or directory holds,
/// and a state that cannot be moved.
pub(crate) fn stage_the_worktrees_own_state(destination: &Path, git_dir: Option<&Path>) -> Result<Staged, String> {
    // The staging name sits beside the build directory it takes, so the rename stays inside the worktree's own subvolume:
    // btrfs refuses to rename a directory out of a subvolume, and a worktree of this repository is one.
    let staging: PathBuf = staging_of(destination);
    let marker: PathBuf = marker_of(staging.as_path());
    // The lease is taken before anything here is removed or renamed, and it is what tells a filling that is still at work from
    //  one that was killed. Without it this add would take a live filling's marker for a dead one's.
    let _lease: TheLease = match claim_the_names(staging.as_path()) {
        TheClaim::Held(lease) => lease,
        TheClaim::Taken => {
            return Err(format!(
                "a filling of {} is still at work on the names beside it, so the copied build state is left where it is; run \
                 `cargo clean --offline --package <member>` in {} instead",
                destination.display(),
                destination.display()
            ));
        }
        TheClaim::Refused(reason) => {
            return Err(format!(
                "{reason}, so nothing proves who holds the names beside {}, and the copied build state is left where it is; run \
                 `cargo clean --offline --package <member>` in {} instead",
                destination.display(),
                destination.display()
            ));
        }
    };
    let standing: TheMarkerThere = the_marker_there(marker.as_path());
    if standing == TheMarkerThere::SomebodyElses {
        return Err(format!(
            "{} already holds something this wrapper did not stage, so the copied build state is left where it is; the worktree's first build would reuse it, so run `cargo clean --offline --package <member>` in {} instead",
            staging.display(),
            destination.display()
        ));
    }
    if let TheMarkerThere::ThisJobs(claim) = &standing {
        // This job's marker for this very build directory, with the staging copy it names standing beside it, is the one proof
        //  an add can read: the state is already out of the worktree, and the filling's clean is what completes the repair.
        if claim.to == destination && staging.exists() {
            return Ok(Staged::Staged(staging));
        }
        // A marker this add cannot place as finished: the add that wrote it was killed before the rename that moves the state
        //  aside, or before a filling published, and a copy may have carried it here from another worktree. The repair runs
        //  again rather than being skipped, and the marker goes so this add claims the name itself.
        debug(
            TAG,
            format!(
                "{} holds the marker of the add {} for {}, which never finished, so the repair runs again",
                marker.display(),
                claim.begun_by,
                claim.to.display()
            )
            .as_str(),
        );
        take_the_marker_back(marker.as_path())?;
        if staging.exists() {
            // A copy carried the state another worktree had staged. Where this worktree also has a build directory, that copy
            //  is the older copy of the same tree and goes, so this worktree's own state is what the filling cleans; where it
            //  has none, the carried state is the only copy and is cleaned and published itself.
            if !destination.exists() {
                claim_the_staging_names(marker.as_path(), destination, git_dir)?;
                return Ok(Staged::Staged(staging));
            }
            remove_if_any(staging.as_path())?;
        }
    }
    if !destination.exists() {
        return Ok(Staged::Nothing);
    }
    if staging.exists() || marker.exists() {
        // A name nothing here claimed stands beside the build directory: it may be somebody's directory or a file this job never
        //  wrote, and a rename would replace an empty directory of theirs without a word.
        return Err(format!(
            "{} already holds something this wrapper did not stage, so the copied build state is left where it is; the worktree's first build would reuse it, so run `cargo clean --offline --package <member>` in {} instead",
            staging.display(),
            destination.display()
        ));
    }
    // The staging name and the target directory its clean uses stand inside the worktree, so the repository's own exclude file
    // names them and no `git status` or `git add -A` of the person's sees them.
    claim_the_staging_names(marker.as_path(), destination, git_dir)?;
    return match std::fs::rename(destination, staging.as_path()) {
        Ok(()) => Ok(Staged::Staged(staging)),
        Err(error) => {
            // Nothing moved, so the marker claims a name nothing is in; take it back and say the state stays where it is.
            let _taken_back = std::fs::remove_file(marker.as_path());
            Err(format!(
                "the copied build state at {} could not be moved aside ({error}), so it stays where it is; run `cargo clean --offline --package <member>` in that worktree instead",
                destination.display()
            ))
        }
    };
}

/// The patterns that keep this job's staging names out of the person's `git status` and `git add -A`.
///
/// They are written into the repository's own exclude file (`<git-dir>/info/exclude`), which applies to every worktree of it,
/// because the names stand beside the build directory they take, inside the worktree.
pub(crate) const EXCLUDED_STAGING_PATTERNS: [&str; 4] =
    ["*.monochromatic-prewarm/", "*.monochromatic-prewarm.target/", "*.monochromatic-prewarm.owner", "*.monochromatic-prewarm.claim"];

/// What:
///  Name this job's staging patterns in the repository's exclude file,
/// once.
/// Why:
///   The staging name sits inside the worktree beside the build directory it takes,
/// because btrfs refuses a rename of a directory out of a worktree that is a subvolume,
/// so without this the person's `git status` would show it and a `git add -A` would commit a copy of `target/`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const excludeTheStagingNames = (gitDir) => { const path = join(gitDir, 'info/exclude'); if (!read(path, '').includes(PATTERNS[0])) appendFileSync(path, PATTERNS.join('\n') + '\n'); };
/// ```
pub(crate) fn exclude_the_staging_names(git_dir: &Path) -> Result<(), String> {
    let info: PathBuf = git_dir.join("info");
    let exclude: PathBuf = info.join("exclude");
    let already: String = std::fs::read_to_string(exclude.as_path()).unwrap_or_default();
    if already.contains(EXCLUDED_STAGING_PATTERNS[0]) {
        return Ok(());
    }
    if let Err(error) = std::fs::create_dir_all(info.as_path()) {
        return Err(format!("{} could not be made ({error})", info.display()));
    }
    let named_before: usize = already.len();
    let mut text: String = already;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    for pattern in EXCLUDED_STAGING_PATTERNS {
        text.push_str(pattern);
        text.push('\n');
    }
    // The file is the person's own and Git made it when the repository was, so this appends to it and never rewrites it:
    // a kill in the middle leaves their lines whole and at most adds this job's patterns twice.
    let opened = std::fs::OpenOptions::new().create(true).append(true).open(exclude.as_path());
    let mut file = match opened {
        Ok(file) => file,
        Err(error) => return Err(format!("{} could not be opened ({error})", exclude.display())),
    };
    return match std::io::Write::write_all(&mut file, &text.as_bytes()[named_before..]) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} could not be appended to ({error})", exclude.display())),
    };
}

/// What:
///  Claim the staging name of one build directory for this add:
/// name the repository's exclude patterns for the staging names,
/// then put this add's marker beside the staging name in one exclusive write.
/// Why:
///   The staging name sits inside the worktree beside the build directory it takes,
/// because btrfs refuses a rename of a directory out of a worktree that is a subvolume,
/// so without the exclude file the person's `git status` would show it and a `git add -A` would
///  stage a copy of the build directory.
/// The marker is what proves the names are this job's own,
/// and this one names the build directory and the add that claimed them,
/// so a copy that carries it into another worktree is not read as a claim on that worktree's names.
/// A name another writer takes between this add's look and its write is theirs,
/// so nothing is written there and the state stays where it is for the add's own clean.
///
/// # Errors
///
/// A Git directory the patterns cannot be named in,
/// and a marker another writer took.
fn claim_the_staging_names(marker: &Path, destination: &Path, git_dir: Option<&Path>) -> Result<(), String> {
    if let Some(directory) = git_dir
        && let Err(reason) = exclude_the_staging_names(directory)
    {
        return Err(format!("{reason}, so the copied build state stays where it is; run `cargo clean --offline --package <member>` in the worktree instead"));
    }
    return match write_marker(marker, destination)? {
        TheMarkersAnswer::WrittenByThisRun => Ok(()),
        TheMarkersAnswer::WrittenByAnother => Err(format!(
            "{} was taken by another writer, so the copied build state is left where it is; run `cargo clean --offline --package <member>` in {} instead",
            marker.display(),
            destination.display()
        )),
    };
}

/// What a staging step came to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Staged {
    /// The worktree's own copied state now sits at this staging path, under this job's marker.
    Staged(PathBuf),
    /// The worktree held no build state of its own to move.
    Nothing,
}

/// What:
///  Remove the staging copy, its target directory and the marker a filling of this workspace left behind
///  when it was killed after it published,
/// and nothing at all when this job's own marker for this very build directory is not there.
/// Why:
///   A publish is one rename that leaves no trace of the filling that made it,
/// so a kill between the publish and the giving back of the names strands an empty directory and its marker for ever:
/// every later filling of that workspace answers `Filled::Kept` at the destination and never looks at the names again.
/// The marker is what proves the names are this job's own,
/// and it proves that only for the build directory it names:
/// one a copy carried from another worktree names a build directory that is not this one,
/// so a filling here never gives back what it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const giveBackWhatAKilledFillingLeft = (destination) => { const staging = stagingOf(destination); const marker = markerOf(staging); if (!markerClaims(marker, destination)) return; removeIfAny(staging); removeIfAny(targetOf(staging)); removeFileIfPresent(marker); };
/// ```
fn give_back_what_a_killed_filling_left(entry: &PrewarmEntry) {
    let staging: PathBuf = match entry.staged_in.as_ref() {
        Some(staged) => staged.clone(),
        None => staging_of(entry.to.as_path()),
    };
    let marker: PathBuf = marker_of(staging.as_path());
    if !marker_claims(marker.as_path(), entry.to.as_path()) {
        return;
    }
    // The marker names this build directory, and only the lease can say the filling that wrote it is gone: a filling that is
    //  still cleaning or publishing this staging holds it, and then nothing here is removed.
    let lease: TheLease = match claim_the_names(staging.as_path()) {
        TheClaim::Held(lease) => lease,
        TheClaim::Taken => return,
        TheClaim::Refused(reason) => {
            debug(TAG, format!("{reason}; the names a killed filling left are left for the next filling").as_str());
            return;
        }
    };
    let names: TheNames = TheNames { staging: staging.clone(), target: target_of(staging.as_path()), marker: Some(marker), lease: Some(lease) };
    if let Err(reason) = remove_if_any(names.staging.as_path()) {
        // Nothing of the worktree depends on this directory; the next filling of the same workspace tries again.
        debug(TAG, format!("{reason}; a staging copy a killed filling left stays for the next one").as_str());
    }
    give_back(&names);
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
    if entry.staged_in.is_none() {
        let Some(from) = entry.from.as_ref() else {
            return Err(String::from("a filling that clones was handed no source build directory, so nothing was cleaned"));
        };
        let made: Result<Cloned, String> = clone_tree_awaiting(from.as_path(), names.staging.as_path(), 1).await;
        if let Err(reason) = made {
            let _removed = remove_if_any(names.staging.as_path());
            return Err(format!("the build state of {} could not be cloned ({reason})", from.display()));
        }
    }
    // A build that started during the clone owns the destination now, and this job touches nothing of it.
    if entry.to.exists() {
        let _removed = remove_if_any(names.staging.as_path());
        return Ok(Filled::Kept);
    }
    // Cargo resolves the worktree's workspace for this clean even though it deletes only from directories of this job's
    //  own, and that rewrites a `Cargo.lock` that no longer matches its manifest or creates one that is missing. This job
    //  runs after the add returned, when the person may be working in the worktree, so the clean itself must never write
    //  that lock (`--locked`): a lock it refuses is resolved by Cargo first, and the checked-out lock goes back only when
    //  the file still holds exactly the bytes that resolution wrote (`package_repair_cargo_lock.rs`).
    let lock_before: Option<Vec<u8>> = read_lock(entry.root.as_path());
    let mut run: ToolRun = clean_the_copy(fill, entry, names).await;
    if !matches!(run.end, ToolEnd::Succeeded) {
        // A `--locked` clean refuses a lock that no longer matches its manifest, and making it match means writing that lock.
        //  The resolution is made where the person's file is not, and its bytes are placed with a step that hands back whatever
        //  stood there, so a write of theirs made at any moment before the placement is in this run's hands and not under it.
        let resolution: Vec<u8> = match resolve_the_workspace_elsewhere(fill, entry).await {
            Ok(bytes) => bytes,
            Err(reason) => {
                let _removed = remove_if_any(names.staging.as_path());
                return Err(format!(
                    "{reason}, so nothing of this workspace was cleaned and the copy of it was removed. {} A write of yours to that file is \
                     never replaced: one the repair meets while it resolves is handed back and the repair gives way.",
                    what_to_do_next(entry, LOCK_FILE),
                ));
            }
        };
        match place_the_resolution(entry.root.as_path(), &resolution, lock_before.as_deref(), fill.kept_in.as_deref()) {
            Placed::Placed => {
                run = clean_the_copy(fill, entry, names).await;
                match restore_only_the_cleans_own(entry.root.as_path(), lock_before.as_deref(), fill.kept_in.as_deref(), resolution.as_slice()) {
                    None => {}
                    Some(said) => debug(TAG, format!("{}: {said}", entry.root.display()).as_str()),
                }
            }
            Placed::GaveWay(reason) => {
                let _removed = remove_if_any(names.staging.as_path());
                return Err(format!(
                    "{reason}, so nothing of this workspace was cleaned and the copy of it was removed. {} The repair gave way rather than \
                     replace a write of yours: the file is exactly as you left it.",
                    what_to_do_next(entry, LOCK_FILE),
                ));
            }
        }
    }
    if !matches!(run.end, ToolEnd::Succeeded) {
        let _removed = remove_if_any(names.staging.as_path());
        let reason: String = end_reason(&run.end);
        return Err(format!(
            "the copy of {} could not be cleaned ({reason}), so nothing was published and everything staged for it was removed. The clean itself \
             never writes the worktree's {LOCK_FILE} (`{LOCK_FLAG}`): a lock it refuses is resolved in a mirror of the workspace, and the resolution's \
             bytes are placed with the step that hands back whatever stood there, so a write of yours is never replaced. {}",
            entry.from.as_deref().unwrap_or(names.staging.as_path()).display(),
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
    let fill: Fill = Fill { cargo, limits: environment.limits(), kept_in: Some(item.git_dir.clone()), worktree: item.repository.clone() };
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
