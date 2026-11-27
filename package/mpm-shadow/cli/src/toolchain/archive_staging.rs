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
//! a `.recovered` sibling when the recovery itself is killed after it claimed that copy,
//! which the next install of that version publishes;
//! and an unfinished `.replaced` sibling after the rename,
//! which the next install of that tool removes.
//! `leftovers_of_a_killed_install` holds that list with its reasoning,
//! and tells a dead installer's sibling from a running one's by the four facts in its name.
//!
//! A publish that fails costs neither copy:
//! the finished copy stays under the name this run holds it at,
//! which the next install publishes without downloading,
//! and the message names that path and the version's name,
//! so a person reads what is on disk.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // archive-staging.ts
//! export function stagingDirOf(versionDir: string): string
//! export function hasCompleteMarker(dir: string): boolean
//! export function writeCompleteMarker(dir: string, toolId: string, version: string, archiveSha256: string): void
//! export function publishStaged(staging: string, finalDir: string, firstBin: string): 'here' | 'already-there'
//! export function discardStaging(staging: string): void
//! export function recoverLeftovers(versionDir: string, firstBin: string, olderVersion: AnOlderVersion, expected: CopyRecord | undefined): Recovered
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `io` from `std` for the error a rename answers with.
use std::io;
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
/// The completion marker,
/// the facts it records and the lock that vouches for them.
mod the_completion_marker;
/// The process that made a hidden sibling,
/// as the facts that no reuse repeats.
mod the_owner_of_a_hidden_sibling;

/// Re-exports the marker file's name,
/// the question whether a directory carries it,
/// and the write an install makes last.
pub use self::the_completion_marker::{
    COMPLETE_MARKER,
    has_complete_marker,
    write_complete_marker,
};
/// Re-exports what the lock verified about a copy,
/// and the questions a publish and a cleanup ask of the marker it holds.
pub(crate) use self::the_completion_marker::{
    CopyRecord,
    has_vouched_marker,
    marker_vouches,
};

/// Imports the renames a control can refuse,
/// which the rename below names whether or not a control is planted.
pub(crate) use self::a_control_plants_these_steps::RenameRefused;
/// Imports the plant a control arms,
/// together with the functions that arm and forget it.
#[cfg(test)]
pub(crate) use self::a_control_plants_these_steps::{
    Plant,
    clear_the_plant,
    plant_these_steps,
};
/// Imports the steps of a publish and the answers a control plants for them.
use self::a_control_plants_these_steps::{
    exchange_is_refused,
    reached_the_instant_before_the_aside_rename,
    reached_the_instant_before_the_name_is_given,
    reached_the_instant_between_the_two_renames,
    refused_or_none,
    the_exchanges_error,
};
/// Imports the exchange and the errors that mean the file system cannot do it.
use self::exchanging_the_two_names::{
    a_permission_denial,
    cannot_exchange,
    exchange_the_two_names,
};
/// Imports what a later install does with the leftovers of a killed one,
/// and the hidden name that holds the copy a kill moved aside.
pub(crate) use self::leftovers_of_a_killed_install::{
    AnOlderVersion,
    FinishedCopy,
    HoldingInstaller,
    Recovered,
    RemovedFinishedCopy,
    RemovedLeftover,
    WhyNotPublished,
    hidden_siblings,
    recover_leftovers,
};
/// Re-exports what a process's record answers,
/// and the reader a control uses to name a process of its own choosing.
#[cfg(test)]
pub(crate) use self::the_owner_of_a_hidden_sibling::{
    ForcedStart,
    ReaderView,
    moment_in,
    read_as,
    read_as_a_proc_that_hides_records,
    start_time_of_a_process,
};
/// Imports the owner of a hidden sibling,
/// and re-exports it and the two reads of a process's record for a caller that builds one of these names itself.
pub(crate) use self::the_owner_of_a_hidden_sibling::{
    Liveness,
    Owner,
};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `is_finished` from `super::published_archive`.
use super::published_archive::is_finished;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// The purpose in a hidden sibling's name that holds an install being unpacked.
pub(crate) const STAGING_PURPOSE: &str = "staging";

