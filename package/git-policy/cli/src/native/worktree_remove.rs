//! What:
//!  The removal of a clean linked worktree that is a btrfs subvolume, by one `BTRFS_IOC_SNAP_DESTROY` call and the
//!  deletion of its administrative directory, in place of Git's file-by-file removal.
//! Why:
//!  Git unlinks every file of the worktree (0.7 s without ignored state, 3.6 s with a large ignored tree in the
//!  measurements of `doc/planning/cli-git-btrfs-worktree.md`), and the subvolume call takes milliseconds. The call
//!  cannot be undone, so the wrapper takes it only when the same conditions hold that make Git remove without
//!  `--force`: the worktree is registered under the path given, is not locked, is clean (`worktree_clean.rs`), and
//!  holds no submodule. The mount must also allow the call to an unprivileged user (`user_subvol_rm_allowed`). Every
//!  other case, and every call that the kernel refuses, is left to Git, which then words the refusal itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (await removeSubvolumeWorktree(context, plan)) return 0; // else: run git worktree remove
//! ```

use super::diagnostic_log::{debug, warn};
use super::fs_btrfs_ioctl::subvolume_destroy_at;
use super::fs_kind::is_subvolume_root;
use super::fs_mount_options::read_mount_facts;
use super::transaction_git::GitContext;
use super::worktree_clean::check_clean;
use super::worktree_remove_plan::RemovePlan;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemove";

/// The mount option that lets a regular user destroy a subvolume with the ioctl.
const REMOVAL_OPTION: &str = "user_subvol_rm_allowed";

/// What:
///  The administrative directory a linked worktree's `.git` file names, when the worktree is registered under it.
/// Why:
///   The path in `.git` and the `gitdir` file in the administrative directory must name each other, which is how Git
///  knows a directory is the worktree registered there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registeredAdminDirectory(target: string): string | undefined;
/// ```
fn registered_admin_directory(target: &Path) -> Option<PathBuf> {
    let pointer: Vec<u8> = std::fs::read(target.join(".git")).ok()?;
    let line: &[u8] = pointer.strip_suffix(b"\n").unwrap_or(pointer.as_slice());
    let named: &OsStr = OsStr::from_bytes(line.strip_prefix(b"gitdir: ")?);
    let admin: PathBuf = std::fs::canonicalize(target.join(named)).ok()?;
    if admin.parent()?.file_name()? != "worktrees" {
        return None;
    }
    let recorded: Vec<u8> = std::fs::read(admin.join("gitdir")).ok()?;
    let recorded_path: PathBuf = PathBuf::from(OsStr::from_bytes(recorded.strip_suffix(b"\n").unwrap_or(recorded.as_slice())));
    return (recorded_path == target.join(".git")).then_some(admin);
}

/// What:
///  Remove the worktree if every condition holds; report whether it was removed.
/// Why:
///   `false` means Git has to do the removal, and nothing was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeSubvolumeWorktree(context: GitContext, plan: RemovePlan): Promise<boolean>;
/// ```
pub fn remove_subvolume_worktree(context: &GitContext, plan: &RemovePlan) -> bool {
    let target: &Path = plan.target.as_path();
    let Some(admin) = registered_admin_directory(target) else {
        debug(TAG, format!("{} is not a registered linked worktree; Git removes it", target.display()).as_str());
        return false;
    };
    if admin.join("locked").symlink_metadata().is_ok() {
        debug(TAG, format!("{} is locked; Git decides", target.display()).as_str());
        return false;
    }
    if !is_subvolume_root(target).unwrap_or(false) {
        debug(TAG, format!("{} is not a subvolume; Git removes it file by file", target.display()).as_str());
        return false;
    }
    let allowed: bool = read_mount_facts(target).ok().flatten().is_some_and(|facts| { return facts.has_option(REMOVAL_OPTION); });
    if !allowed {
        debug(TAG, format!("the mount of {} lacks {REMOVAL_OPTION}; Git removes the worktree file by file", target.display()).as_str());
        return false;
    }
    let Some(common_dir) = admin.parent().and_then(Path::parent) else { return false };
    if let Err(reason) = check_clean(context, target, admin.as_path(), common_dir) {
        debug(TAG, format!("{} is not proven clean ({reason}); Git decides", target.display()).as_str());
        return false;
    }
    if let Err(error) = subvolume_destroy_at(target) {
        debug(TAG, format!("the subvolume of {} was not destroyed ({error}); Git removes it", target.display()).as_str());
        return false;
    }
    if let Err(error) = std::fs::remove_dir_all(admin.as_path()) {
        warn(TAG, format!("{} was removed, but its administrative directory {} was not ({error}); `git worktree prune` removes it", target.display(), admin.display()).as_str());
    }
    debug(TAG, format!("destroyed the subvolume {} and its administrative directory", target.display()).as_str());
    return true;
}

/// Subvolume removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_tests.rs"]
mod tests;
