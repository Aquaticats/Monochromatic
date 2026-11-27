//! What:
//! How an archive install becomes visible:
//! the archive is unpacked into a hidden staging directory beside the version directory,
//! a completion marker is written there last,
//! and the version's name is then given the finished copy in one step.
//! Why:
//! `tar --extract` creates a file with its final mode before it writes the content,
//! so an interrupted unpacking (a truncated archive,
//! a killed process,
//! a full disk) leaves an executable file of the right name with the wrong bytes.
//! If that sat at the version's name,
//! the install would count as finished,
//! the working older version would be deleted as superseded,
//! and `current` would point at the broken copy.
//! Staging keeps a failed unpacking away from the version's name and from any existing copy;
//! the marker is what tells a directory made by this scheme from one an earlier release unpacked in place
//! (which cannot be told from a broken one),
//! so such a directory is never the reason to delete anything and a reinstall replaces it.
//!
//! The last step replaces a copy that is already at the version's name.
//! Where the file system can exchange two names in one call,
//! the names change places in that call and the version's name is never absent
//! (`exchanging_the_two_names` quotes the call and the errors that say the file system cannot).
//! Where it cannot,
//! the copy that is there is moved aside first and the finished one renamed in after it,
//! which leaves one instant in which the version's name is absent.
//!
//! What a kill leaves at each step of an archive install,
//! and what removes it:
//! the half file `<file>.download` during the download,
//! which the next download writes over;
//! an unfinished `.staging` sibling during the unpacking,
//! which the next install of that tool removes;
//! a finished `.staging` sibling after the marker,
//! which the next install of that version publishes;
//! a finished `.staging` sibling with the copy that was there at a `.replaced` sibling and the version's name absent
//! after a kill in the instant above,
//! which the next install of that version publishes,
//! putting the `.replaced` copy back only when the finished one is gone;
//! and an unfinished `.replaced` sibling after the rename,
//! which the next install of that tool removes.
//! `leftovers_of_a_killed_install` holds that list with its reasoning,
//! and tells a dead installer's sibling from a running one's by the pair in its name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // archive-staging.ts
//! export function stagingDirOf(versionDir: string): string
//! export function hasCompleteMarker(dir: string): boolean
//! export function writeCompleteMarker(dir: string, toolId: string, version: string): void
//! export function publishStaged(staging: string, finalDir: string, firstBin: string): void
//! export function discardStaging(staging: string): void
//! export function recoverLeftovers(versionDir: string, firstBin: string): Recovered
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// The steps of a publish a build made for controls can arm.
mod a_control_plants_these_steps;
/// The one call that exchanges two names,
/// and the errors that say it is impossible here.
mod exchanging_the_two_names;
/// The hidden siblings a killed install leaves,
/// and what a later install does with them.
mod leftovers_of_a_killed_install;
/// The process that made a hidden sibling,
/// as the pair that no reuse repeats.
mod the_owner_of_a_hidden_sibling;

/// Imports the plant a control arms,
/// together with the functions that arm and forget it.
#[cfg(test)]
pub(crate) use self::a_control_plants_these_steps::{
    Plant,
    clear_the_plant,
    plant_these_steps,
};
/// Imports the two steps a control can arm.
use self::a_control_plants_these_steps::{
    exchange_is_refused,
    reached_the_instant_between_the_two_renames,
};
/// Imports the exchange and the errors that mean the file system cannot do it.
use self::exchanging_the_two_names::{
    cannot_exchange,
    exchange_the_two_names,
};
/// Imports what a later install does with the leftovers of a killed one.
pub(crate) use self::leftovers_of_a_killed_install::{
    Recovered,
    recover_leftovers,
};
/// Imports the owner of a hidden sibling,
/// and re-exports it for a caller that builds one of these names itself.
pub(crate) use self::the_owner_of_a_hidden_sibling::Owner;
/// Re-exports the reader of a process's start time,
/// for a control that names an installer of its own choosing.
#[cfg(test)]
pub(crate) use self::the_owner_of_a_hidden_sibling::start_time_of;

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `is_finished` from `super::published_archive`.
use super::published_archive::is_finished;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// The file whose presence shows an install finished;
/// it is the last file written before the version directory takes its name.
pub const COMPLETE_MARKER: &str = ".mpm-shadow-installed";