/// The purpose in a hidden sibling's name that holds the copy being replaced.
pub(crate) const REPLACED_PURPOSE: &str = "replaced";

/// The purpose in a hidden sibling's name that holds a finished copy an install claimed
/// before it published it,
/// so no other install publishes from the same directory.
pub(crate) const RECOVERED_PURPOSE: &str = "recovered";

/// `<parent>/.<name>.<purpose>.<process id>.<start time>.<namespace>.<boot>`,
/// a hidden sibling of `version_dir` that no version scan lists.
/// The last four fields name the installer that made it,
/// so a later install can tell one that no longer runs from one that does
/// (`the_owner_of_a_hidden_sibling`).
/// A name whose owner could not read its boot or namespace
/// falls back to the earlier shape that carries only the process id and its start time,
/// which no later install can judge.
pub(crate) fn hidden_sibling(
    version_dir: &Path,
    purpose: &str,
    owner: Owner,
) -> PathBuf {
    let version: String = file_name_text(version_dir);
    let name: String = match (owner.boot.as_deref(), owner.namespace) {
        (Some(boot), Some(namespace)) => {
            format!(
                ".{version}.{purpose}.{}.{}.{namespace}.{boot}",
                owner.pid, owner.started
            )
        }
        _ => format!(".{version}.{purpose}.{}.{}", owner.pid, owner.started),
    };
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
/// What giving a version's name a finished copy did.
/// Why:
/// The install report says whether the publish was this run's,
/// and a publish that found the version finished by another install dropped its own copy without publishing anything,
/// so the two are told apart rather than both called a publish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Published = 'here' | 'already-there';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Published {
    /// This call gave the version's name the finished copy it holds.
    Here,
    /// The version's name already held a finished copy,
    /// so this call's copy was dropped and nothing was published.
    AlreadyThere,
}

