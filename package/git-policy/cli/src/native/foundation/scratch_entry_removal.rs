//! What:
//!  How one scratch entry is removed,
//!  bounded so a device that stalls cannot hold a command,
//!  and which commit scratch trees a later command takes for abandoned.
//! Why:
//!  A recursive removal cannot be interrupted once it starts,
//!  and the commit timing record holds one `unlinkat` of a shaped tree at 2.5 s
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
//!  A command that waits for one therefore waits for the device,
//!  which the owner's "commits vs native git should not take 5s longer" rules against,
//!  so every removal here runs on a thread of its own,
//!  stops being waited for at `SCRATCH_REMOVAL_CEILING`,
//!  and what it spent and what became of it go to the command's timing record.
//!  The same ceiling is what a later command uses to collect the trees of commands that no service held.
//!  What a name may be before it is removed is here too:
//!  the wrapper's own prefix and an identity of letters,
//!  digits and dashes,
//!  and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removal = removeWithin(path, 30_000); // 'finished' | 'unfinished' | { failed }
//! ```

/// Debug lines for the removals that do not finish.
use crate::diagnostic_log::debug;
/// The recursive removal of a directory tree.
use crate::private_storage::remove_tree;
/// The two costs a command pays outside its phases,
/// one of which is removing a scratch tree.
use crate::time_spent_outside_the_phases::note_scratch_removal;
/// The name prefix of a commit's scratch tree,
/// and what an identity may hold.
use crate::wrapper_names_old_and_new::{SERVICE_SCRATCH_PREFIX, is_identity_byte};
/// A path,
/// a span of time and a moment.
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// The tag of this module's diagnostics.
const TAG: &str = "monochromatic-git-policy-cli";

/// The ceiling on removing one scratch entry,
/// which is where a recursive removal could otherwise wait for as long as the device stalls.
///  Measured:
/// the commit timing record of 2026-11-09 holds one `unlinkat` of a scratch tree at 2.5 s in a shaped fixture
///  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
/// "What the steps wait on"),
///  and a shaped tree measured on this host on 2026-11-09 holds 9,081,951 bytes in a handful of entries,
///  whose whole removal took 34 `unlinkat` calls and 1.5 ms of system time under `strace`.
///  Thirty seconds is twelve times the slowest single call the record holds,
///  and a removal that reaches it is reported rather than waited for.
///  It is never a limit on a removal that has to finish before the person can go on:
///  the commit's own scratch tree is handed to the service or left to the sweep instead.
pub const SCRATCH_REMOVAL_CEILING: Duration = Duration::from_secs(30);

/// How old a commit scratch tree must be before a later command removes it whatever the sweep's day says,
/// so a machine that commits while the service is away cannot pile trees up for a whole day.
///  Measured:
/// the longest whole commit in the commit timing log of 2026-11-03 was 750.6 s,
///  and `ANSWER_CEILING`
///  (`wait_ceilings.rs`)
///  ends every wait of a commit at 20 minutes,
///  so no live command owns a tree this old.
///  Two hours is six times that ceiling,
///  and a commit that waits renews its tree's time every `SCRATCH_REFRESH_EVERY`,
///  which is shorter still.
pub const COMMIT_SCRATCH_TREE_ABANDONED_AFTER: Duration = Duration::from_secs(2 * 60 * 60);

/// The most abandoned commit scratch trees one command removes,
/// so removing them never becomes the work the command is doing.
///  Measured:
/// a commit's scratch tree of 2026-11-09 held 9,081,951 bytes in three files on this host,
///  so eight of them are about 73 MiB,
///  and the next command removes the next eight.
pub const COMMIT_SCRATCH_TREES_REMOVED_PER_COMMAND: usize = 8;

/// What a removal of one scratch entry came to.
#[derive(Debug)]
pub enum Removal {
    /// The entry is gone.
    Finished,
    /// The removal was still running when its ceiling passed,
    /// and the entry is partly or wholly there;
    /// the next sweep tries again.
    Unfinished,
    /// The removal was refused,
    /// with the refusal that says why.
    Failed(std::io::Error),
}

impl Removal {
    /// Whether the entry is gone.
    pub fn finished(&self) -> bool {
        return matches!(self, Self::Finished);
    }
}

