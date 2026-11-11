//! What:
//!  Whether a directory a repair deletes from or writes in is inside the worktree it serves,
//! once every link along the way is followed.
//! Why:
//!   A copied `target/` or `node_modules` can be a link to a directory elsewhere,
//! and the copy of the ignored state copies the link rather than what it names,
//! so the path a repair is handed lies inside the worktree while the files a clean or an install
//!  reaches do not:
//! `cargo clean --package` would delete the source checkout's build output through it.
//! No repair may touch a directory outside the worktree it serves,
//! so the two repairs that do (
//! `package_repair_cargo.rs`,
//! `package_repair_pnpm.rs`)
//! ask this first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const isInsideTheWorktree = (worktree, directory) => directory.startsWith(worktree) && realpath(directory).startsWith(realpath(worktree));
//! ```

/// What:
///  Whether `directory`,
/// which the caller expects below `worktree`,
/// is still there when its links are followed,
/// and whether it is inside `worktree` at all.
/// Why:
///   A directory the worktree has not made yet is the worktree's own place to make,
/// so a path that is absent and inside the worktree answers true;
/// a directory that exists answers true only when its real path is inside the worktree's,
/// which is what a clean or an install actually reaches.
/// A worktree that cannot be resolved has no inside,
/// so nothing answers true for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isInsideTheWorktree(worktree: string, directory: string): boolean;
/// ```
pub fn is_inside_the_worktree(worktree: &std::path::Path, directory: &std::path::Path) -> bool {
    if !directory.starts_with(worktree) {
        return false;
    }
    let Ok(root) = std::fs::canonicalize(worktree) else {
        return false;
    };
    return match std::fs::canonicalize(directory) {
        Ok(real) => real.starts_with(root),
        // Nothing is there yet, so nothing outside the worktree can be reached through the name.
        Err(_absent) => true,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_touches_only_its_own_worktree_tests.rs"]
mod tests;