/// The purpose in a hidden sibling's name that holds an install being unpacked.
pub(crate) const STAGING_PURPOSE: &str = "staging";

/// The purpose in a hidden sibling's name that holds the copy being replaced.
pub(crate) const REPLACED_PURPOSE: &str = "replaced";

/// `<parent>/.<name>.<purpose>.<process id>.<start time>`,
/// a hidden sibling of `version_dir` that no version scan lists.
/// The two last fields name the installer that made it,
/// so a later install can tell one that no longer runs from one that does
/// (`the_owner_of_a_hidden_sibling`).
pub(crate) fn hidden_sibling(
    version_dir: &Path,
    purpose: &str,
    owner: Owner,
) -> PathBuf {
    let name: String = format!(
        ".{}.{purpose}.{}.{}",
        file_name_text(version_dir),
        owner.pid,
        owner.started
    );
    return version_dir.with_file_name(name);
}

/// The staging directory for the install this process is unpacking into `version_dir`.
pub fn staging_dir_of(version_dir: &Path) -> PathBuf {
    return hidden_sibling(version_dir, STAGING_PURPOSE, Owner::here());
}

/// The name this process moves the copy at `version_dir` aside to,
/// where the file system cannot exchange the two names.
fn replaced_dir_of(version_dir: &Path) -> PathBuf {
    return hidden_sibling(version_dir, REPLACED_PURPOSE, Owner::here());
}

/// Whether `dir` holds the completion marker as a regular file
/// (a link of that name does not count).
pub fn has_complete_marker(dir: &Path) -> bool {
    return match fs::symlink_metadata(dir.join(COMPLETE_MARKER)) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    };
}

/// Writes the completion marker into `dir`;
/// call it after everything else in the install has been checked.
pub fn write_complete_marker(
    dir: &Path,
    tool_id: &str,
    version: &str,
) -> Result<(), ToolchainError> {
    let marker: PathBuf = dir.join(COMPLETE_MARKER);
    if let Err(error) = fs::write(&marker, format!("{tool_id} {version}\n")) {
        return Err(ToolchainError::failure(format!(
            "cannot write {}: {error}",
            marker.display()
        )));
    }
    return Ok(());
}

/// Removes a path the installer made and no longer needs,
/// whatever it turned out to be;
/// a failure is ignored because the caller is already on its way out,
/// and the leftover is hidden and harmless.
pub fn discard_path(path: &Path) {
    if fs::remove_dir_all(path).is_err() {
        // Best effort: the path may be a file or a link, or already gone.
        let _ = fs::remove_file(path);
    }
}

/// What:
/// Makes the staged install the version directory.
/// A copy already at the version's name that is finished
/// (it carries the marker and still holds `first_bin` as an executable file,
/// as `is_finished` checks)
/// is kept,
/// because another run installed the same verified release meanwhile,
/// and the staging directory is discarded;
/// any other copy is replaced in one step where the file system can exchange two names,
/// and in two renames where it cannot.
/// Why:
/// A copy that was working is never deleted before its replacement is in place,
/// and a reader of the version's name finds either no copy (before the first install)
/// or a whole finished one.
/// A marker alone is not enough to keep the old copy:
/// a person who deleted or made non-executable the program in a marked directory
/// would otherwise see every later install discard the good copy and report success.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function publishStaged(staging: string, finalDir: string, firstBin: string): void  // throws
/// ```
pub fn publish_staged(
    staging: &Path,
    final_dir: &Path,
    first_bin: &str,
) -> Result<(), ToolchainError> {
    if is_finished(final_dir, first_bin) {
        discard_path(staging);
        return Ok(());
    }
    if fs::symlink_metadata(final_dir).is_err() {
        return move_the_finished_install(staging, final_dir, first_bin);
    }
    return replace_the_copy_that_is_there(staging, final_dir, first_bin);
}

