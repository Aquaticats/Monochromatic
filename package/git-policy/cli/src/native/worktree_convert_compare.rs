//! What:
//!  A read-only comparison of the old tree of a conversion with the installed one: the first entry of the old tree that the
//!  installed tree does not hold in the same form, or that this command does not delete.
//! Why:
//!  The old tree is deleted only after the installed copy is known to hold everything in it. The comparison changes neither
//!  tree and copies nothing: a difference keeps the old tree and says which entry differs, and whoever knows which version is
//!  wanted settles it. An entry on another device (a nested subvolume or a mount point) is reported the same way, because
//!  deleting it is not this command's to do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const difference = await firstDifference(oldRoot, installedRoot); // undefined when the installed tree holds all of it
//! ```

use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// What:
///  The first way the old tree at `old` is not held by the installed tree at `installed`, or nothing when it is held whole.
/// Why:
///   See the module description. A file is the same when its type, size and modification time are, a link when its target is;
///  the installed tree may hold more than the old one, because it has been in use since the exchange.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// firstDifference(old, installed);
/// ```
pub fn first_difference(old: &Path, installed: &Path) -> std::io::Result<Option<String>> {
    // The root may be named through a descriptor's own path, a link that is followed to the directory it holds.
    let device: u64 = std::fs::metadata(old)?.dev();
    let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        for item in std::fs::read_dir(old.join(&relative))? {
            let entry: std::fs::DirEntry = item?;
            let inside: PathBuf = relative.join(entry.file_name());
            let before: std::fs::Metadata = std::fs::symlink_metadata(old.join(&inside))?;
            if before.dev() != device {
                return Ok(Some(format!("{} is a subvolume or a mount point inside the old tree, which this command does not delete", inside.display())));
            }
            let after: std::fs::Metadata = match std::fs::symlink_metadata(installed.join(&inside)) {
                Ok(found) => found,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Some(format!("{} is missing from the installed worktree", inside.display()))),
                Err(error) => return Err(error),
            };
            if before.file_type() != after.file_type() {
                return Ok(Some(format!("{} is not the same kind of entry in the installed worktree", inside.display())));
            }
            if before.file_type().is_dir() {
                pending.push(inside);
            } else if before.file_type().is_symlink() {
                if std::fs::read_link(old.join(&inside))? != std::fs::read_link(installed.join(&inside))? {
                    return Ok(Some(format!("the link {} points elsewhere in the installed worktree", inside.display())));
                }
            } else if before.file_type().is_file() && (before.size() != after.size() || before.mtime() != after.mtime() || before.mtime_nsec() != after.mtime_nsec()) {
                return Ok(Some(format!("{} differs from the installed worktree (size or modification time)", inside.display())));
            }
        }
    }
    return Ok(None);
}

/// Comparison controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_compare_tests.rs"]
mod tests;
