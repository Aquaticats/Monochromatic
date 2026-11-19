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
//! and what proves a directory at one of them is its leftover is a making and the record of that making:
//! each is created by a call that fails when the name is taken,
//! the target directory by this job's own exclusive create,
//! and the marker records the target directory it made,
//! so anything at those names without that proof is somebody's work and is never removed,
//! and a workspace whose usual names are taken is filled under names of this run's own.
//! A marker a copy carried from another worktree names another build directory,
//! so no add believes it and the repair it would have skipped runs again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await prewarmAwaiting(item); // for each entry: clone to the staging name, clean it, publish it unless the destination exists
//! ```

use crate::diagnostic_log::debug;
use crate::package_repair_cargo::{CargoTool, locked_clean_arguments, manual_command};
use crate::package_repair_cargo_clean_in_mirror::{MirrorClean, clean_in_a_mirror};
/// The lock file the filling's report names,
/// and the faces that read it and put the checked-out bytes back over a resolution's.
use crate::package_repair_cargo_lock::LOCK_FILE;
use crate::package_repair_command::{ToolCommand, ToolEnd, ToolLimits, ToolRun, find_program, run_tool_awaiting};
use crate::package_repair_prewarm::{BUILD_DIRECTORY_VARIABLE, PrewarmEntry, STAGING_SUFFIX, TARGET_DIRECTORY_VARIABLE, decode};
use crate::package_repair_prewarm_claim::{TheClaim, TheLease, claim_the_names, claim_the_names_at, lease_of};
use crate::package_repair_request::RepairEnvironment;
// The marker's own vocabulary, reading and writing live beside this job
// (`package_repair_prewarm_marker.rs`), and this module names what it uses so its controls reach them too.
pub(crate) use crate::package_repair_prewarm_marker::{
    TheEntry, TheMarker, TheMarkerThere, TheMarkersAnswer, TheRemoval, WhatTheMarkerRecords, a_claim_inside, a_name_for_this_run, marker_of,
    take_the_marker_back, the_claim_a_directory_carries, the_claim_text, the_marker_there, write_marker,
};
/// The controls read a marker through the same question the add asks of it.
#[cfg(test)]
pub(crate) use crate::package_repair_prewarm_marker::marker_claims;
use crate::package_repair_tree_clone::{Cloned, TheDestination, clone_tree_awaiting};
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
    /// This filling's own name,
    /// so the mirror one resolution is made in belongs to one filling.
    ///
    /// Two fillings of one workspace run at once when a worktree is removed and added again at the same
    /// path while its first record is still queued,
    /// and the lease keeps them out of each other's staging names;
    /// a mirror named after the workspace alone lets one delete the other's while its Cargo runs,
    /// which fails a healthy workspace's repair and tells the person to update a lock that is fine.
    pub(crate) filling: String,
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
///  Whether any entry at all stands at this name:
/// a directory,
/// a file,
/// a link,
/// or a link whose target is not there.
/// Why:
///   `Path::exists` follows a link and answers `false` for one that points at nothing,
/// and a name a link holds is not a free name:
/// this job replaces nothing and removes nothing it did not make,
/// so the question a guard asks is whether the name is taken,
/// never whether some path leads somewhere from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theNameIsTaken = (path) => { try { lstatSync(path); return true; } catch { return false; } };
/// ```
fn the_name_is_taken(path: &Path) -> bool {
    return std::fs::symlink_metadata(path).is_ok();
}

/// What:
///  A boundary between two steps of one filling:
/// a place where a kill can fall and where another command can act.
/// Why:
///   The control that puts another command's entry at every such boundary takes its rows from this list,
/// so a step added to the filling later gets its rows without anyone remembering to write them.
///   Each step is reached by `a_step`, which does nothing in a release build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AStep = 'before the names are taken' | 'after the lease is taken' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AStep {
    /// Before the filling looks at the names.
    BeforeTheNamesAreTaken,
    /// After the lease on the names is taken.
    AfterTheLeaseIsTaken,
    /// After the marker's claim has been read.
    AfterTheMarkerIsRead,
    /// After this run made the clean's target directory.
    AfterTheTargetIsMade,
    /// After this run wrote the marker.
    AfterTheMarkerIsWritten,
    /// Before the clone fills the staging name.
    BeforeTheClone,
    /// After the clone came to its answer.
    AfterTheClone,
    /// Before the clean runs.
    BeforeTheClean,
    /// Before the clean's child is given the two paths it deletes from,
    /// which is the act the paths belong to.
    BeforeTheChildIsGivenItsPaths,
    /// After the clean came to its answer.
    AfterTheClean,
    /// Before the mirror's clean runs,
    /// which is the second child this filling starts with the same two names.
    BeforeTheMirrorClean,
    /// Before the claimed directory the act will remove is taken aside.
    BeforeTheClaimedDirectoryIsTakenAside,
    /// Before the directory this run took aside is removed.
    BeforeTheHeldDirectoryIsRemoved,
    /// Before the marker the act will take back is taken aside.
    BeforeTheMarkerIsTakenAside,
    /// Before the file this run took aside at the marker's name is unlinked.
    BeforeTheHeldMarkerIsRemoved,
    /// Before the claim file a failed move aside wrote is taken aside.
    BeforeTheClaimInsideIsTakenAside,
    /// Before the marker a failed move aside wrote is taken back by name.
    BeforeTheMarkerOfAFailedMoveIsTakenBack,
    /// Before the staging copy is taken aside for the publish,
    /// which is the act that moves whatever stands at the name.
    BeforeThePublishTakesTheCopyAside,
    /// Before the staging copy is published.
    BeforeThePublish,
    /// After the publish came to its answer.
    AfterThePublish,
    /// Before the names are given back.
    BeforeTheNamesAreGivenBack,
    /// After the target directory was given back.
    AfterTheTargetIsGivenBack,
    /// After the marker was given back.
    AfterTheMarkerIsGivenBack,
}

impl AStep {
    /// Every step of a filling a successful one reaches, in the order the filling reaches them.
    /// The mirror's clean and the failed move aside's removals are reached only on a path a control
    /// takes on purpose, and their own controls reach them.
    #[cfg(test)]
    pub(crate) const EVERY: [AStep; 18] = [
        AStep::BeforeTheNamesAreTaken,
        AStep::AfterTheLeaseIsTaken,
        AStep::AfterTheMarkerIsRead,
        AStep::AfterTheTargetIsMade,
        AStep::AfterTheMarkerIsWritten,
        AStep::BeforeTheClone,
        AStep::AfterTheClone,
        AStep::BeforeTheClean,
        AStep::BeforeTheChildIsGivenItsPaths,
        AStep::AfterTheClean,
        AStep::BeforeTheClaimedDirectoryIsTakenAside,
        AStep::BeforeTheMarkerIsTakenAside,
        AStep::BeforeThePublishTakesTheCopyAside,
        AStep::BeforeThePublish,
        AStep::AfterThePublish,
        AStep::BeforeTheNamesAreGivenBack,
        AStep::AfterTheTargetIsGivenBack,
        AStep::AfterTheMarkerIsGivenBack,
    ];
}

