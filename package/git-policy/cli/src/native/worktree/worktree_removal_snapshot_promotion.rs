//! What:
//!  Turn what a removal snapshot kept into notices any worktree shows:
//! its notices and the record of each unfinished push.
//! Why:
//!  A worktree Git removed took its Git directory with it,
//! so what a snapshot holds is promoted to the directory whose notices any
//!  worktree shows,
//! and each item is removed from the snapshot as soon as it is promoted,
//! so a snapshot kept after a
//!  failure promotes only what is left on the next try and a notice already shown is not made again.
//! A failure is
//!  reported to the person and the snapshot stays,
//! because the snapshot is then the last copy of that record.
//! The promotion
//!  is its own file because the settle of every snapshot and the finish of an interrupted destroy both use it,
//! and
//!  neither may depend on the other's file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const complete = await promote(commonDir, snapshot, problems);
//! promoteSnapshot(commonDir, snapshot, problems);
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::private_storage::create_private_directory_all;
use crate::push_failure_notice::{ORPHAN_NOTICE_DIRECTORY, record_lost_push};
use crate::work_item::WorkItem;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalSnapshotPromotion";

/// Directory in a snapshot that holds copies of the notices.
pub(crate) const NOTICES: &str = "notices";

/// Directory in a snapshot that holds copies of the unfinished push records,
/// named `<state>-<record>`.
pub(crate) const WORK: &str = "work";

/// The files of a directory,
/// or none when it cannot be read.
pub(crate) fn files_of(directory: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(directory) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();
    return paths;
}

/// The name of a file as text.
pub(crate) fn name_of(path: &Path) -> String {
    return match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::new(),
    };
}

/// Moves one saved notice to the notices any worktree shows;
/// a notice another command already moved is not a failure.
pub(crate) fn promote_notice(orphans: &Path, path: &Path) -> std::io::Result<()> {
    create_private_directory_all(orphans)?;
    return match std::fs::rename(path, orphans.join(name_of(path))) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
        Ok(()) | Err(_) => Ok(()),
    };
}

/// What:
///  Turns what a snapshot of a removed worktree kept into notices any worktree shows,
/// and removes what it promoted;
/// whether
///  everything was promoted.
/// Why:
///   Each item is removed from the snapshot as soon as it is promoted,
/// so a snapshot kept after a failure promotes only what is
///  left on the next try and a notice already shown is not made again.
/// A failure is reported to the person and the snapshot
///  stays,
/// because the snapshot is then the last copy of that record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const complete = await promote(commonDir, snapshot, problems)
/// ```
pub(crate) fn promote(common_dir: &Path, snapshot: &Path, problems: &mut Vec<String>) -> bool {
    let orphans: PathBuf = common_dir.join(ORPHAN_NOTICE_DIRECTORY);
    let mut complete: bool = true;
    for path in files_of(snapshot.join(NOTICES).as_path()) {
        if let Err(error) = promote_notice(orphans.as_path(), path.as_path()) {
            complete = false;
            let said: String = format!(
                "The push failure notice {} of a removed worktree could not be kept for you ({error}). It stays in {}; once that is fixed, any git command shows it.",
                name_of(path.as_path()),
                snapshot.display()
            );
            debug(TAG, said.as_str());
            problems.push(said);
        }
    }
    for path in files_of(snapshot.join(WORK).as_path()) {
        let text: String = match std::fs::read_to_string(path.as_path()) {
            Ok(read) => read,
            // Another command promoted it first.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                complete = false;
                let said: String = format!("The saved record {} of a removed worktree's unfinished push could not be read ({error}); it stays in {}.", name_of(path.as_path()), snapshot.display());
                debug(TAG, said.as_str());
                problems.push(said);
                continue;
            }
        };
        match WorkItem::decode(text.as_str()) {
            Ok(found) => match record_lost_push(common_dir, found.branch_ref.as_str(), found.oid.as_str()) {
                Ok(()) => {
                    // Removed now, so that a try after a failure elsewhere in this snapshot does not record the push again.
                    if let Err(error) = std::fs::remove_file(path.as_path()) {
                        warn(TAG, format!("removing the promoted record {} failed: {error}", path.display()).as_str());
                    }
                }
                Err(reason) => {
                    complete = false;
                    let said: String = format!(
                        "The backup push of {} for commit {} had not finished when its worktree was removed, and noting that failed ({reason}). \
                         The record stays in {}; once that is fixed, any git command shows it.",
                        found.branch_ref,
                        found.oid,
                        snapshot.display()
                    );
                    debug(TAG, said.as_str());
                    problems.push(said);
                }
            },
            Err(reason) => {
                // A record that cannot be read now cannot be read later: it is said once and dropped.
                let said: String = format!("A saved record of an unfinished push of a removed worktree is unreadable ({reason}), so that push is not reported: {}", path.display());
                debug(TAG, said.as_str());
                problems.push(said);
                let _dropped = std::fs::remove_file(path.as_path());
            }
        }
    }
    return complete;
}

/// What:
///  Promote what one snapshot kept and remove it when everything went through.
/// Why:
///   Two callers settle one snapshot:
/// the settle of every snapshot,
/// and the finish of an interrupted destroy,
/// which
///  promotes the snapshot it just took before that registration goes.
/// A failure keeps the snapshot,
/// which is then the last copy of that record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// promoteSnapshot(commonDir, snapshot, problems)
/// ```
pub(crate) fn promote_snapshot(common_dir: &Path, snapshot: &Path, problems: &mut Vec<String>) {
    if promote(common_dir, snapshot, problems) {
        let _settled = std::fs::remove_dir_all(snapshot);
    }
}
