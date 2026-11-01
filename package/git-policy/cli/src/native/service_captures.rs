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
//! recordLanded(commonDir, worktree, commit, captured);
//! const known = remembered(commonDir, worktree); // what a commit sent back is told
//! ```

use super::service_protocol::LandedCapture;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// How many landed commits are remembered for one worktree; older ones are forgotten, which makes them unknown.
pub const REMEMBERED: usize = 128;

type Key = (PathBuf, PathBuf);

fn memory() -> &'static Mutex<HashMap<Key, VecDeque<LandedCapture>>> {
    static MEMORY: OnceLock<Mutex<HashMap<Key, VecDeque<LandedCapture>>>> = OnceLock::new();
    return MEMORY.get_or_init(|| { return Mutex::new(HashMap::new()); });
}

/// What:
///  Remembers a landed commit and the stamp of the request that made it, when the stamp is known.
/// Why:
///   A request with no stamp (zero) cannot be ordered, so it is not remembered and its commit stays unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordLanded(commonDir, worktree, commit, captured)
/// ```
pub fn record_landed(common_dir: &Path, worktree: &Path, commit: &str, captured: u64) {
    if captured == 0 {
        return;
    }
    let Ok(mut all) = memory().lock() else { return };
    let entries: &mut VecDeque<LandedCapture> = all.entry((common_dir.to_path_buf(), worktree.to_path_buf())).or_default();
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
/// remembered(commonDir, worktree)
/// ```
pub fn remembered(common_dir: &Path, worktree: &Path) -> Vec<LandedCapture> {
    let Ok(all) = memory().lock() else { return Vec::new() };
    return all.get(&(common_dir.to_path_buf(), worktree.to_path_buf())).map(|entries| { return entries.iter().cloned().collect(); }).unwrap_or_default();
}

/// Capture memory controls stay out of the release executable.
#[cfg(test)]
#[path = "service_captures_tests.rs"]
mod tests;