#[cfg(test)]
thread_local! {
    /// What a control does at one boundary: `true` stops the filling there, as a kill would stop it.
    /// Why:
    ///   The control's rows are the boundaries of the filling,
    /// and a row stops the filling at its own boundary so the leftovers a kill leaves are the state it plants into.
    pub(crate) static THE_STEP_HOOK: std::cell::RefCell<Option<Box<dyn Fn(AStep) -> bool>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Reach one boundary of a filling,
///  and say whether the filling stops there,
///  which is what a control asks for so the state after a kill can be inspected.
/// Why:
///   Everything a filling does crosses one of these boundaries,
/// so a control that plants another command's entry at every one of them covers the seams the code has,
/// not the seams someone remembered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aStep = (step) => theStepHook?.(step) ?? false;
/// ```
fn a_step(step: AStep) -> bool {
    #[cfg(test)]
    {
        return THE_STEP_HOOK.with(|hook| {
            let borrowed = hook.borrow();
            let Some(what_to_do) = borrowed.as_ref() else { return false };
            return what_to_do(step);
        });
    }
    #[cfg(not(test))]
    {
        let _ = step;
        return false;
    }
}

/// What:
///  The same boundary, for the marker module, whose removals are steps of this filling.
/// Why:
///   A control plants the entry at the act, and the act of taking a marker back lives in the marker file;
/// this is the one hook both files read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aStepFromTheMarker = (step) => aStep(step);
/// ```
#[cfg(test)]
pub(crate) fn a_step_from_the_marker(step: AStep) -> bool {
    return a_step(step);
}

/// What:
///  What a filling answers when a control stops it at one of its boundaries.
/// Why:
///   Every boundary is reached by `a_step`,
/// and a control that stops the filling there is asking for the state a kill leaves,
/// so the answer names what happened rather than pretending to be a tool's reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopWords = () => 'the filling stopped at a boundary for a control';
/// ```
fn stop_words() -> String {
    return String::from("the filling stopped at a boundary for a control");
}

/// What:
///  Make one directory this job will later remove,
///  and write the claim inside it that says which run made it and for which build directory.
/// Why:
///   A name is not proof of having made what stands under it:
/// another command can replace an entry under any name while this job works on the name,
/// so ownership is carried by the entry itself,
/// written by the same call that made it (an exclusive create that refuses a name another command holds).
///
/// # Errors
///
/// A name another command holds, a directory that cannot be made, and a claim that cannot be written.
fn make_the_claimed_directory(directory: &TheDirectory, named: &Path, to: &Path, begun_by: &str) -> Result<(), String> {
    let at: PathBuf = directory.along(named);
    return match std::fs::create_dir(at.as_path()) {
        Ok(()) => write_the_claim_inside(at.as_path(), to, begun_by),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(format!(
            "{} holds something this run did not make, so nothing of {} was cleaned",
            named.display(),
            to.display()
        )),
        Err(error) => Err(format!("{} could not be made ({error}), so nothing of {} was cleaned", named.display(), to.display())),
    };
}

/// What:
///  Write the claim that says this directory was made by this run for this build directory.
/// Why:
///   The claim is what a later run (or this one) reads to know the directory is this job's,
/// and it is written by the call that made the directory,
/// so a directory that carries one was made by a filling of this wrapper and nothing else.
///
/// # Errors
///
/// A claim that cannot be written, and a build directory whose path no claim can name.
fn write_the_claim_inside(directory: &Path, to: &Path, begun_by: &str) -> Result<(), String> {
    let text: Vec<u8> = the_claim_text(to, begun_by)?;
    let claim: PathBuf = directory.join(a_claim_inside());
    return match crate::private_storage::write_private_file_unsynced(claim.as_path(), text.as_slice()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} could not be written ({error}), so nothing of {} was cleaned", claim.display(), to.display())),
    };
}

/// What:
///  Make the clean's target directory for this run:
/// the usual name when nothing holds it,
/// and a name of this run's own when something does,
/// with this run's claim written inside either.
/// Why:
///   The clean deletes from this directory and the filling removes it,
/// so it is this run's own in both cases,
/// and a directory somebody else's at the usual name is neither used nor removed.
///
/// # Errors
///
/// Both names held, and a directory or claim that cannot be made.
fn the_target_this_run_makes(directory: &TheDirectory, staging: &Path, to: &Path, begun_by: &str) -> Result<PathBuf, String> {
    let usual: PathBuf = target_of(staging);
    if make_the_claimed_directory(directory, usual.as_path(), to, begun_by).is_ok() {
        return Ok(usual);
    }
    let own: PathBuf = target_of(a_name_of_this_runs_own(staging).as_path());
    make_the_claimed_directory(directory, own.as_path(), to, begun_by)?;
    return Ok(own);
}

/// What:
///  Whether the entry at this name still carries this run's claim for this build directory.
/// Why:
///   A removal and a write both reach the entry the name finds now:
/// the claim written when this run made it proves that entry is this run's,
/// so a clean and a publish that find a claim gone (another command replaced the entry) stop and say so,
/// and nothing of this run's is written into another command's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillThisRuns = (directory, named, to) => theClaimADirectoryCarries(join(directory.at, basename(named)))?.to === to;
/// ```
fn the_entry_is_still_this_runs(directory: &TheDirectory, named: &Path, to: &Path) -> bool {
    return matches!(the_claim_a_directory_carries(directory.along(named).as_path()), Some(claim) if claim.to == to);
}

/// What:
///  Remove one directory this job made,
///  and only when the entry found at the act carries this job's claim for this very build directory.
/// Why:
///   The claim travels with the entry,
/// so a directory another command made,
/// or one whose claim names another build directory,
/// is left where it stands and the filling says so:
/// a directory left behind costs disk, and a person's file removed is what the rules forbid.
///
/// # Errors
///
/// A claimed directory that cannot be removed.
fn remove_the_claimed_directory(directory: &TheDirectory, named: &Path, to: &Path) -> Result<TheRemoval, String> {
    let at: PathBuf = directory.along(named);
    let carries: bool = matches!(the_claim_a_directory_carries(at.as_path()), Some(claim) if claim.to == to);
    if !carries {
        debug(
            TAG,
            format!("{} holds no claim of this job for {}, so it is left as it is found", named.display(), to.display()).as_str(),
        );
        return Ok(TheRemoval::Left);
    }
    if a_step(AStep::BeforeTheClaimedDirectoryIsTakenAside) {
        return Err(stop_words());
    }
    remove_if_any(at.as_path())?;
    return Ok(TheRemoval::Removed);
}

