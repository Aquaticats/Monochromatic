//! What:
//!  Which commit scratch trees a later command removes,
//!  and which it leaves alone because the command that owns them still runs.
//! Why:
//!  The wrapper never removes what it did not create,
//!  and it must not remove a tree a live command still uses:
//!  a commit can hold its tree for as long as a person leaves an editor open in the scratch repository,
//!  so age alone proves nothing about ownership.
//!  Each command writes a marker into its own tree naming its process and that process's birth identity,
//!  the same identity a stale-lock reclaim reads
//!  (`service_index_marker.rs`),
//!  and a later command removes a tree only when that marker names a process that is gone.
//!  A tree with no marker is one a release before this one left,
//!  or a command killed between making the tree and writing the marker;
//!  it goes by age,
//!  and the sweep of the repository scratch directory goes by age as well
//!  (`scratch_location.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! markTheOwner(tree); const removed = removeAbandonedTrees(join(commonDir, 'monochromatic-git-policy-cli-scratch'));
//! ```

/// The marker,
/// who it names,
/// and whether that process is gone.
use crate::service_index_marker::{is_reclaimable, marker_text};
use crate::service_index_identity::own_identity;
/// The bounded removal of one entry.
use crate::scratch_entry_removal::{Removal, SCRATCH_REMOVAL_CEILING, remove_within};
/// The prefix of a commit's scratch tree,
/// and what an identity in a name may hold.
use crate::wrapper_names_old_and_new::{SERVICE_SCRATCH_PREFIX, is_identity_byte};
/// Debug lines for what a later command leaves alone.
use crate::diagnostic_log::debug;
/// A path and a moment.
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The tag of this module's diagnostics.
const TAG: &str = "monochromatic-git-policy-cli";

/// The file inside a commit's scratch tree that names the process holding it.
pub const OWNER_MARKER: &str = "owner";

/// How old a commit scratch tree must be before a later command removes it without a marker that names a gone process.
///  Measured:
/// the longest whole commit in the commit timing log of 2026-11-03 was 750.6 s,
///  and `ANSWER_CEILING`
///  (`wait_ceilings.rs`)
///  ends every wait of a commit at 20 minutes;
///  a tree whose command still lives is renewed every `SCRATCH_REFRESH_EVERY` and carries a marker,
///  so only a leftover none of those rules reaches waits this long.
///  Two hours is six times that ceiling.
pub const UNMARKED_SCRATCH_TREE_AGE: Duration = Duration::from_secs(2 * 60 * 60);

/// The most commit scratch trees one command removes,
/// so collecting them never becomes the work the command is doing.
///  Measured:
/// a commit's scratch tree of 2026-11-09 held 9,081,951 bytes in three files on this host,
///  so eight of them are about 73 MiB,
///  and the next command removes the next eight.
pub const SCRATCH_TREES_REMOVED_PER_COMMAND: usize = 8;

/// What:
///  Writes the marker that names this process as the owner of `tree`.
/// Why:
///   A later command decides whether to remove a tree from this marker,
///  so a command that leaves no marker leaves a tree no one may remove before it is old.
///  A marker that cannot be written,
///  or a `/proc` that cannot identify this process,
///  is a debug line and never a failure of the commit:
///  the tree then waits for the age rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeOwnerMarker(tree);
/// ```
pub fn mark_the_owner(tree: &Path) {
    let marker: Result<String, crate::process_identity::ProcessIdentityError> = marker_text(0, &own_identity);
    let written: std::io::Result<()> = match marker {
        Ok(text) => crate::private_storage::write_private_file_unsynced(tree.join(OWNER_MARKER).as_path(), text.as_bytes()),
        Err(error) => {
            debug(TAG, format!("the scratch tree {} is not marked as this command's: {error}", tree.display()).as_str());
            return;
        }
    };
    if let Err(error) = written {
        debug(TAG, format!("the scratch tree {} is not marked as this command's: {error}", tree.display()).as_str());
    }
}

/// Whether an entry is old enough to go without a marker that names a gone owner.
fn older_than_the_unmarked_age(age: Duration) -> bool {
    return age > UNMARKED_SCRATCH_TREE_AGE;
}