/// What:
///  Removes one scratch entry,
///  a directory with its contents or a single name,
///  and returns what became of it at `ceiling`.
/// Why:
///   A recursive removal cannot be interrupted once it starts,
///  so it runs on a thread of its own and the caller stops waiting at the ceiling.
///  A thread still working when the process ends takes its part of the removal with it,
///  which leaves a partly removed entry for the next sweep,
///  never a wrong one.
///  The time and the outcome go to the command's timing record,
///  so a machine where removals stall says so where a reading looks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removal = removeWithin(path, ceiling); // 'finished' | 'unfinished' | { failed }
/// ```
pub fn remove_within(path: &Path, ceiling: Duration) -> Removal {
    let began: Instant = Instant::now();
    let target: PathBuf = path.to_path_buf();
    // What the thread ended with; a thread that outlives the ceiling sends into a channel nobody reads, which is no error.
    let (ended, waited) = std::sync::mpsc::channel::<std::io::Result<()>>();
    let started = std::thread::Builder::new().name(String::from("scratch-removal")).spawn(move || {
        let _sent = ended.send(remove_entry(&target));
    });
    let removal: Removal = match started {
        Err(error) => Removal::Failed(error),
        Ok(_thread) => match waited.recv_timeout(ceiling) {
            Ok(Ok(())) => Removal::Finished,
            Ok(Err(error)) => Removal::Failed(error),
            Err(_late) => Removal::Unfinished,
        },
    };
    let took: Duration = began.elapsed();
    if let Removal::Failed(error) = &removal {
        debug(TAG, format!("could not remove the scratch entry {}: {error}", path.display()).as_str());
    }
    if let Removal::Unfinished = &removal {
        debug(TAG, format!("the scratch entry {} was still being removed after {ceiling:?}", path.display()).as_str());
    }
    note_scratch_removal(took, removal.finished());
    return removal;
}

/// One scratch entry removed:
/// a real directory with its contents,
/// or anything else as the one name it is,
/// so a link is removed rather than followed.
fn remove_entry(path: &Path) -> std::io::Result<()> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    return if metadata.is_dir() { remove_tree(path) } else { std::fs::remove_file(path) };
}

/// The order two scratch entries are removed in:
/// the older one first,
/// so a machine that keeps making them loses the oldest.
fn oldest_first(left: &(SystemTime, PathBuf), right: &(SystemTime, PathBuf)) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// What:
///  Removes the commit scratch trees of `directory` that are older than `abandoned_after`,
///  oldest first,
///  at most `at_most` of them,
///  and returns how many are gone.
/// Why:
///   The sweep removes a scratch tree once it is a day old,
///  which bounds how long one lives and not how many a machine can make:
///  a machine that commits while the service is away makes one tree per commit and nothing collects them before their age.
///  A tree older than the longest a commit can live belongs to no live command,
///  so removing it takes nothing from anyone,
///  and the age is what bounds the pile.
///  Every name that is not a commit scratch tree's is left alone,
///  and so is a name that holds a link or a file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = removeAbandonedCommitScratchTrees(directory, ABANDONED_AFTER, 8);
/// ```
pub fn remove_abandoned_commit_scratch_trees(directory: &Path, abandoned_after: Duration, at_most: usize) -> usize {
    let now: SystemTime = SystemTime::now();
    let mut abandoned: Vec<(SystemTime, PathBuf)> = Vec::new();
    let entries: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(error) => {
            debug(TAG, format!("could not list {} for the commit scratch trees it holds: {error}", directory.display()).as_str());
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
        // An entry whose age cannot be told is never old enough to be abandoned.
        let Ok(modified) = metadata.modified() else { continue };
        let Ok(age) = now.duration_since(modified) else { continue };
        if age > abandoned_after {
            abandoned.push((modified, path));
        }
    }
    abandoned.sort_by(oldest_first);
    let mut removed: usize = 0;
    for (_, path) in abandoned.iter().take(at_most) {
        if remove_within(path.as_path(), SCRATCH_REMOVAL_CEILING).finished() {
            removed += 1;
        }
    }
    return removed;
}

/// The removal and the abandonment rule are controlled in the test module.
#[cfg(test)]
#[path = "scratch_entry_removal_tests.rs"]
mod tests;
