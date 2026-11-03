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
///  Whether the administrative directory holds a `locked` entry.
/// Why:
///   A lock is a promise to keep the worktree; whether it is present is read from the entry itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockState(admin: string): boolean; // throws when the entry cannot be looked up
/// ```
fn lock_state(admin: &Path) -> Result<bool, String> {
    return Ok(admin.join("locked").symlink_metadata().is_ok());
}

/// What:
///  How one attempt to remove a worktree with a subvolume destroy ended.
/// Why:
///   Three endings need three different next steps: Git removes it (nothing changed), the work is done, or the files
///  are gone and the registration is not, which only the person can finish and which must never reach Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoveOutcome = { kind: 'left-to-git' } | { kind: 'removed' } | { kind: 'incomplete'; report: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoveOutcome {
    /// Nothing was changed; real Git removes the worktree or states its refusal.
    LeftToGit,
    /// The subvolume and the administrative directory are gone.
    Removed,
    /// The subvolume was destroyed but the registration could not be removed; the report is for standard error.
    Incomplete(String),
}

/// What:
///  What one removal is asked to do and in which repository.
/// Why:
///   The repository that was invoked decides which registrations count; the worktree's own pointers never do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemoveRequest = { context: GitContext; plan: RemovePlan; commonDir: string };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct RemoveRequest<'a> {
    /// How to start real Git for the cleanliness proof.
    pub context: &'a GitContext,
    /// The directory the person named.
    pub plan: &'a RemovePlan,
    /// The canonical common Git directory of the repository the command was run in.
    pub common_dir: &'a Path,
}

/// What:
///  The operations of one removal that touch the kernel or start Git, behind one trait.
///  `&self` borrows the implementation read-only; `dyn RemoveOperations` is how a caller holds "any implementation"
///  (a TS interface).
/// Why:
///   The ordering of the checks is the same on any filesystem; a stand-in lets the controls run it on a plain
///  directory tree, and swap objects between two steps to prove the identity checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemoveOperations { isSubvolumeRoot(path): boolean; removalAllowed(path): boolean; checkClean(...): void; destroy(path): void }
/// ```
pub trait RemoveOperations {
    /// Whether `path` is the root of a subvolume.
    fn is_subvolume_root(&self, path: &Path) -> bool;
    /// Whether an unprivileged destroy is allowed on the mount holding `path`.
    fn removal_allowed(&self, path: &Path) -> bool;
    /// The proof that the worktree is clean, or the reason it is not proven.
    fn check_clean(&self, request: &RemoveRequest<'_>, admin: &Path, common_dir: &Path) -> Result<(), String>;
    /// Destroy the subvolume at `path`.
    fn destroy(&self, path: &Path) -> Result<(), String>;
}

/// The kernel and Git, as they are.
#[derive(Clone, Copy, Debug, Default)]
pub struct LinuxOperations;

/// Every operation is the primitive of the same name.
impl RemoveOperations for LinuxOperations {
    fn is_subvolume_root(&self, path: &Path) -> bool {
        return is_subvolume_root(path).unwrap_or(false);
    }

    fn removal_allowed(&self, path: &Path) -> bool {
        return read_mount_facts(path).ok().flatten().is_some_and(|facts| { return facts.has_option(REMOVAL_OPTION); });
    }

    fn check_clean(&self, request: &RemoveRequest<'_>, admin: &Path, common_dir: &Path) -> Result<(), String> {
        return check_clean(request.context, request.plan.target.as_path(), admin, common_dir).map(|_clean| { return (); });
    }

    fn destroy(&self, path: &Path) -> Result<(), String> {
        return subvolume_destroy_at(path).map_err(|error| { return error.to_string(); });
    }
}

/// What:
///  Remove the worktree if every condition holds, with the kernel and Git as they are.
/// Why:
///   See `remove_with`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeSubvolumeWorktree(request: RemoveRequest): Promise<RemoveOutcome>;
/// ```
pub fn remove_subvolume_worktree(request: &RemoveRequest<'_>) -> RemoveOutcome {
    return remove_with(&LinuxOperations, request);
}

/// What:
///  Remove the worktree if every condition holds; report how it ended.
/// Why:
///   `LeftToGit` means Git has to do the removal, and nothing was changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeWith(operations: RemoveOperations, request: RemoveRequest): Promise<RemoveOutcome>;
/// ```
pub fn remove_with(operations: &dyn RemoveOperations, request: &RemoveRequest<'_>) -> RemoveOutcome {
    let target: &Path = request.plan.target.as_path();
    let Some(admin) = registered_admin_directory(target) else {
        debug(TAG, format!("{} is not a registered linked worktree; Git removes it", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    };
    match lock_state(admin.as_path()) {
        Ok(false) => {}
        Ok(true) => {
            debug(TAG, format!("{} is locked; Git decides", target.display()).as_str());
            return RemoveOutcome::LeftToGit;
        }
        Err(reason) => {
            debug(TAG, format!("{reason}; Git decides").as_str());
            return RemoveOutcome::LeftToGit;
        }
    }
    if !operations.is_subvolume_root(target) {
        debug(TAG, format!("{} is not a subvolume; Git removes it file by file", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    }
    if !operations.removal_allowed(target) {
        debug(TAG, format!("the mount of {} lacks {REMOVAL_OPTION}; Git removes the worktree file by file", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    }
    let Some(common_dir) = admin.parent().and_then(Path::parent) else { return RemoveOutcome::LeftToGit };
    if let Err(reason) = operations.check_clean(request, admin.as_path(), common_dir) {
        debug(TAG, format!("{} is not proven clean ({reason}); Git decides", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    }
    if let Err(error) = operations.destroy(target) {
        debug(TAG, format!("the subvolume of {} was not destroyed ({error}); Git removes it", target.display()).as_str());
        return RemoveOutcome::LeftToGit;
    }
    if let Err(error) = std::fs::remove_dir_all(admin.as_path()) {
        warn(TAG, format!("{} was removed, but its administrative directory {} was not ({error}); `git worktree prune` removes it", target.display(), admin.display()).as_str());
    }
    debug(TAG, format!("destroyed the subvolume {} and its administrative directory", target.display()).as_str());
    return RemoveOutcome::Removed;
}

/// Subvolume removal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_tests.rs"]
mod tests;

/// Controls that swap objects between the steps of a removal run on a stand-in for the kernel.
#[cfg(test)]
#[path = "worktree_remove_guard_tests.rs"]
mod guard_tests;