/// What:
///  Remove the claim file this run wrote inside a directory,
///  and only while the entry that was written still stands at the name.
/// Why:
///   The claim is written by the call that makes the directory,
/// so it is this run's file,
/// and another file renamed over its name afterwards is not,
/// which the identity answered at the write decides.
///
/// # Errors
///
/// A claim that cannot be removed.
fn remove_the_claim_of(claim: &Path, written: Option<TheEntry>) -> Option<TheRemoval> {
    if let Some(was) = written
        && TheEntry::at(claim) != Some(was)
    {
        return Some(TheRemoval::Left);
    }
    if a_step(AStep::BeforeTheClaimInsideIsTakenAside) {
        return None;
    }
    return match std::fs::remove_file(claim) {
        Ok(()) => Some(TheRemoval::Removed),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(TheRemoval::Left),
        Err(error) => {
            debug(TAG, format!("{} could not be removed ({error})", claim.display()).as_str());
            None
        }
    };
}

/// What:
///  The directory this run took the names in,
/// held open,
/// and the path that reaches it through the descriptor.
/// Why:
///   A worktree can be removed and another added at the same path while a filling works on the names inside it,
/// and then a lease proves nothing:
/// the old filling holds a descriptor on a file no name reaches,
/// so the directory this run opened is what every removal is addressed through,
/// and a directory made again at the same path is a different directory.
///   The descriptor says nothing about which entry stands under a name inside it:
/// that is what the claims written inside the entries are for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theDirectory = (path) => { const held = openSync(path, O_DIRECTORY | O_NOFOLLOW); return { held, at: `/proc/self/fd/${held}` }; };
/// const entry = (directory, name) => join(directory.at, name);
/// ```
struct TheDirectory {
    /// The descriptor of the directory the names live in.
    /// It is what keeps the path beside it pointing at this directory and not at whatever stands there later.
    #[allow(dead_code, reason = "the descriptor is what the path beside it reaches: dropping it closes the path")]
    held: std::fs::File,
    /// The path that reaches that directory through the descriptor,
    /// `/proc/self/fd/<the descriptor>`.
    at: PathBuf,
}

impl TheDirectory {
    /// What:
    ///  Take the directory the names live in, without following a link,
    /// and answer the path that reaches it through the descriptor.
    /// Why:
    ///   The descriptor is what keeps the names this run took pointing at the directory it took them in,
    ///  whatever now stands at the path.
    ///
    /// # Errors
    ///
    /// A directory that cannot be opened.
    fn of(directory: &Path) -> Result<TheDirectory, String> {
        use std::os::unix::io::AsRawFd as _;
        let opened: std::io::Result<std::fs::File> = {
            use std::os::unix::fs::OpenOptionsExt as _;
            std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(directory)
        };
        return match opened {
            Ok(held) => Ok(TheDirectory { at: PathBuf::from(format!("/proc/self/fd/{}", held.as_raw_fd())), held }),
            Err(error) => Err(format!("{} could not be taken as the directory the staging names live in ({error})", directory.display())),
        };
    }

    /// What:
    ///  The path of this name as it stands in the directory taken here.
    /// Why:
    ///   Every removal, every claim and the move aside go through this path,
    /// so none of them can reach a directory that was made at the same path afterwards.
    fn along(&self, named: &Path) -> PathBuf {
        return match named.file_name() {
            Some(name) => self.at.join(name),
            None => self.at.clone(),
        };
    }