/// What:
/// Gives a name nothing holds to the finished install,
/// in the one rename that this case needs.
/// Why:
/// The version's name was absent already,
/// so no reader loses a copy here,
/// and a rename cannot leave the name half written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveTheFinishedInstall(staging: string, finalDir: string): void  // throws
/// ```
fn move_the_finished_install(
    staging: &Path,
    final_dir: &Path,
    first_bin: &str,
) -> Result<(), ToolchainError> {
    if let Err(error) = fs::rename(staging, final_dir) {
        // Another run may have published this version in the instant since: the name is then finished, and nothing is
        // missing.
        if is_finished(final_dir, first_bin) {
            discard_path(staging);
            return Ok(());
        }
        discard_path(staging);
        return Err(ToolchainError::failure(format!(
            "cannot move the finished install to {}: {error}",
            final_dir.display()
        )));
    }
    return Ok(());
}

/// What:
/// Replaces an unfinished copy at the version's name with the staged one,
/// exchanging the two names in one call where the file system can.
/// Why:
/// This is the only case with a window:
/// moving the old copy aside and renaming the new one in leaves an instant in which the version's name is absent,
/// and a kill in it leaves a copy nothing reads at the aside name
/// (`leftovers_of_a_killed_install` repairs that,
/// and the exchange means the instant need not exist at all).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceTheCopyThatIsThere(staging: string, finalDir: string, firstBin: string): void  // throws
/// ```
fn replace_the_copy_that_is_there(
    staging: &Path,
    final_dir: &Path,
    first_bin: &str,
) -> Result<(), ToolchainError> {
    if exchange_is_refused() {
        return two_renames(staging, final_dir);
    }
    match exchange_the_two_names(staging, final_dir) {
        Ok(()) => {
            // The exchange left the copy that was there at the staging name, where nothing reads it.
            discard_path(staging);
            return Ok(());
        }
        Err(error) if cannot_exchange(&error) => return two_renames(staging, final_dir),
        Err(error) => {
            // The exchange changed neither name.
            if is_finished(final_dir, first_bin) {
                discard_path(staging);
                return Ok(());
            }
            discard_path(staging);
            return Err(ToolchainError::failure(format!(
                "cannot put the finished install in place of the copy at {}: {error}; both copies are as they were, so check that the directory is writable and has room, and run the command again",
                final_dir.display()
            )));
        }
    }
}

/// What:
/// Replaces the copy at the version's name in two renames,
/// for a file system that cannot exchange the two names in one call.
/// Why:
/// The copy that is there is moved aside first,
/// so a failed second rename puts it back and a working copy is never deleted before its replacement is in place.
/// The instant between the two renames is the one window this module has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function twoRenames(staging: string, finalDir: string): void  // throws
/// ```
fn two_renames(
    staging: &Path,
    final_dir: &Path,
) -> Result<(), ToolchainError> {
    let aside: PathBuf = replaced_dir_of(final_dir);
    // A leftover of a killed run with the same owner would block the rename.
    discard_path(&aside);
    if let Err(error) = fs::rename(final_dir, &aside) {
        discard_path(staging);
        return Err(ToolchainError::failure(format!(
            "cannot move the copy at {} aside to install over it: {error}",
            final_dir.display()
        )));
    }
    reached_the_instant_between_the_two_renames();
    if let Err(error) = fs::rename(staging, final_dir) {
        if let Err(restore_error) = fs::rename(&aside, final_dir) {
            discard_path(staging);
            return Err(ToolchainError::failure(format!(
                "cannot move the finished install to {}: {error}; the copy that was there is at {} and could not be put back: {restore_error}",
                final_dir.display(),
                aside.display()
            )));
        }
        discard_path(staging);
        return Err(ToolchainError::failure(format!(
            "cannot move the finished install to {}: {error}",
            final_dir.display()
        )));
    }
    discard_path(&aside);
    return Ok(());
}
