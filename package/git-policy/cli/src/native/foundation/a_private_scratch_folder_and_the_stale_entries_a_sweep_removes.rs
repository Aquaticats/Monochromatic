//! What:
//!  What any scratch folder of this wrapper must be and what it sheds:
//!  the check that a folder is this account's private directory,
//!  how old an entry must be before a preparation removes it,
//!  and the sweep that removes the stale entries of one folder,
//!  oldest first,
//!  each within a ceiling.
//! Why:
//!  `scratch_location.rs` says where the two scratch folders are and prepares them;
//!  what a folder must be before anything of this account is written in it,
//!  and what a later command collects out of it,
//!  are the same for the per-user folder and the repository one,
//! so they are here and both callers read one rule.
//!  The sweep runs on every preparation,
//!  so what a killed process left does not accumulate without bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const s = await lstat(path); // throws unless a directory of this account, mode 0700
//! const stale = (await readdir(dir)).filter(mayGo).sort(byAge).slice(0, 8);
//! for (const entry of stale) await removeWithin(entry, ceiling);
//! ```

/// Debug lines for problems that never fail a command.
use crate::diagnostic_log::debug;
/// The mode a private directory ends with,
/// and the call that sets it,
/// which only Unix has.
#[cfg(unix)]
use crate::private_storage::{
    PRIVATE_DIRECTORY_MODE,
    protect_path,
};
/// What one removal answered,
/// and the call that removes one entry within the ceiling the caller gives it.
use crate::scratch_entry_removal::{
    Removal,
    remove_within,
};
/// The record of which paths a preparation proved private,
/// which the check below notes in a test build.
#[cfg(test)]
use crate::scratch_location::test_proofs;
/// The folder's path and the path of one entry in it,
/// and the clock their ages are measured on.
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    SystemTime,
};

/// The tag of this module's diagnostics.
pub(crate) const TAG: &str = "monochromatic-git-policy-cli";

/// How old an entry must be before a preparation removes it:
///  far longer than any command of the wrapper runs,
/// so a live process never loses a file.
/// Measured:
/// the longest command in the
///  commit timing log (1,051 commits,
/// 2026-11-02T09:01:37Z to 2026-11-03T15:10:12Z) took 750.6 s and the longest worktree add in the
///  add timing log (48 adds) 539.7 s;
/// a day is about a hundred times the longest.
/// The patience for an owner of another namespace
///  (`FOREIGN_OWNER_PATIENCE`,
/// twice `MERGE_LOCK_CEILING`) is far under a day.
/// A wait for a live program's lock ends at a ceiling:
/// one minute (`LANDING_LOCK_CEILING`,
/// `MERGE_LOCK_CEILING`)
///  or ten for the service's own hold (`SERVICE_HOLD_CEILING`,
/// `wait_ceilings.rs`),
/// so no command that waits on one comes near this age.
///  Kept as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const SCRATCH_STALE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

/// What:
///  Accept `path` only if it is a real directory (not a link) of this user that no one else can enter,
///  tightening a looser mode of a directory this user owns.
/// Why:
///   Another account that could write into the directory could plant a link or read the files,
///  and a link in place of the directory could redirect every write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const s = await lstat(path); if (!s.isDirectory() || s.uid !== process.geteuid()) throw ...; if (s.mode & 0o077) await chmod(path, 0o700);
/// ```
#[cfg(unix)]
pub(crate) fn assert_private_directory(path: &Path) -> std::io::Result<()> {
    // `MetadataExt` gives the owner and the mode bits.
    use std::os::unix::fs::MetadataExt;
    // A control reads which paths were proved and which were never asked about
    //  (`scratch_location_privacy_proofs_test_support.rs`).
    #[cfg(test)]
    test_proofs::note(path);
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        return Err(std::io::Error::other(
            "it is a link or a file, not a directory",
        ));
    }
    // SAFETY: `geteuid` has no preconditions.
    let owner: u32 = unsafe { libc::geteuid() };
    if metadata.uid() != owner {
        return Err(std::io::Error::other("it belongs to another account"));
    }
    if metadata.mode() & 0o077 != 0 {
        protect_path(path, PRIVATE_DIRECTORY_MODE)?;
    }
    return Ok(());
}