/// What:
/// The rename this module makes,
/// or the error a control planted in its place.
/// Why:
/// Every failure arm of a rename needs a control,
/// and no disposable fixture can make a rename fail on demand,
/// so this one place answers "the rename,
/// or what a control asked for".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theRename(from: string, to: string, what: RenameRefused): void  // throws
/// ```
fn the_rename(
    from: &Path,
    to: &Path,
    what: RenameRefused,
) -> io::Result<()> {
    if let Some(error) = refused_or_none(what) {
        return Err(error);
    }
    // What: `fs::rename` asks the kernel to make `to` name what `from` named, and to leave the two names as they were
    //  when it cannot (TS: `fs.renameSync`).
    // Why: it is the one step that changes a name, so a reader finds the old copy or the new one and never half of one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fs.renameSync(from, to);
    // ```
    return fs::rename(from, to);
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
/// A publish that fails keeps the finished copy it holds under `staging` and says where it is.
/// Why:
/// A copy that was working is never deleted before its replacement is in place,
/// and a reader of the version's name finds either no copy (before the first install)
/// or a whole finished one.
/// A marker alone is not enough to keep the old copy:
/// a person who deleted or made non-executable the program in a marked directory
/// would otherwise see every later install discard the good copy and report success.
/// A failed publish costs nothing either:
/// the finished copy is the only way an offline machine gets the version back,
/// so it stays under the name this run claimed it at,
/// where the next install publishes it without downloading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function publishStaged(staging: string, finalDir: string, firstBin: string): Published  // throws
/// ```
pub fn publish_staged(
    staging: &Path,
    final_dir: &Path,
    first_bin: &str,
) -> Result<Published, ToolchainError> {
    if is_finished(final_dir, first_bin) {
        discard_path(staging);
        return Ok(Published::AlreadyThere);
    }
    if fs::symlink_metadata(final_dir).is_err() {
        return move_the_finished_install(staging, final_dir, first_bin);
    }
    return replace_the_copy_that_is_there(staging, final_dir, first_bin);
}

/// What:
/// Gives a name nothing holds to the finished install,
/// in the one rename that this case needs.
/// A finished copy another install published in the instant since the check is kept,
/// and any other copy that install put there is replaced as if it had been there all along.
/// Why:
/// The version's name was absent when this call looked,
/// so no reader loses a copy here,
/// and a rename cannot leave the name half written.
/// A second install is what makes this a window:
/// it can publish,
/// or put a copy the version had back,
/// between this call's check and its rename,
/// and neither may cost the finished copy this call holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveTheFinishedInstall(staging: string, finalDir: string, firstBin: string): Published  // throws
/// ```
fn move_the_finished_install(
    staging: &Path,
    final_dir: &Path,
    first_bin: &str,
) -> Result<Published, ToolchainError> {
    reached_the_instant_before_the_name_is_given(staging, final_dir);
    if let Err(error) = the_rename(staging, final_dir, RenameRefused::GivesTheVersionItsName) {
        // Another run may have published this version in the instant since: the name is then finished, and nothing is
        // missing.
        if is_finished(final_dir, first_bin) {
            discard_path(staging);
            return Ok(Published::AlreadyThere);
        }
        // Another install put a copy there: this publish replaces it, as it replaces a copy that was there already.
        if fs::symlink_metadata(final_dir).is_ok() {
            return replace_the_copy_that_is_there(staging, final_dir, first_bin);
        }
        return Err(ToolchainError::failure(format!(
            "cannot move the finished install to {}: {error}; the finished copy this run holds is still at {}, and running the command again publishes it without downloading",
            final_dir.display(),
            staging.display()
        )));
    }
    return Ok(Published::Here);
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
/// A failure here changes neither name and keeps the finished copy this call holds,
/// so the next install publishes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceTheCopyThatIsThere(staging: string, finalDir: string, firstBin: string): Published  // throws
/// ```
fn replace_the_copy_that_is_there(
    staging: &Path,
    final_dir: &Path,
    first_bin: &str,
) -> Result<Published, ToolchainError> {
    if exchange_is_refused() {
        return two_renames(staging, final_dir, first_bin);
    }
    // What: the answer the exchange gives: what a control planted, or the one call that changes the two names.
    // Why: the arm that meets an error outside the "cannot exchange" list has to be reachable from a control.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const answer = theExchangesError() ?? attempt(() => exchangeTheTwoNames(staging, finalDir));
    // ```
    let answer: io::Result<()> = match the_exchanges_error() {
        Some(error) => Err(error),
        None => exchange_the_two_names(staging, final_dir),
    };
    match answer {
        Ok(()) => {
            // The exchange left the copy that was there at the staging name, where nothing reads it.
            discard_path(staging);
            return Ok(Published::Here);
        }
        Err(error) if cannot_exchange(&error) => {
            if a_permission_denial(&error) {
                // What: `eprintln!` writes one line to standard error, the channel this command's failures use
                //  (TS: `console.error`).
                // Why: a policy that refuses only the exchange would otherwise be silent, and the instant the two
                //  renames leave, in which the version's name holds nothing, would come back unannounced.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // console.error(`mpm-shadow: warning: ${error}: the exchange ... was refused, so ...`);
                // ```
                eprintln!(
                    "mpm-shadow: warning: {error}: changing {} and {} in one exchange was refused, so they are changed in two renames instead, and the instant between those renames, in which {} names nothing, is back; a file system that cannot exchange two names answers the same way, and so does a policy that refuses only that one call",
                    staging.display(),
                    final_dir.display(),
                    final_dir.display()
                );
            }
            return two_renames(staging, final_dir, first_bin);
        }
        Err(error) => {
            // The exchange changed neither name.
            if is_finished(final_dir, first_bin) {
                discard_path(staging);
                return Ok(Published::AlreadyThere);
            }
            return Err(ToolchainError::failure(format!(
                "cannot put the finished install in place of the copy at {}: {error}; that copy is as it was, the finished copy this run holds is still at {}, and running the command again publishes it without downloading",
                final_dir.display(),
                staging.display()
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
/// The instant between the two renames is the one window in which the version's name is absent,
/// and a second install can also publish the version since this call read the name,
/// which makes the copy this call moves aside a finished one.
/// A copy that a second install published in that instant is kept rather than replaced,
/// and the copy this call holds and the copy it moved aside are dropped;
/// a copy that a second install put back in it goes back rather than being replaced,
/// and the finished copy this call holds stays wherever it is when anything else fails,
/// so the next install publishes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function twoRenames(staging: string, finalDir: string, firstBin: string): Published  // throws
/// ```
fn two_renames(
    staging: &Path,
    final_dir: &Path,
    first_bin: &str,
) -> Result<Published, ToolchainError> {
    let aside: PathBuf = replaced_dir_of(final_dir);
    // A leftover of a killed run with the same owner would block the rename.
    discard_path(&aside);
    // What: the instant before whatever stands at the version's name is moved aside, which a control may fill.
    // Why: a second install can give the version its name in the instant since this call read it,
    //  and then the copy this rename moves aside is a finished one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // reachedTheInstantBeforeTheAsideRename(staging, finalDir);
    // ```
    reached_the_instant_before_the_aside_rename(staging, final_dir);
    if let Err(error) = the_rename(final_dir, &aside, RenameRefused::MovesTheCopyAside) {
        return Err(ToolchainError::failure(format!(
            "cannot move the copy at {} aside to install over it: {error}; that copy is as it was, the finished copy this run holds is still at {}, and running the command again publishes it without downloading",
            final_dir.display(),
            staging.display()
        )));
    }
    // What: whether the copy this rename just moved aside is a finished install, which says another install published
    //  the version in the instant between this call's check and this rename.
    // Why: a finished copy is never replaced here: it goes back and this call's copy is dropped, the same answer as
    //  when the version's name was finished before this call started.
    if is_finished(&aside, first_bin) {
        if let Err(error) = the_rename(&aside, final_dir, RenameRefused::PutsTheCopyBack) {
            return Err(ToolchainError::failure(format!(
                "cannot put the finished install that was at {} back from {}: {error}; the finished copy this run holds is still at {}, and running the command again publishes it without downloading",
                final_dir.display(),
                aside.display(),
                staging.display()
            )));
        }
        discard_path(staging);
        return Ok(Published::AlreadyThere);
    }
    reached_the_instant_between_the_two_renames(staging, final_dir);
    if let Err(error) = the_rename(staging, final_dir, RenameRefused::GivesTheVersionItsName) {
        // What: whether another install published the version in the instant between the two renames, which makes both
        //  this rename and the put-back below fail.
        // Why: the version is then finished by that install, so this call has nothing to put in place and nothing to
        //  restore, and the copies it holds are the leftovers a later install would only remove.
        if is_finished(final_dir, first_bin) {
            discard_path(staging);
            discard_path(&aside);
            return Ok(Published::AlreadyThere);
        }
        if let Err(restore_error) = the_rename(&aside, final_dir, RenameRefused::PutsTheCopyBack) {
            return Err(ToolchainError::failure(format!(
                "cannot move the finished install to {}: {error}; the copy that was there could not be put back from {}: {restore_error}; the finished copy this run holds is still at {}",
                final_dir.display(),
                aside.display(),
                staging.display()
            )));
        }
        return Err(ToolchainError::failure(format!(
            "cannot move the finished install to {}: {error}; the copy that was there is back at {}, the finished copy this run holds is still at {}, and running the command again publishes it without downloading",
            final_dir.display(),
            final_dir.display(),
            staging.display()
        )));
    }
    discard_path(&aside);
    return Ok(Published::Here);
}
