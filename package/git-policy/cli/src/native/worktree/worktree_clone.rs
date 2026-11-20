//! What:
//!  Rung 2 of the cheap-worktree ladder:
//! each tracked file of the clean source is cloned (`FICLONE`) into the new worktree
//!  with the source's mode and times,
//! and the source's index is installed with fresh stat fields.
//! Why:
//!  A clone shares every data block with the source on btrfs,
//! on XFS with reflink and inside an overlay,
//! which the
//!  snapshot rung cannot reach (overlayfs and XFS have reflink but no subvolumes).
//! It takes no less time than Git's checkout
//!  on btrfs (`doc/planning/cli-git-btrfs-worktree.md`,
//! option B),
//! so it saves disk,
//! not time,
//! and it is the accelerated
//!  rung where a snapshot is impossible.
//! The wrapper's own index writer is `rewrite_index_stat_with`:
//! with each file's
//!  size and modification time kept,
//! only the inode,
//! change time and device of every entry change.
//! The first failure ends
//!  the rung with its reason,
//! the entries this run created stay (a name cannot be checked and unlinked in one step),
//! and
//!  Git's own checkout completes the worktree and overwrites the tracked ones.
//!  Every source file is opened below a held handle of the source root without following a link,
//! and its descriptor's
//!  metadata,
//! not a path,
//! is compared with the frozen index entry before and after the clone;
//! every destination entry is
//!  created below a held handle of the destination.
//! A directory swapped for a link while the run goes on is an error,
//!  never a detour.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await cloneTrackedFiles(source, destination, admin); // throws a reason
//! ```

use crate::worktree_clone_paths::Created;
use crate::worktree_clone_slices::clone_all;
use crate::worktree_snapshot_source::SnapshotSource;
use std::ffi::OsString;
use std::fs::File;
use std::path::Path;

/// What:
///  The components of a path joined with slashes,
/// as text.
/// Why:
///   The report names a few of the entries the clones created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.join('/')
/// ```
fn joined_names(names: &[OsString]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for name in names {
        parts.push(name.to_string_lossy().into_owned());
    }
    return parts.join("/");
}

/// What:
///  What a successful clone run holds:
/// the entries it created.
/// Why:
///   A later step that fails (the tree difference) reports them;
/// Git's own checkout then overwrites the tracked ones,
/// and the
///  rest stay as untracked files for the person to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cloned = { created: Created[] };
/// ```
#[derive(Debug)]
pub struct Cloned {
    /// The entries created,
    /// in creation order.
    created: Vec<Created>,
}

impl Cloned {
    /// The entries the run created,
    /// in creation order.
    pub fn created(&self) -> &[Created] {
        return self.created.as_slice();
    }
}

/// What:
///  The report of entries a clone run created and leaves in place.
/// Why:
///   The tracked ones are overwritten by the checkout that completes the worktree;
/// any other stays as an untracked file,
/// and
///  the person is told where to look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = leftover_report(created, destination);
/// ```
pub fn leftover_report(created: &[Created], destination: &Path) -> String {
    // The first three entries that are not directories, as `filter` and `take` chose them.
    let mut shown: Vec<String> = Vec::new();
    for entry in created {
        if entry.directory {
            continue;
        }
        if shown.len() == 3 {
            break;
        }
        shown.push(joined_names(entry.names.as_slice()));
    }
    return format!("{} entries the clones created stay in {} (for example {}); the checkout that completes the worktree overwrites the tracked ones, and `git -C {} status` lists any other as untracked", created.len(), destination.display(), shown.join(", "), destination.display());
}

/// What:
///  Clone every tracked file of the frozen index into the held `destination`,
/// and install that index in the held `admin`
///  with the stat fields of the regular files made.
/// On an error nothing is removed:
/// the entries this run created stay and
///  are counted in the reason.
/// Why:
///   The index bytes are the ones proven clean,
/// so what is installed is what was proven;
/// the destination may hold files
///  of other writers,
/// and they are not this run's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cloned = await cloneTrackedFiles(source, destination, admin); // throws a reason
/// ```
pub fn clone_tracked_files(source: &SnapshotSource, destination: &File, admin: &File) -> Result<Cloned, String> {
    let mut created: Vec<Created> = Vec::new();
    if let Err(reason) = clone_all(source, destination, admin, &mut created) {
        return Err(if created.is_empty() { reason } else { format!("{reason}; {} entries it created stay", created.len()) });
    }
    return Ok(Cloned { created });
}

/// Clone rung controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_clone_tests.rs"]
mod tests;
