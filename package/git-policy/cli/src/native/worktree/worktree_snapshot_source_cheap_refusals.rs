//! What:
//!  The cheap reasons a snapshot rung or a clone rung cannot serve a planned `git worktree add`:
//! the creating worktree is no subvolume root,
//! the two paths are not on one btrfs filesystem,
//! or the filesystem that would receive the clones cannot share data blocks.
//! Why:
//!  These ask two `statfs` calls and one inode number,
//! so they run before the dear checks,
//! and the clone rung's three questions are asked through calls a control can answer on any filesystem.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cheap = snapshotRefusal(identity, plan); // a string, or undefined when the rung may try
//! const cloneCheap = cloneRefusal(identity, plan);
//! ```

use crate::fs_kind::{
    FilesystemKind,
    filesystem_kind,
    is_subvolume_root,
    may_clone,
};
use crate::fs_mount_options::read_mount_facts;
use crate::worktree_clone_block_cloning::{
    ZFS_BLOCK_CLONING_SWITCH,
    block_cloning_refusal_at,
};
use crate::worktree_clone_domain::{
    clone_domain_refusal_of,
    clone_write_target,
    unclonable_filesystem_asked,
};
use crate::worktree_command_facts::AddPlan;
use crate::worktree_identity::WorktreeIdentity;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Whether the mounts that hold two paths are one btrfs filesystem.
/// Why:
///   A snapshot and a clone cross subvolumes of one filesystem only;
/// the mount table names the device,
/// which is the
///  same for every subvolume of a filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function onOneBtrfs(first: string, second: string): boolean;
/// ```
fn on_one_btrfs(
    first: &Path,
    second: &Path,
) -> bool {
    if filesystem_kind(first) != Ok(FilesystemKind::Btrfs)
        || filesystem_kind(second) != Ok(FilesystemKind::Btrfs)
    {
        return false;
    }
    let (Ok(Some(one)), Ok(Some(other))) = (read_mount_facts(first), read_mount_facts(second))
    else {
        return false;
    };
    return one.source == other.source;
}

/// What:
///  The root,
/// Git directory and common directory of the creating worktree,
/// or why there is none.
/// Why:
///   A bare repository and a location outside any worktree have no files to copy from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeOf(identity: WorktreeIdentity): { root: string; gitDir: string; commonDir: string }; // throws a reason
/// ```
pub(crate) fn worktree_of(
    identity: &WorktreeIdentity
) -> Result<(&PathBuf, &PathBuf, &PathBuf), String> {
    return match identity {
        WorktreeIdentity::MainWorktree {
            worktree_root,
            git_dir,
            common_dir,
        }
        | WorktreeIdentity::LinkedWorktree {
            worktree_root,
            git_dir,
            common_dir,
        } => Ok((worktree_root, git_dir, common_dir)),
        WorktreeIdentity::BareRepository {
            ..
        }
        | WorktreeIdentity::OutsideWorktree => {
            Err(String::from("there is no worktree to copy from"))
        }
    };
}

/// What:
///  The cheap reasons the snapshot rung cannot serve a planned add:
/// the source is no subvolume root,
/// or it and the
///  destination are not on one btrfs filesystem.
/// Why:
///   These need two `statfs` calls and one inode number,
/// so they run before the dear checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function snapshotRefusal(identity: WorktreeIdentity, plan: AddPlan): string | undefined;
/// ```
pub fn snapshot_refusal(
    identity: &WorktreeIdentity,
    plan: &AddPlan,
) -> Option<String> {
    let (root, _git_dir, _common_dir): (&PathBuf, &PathBuf, &PathBuf) = match worktree_of(identity)
    {
        Ok(found) => found,
        Err(reason) => return Some(reason),
    };
    let Some(parent) = plan.destination.parent() else {
        return Some(String::from("the destination has no parent directory"));
    };
    if !is_subvolume_root(root.as_path()).unwrap_or(false) {
        return Some(format!("{} is not a btrfs subvolume", root.display()));
    }
    if !on_one_btrfs(root.as_path(), parent) {
        return Some(format!(
            "{} and {} are not on one btrfs filesystem",
            root.display(),
            parent.display()
        ));
    }
    return None;
}

