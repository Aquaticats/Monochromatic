//! What:
//!  Cloning one directory tree for a worktree repair: every file cloned (or copied when the filesystem cannot clone), every
//!  directory recreated with its mode, every link recreated with its exact target, modes and times kept.
//! Why:
//!  Cargo keeps its build state outside the worktree when the person's configuration says so, so a copy of the worktree's
//!  ignored files does not carry it. The state is reused by the new worktree's registry and git dependencies, and Cargo
//!  decides what is fresh from file times, so the clone keeps them. The destination must not exist: nothing is overwritten.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cloned = await cloneTree({ source, destination, workers });
//! ```

use std::path::Path;

/// What:
///  Clone `source` into the new directory `destination`.
/// Why:
///   The count of entries made is returned for the log; a failure names the entry that could not be made, and leaves what was
///  made in place for the person to remove with the destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cloneTree(args: { source: string; destination: string; workers: number }): Promise<number>;
/// ```
pub fn clone_tree(_source: &Path, _destination: &Path, _workers: usize) -> Result<usize, String> {
    return Err(String::from("not written yet"));
}

/// Tree clone controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_tree_clone_tests.rs"]
mod tests;
