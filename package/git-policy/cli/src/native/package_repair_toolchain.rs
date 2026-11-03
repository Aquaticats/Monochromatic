//! What:
//!  The toolchain repair: when the launchers copied into a new worktree's `.cache/toolchain-launchers` still name another
//!  checkout's `toolchain.lock.json`, rebuild the worktree's own links and launchers with `mpm-shadow install
//!  --frozen-lockfile`.
//! Why:
//!  mpm-shadow owns `toolchain.plan.jsonc`, `toolchain.lock.json` and the pinned tools. Its exposed links and launcher records
//!  hold absolute paths, so a copy would run the source checkout's tool versions and guard, and a branch that changes the
//!  lock would silently use the source's. It installs nothing here: the repair runs only when `mpm-shadow list` finds every
//!  tool in sync, so `install` only relinks and never reaches the network. A tool that is out of sync is left for the person,
//!  with `mise run toolchain:install` named.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairToolchain(destination, environment); // undefined when the launchers already name this worktree
//! ```

use super::package_repair::RepairEnvironment;
use super::package_repair_report::Finding;
use std::path::{Path, PathBuf};

/// What:
///  The launcher records of a worktree that name a lock file other than the worktree's own.
/// Why:
///   They are what a copy leaves behind; a record naming this worktree's lock is already right and needs no repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staleLaunchRecords(destination: string): string[];
/// ```
pub fn stale_launch_records(_destination: &Path) -> Vec<PathBuf> {
    return Vec::new();
}

/// What:
///  Rebuild the worktree's toolchain links and launchers, when its launcher records are stale and its tools are in sync.
/// Why:
///   See the module comment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairToolchain(destination: string, environment: RepairEnvironment): Promise<Finding | undefined>;
/// ```
pub fn repair_toolchain(_destination: &Path, _environment: &RepairEnvironment) -> Option<Finding> {
    return None;
}

/// Toolchain repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_toolchain_tests.rs"]
mod tests;
