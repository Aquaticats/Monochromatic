//! What:
//!  The removal of a worktree that a `git worktree add` of the wrapper registered and never completed: the directory
//!  at the destination and the administrative directory that registers it, removed only when the destination's `.git`
//!  pointer and the registration name each other.
//! Why:
//!  Real Git runs without a checkout first, so a kill, a full disk or a refused rung leaves a registered worktree that
//!  holds only Git's pointer or a partial checkout. Git's own `worktree add` removes its junk when the checkout fails;
//!  the wrapper does the same for the part it took over, and a later command does it for a wrapper that was killed.
//!  Ownership is proven from the repository, never assumed from a path: the pointer must name an administrative directory
//!  directly under this repository's `worktrees`, and that directory's `gitdir` must name the pointer back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeIncompleteWorktree(commonDir, destination); // throws when the pair is not a registered worktree of commonDir
//! ```

use super::fs_btrfs_ioctl::subvolume_destroy_at;
use super::fs_kind::is_subvolume_root;
use super::subvolume_remove::remove_tree;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// The administrative directory that `destination`'s `.git` pointer names, when it is a registration of `common_dir` that
///  names the pointer back.
///
/// Why: the removal below deletes two trees, so both must be proven to belong together first.
pub fn registered_admin(common_dir: &Path, destination: &Path) -> Result<PathBuf, String> {
    let shown: String = destination.display().to_string();
    if std::fs::canonicalize(destination).ok().as_deref() != Some(destination) {
        return Err(format!("{shown} is not a real path"));
    }
    let pointer_path: PathBuf = destination.join(".git");
    let pointer: Vec<u8> = std::fs::read(pointer_path.as_path()).map_err(|error| { return format!("{shown}/.git is unreadable ({error})"); })?;
    let line: &[u8] = pointer.strip_suffix(b"\n").unwrap_or(pointer.as_slice());
    let named: &[u8] = line.strip_prefix(b"gitdir: ").ok_or_else(|| { return format!("{shown}/.git is not a gitdir pointer"); })?;
    let admin: PathBuf = std::fs::canonicalize(destination.join(std::ffi::OsStr::from_bytes(named))).map_err(|error| { return format!("the administrative directory of {shown} is unreadable ({error})"); })?;
    let worktrees: PathBuf = std::fs::canonicalize(common_dir.join("worktrees")).map_err(|error| { return format!("the worktrees of the repository are unreadable ({error})"); })?;
    if admin.parent() != Some(worktrees.as_path()) {
        return Err(format!("{shown} is not registered in this repository"));
    }
    let recorded: Vec<u8> = std::fs::read(admin.join("gitdir")).map_err(|error| { return format!("the registration of {shown} is unreadable ({error})"); })?;
    let recorded_line: &[u8] = recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice());
    if recorded_line != pointer_path.as_os_str().as_bytes() {
        return Err(format!("the registration {} does not name {shown}", admin.display()));
    }
    return Ok(admin);
}

/// What:
///  Remove the destination and the registration of a worktree the wrapper did not complete.
/// Why:
///   Destination first, registration last, as Git's own junk removal does: a kill between the two leaves a registration
///  `git worktree prune` removes, never a directory nothing names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeIncompleteWorktree(commonDir, destination);
/// ```
pub fn remove_incomplete_worktree(common_dir: &Path, destination: &Path) -> Result<(), String> {
    let admin: PathBuf = registered_admin(common_dir, destination)?;
    let destroyed: bool = is_subvolume_root(destination).unwrap_or(false) && subvolume_destroy_at(destination).is_ok();
    if !destroyed {
        remove_tree(destination).map_err(|error| { return format!("{} cannot be removed ({error})", destination.display()); })?;
    }
    return remove_tree(admin.as_path()).map_err(|error| { return format!("the registration {} cannot be removed ({error})", admin.display()); });
}

/// Rollback controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_add_rollback_tests.rs"]
mod tests;
