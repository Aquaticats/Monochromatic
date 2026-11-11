//! What:
//!  The refusal of the one-step removal for a worktree that holds a subvolume or a mount point below its own root,
//! and the
//!  words for the person.
//! Why:
//!  The removal of a subvolume worktree is one `BTRFS_IOC_SNAP_DESTROY`,
//! and the kernel refuses that call for a subvolume that
//!  contains another subvolume (`ENOTEMPTY`),
//! so a worktree holding a linked worktree of its own could never be destroyed that way:
//! the
//!  rename took it out of the person's way,
//! the directory stayed for ever under the service's own name with every file of it,
//! and the
//!  command said the files were gone (a Sonnet finding,
//! F5).
//! Git's own removal is no answer:
//! it unlinks the inner
//!  worktree's files and leaves its empty subvolume,
//! so the one worktree inside another keeps nothing
//!  (a Sonnet finding,
//! F2).
//! The removal therefore proves the kernel can take the whole tree before it renames anything,
//! with the
//!  same walk the emptying makes,
//! and refuses with the path it found,
//! the branch the worktree there has checked out,
//! and
//!  every way forward.
//! A walk that cannot read some directory proves nothing about it,
// and the removal is then Git's,
// which is
//!  what it was before this check existed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = refusalForNestedVolume({ root, target, commonDir, found }); // the report, or undefined
//! ```

use crate::subvolume_remove_walk::Boundary;
use crate::worktree_landed_registrations::{Registration, registrations_of};
use std::path::Path;

/// The prefix every line of this program carries.
const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// What:
///  The report of a removal refused because a subvolume or a mount point stands inside the worktree.
/// Why:
///   The destroy cannot be undone,
/// so what stands in its way is named before it runs,
/// and the person is sent to
///  the one command that clears it while keeping what is there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = refusalForNestedVolume({ target, commonDir, found }); // string
/// ```
pub(crate) fn refusal_for_nested_volume(target: &Path, common_dir: &Path, found: &Boundary) -> String {
    let inner: Option<Registration> = registration_at(common_dir, found.path.as_path());
    return match inner {
        Some(worktree) => named_worktree_report(target, &worktree),
        None => unnamed_volume_report(target, found),
    };
}

/// What:
///  The registration of this repository whose worktree is exactly `path`,
/// when there is one.
/// Why:
///   A nested linked worktree is the shape a person makes,
/// and its branch is what the refusal keeps:
/// naming `git worktree
///  remove` for it sends the person to a command that keeps the branch and the commits it holds,
/// while a
///  subvolume that is no worktree has no branch to name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worktree = registrationAt(commonDir, path); // Registration | undefined
/// ```
fn registration_at(common_dir: &Path, path: &Path) -> Option<Registration> {
    for one in registrations_of(common_dir).ok()? {
        if one.worktree == path {
            return Some(one);
        }
    }
    return None;
}

/// What:
///  The refusal for a worktree that holds a linked worktree of this repository,
/// naming it,
/// its branch,
/// and the
///  command that removes it.
/// Why:
///   The inner worktree's own files are the reason nothing is removed,
/// and the person is the one who reads them and
///  decides,
/// so the way forward keeps every one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = namedWorktreeReport(target, worktree);
/// ```
fn named_worktree_report(target: &Path, worktree: &Registration) -> String {
    return format!(
        "{PREFIX}the worktree {target} holds the linked worktree {inner}, which is on branch {branch}, and the kernel \
         cannot destroy a subvolume that contains another one. Nothing was removed, and nothing of {inner} was deleted.\n\
         Remove {inner} first, with `git worktree remove {inner}`, which keeps its branch and the commits on it and \
         refuses while anything there is uncommitted; then run this removal again. To keep both, leave them as they \
         are: {target} can only be removed by hand, one file at a time, once {inner} is gone.",
        target = target.display(),
        inner = worktree.worktree.display(),
        branch = worktree.branch
    );
}

/// What:
///  The refusal for a worktree that holds a subvolume or a mount point that is no worktree of this repository.
/// Why:
///   A subvolume of the person's own is the same to the kernel as a nested worktree,
/// and a mount point stops the
///  deletion too,
/// so the report names it and every way to move,
/// unmount or delete it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = unnamedVolumeReport(target, found);
/// ```
fn unnamed_volume_report(target: &Path, found: &Boundary) -> String {
    return format!(
        "{PREFIX}the worktree {target} holds {inner}, which is {what}, and the kernel cannot destroy a subvolume that \
         contains one. Nothing was removed, and nothing of {inner} was deleted.\n\
         Move {inner} out of {target}, or delete it yourself (`btrfs subvolume delete {inner}`, or `umount {inner}` for \
         a mount point), and then run this removal again. To keep it, remove the worktree by hand, one file at a time.",
        target = target.display(),
        inner = found.path.display(),
        what = found.what
    );
}

/// The refusals of a worktree that holds a volume of its own stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_refuses_a_nested_volume_tests.rs"]
mod tests;