/// What:
///  Whether the entry at `path`, listed with `metadata`,
///  may be removed now:
///  the one decision the sweep of a scratch place and the collection a preparation does both make.
/// Why:
///   Two rules that disagree undo each other:
///  a sweep that removed a tree by its age alone took the private index and the typed message of a command that still runs,
///  because age says nothing about the command that made the tree.
///  A commit tree therefore goes only when its marker names a process `/proc` proves is gone,
///  or, with no marker at all, once it is older than `UNMARKED_SCRATCH_TREE_AGE`;
///  a live command's tree stays however old it is,
///  and one whose marker the reader cannot judge stays too,
///  which keeps a live command's work rather than taking it.
///  Every other entry of a scratch place is the wrapper's own and goes by the age the caller gives it,
///  because nothing else ever collects it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mayGo(entry, metadata, now, day) // true when nothing may still read it
/// ```
pub fn may_go(path: &Path, metadata: &std::fs::Metadata, now: SystemTime, max_age: Duration) -> bool {
    let name: Option<&str> = path.file_name().and_then(std::ffi::OsStr::to_str);
    let named_a_tree: bool = name
        .and_then(|text| { return text.strip_prefix(SERVICE_SCRATCH_PREFIX); })
        .is_some_and(|identity| { return !identity.is_empty() && identity.as_bytes().iter().all(is_identity_byte); })
        && metadata.is_dir();
    // An entry whose age cannot be told is never old enough to remove, and one dated in the future is kept: a wrong clock must
    //  not delete live files.
    let age: Option<Duration> = metadata.modified().ok().and_then(|modified| { return now.duration_since(modified).ok(); });
    if !named_a_tree {
        return age.is_some_and(|found| { return found > max_age; });
    }
    let marker: PathBuf = path.join(OWNER_MARKER);
    if is_reclaimable(marker.as_path()) {
        return true;
    }
    // A marker this reader can parse names a process, and only `/proc` may then decide: age never does, because a commit can hold
    //  a tree while a person leaves an editor open in it.
    let marked: bool = marker.try_exists().unwrap_or(true);
    if marked {
        return false;
    }
    return age.is_some_and(older_than_the_unmarked_age);
}

/// What:
///  Removes the commit scratch trees of `directory` that belong to no live command,
///  oldest first,
///  at most `SCRATCH_TREES_REMOVED_PER_COMMAND` of them,
///  and returns how many are gone.
/// Why:
///   A tree whose marker names a process that is gone belongs to a command that was killed,
///  and nothing would ever collect it before the sweep's day.
///  A tree a live command holds is left alone however old it is,
///  and so is one whose marker names a process the reader cannot judge,
///  which keeps it rather than taking a live command's work.
///  `may_go` is the same decision the sweep of the place makes,
///  so the two cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = removeAbandonedTrees(directory);
/// ```
pub fn remove_abandoned_trees(directory: &Path) -> usize {
    let now: SystemTime = SystemTime::now();
    let mut candidates: Vec<(SystemTime, PathBuf)> = Vec::new();
    let entries: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(error) => {
            debug(TAG, format!("could not list {} for the scratch trees it holds: {error}", directory.display()).as_str());
            return 0;
        }
    };
    for listed in entries {
        let Ok(entry) = listed else { continue };
        let name: std::ffi::OsString = entry.file_name();
        let Some(text) = name.to_str() else { continue };
        let Some(identity) = text.strip_prefix(SERVICE_SCRATCH_PREFIX) else { continue };
        if identity.is_empty() || !identity.as_bytes().iter().all(is_identity_byte) {
            continue;
        }
        let path: PathBuf = entry.path();
        let Ok(metadata) = std::fs::symlink_metadata(&path) else { continue };
        // A name that holds a link or a file is not a tree this wrapper made, and is left alone.
        if !metadata.is_dir() {
            continue;
        }
        if !may_go(path.as_path(), &metadata, now, UNMARKED_SCRATCH_TREE_AGE) {
            continue;
        }
        let Ok(modified) = metadata.modified() else { continue };
        candidates.push((modified, path));
    }
    candidates.sort_by(oldest_first);
    let mut removed: usize = 0;
    for (_, path) in candidates.iter().take(SCRATCH_TREES_REMOVED_PER_COMMAND) {
        let removal: Removal = remove_within(path.as_path(), SCRATCH_REMOVAL_CEILING);
        if removal.finished() {
            removed += 1;
        }
    }
    return removed;
}

/// The order two candidates are removed in:
/// the older one first.
fn oldest_first(left: &(SystemTime, PathBuf), right: &(SystemTime, PathBuf)) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// The marker and the rule are controlled in the test module.
#[cfg(test)]
#[path = "service_scratch_abandonment_tests.rs"]
mod tests;