/// What:
///  The cheap reasons the clone rung cannot serve a planned add:
/// either side is on a filesystem that never clones,
/// or the
///  two are not in one clone domain (one btrfs filesystem,
/// one ZFS dataset,
/// or one device elsewhere).
/// Why:
///   A clone crosses subvolumes of one btrfs filesystem,
/// shares blocks inside one ZFS dataset and nowhere else,
/// and
///  an overlay or XFS is one device;
/// the question is asked of the filesystem that would receive the clones,
/// which is
///  the destination when something already occupies it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneRefusal(identity: WorktreeIdentity, plan: AddPlan): string | undefined;
/// ```
pub fn clone_refusal(
    identity: &WorktreeIdentity,
    plan: &AddPlan,
) -> Option<String> {
    let mut questions: CloneRungQuestions<'_> = CloneRungQuestions {
        may_clone: &mut may_clone,
        block_cloning: &mut block_cloning_refusal_at_switch,
        domain_refusal: &mut clone_domain_refusal_of,
    };
    return clone_refusal_asked(identity, plan, &mut questions);
}

/// What:
///  Why the pool of the filesystem that would receive the clones has block cloning off,
///  asked of the kernel's own switch file.
/// Why:
///  The rung's real answer,
///  named so the questions struct can hold a call and a control can stand in front of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blockCloningAtSwitch = (target) => blockCloningRefusalAt(target, ZFS_BLOCK_CLONING_SWITCH);
/// ```
fn block_cloning_refusal_at_switch(target: &Path) -> Option<String> {
    return block_cloning_refusal_at((target, Path::new(ZFS_BLOCK_CLONING_SWITCH)));
}

/// The questions the clone rung asks of the two filesystems before it tries a clone,
/// as calls a control can answer on any filesystem.
pub(crate) struct CloneRungQuestions<'a> {
    /// Whether a path's filesystem may share data blocks between files at all.
    pub(crate) may_clone: &'a mut dyn FnMut(&Path) -> bool,
    /// Why the filesystem that would receive the clones cannot take them,
    /// or nothing when it can.
    pub(crate) block_cloning: &'a mut dyn FnMut(&Path) -> Option<String>,
    /// Why the two paths are not one clone domain,
    /// or nothing when they are.
    pub(crate) domain_refusal: &'a mut dyn FnMut((&Path, &Path)) -> Option<String>,
}

/// What:
///  The cheap reasons the clone rung cannot serve a planned add,
/// asked through `questions` rather than of the kernel.
/// Why:
///  The rung's three questions (may either side clone,
/// does the receiving pool have block cloning on,
/// and are the two sides one clone domain) are answered by the kernel for a real add,
/// and a
///  control has to answer them itself to prove the rung asks each one on a filesystem it never runs on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cloneRefusalAsked(identity, plan, questions): string | undefined;
/// ```
pub(crate) fn clone_refusal_asked(
    identity: &WorktreeIdentity,
    plan: &AddPlan,
    questions: &mut CloneRungQuestions<'_>,
) -> Option<String> {
    let (root, _git_dir, _common_dir): (&PathBuf, &PathBuf, &PathBuf) = match worktree_of(identity)
    {
        Ok(found) => found,
        Err(reason) => return Some(reason),
    };
    let Some(parent) = plan.destination.parent() else {
        return Some(String::from("the destination has no parent directory"));
    };
    let target: PathBuf = clone_write_target((plan.destination.as_path(), parent));
    if let Some(reason) =
        unclonable_filesystem_asked((root.as_path(), target.as_path()), questions.may_clone)
    {
        return Some(reason);
    }
    if let Some(reason) = (questions.block_cloning)(target.as_path()) {
        return Some(reason);
    }
    return (questions.domain_refusal)((root.as_path(), target.as_path()));
}