    /// What:
    ///  Remove one file through the descriptor of the directory it stands in,
    ///  where a name that is not there is nothing to do.
    /// Why:
    ///   A marker this add wrote is removed the same way a directory is.
    fn remove_the_file(&self, named: &Path) -> std::io::Result<()> {
        return match std::fs::remove_file(self.along(named)) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        };
    }
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
/// type TheNames = { staging: string; target: string; marker?: string; lease?: TheLease };
/// ```
struct TheNames {
    /// The directory the three names live in, taken and held when this run took them,
    /// which is what every removal and the move aside are addressed through.
    directory: TheDirectory,
    /// The build directory these names serve,
    /// which every claim written inside them names.
    to: PathBuf,
    /// The name this run has for itself,
    /// which every claim it writes carries.
    begun_by: String,
    /// The entry the marker was, when this run wrote it or read it,
    /// so its removal is bound to that very entry.
    marker_made: Option<TheEntry>,
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
    /// It is held for the whole filling and released when it ends:
    /// the descriptor is the lease, and nothing reads it (`package_repair_prewarm_claim.rs`).
    #[allow(dead_code, reason = "the descriptor is the lease: dropping it releases the lock at the end of the filling")]
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
fn names_for(destination: &Path, git_dir: Option<&Path>) -> Result<TheNames, String> {
    let begun_by: String = a_name_for_this_run();
    // The directory the names live in is taken first, before the first boundary: every removal, every claim and every
    //  create below is addressed through this descriptor, so a worktree made again at the same path is never reached.
    let Some(names_live_in) = destination.parent() else {
        return Err(format!("{} has no directory to stage beside", destination.display()));
    };
    let directory: TheDirectory = TheDirectory::of(names_live_in)?;
    if a_step(AStep::BeforeTheNamesAreTaken) {
        return Err(stop_words());
    }
    let usual: PathBuf = staging_of(destination);
    let usual_target: PathBuf = target_of(usual.as_path());
    let marker: PathBuf = marker_of(usual.as_path());
    // The exclude file names the lease before the lease file exists, so a person's `git status` never shows it.
    if let Some(git_directory) = git_dir
        && let Err(reason) = exclude_the_staging_names(git_directory)
    {
        return Err(format!("{reason}, so nothing of {} was filled", destination.display()));
    }
    // The lease is taken before the names are read, and it is what tells a filling that is still at work from one that was
    //  killed: a marker says a filling wrote these names, and only the kernel can say whether it is still writing them.
    let lease: TheLease = match claim_the_names_at(directory.along(lease_of(usual.as_path()).as_path()).as_path()) {
        TheClaim::Held(lease) => lease,
        // A filling of this very build directory holds the names and has not finished: this run touches none of them and
        //  stages under names of its own, so it can never delete a live filling's copy.
        TheClaim::Taken => return names_of_this_runs_own(usual.as_path(), directory, destination, begun_by.as_str()),
        // Nothing proves who holds the names, so nothing here may be removed for a dead filling.
        TheClaim::Refused(reason) => {
            debug(TAG, format!("{reason}; this run stages under names of its own rather than remove what it cannot prove is a dead filling's").as_str());
            return names_of_this_runs_own(usual.as_path(), directory, destination, begun_by.as_str());
        }
    };
    if a_step(AStep::AfterTheLeaseIsTaken) {
        return Err(stop_words());
    }
    let standing: TheMarkerThere = the_marker_there(marker.as_path());
    if let TheMarkerThere::ThisJobs(claim) = &standing {
        if claim.to == destination {
            // An earlier filling of this same workspace was interrupted, so the names its marker records are this job's
            //  and are given back before this run claims them afresh.
            give_back_the_names_a_marker_claims(usual.as_path(), marker.as_path(), claim, &directory)?;
        } else {
            // A copy carried another worktree's claim here, and it says nothing about this one's names, so it goes and this
            //  fill claims them.
            match take_the_marker_back(directory.along(marker.as_path()).as_path(), claim.read)? {
                TheRemoval::Removed | TheRemoval::Left => {}
            }
        }
    }
    if a_step(AStep::AfterTheMarkerIsRead) {
        return Err(stop_words());
    }
    // A killed filling can leave a directory it made before it wrote its marker,
    //  and the claim inside the entry is what proves it is this job's:
    // such a directory is given back rather than left to hold the usual names for ever.
    for leftover in [usual.as_path(), usual_target.as_path()] {
        let claimed: bool = matches!(the_claim_a_directory_carries(directory.along(leftover).as_path()), Some(claim) if claim.to == destination);
        if claimed {
            let _gave: TheRemoval = remove_the_claimed_directory(&directory, leftover, destination)?;
        }
    }
    if the_name_is_taken(usual.as_path()) || the_name_is_taken(usual_target.as_path()) || the_name_is_taken(marker.as_path()) {
        // Somebody else's file, directory or link holds one of the names, so this run stages under names of its own.
        return names_of_this_runs_own(usual.as_path(), directory, destination, begun_by.as_str());
    }
    // The target directory is made before the marker that records it: a marker is the proof a later run removes a directory
    //  by, and it can only witness a making that has already happened. A directory somebody else's at the usual target name
    //  is left where it is, and the clean is given a name of this run's own.
    #[cfg(test)]
    BEFORE_THE_TARGET_IS_MADE.with(|hook| {
        if let Some(another) = hook.borrow().as_ref() {
            another();
        }
    });
    // The clean's target directory is made here, exclusively, with its claim inside, so the directory this job deletes
    //  from carries the proof that it is this job's.
    let target: PathBuf = the_target_this_run_makes(&directory, usual.as_path(), destination, begun_by.as_str())?;
    if a_step(AStep::AfterTheTargetIsMade) {
        return Err(stop_words());
    }
    // The marker records the names this run made under the usual ones, and nothing else: a target directory at a name of
    //  this run's own is not the one the usual name reaches, so it is not recorded, and a later filling leaves whatever
    //  stands at the usual name rather than remove a directory this run saw was another's.
    let records: WhatTheMarkerRecords = if target == usual_target {
        WhatTheMarkerRecords::TheStagingAndItsTarget
    } else {
        WhatTheMarkerRecords::TheStaging
    };
    let written: TheMarkersAnswer = write_marker(directory.along(marker.as_path()).as_path(), destination, records, begun_by.as_str())?;
    // The entry that was written is read here, before any boundary: a file renamed over the name afterwards is not this
    //  run's, and a removal bound to this identity leaves it where its writer put it.
    let marker_made: Option<TheEntry> = TheEntry::at(marker.as_path());
    if a_step(AStep::AfterTheMarkerIsWritten) {
        return Err(stop_words());
    }
    return match written {
        TheMarkersAnswer::WrittenByThisRun => Ok(TheNames {
            directory,
            to: destination.to_path_buf(),
            begun_by,
            marker_made,
            staging: usual,
            target,
            marker: Some(marker),
            lease: Some(lease),
        }),
        // Another command has the name: what is there is not this run's to remove, and the target this run made goes with the
        //  names it will not use.
        TheMarkersAnswer::WrittenByAnother => {
            let _left = remove_the_claimed_directory(&directory, target.as_path(), destination);
            return names_of_this_runs_own(usual.as_path(), directory, destination, begun_by.as_str());
        }
    };
}

#[cfg(test)]
thread_local! {
    /// A control may take the usual target name between this run's look at the names and its own exclusive create,
    /// which is where another command's directory appears.
    /// Why:
    ///   The window is two adjacent calls,
    /// and a control that waited on a wall clock would pass whatever the code did.
    pub(crate) static BEFORE_THE_TARGET_IS_MADE: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Give back the names one marker of this job records for this very build directory:
/// the staging copy it was written for,
/// the clean's target directory when the marker records making it,
/// and the marker itself.
/// Why:
///   A killed filling and a killed add both leave names behind,
/// and the marker is the only thing that says which of them this job made:
/// the staging name always,
/// because the marker is written before anything is cloned into it,
/// and the target name only when the marker says the run that wrote it made that directory.
/// A directory at the target name that no marker records is somebody else's or an older release's:
/// it is left exactly as it is found, with one line,
/// and the filling that follows cleans into a name of its own.
///   The staging copy is removed on the marker and on the call that makes the name:
/// the clone and the add's move aside both fail when the name is taken,
/// so a directory there is what that call made,
/// with the one window of a kill between the marker's write and the call that creates the name.
///
/// # Errors
///
/// A recorded directory that cannot be removed,
/// and a marker that cannot be taken back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const giveBackTheNamesAMarkerClaims = (staging, marker, claim) => { removeIfAny(staging); if (claim.madeTarget) removeIfAny(targetOf(staging)); else if (taken(targetOf(staging))) debug(oneLine); takeTheMarkerBack(marker); };
/// ```
fn give_back_the_names_a_marker_claims(staging: &Path, marker: &Path, claim: &TheMarker, directory: &TheDirectory) -> Result<(), String> {
    // Each directory is removed only when the entry itself carries this job's claim for this very build directory, and
    //  the marker only while the entry that was read still stands at its name.
    let _staging: TheRemoval = remove_the_claimed_directory(directory, staging, &claim.to)?;
    let target: PathBuf = target_of(staging);
    if claim.made_target {
        let _target: TheRemoval = remove_the_claimed_directory(directory, target.as_path(), &claim.to)?;
    } else if the_name_is_taken(target.as_path()) {
        debug(
            TAG,
            format!("{} holds a directory no marker of this job records making, so it is left as it is found", target.display()).as_str(),
        );
    }
    let _marker: TheRemoval = take_the_marker_back(directory.along(marker).as_path(), claim.read)?;
    return Ok(());
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
fn names_of_this_runs_own(usual: &Path, directory: TheDirectory, to: &Path, begun_by: &str) -> Result<TheNames, String> {
    let own: PathBuf = a_name_of_this_runs_own(usual);
    // The clean deletes from this directory and the filling removes it, so it is made here, exclusively, with its claim
    //  inside, and nowhere else.
    let target: PathBuf = the_target_this_run_makes(&directory, own.as_path(), to, begun_by)?;
    return Ok(TheNames { directory, to: to.to_path_buf(), begun_by: String::from(begun_by), marker_made: None, staging: own, target, marker: None, lease: None });
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
/// Both are names this run made:
/// the target directory by an exclusive create of its own (`make_the_target_of`),
/// and the marker by the exclusive write that wrote it.
/// The marker goes after the target,
/// so a filling that is killed in between leaves the proof its successor needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeIfAny(target); removeFile(marker, { force: true });
/// ```
fn give_back(names: &TheNames) {
    if a_step(AStep::BeforeTheNamesAreGivenBack) {
        return;
    }
    match remove_the_claimed_directory(&names.directory, names.target.as_path(), &names.to) {
        Ok(TheRemoval::Removed) | Ok(TheRemoval::Left) => {}
        // The filling itself is done and published, and this directory is this job's own: the next filling of the same
        //  workspace clears it.
        Err(reason) => debug(TAG, format!("{reason}; the filling is done and the directory stays for the next one").as_str()),
    }
    if a_step(AStep::AfterTheTargetIsGivenBack) {
        return;
    }
    if let Some(marker) = names.marker.as_ref() {
        match take_the_marker_back(names.directory.along(marker).as_path(), names.marker_made) {
            Ok(TheRemoval::Removed) | Ok(TheRemoval::Left) => {}
            Err(reason) => debug(TAG, format!("{reason}; the marker stays where it is").as_str()),
        }
    }
    let _after: bool = a_step(AStep::AfterTheMarkerIsGivenBack);
    // The lease file stays where it is. Unlinking it while the lock is still held let two fillings hold "the lease" at once: a
    //  taker that opened the name before the unlink locked the inode the name no longer pointed at, while a taker after it made a
    //  fresh file and locked that, and both believed they held the names. The file holds no bytes of anybody's, one per build
    //  directory, and the next holder locks it as it stands (`claim_the_names`).
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
pub(crate) fn end_reason(end: &ToolEnd) -> String {
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
///  What a person reads when the resolution in a mirror could not be made:
///  the mirror's reason,
///  what this job's clean gave up,
///  and the cause a mirror has,
///  never the lock,
///  because the person's lock is read and never written and a lock is therefore never what stopped this.
/// Why:
///   A dependency whose path leaves the worktree is not carried into a mirror of it,
/// and a message that sent a person to update their lock for that would name the one file that cannot be
///  the cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theWordsForAMirrorThatGaveWay = (reason, entry) => `${reason}, so nothing of this workspace was cleaned and the copy of it was removed. The resolution runs in a mirror of the workspace, ...`;
/// ```
fn the_words_for_a_mirror_that_gave_way(reason: &str, entry: &PrewarmEntry) -> String {
    return format!(
        "{reason}, so nothing of this workspace was cleaned and the copy of it was removed. The resolution runs in a mirror \
         of the workspace, which carries no dependency whose path leaves the worktree, and that is what the manifest error above \
         names. Your {LOCK_FILE} is never written: a lock Cargo refused is resolved in that mirror, so nothing of yours is \
         replaced and nothing of yours moves. {}",
        what_to_do_next(entry, ""),
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
        // The two directories the clean deletes from are reached through the directory this filling took the names in,
        //  so a worktree made again at the same path gets no clean of the old filling's.
        set: vec![
            (OsString::from(BUILD_DIRECTORY_VARIABLE), names.directory.along(names.staging.as_path()).into_os_string()),
            (OsString::from(TARGET_DIRECTORY_VARIABLE), names.directory.along(names.target.as_path()).into_os_string()),
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
/// The rename's two ends are reached through the directory this run took the names in,
/// so a worktree made again at the same path never has its own build state moved aside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const publish = (names, destination) => renameNoReplace(entry(names.directory, names.staging), entry(names.directory, destination));
/// ```
fn publish(names: &TheNames, destination: &Path) -> Result<bool, String> {
    let staging: &Path = names.staging.as_path();
    return match rename_noreplace(names.directory.along(staging).as_path(), names.directory.along(destination).as_path()) {
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
        None => names_for(entry.to.as_path(), fill.kept_in.as_deref())?,
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
    // The staged state stands in the directory this run must hold, so every removal here reaches that directory and never
    //  one made at the same path afterwards.
    let Some(names_live_in) = staging.parent() else {
        return Err(format!("{} has no directory of its own, so nothing of it is this job's", staging.display()));
    };
    let directory: TheDirectory = TheDirectory::of(names_live_in)?;
    // The add staged this state and no filling of this worktree is at work on it yet; a second record naming the same
    //  staging is the case where one is, and then this run leaves it to that filling.
    let lease: Option<TheLease> = match claim_the_names_at(directory.along(lease_of(staging).as_path()).as_path()) {
        TheClaim::Held(lease) => Some(lease),
        TheClaim::Taken => return Ok(None),
        TheClaim::Refused(reason) => {
            debug(TAG, format!("{reason}; this run leaves the staged state to whichever filling holds it").as_str());
            return Ok(None);
        }
    };
    let marker: PathBuf = marker_of(staging);
    let TheMarkerThere::ThisJobs(claim) = the_marker_there(marker.as_path()) else {
        return Err(format!("{} holds what no filling of this job staged, so it is left exactly as it is found", staging.display()));
    };
    if claim.to != destination {
        return Err(format!("{} holds what no filling of this job staged, so it is left exactly as it is found", staging.display()));
    }
    // The add's marker records the staging name alone, because the add made no target directory, so a directory left at the
    //  target name by a killed filling is removed only when a marker records making it; otherwise the clean is given a name
    //  this run made and that directory is left as it is found.
    let usual_target: PathBuf = target_of(staging);
    if claim.made_target {
        let _usual: TheRemoval = remove_the_claimed_directory(&directory, usual_target.as_path(), destination)?;
    }
    let target: PathBuf = the_target_this_run_makes(&directory, staging, destination, claim.begun_by.as_str())?;
    return Ok(Some(TheNames {
        directory,
        to: destination.to_path_buf(),
        begun_by: claim.begun_by.clone(),
        marker_made: claim.read,
        staging: staging.to_path_buf(),
        target,
        marker: Some(marker),
        lease,
    }));
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
    let target: PathBuf = target_of(staging.as_path());
    let marker: PathBuf = marker_of(staging.as_path());
    // The directory the names live in, taken before anything here is removed or moved:
    // the move aside and every removal below are addressed through this descriptor, so a worktree made again at the
    //  same path while this add runs is never reached.
    let Some(names_live_in) = destination.parent() else {
        return Err(format!("{} has no directory to stage beside", destination.display()));
    };
    let directory: TheDirectory = TheDirectory::of(names_live_in)?;
    let standing: TheMarkerThere = the_marker_there(marker.as_path());
    if !destination.exists()
        && standing == TheMarkerThere::Nothing
        && !the_name_is_taken(staging.as_path())
        && !the_name_is_taken(target.as_path())
    {
        // Nothing stands at the names and the worktree holds nothing to move: no lease is taken and no exclude file is written for an
        //  add that stages nothing.
        return Ok(Staged::Nothing);
    }
    // The exclude file names the lease before the lease file exists, so a person's `git status` never shows it.
    if let Some(git_directory) = git_dir
        && let Err(reason) = exclude_the_staging_names(git_directory)
    {
        return Err(format!("{reason}, so the copied build state stays where it is; run `cargo clean --offline --package <member>` in the worktree instead"));
    }
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
        if claim.to == destination && the_name_is_taken(staging.as_path()) {
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
        take_the_marker_back(directory.along(marker.as_path()).as_path(), claim.read).map_err(|reason| {
            return format!(
                "{reason}; the copied build state is left where it is, so run `cargo clean --offline --package <member>` in {} instead",
                destination.display()
            );
        })?;
    }
    if the_name_is_taken(staging.as_path()) || the_name_is_taken(target.as_path()) || the_name_is_taken(marker.as_path()) {
        // A name this marker does not record stands beside the build directory: a directory, a file or a link somebody else's,
        //  and this job replaces nothing it did not make. A marker that named another worktree's state is taken back above and
        //  what it left here is still not this job's to remove, so it is named and left.
        return Err(what_stands_at_the_names(staging.as_path(), destination));
    }
    if !destination.exists() {
        return Ok(Staged::Nothing);
    }
    // The staging name and the target directory its clean uses stand inside the worktree, so the repository's own exclude file
    // names them and no `git status` or `git add -A` of the person's sees them.
    let begun_by: String = a_name_for_this_run();
    claim_the_staging_names(marker.as_path(), destination, git_dir, &directory, begun_by.as_str())?;
    // The claim goes inside the state before it is moved, so the directory that lands at the staging name carries the proof
    //  that this add made it: a kill between the two leaves a name this job can still prove is its own.
    let claim: PathBuf = destination.join(a_claim_inside());
    if let Err(reason) = write_the_claim_inside(destination, destination, begun_by.as_str()) {
        return Err(format!("{reason}, so the copied build state stays where it is"));
    }
    let claim_entry: Option<TheEntry> = TheEntry::at(claim.as_path());
    // A rename that never replaces, so a name taken between the check above and this call is refused rather than overwritten,
    //  and both ends reached through the directory this add took, so a worktree made again at the same path never has its
    //  build state moved aside.
    return match rename_noreplace(directory.along(destination).as_path(), directory.along(staging.as_path()).as_path()) {
        Ok(()) => Ok(Staged::Staged(staging)),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            // Something took the staging name between the check and the move: it is not this job's to replace, and the marker
            //  claims a name nothing was moved into, so it goes back, and so does the claim written in the state that stayed.
            let _claim: Option<TheRemoval> = remove_the_claim_of(directory.along(claim.as_path()).as_path(), claim_entry);
            if a_step(AStep::BeforeTheMarkerOfAFailedMoveIsTakenBack) {
                return Err(stop_words());
            }
            let _taken_back = directory.remove_the_file(marker.as_path());
            Err(what_stands_at_the_names(staging.as_path(), destination))
        }
        Err(error) => {
            // Nothing moved, so the marker claims a name nothing is in; take it back and say the state stays where it is.
            let _claim: Option<TheRemoval> = remove_the_claim_of(directory.along(claim.as_path()).as_path(), claim_entry);
            if a_step(AStep::BeforeTheMarkerOfAFailedMoveIsTakenBack) {
                return Err(stop_words());
            }
            let _taken_back = directory.remove_the_file(marker.as_path());
            Err(format!(
                "the copied build state at {} could not be moved aside ({error}), so it stays where it is; run `cargo clean --offline --package <member>` in that worktree instead",
                destination.display()
            ))
        }
    };
}

/// What:
///  What a person reads when something this job did not make stands at one of the staging names:
/// the staging name,
/// the clean's target name,
/// or the marker's,
/// so nothing there is removed and the repair goes back to the worktree's own clean.
/// Why:
///   A directory, a file or a link at one of these names is somebody else's unless a marker of this job records making it,
/// and the next build in the worktree would reuse the wrong artifacts,
/// so the person is told what stands there and what completes the repair instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const whatStandsAtTheNames = (staging, destination) => `${staging} already holds something this wrapper did not stage, so the copied build state is left where it is; ...`;
/// ```
fn what_stands_at_the_names(staging: &Path, destination: &Path) -> String {
    return format!(
        "{} already holds something this wrapper did not stage, so the copied build state is left where it is; the worktree's first build would reuse it, so run `cargo clean --offline --package <member>` in {} instead",
        staging.display(),
        destination.display()
    );
}

/// What the tail of a name of this run's own begins with,
/// as the exclude file reads a pattern.
/// Why:
///   A filling that finds the usual names taken works under `<usual>-<its process>-<the moment it asked>`,
/// and no pattern can name that moment,
/// so the tail is read from the one function that makes those names (`a_name_of_this_runs_own`)
/// and the part no pattern can predict is the `*`.
const OWN_NAME_TAIL: &str = "-*";

/// What:
///  The patterns that keep this job's staging names out of the person's `git status` and `git add -A`,
///  every form its name makers can produce.
/// Why:
///   The names are written into the repository's own exclude file (`<git-dir>/info/exclude`),
/// which applies to every worktree of it,
/// because they stand beside the build directory they take, inside the worktree.
/// A rule written for the ordinary name alone left the names of a second filling visible,
/// and those are the names a person's `git add --all` met while another filling was at work.
/// The forms are read from the functions that make the names (`staging_of`, `target_of`, `marker_of`, `lease_of`),
/// so a form one of them can produce is a form these patterns cover:
/// the staging directory and the target directory are directories,
/// and the marker and the lease are files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const excludedStagingPatterns = () => [stagingOf('*'), withOwnTail(stagingOf('*'))].flatMap((staging) => [staging + '/', targetOf(staging) + '/', markerOf(staging), leaseOf(staging)]);
/// ```
pub(crate) fn excluded_staging_patterns() -> Vec<String> {
    let usual: PathBuf = staging_of(Path::new("*"));
    let mut own: OsString = usual.as_os_str().to_os_string();
    own.push(OWN_NAME_TAIL);
    let mut patterns: Vec<String> = Vec::new();
    for staging in [usual, PathBuf::from(own)] {
        for (named, directory) in [
            (staging.clone(), true),
            (target_of(staging.as_path()), true),
            (marker_of(staging.as_path()), false),
            (lease_of(staging.as_path()), false),
        ] {
            let mut pattern: String = named.to_string_lossy().into_owned();
            if directory {
                pattern.push('/');
            }
            patterns.push(pattern);
        }
    }
    return patterns;
}

/// The most bytes of the repository's exclude file this job reads.
/// Why:
///   The file is the person's own and may hold any number of their lines,
/// and a file far larger than an exclude list ever is could not be one:
/// reading it into memory whole would be a read whose size somebody else decides.
const EXCLUDE_LIMIT: u64 = 1 << 20;

/// What:
///  Why a repository's exclude file could not be read for naming this job's patterns,
/// in the words a person reads.
/// Why:
///   Each refusal is a different thing to fix,
/// and the caller formats the same reason into the add's line and the filling's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusalWords = (refusal) => refusal.kind === 'not-regular' ? 'is not a regular file' : ...;
/// ```
fn exclusion_refusal_words(refusal: &crate::private_storage::ReadRefusal) -> String {
    use crate::private_storage::ReadRefusal;
    return match refusal {
        ReadRefusal::Missing => String::from("is not there"),
        ReadRefusal::NotRegular => String::from("is not a regular file (a link, a directory or a named pipe)"),
        ReadRefusal::TooLarge => format!("holds more than {EXCLUDE_LIMIT} bytes"),
        ReadRefusal::Io(error) => format!("could not be read ({error})"),
    };
}

/// What:
///  The lines of one exclude file,
/// as the bytes the file holds split at each line end.
/// Why:
///   The file is the person's own and need not be UTF-8,
/// so whether a pattern stands there is decided on the bytes and never on a lossy text:
/// the only bytes this job ever writes are its own pattern lines.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesOf = (bytes) => bytes.split(0x0a).map((line) => trimEnd(line));
/// ```
fn lines_of_excluding(bytes: &[u8]) -> Vec<&[u8]> {
    return bytes
        .split(|byte| return *byte == b'\n')
        // Each line as Git reads it as a pattern: the ending and the trailing spaces gone, leading spaces and tabs kept,
        //  so a form Git does not ignore by is never taken for one of this job's own
        //  (`exclude_line_as_git_reads_it.rs`, measured with Git 2.56.0).
        .map(crate::exclude_line_as_git_reads_it::the_pattern_git_reads)
        .collect();
}

/// What:
///  Name this job's staging patterns in the repository's exclude file,
/// every pattern of them that is not named there yet,
/// the shapes of a run's own names included.
/// Why:
///   The staging name sits inside the worktree beside the build directory it takes,
/// because btrfs refuses a rename of a directory out of a worktree that is a subvolume,
/// so without this the person's `git status` would show it and a `git add -A` would commit a copy of `target/`.
/// Each pattern is looked for as a line of its own,
/// so a file an older release wrote with fewer patterns,
/// an append a kill cut short,
/// and a list a person edited by hand all get the patterns they lack,
/// and one already there is never written twice.
///   The file is read without following a link and without waiting for a writer:
/// a named pipe at that path would hold a plain read for ever,
/// and the filling calls this before the future's first await,
/// where no ceiling could expire
/// (`SPEC.md`,
/// "Every wait has a bound";
/// the marker beside it is read the same way).
/// A read that is not a regular file,
/// too large,
/// or refused for another reason is this job's refusal and not somebody's empty file:
/// the add then cleans the copied state where it stands, as it does when the file cannot be written,
/// and the filling fails that record with this reason,
/// so neither of them ever stages under names a person's `git status` would show without having said so.
///   The write is one append of the patterns that were missing,
/// opened without following a link and without waiting,
/// so the person's own lines are never rewritten and a named pipe cannot hold the write either.
///
/// # Errors
///
/// An exclude file that is not a regular file, cannot be read or cannot be appended to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const excludeTheStagingNames = (gitDir) => { const path = join(gitDir, 'info/exclude'); const bytes = readRegularFile(path, LIMIT) ?? empty; const missing = PATTERNS.filter((p) => !linesOf(bytes).includes(p)); if (missing.length === 0) return ok; const text = (bytes.length && bytes.at(-1) !== 0x0a ? '\n' : '') + missing.map((p) => p + '\n').join(''); appendNoWait(path, text); };
/// ```
pub(crate) fn exclude_the_staging_names(git_dir: &Path) -> Result<(), String> {
    let info: PathBuf = git_dir.join("info");
    let exclude: PathBuf = info.join("exclude");
    // The read the marker beside it gets: a regular file only, opened without following a link and without waiting for a writer.
    let already: Vec<u8> = match crate::private_storage::read_regular_file(exclude.as_path(), EXCLUDE_LIMIT) {
        Ok(bytes) => bytes,
        // A repository that never named an exclusion has none, and an empty list lacks every pattern.
        Err(crate::private_storage::ReadRefusal::Missing) => Vec::new(),
        Err(refusal) => {
            return Err(format!(
                "{} {} and no staging name of this job's could be named in it without waiting for whatever it is",
                exclude.display(),
                exclusion_refusal_words(&refusal)
            ));
        }
    };
    let lines: Vec<&[u8]> = lines_of_excluding(already.as_slice());
    let wanted: Vec<String> = excluded_staging_patterns();
    let missing: Vec<&String> = wanted
        .iter()
        .filter(|pattern| return !lines.contains(&pattern.as_bytes()))
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    #[cfg(test)]
    AFTER_THE_EXCLUDE_IS_READ.with(|hook| {
        if let Some(another) = hook.borrow().as_ref() {
            another();
        }
    });
    if let Err(error) = std::fs::create_dir_all(info.as_path()) {
        return Err(format!("{} could not be made ({error})", info.display()));
    }
    // The batch starts with a line end whatever the file ended with when it was read. A decision taken from that reading is stale
    //  by the time of the write: another command can append an unterminated pattern after it, and a batch without a leading line
    //  end then joins that pattern and the first of these into a line that is neither, so the staging name is ignored by nothing
    //  and a copied build directory is stageable. Git reads a blank line as nothing, so the extra line end costs nothing when
    //  nothing raced, and the whole batch is one `write`, so nothing can land inside it.
    let mut text: Vec<u8> = Vec::with_capacity(missing.iter().map(|pattern| return pattern.len() + 1).sum::<usize>() + 1);
    text.push(b'\n');
    for pattern in missing {
        text.extend_from_slice(pattern.as_bytes());
        text.push(b'\n');
    }
    // The file is the person's own and Git made it when the repository was, so this appends to it and never rewrites it:
    // a kill in the middle leaves their lines whole and at most cuts this job's last line short, which the next call writes again.
    let opened = {
        use std::os::unix::fs::OpenOptionsExt as _;
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            // A link is not this job's to write through, and a named pipe must refuse or answer rather than wait for a reader.
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(exclude.as_path())
    };
    let mut file = match opened {
        Ok(file) => file,
        Err(error) => return Err(format!("{} could not be opened without waiting ({error})", exclude.display())),
    };
    return match std::io::Write::write_all(&mut file, text.as_slice()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} could not be appended to ({error})", exclude.display())),
    };
}

#[cfg(test)]
thread_local! {
    /// A control may append to the exclude file between this job's reading of it and its own write,
    /// which is where another command's append lands.
    /// Why:
    ///   The window is two adjacent calls,
    /// and a control that waited on a wall clock would pass whatever the code did.
    pub(crate) static AFTER_THE_EXCLUDE_IS_READ: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
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
fn claim_the_staging_names(marker: &Path, destination: &Path, git_dir: Option<&Path>, directory: &TheDirectory, begun_by: &str) -> Result<(), String> {
    if let Some(git_directory) = git_dir
        && let Err(reason) = exclude_the_staging_names(git_directory)
    {
        return Err(format!("{reason}, so the copied build state stays where it is; run `cargo clean --offline --package <member>` in the worktree instead"));
    }
    // The marker is written through the directory this add took the names in: a worktree made again at the same path
    //  while this add runs gets no marker of this add's, so a later filling there believes nothing about its names.
    return match write_marker(directory.along(marker).as_path(), destination, WhatTheMarkerRecords::TheStaging, begun_by)? {
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
    let TheMarkerThere::ThisJobs(claim) = the_marker_there(marker.as_path()) else { return };
    if claim.to != entry.to {
        return;
    }
    let Some(names_live_in) = staging.parent() else { return };
    let directory: TheDirectory = match TheDirectory::of(names_live_in) {
        Ok(directory) => directory,
        Err(reason) => {
            debug(TAG, format!("{reason}; the names a killed filling left are left for the next filling").as_str());
            return;
        }
    };
    // The marker names this build directory, and only the lease can say the filling that wrote it is gone: a filling that is
    //  still cleaning or publishing this staging holds it, and then nothing here is removed.
    let lease: TheLease = match claim_the_names_at(directory.along(lease_of(staging.as_path()).as_path()).as_path()) {
        TheClaim::Held(lease) => lease,
        TheClaim::Taken => return,
        TheClaim::Refused(reason) => {
            debug(TAG, format!("{reason}; the names a killed filling left are left for the next filling").as_str());
            return;
        }
    };
    let _lease: TheLease = lease;
    if let Err(reason) = give_back_the_names_a_marker_claims(staging.as_path(), marker.as_path(), &claim, &directory) {
        // Nothing of the worktree depends on these names; the next filling of the same workspace tries again.
        debug(TAG, format!("{reason}; what a killed filling left stays for the next one").as_str());
    }
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
        if a_step(AStep::BeforeTheClone) {
            return Err(stop_words());
        }
        // This run makes the staging directory itself, exclusively, with its claim inside, and the clone fills that
        //  directory: a name another command holds is refused here, and a kill at any point of the clone leaves a
        //  directory that proves it is this run's.
        make_the_claimed_directory(&names.directory, names.staging.as_path(), &names.to, names.begun_by.as_str())?;
        let made: Result<Cloned, String> = clone_tree_awaiting(
            from.as_path(),
            names.directory.along(names.staging.as_path()).as_path(),
            1,
            TheDestination::MadeAndClaimedByTheCaller,
        )
        .await;
        if a_step(AStep::AfterTheClone) {
            return Err(stop_words());
        }
        if let Err(reason) = made {
            let _removed = remove_the_claimed_directory(&names.directory, names.staging.as_path(), &names.to);
            return Err(format!("the build state of {} could not be cloned ({reason})", from.display()));
        }
    }
    // A build that started during the clone owns the destination now, and this job touches nothing of it.
    if entry.to.exists() {
        let _removed = remove_the_claimed_directory(&names.directory, names.staging.as_path(), &names.to);
        return Ok(Filled::Kept);
    }
    if a_step(AStep::BeforeTheClean) {
        return Err(stop_words());
    }
    // The clean deletes from both directories, so both are checked at the act: an entry another command replaced
    //  carries no claim of this run's, and nothing here writes into it.
    for named in [names.staging.as_path(), names.target.as_path()] {
        if !the_entry_is_still_this_runs(&names.directory, named, &names.to) {
            return Err(format!(
                "{} no longer holds this run's claim, so nothing of {} was cleaned and nothing of it was written",
                named.display(),
                names.to.display()
            ));
        }
    }
    if a_step(AStep::BeforeTheChildIsGivenItsPaths) {
        return Err(stop_words());
    }
    // The clean is given `--locked` here, so it never writes the worktree's lock: a lock it refuses is what sends this step to
    // a mirror of the workspace, where one clean resolves as it does in the worktree and writes the mirror's own lock, and
    // the person's file is never written, renamed or exchanged. Nothing of theirs is displaced, so no kill can strand their
    // bytes and nothing has to be put back.
    let mut run: ToolRun = clean_the_copy(fill, entry, names).await;
    if !matches!(run.end, ToolEnd::Succeeded) {
        if a_step(AStep::BeforeTheMirrorClean) {
            return Err(stop_words());
        }
        run = match clean_in_a_mirror(MirrorClean {
            tool: CargoTool { program: fill.cargo.clone(), limits: fill.limits },
            worktree: fill.worktree.clone(),
            workspace: entry.root.clone(),
            members: entry.members.clone(),
            build_directory: names.staging.clone(),
            target_directory: names.target.clone(),
            kept_in: fill.kept_in.clone(),
            job: fill.filling.clone(),
        })
        .await
        {
            Ok(in_the_mirror) => in_the_mirror,
            Err(reason) => {
                let _removed = remove_the_claimed_directory(&names.directory, names.staging.as_path(), &names.to);
                return Err(the_words_for_a_mirror_that_gave_way(reason.as_str(), entry));
            }
        };
    }
    if a_step(AStep::AfterTheClean) {
        return Err(stop_words());
    }
    if !matches!(run.end, ToolEnd::Succeeded) {
        let _removed = remove_the_claimed_directory(&names.directory, names.staging.as_path(), &names.to);
        let reason: String = end_reason(&run.end);
        return Err(format!(
            "the copy of {} could not be cleaned ({reason}), so nothing was published and everything staged for it was removed. The clean itself \
             never writes the worktree's {LOCK_FILE}: a lock it refuses is resolved in a mirror of the workspace, so a write of yours is never \
             replaced and nothing of yours is moved. {}",
            entry.from.as_deref().unwrap_or(names.staging.as_path()).display(),
            what_to_do_next(entry, reason.as_str()),
        ));
    }
    if a_step(AStep::BeforeThePublish) {
        return Err(stop_words());
    }
    if !the_entry_is_still_this_runs(&names.directory, names.staging.as_path(), &names.to) {
        return Err(format!(
            "{} no longer holds this run's claim, so nothing of {} was published",
            names.staging.display(),
            names.to.display()
        ));
    }
    if a_step(AStep::BeforeThePublishTakesTheCopyAside) {
        return Err(stop_words());
    }
    let published: Result<bool, String> = publish(names, entry.to.as_path());
    if a_step(AStep::AfterThePublish) {
        return Err(stop_words());
    }
    return match published {
        Ok(true) => Ok(Filled::Published),
        Ok(false) => {
            let _removed = remove_the_claimed_directory(&names.directory, names.staging.as_path(), &names.to);
            Ok(Filled::Kept)
        }
        Err(reason) => {
            let _removed = remove_the_claimed_directory(&names.directory, names.staging.as_path(), &names.to);
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
    let fill: Fill = Fill {
        cargo,
        limits: environment.limits(),
        kept_in: Some(item.git_dir.clone()),
        worktree: item.repository.clone(),
        filling: crate::package_repair_workspace_mirror::a_name_of_this_job(),
    };
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
