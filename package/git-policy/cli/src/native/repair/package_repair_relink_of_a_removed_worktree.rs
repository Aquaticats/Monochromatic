//! What:
//!  Ending the commit service's relinking of a worktree that is being removed:
//!  the record of a relink that had not started,
//!  and the tool of one that had.
//! Why:
//!  `git worktree add` hands the relinking of the new worktree's copied package state to the
//!  repository's commit service (`package_repair_relink.rs`),
//!  and the person may remove the worktree before that work is done:
//!  a fresh worktree a removal is asked for is exactly the state this wrapper is used in.
//!  A record waiting in that queue names a worktree that no longer exists,
//!  so nothing of it is left to relink and it is settled here rather than claimed later;
//!  the tool of a record the service already started ends itself when the directory it runs in is gone
//!  (`package_repair_command.rs`,
//!  `WORKTREE_WATCH_INTERVAL`),
//!  and its record settles in the service as "the worktree was removed"
//!  (`package_repair_relink_job.rs`),
//!  so no relink outlives the worktree it serves and no record is left to fail with an error nobody can act on.
//!  The removal waits for none of this:
//!  the record is one queue entry and the end of a tool is the tool's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const lines = cancelTheRelinkOf(commonDir, worktree); // settled records, in words for the person
//! ```

use crate::diagnostic_log::debug;
use crate::service_work::work_directory;
use crate::work_item::{DONE, PENDING, WorkItem, WorkKind, state_directory};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairRelinkOfARemovedWorktree";

/// The prefix every line a person reads from this program starts with.
const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// What:
///  Settle every relink record that names a worktree this removal took,
///  and answer the lines the person reads.
/// Why:
///   A record for a worktree that is gone is work nothing can finish usefully:
///  the add that asked for it is over,
///  and the tools it names rewrite link layouts inside a directory that no longer exists.
///  Only the records nobody holds are settled here:
///  a claim is the service's own,
///  and the run it describes stops itself and settles with its own words.
///  Settling a record is one atomic move inside the queue,
///  so a service claiming it at the same instant settles it too and nothing is lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines: string[] = cancelTheRelinkOf(commonDir, worktree);
/// ```
pub fn cancel_the_relink_of(common_dir: &Path, worktree: &Path) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let work: PathBuf = work_directory(common_dir);
    let pending: PathBuf = state_directory(work.as_path(), PENDING);
    let Ok(listing) = std::fs::read_dir(pending.as_path()) else { return lines };
    for entry in listing.flatten() {
        let name: std::ffi::OsString = entry.file_name();
        let text: String = match std::fs::read_to_string(entry.path()) {
            Ok(read) => read,
            Err(_unreadable) => continue,
        };
        let item: WorkItem = match WorkItem::decode(text.trim_end()) {
            Ok(decoded) => decoded,
            Err(_unreadable) => continue,
        };
        if item.kind != WorkKind::Relink || item.repository != worktree {
            continue;
        }
        match std::fs::rename(entry.path(), state_directory(work.as_path(), DONE).join(&name)) {
            Ok(()) => {
                debug(TAG, format!("the relink of {} had not started and its worktree is gone, so its record is settled", worktree.display()).as_str());
                lines.push(format!(
                    "{PREFIX}the commit service had not started relinking the package state of {}, and this removal took the worktree, so its record is settled and nothing there is left to relink.",
                    worktree.display()
                ));
            }
            // A record the service claimed between the listing and this move is the service's own, and its run stops itself.
            Err(_taken) => continue,
        }
    }
    return lines;
}

/// What:
///  Write the lines of `cancel_the_relink_of` to standard error,
///  and keep nothing else.
/// Why:
///   The removal the person asked for is the command that reads them,
///  and a closed terminal is not a reason to stop the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const line of cancelTheRelinkOf(commonDir, worktree)) stderr.write(line);
/// ```
pub fn say_what_was_cancelled(common_dir: &Path, worktree: &Path) {
    use std::io::Write as _;
    for line in cancel_the_relink_of(common_dir, worktree) {
        let _closed = writeln!(std::io::stderr(), "{line}");
    }
}

/// Controls of this module stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_relink_of_a_removed_worktree_tests.rs"]
mod tests;
