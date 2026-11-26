//! What:
//!  The names one filling of a workspace works in,
//!  and the proof they are this job's:
//!  the staging name beside the build directory it fills,
//!  the target directory the clean deletes from,
//!  the marker that claims both,
//!  and the code that takes the three names,
//!  writes the marker,
//!  and gives them back.
//! Why:
//!  `package_repair_prewarm_job.rs` would otherwise pass the file-size limit,
//!  and this job removes only directories it can prove it made,
//!  so the names and the marker that proves them belong together in a file of their own,
//!  which the filling and the add's own move of the state both read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stagingOf = (destination: string) => `${destination}${STAGING_SUFFIX}`;
//! ```

/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the staging suffix the add and this job share and the record's entry.
use crate::package_repair_prewarm::{
    PrewarmEntry,
    STAGING_SUFFIX,
};
/// Import the owned file-name buffer a suffixed name is built in.
use std::ffi::OsString;
/// Import native paths.
use std::path::{
    Path,
    PathBuf,
};
/// Tag used for diagnostics from this module.
pub(super) const TAG: &str = "packageRepairPrewarmJob";

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
pub(crate) fn target_of(staging: &Path) -> PathBuf {
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
pub(crate) fn marker_of(staging: &Path) -> PathBuf {
    let mut named: OsString = staging.as_os_str().to_os_string();
    named.push(MARKER_SUFFIX);
    return PathBuf::from(named);
}

/// What the marker of the staging name is suffixed with,
/// beside the staging directory.
const MARKER_SUFFIX: &str = ".owner";

/// The text a marker carries,
/// so a person who finds one knows which program wrote it and what it claims.
pub(crate) const MARKER_TEXT: &[u8] =
    b"monochromatic-git-policy-cli's Cargo prewarm job wrote this file. \
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
pub(super) fn marker_claims(marker: &Path) -> bool {
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
pub(super) struct TheNames {
    /// The directory the source's build state is cloned into,
    /// and published from.
    pub(super) staging: PathBuf,
    /// The target directory the clean deletes from.
    pub(super) target: PathBuf,
    /// The marker that says the two names are this job's,
    /// when this run claimed the usual ones.
    pub(super) marker: Option<PathBuf>,
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
pub(super) fn names_for(destination: &Path) -> Result<TheNames, String> {
    let usual: PathBuf = staging_of(destination);
    let usual_target: PathBuf = target_of(usual.as_path());
    let marker: PathBuf = marker_of(usual.as_path());
    if marker_claims(marker.as_path()) {
        // An earlier filling of this same workspace was interrupted, so what it left under these names is this job's.
        remove_if_any(usual.as_path())?;
        remove_if_any(usual_target.as_path())?;
        return Ok(TheNames {
            staging: usual,
            target: usual_target,
            marker: Some(marker),
        });
    }
    if usual.exists() || usual_target.exists() || marker.exists() {
        // Somebody else's file or directory holds one of the names, so this run stages under names of its own.
        return Ok(names_of_this_runs_own(usual.as_path()));
    }
    return match write_marker(marker.as_path())? {
        TheMarkersAnswer::WrittenByThisRun => Ok(TheNames {
            staging: usual,
            target: usual_target,
            marker: Some(marker),
        }),
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
    return TheNames {
        staging: own,
        target: own_target,
        marker: None,
    };
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
    let since_the_epoch: std::time::Duration = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
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
pub(crate) enum TheMarkersAnswer {
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
pub(crate) fn write_marker(marker: &Path) -> Result<TheMarkersAnswer, String> {
    return match crate::private_storage::write_private_file_unsynced(marker, MARKER_TEXT) {
        Ok(()) => Ok(TheMarkersAnswer::WrittenByThisRun),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Ok(TheMarkersAnswer::WrittenByAnother)
        }
        Err(error) => Err(format!(
            "{} could not be written, so nothing of this workspace was filled ({error})",
            marker.display()
        )),
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
pub(super) fn give_back(names: &TheNames) {
    if let Err(reason) = remove_if_any(names.target.as_path()) {
        // The filling itself is done and published, and this directory is this job's own: the next filling of the same
        //  workspace clears it.
        debug(
            TAG,
            format!("{reason}; the filling is done and the directory stays for the next one")
                .as_str(),
        );
    }
    let Some(marker) = names.marker.as_ref() else {
        return;
    };
    match std::fs::remove_file(marker.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => debug(
            TAG,
            format!(
                "{} could not be removed after the filling ({error})",
                marker.display()
            )
            .as_str(),
        ),
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
pub(super) fn remove_if_any(path: &Path) -> Result<(), String> {
    return match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("{} could not be removed ({error})", path.display())),
    };
}

/// What:
///  Remove the staging copy, its target directory and the marker a filling of this workspace left behind
///  when it was killed after it published,
/// and nothing at all when this job's own marker is not there.
/// Why:
///   A publish is one rename that leaves no trace of the filling that made it,
/// so a kill between the publish and the giving back of the names strands an empty directory and its marker for ever:
/// every later filling of that workspace answers `Filled::Kept` at the destination and never looks at the names again.
/// The marker is what proves the names are this job's own,
/// so anything else standing there is somebody's work and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const giveBackWhatAKilledFillingLeft = (destination) => { const staging = stagingOf(destination); const marker = markerOf(staging); if (!markerClaims(marker)) return; removeIfAny(staging); removeIfAny(targetOf(staging)); removeFileIfPresent(marker); };
/// ```
pub(super) fn give_back_what_a_killed_filling_left(entry: &PrewarmEntry) {
    let staging: PathBuf = match entry.staged_in.as_ref() {
        Some(staged) => staged.clone(),
        None => staging_of(entry.to.as_path()),
    };
    let marker: PathBuf = marker_of(staging.as_path());
    if !marker_claims(marker.as_path()) {
        return;
    }
    let names: TheNames = TheNames {
        staging: staging.clone(),
        target: target_of(staging.as_path()),
        marker: Some(marker),
    };
    if let Err(reason) = remove_if_any(names.staging.as_path()) {
        // Nothing of the worktree depends on this directory; the next filling of the same workspace tries again.
        debug(
            TAG,
            format!("{reason}; a staging copy a killed filling left stays for the next one")
                .as_str(),
        );
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
pub(super) fn make_the_destinations_own_directory(entry: &PrewarmEntry) -> Result<(), String> {
    let Some(parent) = entry.to.parent() else {
        return Ok(());
    };
    return match std::fs::create_dir_all(parent) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!(
            "{} could not be made, so nothing of this workspace was filled ({error})",
            parent.display()
        )),
    };
}
