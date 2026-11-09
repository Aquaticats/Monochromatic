//! What:
//!  Removing a subvolume and every subvolume nested in it,
//! deepest first,
//! by the kernel's destroy call or by emptying it.
//! Why:
//!  The rollback deletes the new subvolume after it restores the original tree,
//! and the kernel refuses to destroy a subvolume
//!  that holds another (`ENOTEMPTY`),
//! so the nested ones go first.
//! Where the mount does not allow an unprivileged
//!  destroy,
//! an emptied subvolume is removed with `rmdir`,
//! which the kernel allows for an empty subvolume root.
//! This is its own file because the rule about nesting and the fallback are one concern that the rollback's steps
//!  only call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await removeSubvolumeTree(run, path);
//! ```

use crate::subvolume_error::MigrateError;
use crate::subvolume_remove::remove_tree;
use crate::subvolume_run::Run;
use std::path::{Path, PathBuf};

/// Remove a file,
/// treating "already gone" as done.
pub(crate) fn remove_if_present(path: &Path) -> Result<(), MigrateError> {
    return match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(MigrateError::io("remove", path, &error)),
    };
}

/// What:
///  Every directory inside `top` (not `top` itself) that is a subvolume root,
/// deepest first.
/// Why:
///   Destroying them in this order never meets a subvolume that still holds another.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nested = await nestedSubvolumes(top); // deepest first
/// ```
pub(crate) fn nested_subvolumes(run: &Run<'_>, top: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![top.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(listing) = std::fs::read_dir(&directory) else { continue };
        for item in listing.flatten() {
            let Ok(file_type) = item.file_type() else { continue };
            if !file_type.is_dir() {
                continue;
            }
            if run.platform.is_subvolume_root(item.path().as_path()).unwrap_or(false) {
                found.push(item.path());
            }
            stack.push(item.path());
        }
    }
    // Each path is paired with its depth so the sort compares numbers; the sort is stable, so equal depths keep their order.
    let mut with_depth: Vec<(usize, PathBuf)> = Vec::new();
    for path in found {
        with_depth.push((path.components().count(), path));
    }
    with_depth.sort_by(deeper_first);
    let mut ordered: Vec<PathBuf> = Vec::new();
    for (_depth, path) in with_depth {
        ordered.push(path);
    }
    return ordered;
}

/// Order two paths with the one that has more components first.
pub(crate) fn deeper_first(left: &(usize, PathBuf), right: &(usize, PathBuf)) -> std::cmp::Ordering {
    return right.0.cmp(&left.0);
}

/// What:
///  Remove one subvolume that holds no other:
/// by the destroy call where the mount allows it,
/// else by emptying it and
///  `rmdir`.
/// Why:
///   Both end with the directory gone.
/// A destroy that fails for any reason falls through to the emptying route,
/// whose
///  own failure is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeOneSubvolume(run, path);
/// ```
pub(crate) fn remove_one_subvolume(run: &mut Run<'_>, path: &Path) -> Result<(), MigrateError> {
    if run.platform.removal_allowed(path) && run.platform.destroy_subvolume(path).is_ok() {
        return Ok(());
    }
    MigrateError::from_io_result(remove_tree(path), "empty and remove the subvolume", path)?;
    return Ok(());
}

/// What:
///  Remove a subvolume and every subvolume nested in it,
/// deepest first.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeSubvolumeTree(run, path);
/// ```
pub fn remove_subvolume_tree(run: &mut Run<'_>, path: &Path) -> Result<(), MigrateError> {
    for nested in nested_subvolumes(run, path) {
        remove_one_subvolume(run, nested.as_path())?;
    }
    return remove_one_subvolume(run, path);
}
