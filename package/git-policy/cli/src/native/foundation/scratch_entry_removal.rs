//! What:
//!  How one scratch entry is removed,
//!  bounded so a device that stalls cannot hold a command.
//! Why:
//!  A recursive removal cannot be interrupted once it starts,
//!  and the commit timing record holds one `unlinkat` of a shaped tree at 2.5 s
//!  (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
//!  A command that waits for one therefore waits for the device,
//!  which the owner's "commits vs native git should not take 5s longer" rules against,
//!  so every removal here runs on a thread of its own,
//!  stops being waited for at `SCRATCH_REMOVAL_CEILING`,
//!  and what it spent and what became of it go to the command's timing record.
//!  A caller that collects the trees of killed commands uses the same ceiling
//!  (`landing/service_scratch_abandonment.rs`).
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
/// A path,
/// a span of time and a moment.
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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

/// The removal is controlled in the test module.
#[cfg(test)]
#[path = "scratch_entry_removal_tests.rs"]
mod tests;
