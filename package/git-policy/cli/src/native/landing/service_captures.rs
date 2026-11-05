//! What:
//!  The service's memory of when the bytes of the commits it landed were captured, kept per worktree.
//! Why:
//!  Two commands that capture a file from one worktree at different moments hold different states of it, and the later
//!  capture holds everything the earlier one did (`doc/decision/cli-git-concurrent-commits.md`, "Serial landing"). When the
//!  service sends a commit back because a landed commit changed the same path, the client needs to know which of the two
//!  captured later, and only the service saw the landed commit's request. The stamp is a reading of `CLOCK_MONOTONIC`, one
//!  clock for every process of the host, so captures made by different wrapper processes compare. The memory lives in the
//!  service process and starts empty with it, so a restart can only make a capture unknown, never misordered: a commit with
//!  no entry is treated as captured by somebody else, and the client then merges instead of preferring either side. A reboot
//!  restarts both the clock and the service together. Entries are kept per worktree, because the rule holds for captures of
//!  one worktree only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! memory.recordLanded(commonDir, worktree, commit, captured);
//! const known = memory.remembered(commonDir, worktree); // what a commit sent back is told
//! ```

use crate::service_protocol::LandedCapture;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};

/// How many landed commits are remembered for one worktree; older ones are forgotten, which makes them unknown. Measured: in the
///  service journal (1,085 landings of the `cli-git@` units, 2026-11-02T09:01:38Z to 2026-11-03T15:11:58Z) the busiest unit landed 15
///  commits in its busiest ten minutes and 349 in all, so this many cover eight times the busiest ten minutes and over a third
///  of everything the busiest worktree landed in the window. Kept as the coordinator's pick under the owner's delegation, open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`, "Sizes and ages of the service's own housekeeping").
pub const REMEMBERED: usize = 128;

type Key = (PathBuf, PathBuf);

/// What:
///  The memory of landed captures, owned by the serve loop that lands the commits and answers the clients.
/// Why:
///   Only that one thread records a landed commit and reads the memory to answer a commit sent back, so the memory is a plain
///  value it holds and passes down (`&mut` to record, `&` to read), not a process-wide table behind a lock: nothing waits for
///  it and nothing else can reach it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const memory = new CaptureMemory(); memory.recordLanded(commonDir, worktree, commit, captured);
/// ```
#[derive(Debug, Default)]
pub struct CaptureMemory {
    /// The landed commits of each worktree, oldest first.
    by_worktree: HashMap<Key, VecDeque<LandedCapture>>,
}

/// Methods over the memory.
impl CaptureMemory {
    /// What:
    ///  Remembers a landed commit and the stamp of the request that made it, when the stamp is known.
    /// Why:
    ///   A request with no stamp (zero) cannot be ordered, so it is not remembered and its commit stays unknown.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// memory.recordLanded(commonDir, worktree, commit, captured)
    /// ```
    pub fn record_landed(&mut self, common_dir: &Path, worktree: &Path, commit: &str, captured: u64) {
        if captured == 0 {
            return;
        }
        let entries: &mut VecDeque<LandedCapture> = self.by_worktree.entry((common_dir.to_path_buf(), worktree.to_path_buf())).or_default();
        entries.push_back(LandedCapture { commit: commit.to_string(), captured });
        while entries.len() > REMEMBERED {
            entries.pop_front();
        }
    }

    /// What:
    ///  The commits landed for a worktree that the service remembers, oldest first.
    /// Why:
    ///   The client matches them by name against the commits that changed a path since its base, so older ones do no harm.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// memory.remembered(commonDir, worktree)
    /// ```
    pub fn remembered(&self, common_dir: &Path, worktree: &Path) -> Vec<LandedCapture> {
        return match self.by_worktree.get(&(common_dir.to_path_buf(), worktree.to_path_buf())) {
            Some(entries) => entries.iter().cloned().collect(),
            None => Vec::new(),
        };
    }
}

/// Capture memory controls stay out of the release executable.
#[cfg(test)]
#[path = "service_captures_tests.rs"]
mod tests;