/// What:
///  Accept `path` only if it is a real directory.
/// Why:
///   Ownership and modes are Unix notions;
///  other platforms keep their inherited access rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!(await lstat(path)).isDirectory()) throw ...;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
pub(crate) fn assert_private_directory(path: &Path) -> std::io::Result<()> {
    if !std::fs::symlink_metadata(path)?.is_dir() {
        return Err(std::io::Error::other(
            "it is a link or a file, not a directory",
        ));
    }
    return Ok(());
}

/// The order two stale entries are removed in:
/// the older one first,
/// so a machine that keeps making them loses the oldest.
fn older_first(
    left: &(SystemTime, PathBuf),
    right: &(SystemTime, PathBuf),
) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// What:
///  Remove every entry of `directory` last modified more than `max_age` before `now`,
///  within `ceiling` for each entry,
///  and return how many it removed.
///  A link is removed itself and never followed;
///  a directory is removed with its contents.
/// Why:
///   A process killed between making a scratch file and removing it leaves the file behind,
///  and nothing else would ever collect it.
///  The directory belongs to the wrapper alone,
///  so every entry in it is the wrapper's;
///  an entry modified in the future is kept,
///  because a wrong clock must not delete live files.
///  Each removal goes through `remove_within`,
///  so an entry whose deletion stalls costs the caller `ceiling` and no more,
///  and what did not finish stays for a later sweep.
///  Which entry may go is `service_scratch_abandonment::may_go`'s to decide,
///  the same decision the collection a preparation does makes,
///  so a tree whose owner still runs is kept however old it is.
///  At most `SCRATCH_TREES_REMOVED_PER_COMMAND` entries go per call,
///  oldest first,
///  so a backlog left by killed commands delays one command by that many removals and never by its whole length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stale = (await readdir(dir)).filter(mayGo).sort(byAge).slice(0, 8);
/// for (const entry of stale) await removeWithin(entry, ceiling);
/// ```
pub fn sweep_stale(
    directory: &Path,
    now: SystemTime,
    max_age: Duration,
    ceiling: Duration,
) -> usize {
    let entries: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(error) => {
            debug(
                TAG,
                format!(
                    "could not list {} for stale entries: {error}",
                    directory.display()
                )
                .as_str(),
            );
            return 0;
        }
    };
    // What may go, in the order it goes: the oldest first, so the entries a machine keeps making are the ones that survive.
    let mut stale: Vec<(SystemTime, PathBuf)> = Vec::new();
    for listed in entries {
        let entry: std::fs::DirEntry = match listed {
            Ok(found) => found,
            Err(error) => {
                debug(
                    TAG,
                    format!(
                        "could not read an entry of {}: {error}",
                        directory.display()
                    )
                    .as_str(),
                );
                continue;
            }
        };
        let path: PathBuf = entry.path();
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(&path) {
            Ok(found) => found,
            Err(error) => {
                debug(
                    TAG,
                    format!("could not inspect {}: {error}", path.display()).as_str(),
                );
                continue;
            }
        };
        // One decision, shared with the collection a preparation does: a commit tree stays while the command that made it runs,
        //  however old it is, and every other entry of the place goes by its age.
        if !crate::service_scratch_abandonment::may_go(path.as_path(), &metadata, now, max_age) {
            continue;
        }
        stale.push((metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH), path));
    }
    stale.sort_by(older_first);
    let mut removed: usize = 0;
    for (_, path) in stale
        .iter()
        .take(crate::service_scratch_abandonment::SCRATCH_TREES_REMOVED_PER_COMMAND)
    {
        // The removal is bounded like every other one: a device that stalls costs this call the ceiling and leaves the entry to the
        //  next sweep, instead of holding the command that asked for scratch for as long as the device takes.
        match remove_within(path.as_path(), ceiling) {
            Removal::Finished => removed += 1,
            Removal::Unfinished => debug(
                TAG,
                format!(
                    "the stale {} was still being removed after {ceiling:?}",
                    path.display()
                )
                .as_str(),
            ),
            Removal::Failed(error) => debug(
                TAG,
                format!("could not remove the stale {}: {error}", path.display()).as_str(),
            ),
        }
    }
    return removed;
}
