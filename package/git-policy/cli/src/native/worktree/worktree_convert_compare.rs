//! What:
//!  A read-only comparison of the old tree of a conversion with the installed one:
//! the first entry of the old tree that the
//!  installed tree does not hold in the same form,
//! or that this command does not delete.
//! Why:
//!  The old tree is deleted only after the installed copy is known to hold everything in it.
//! The comparison changes neither
//!  tree and copies nothing:
//! a difference keeps the old tree and says which entry differs,
//! and whoever knows which version is
//!  wanted settles it.
//! An entry on another device (a nested subvolume or a mount point) is reported the same way,
//! because
//!  deleting it is not this command's to do.
//! A file's type and size,
//! a link's target and,
//!  for every entry,
//! the permission bits,
//! the owner,
//! the group and the carried extended attributes are
//!  compared,
//! because a change of any of the last four touches neither the size nor the modification time;
//! the
//!  extended attributes are two `llistxattr` calls per entry on most filesystems and the rest comes from
//!  the metadata already read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const difference = await firstDifference(oldRoot, installedRoot); // undefined when the installed tree holds all of it
//! ```

use crate::subvolume_sync_attrs::carried_attributes_equal;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

/// What:
///  The first attribute of the old tree's entry that the installed entry does not carry:
/// the permission bits,
///  the owner or the group,
/// or one of the carried extended attributes (user attributes and POSIX
///  ACLs).
/// Why:
///   A change of one of these changes neither the size nor the modification time,
/// so nothing else in this comparison can
///  see it;
/// a mode,
/// an owner or an attribute set last in the window before the removal is a person's work,
/// and deleting the
///  tree that holds it would discard it.
/// The extended attributes are judged by the call the copy itself uses,
/// so the comparison
///  and the copy agree on what "carried" means;
/// a security label the policy sets on the copies is not one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attributeDifference(before, after, oldPath, installedPath): string | undefined;
/// ```
fn attribute_difference(before: &std::fs::Metadata, after: &std::fs::Metadata, old: &Path, installed: &Path) -> Option<&'static str> {
    if before.mode() & 0o7777 != after.mode() & 0o7777 {
        return Some("the permission bits");
    }
    if before.uid() != after.uid() || before.gid() != after.gid() {
        return Some("the owner or group");
    }
    if !carried_attributes_equal(old, installed) {
        return Some("the extended attributes (user attributes and POSIX ACLs)");
    }
    return None;
}

/// What:
///  Why the installed tree does not hold the old tree's entry `inside` at all,
/// naming the entry and,
///  for an entry the copy never recreates,
/// the file to delete.
/// Why:
///   A socket or a device node is left out of every copy by design,
/// so the only way to finish a conversion
///  whose old tree still holds one is to delete it there;
/// a person told to "settle which version is wanted
///  by hand" has no version to choose and no path to act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function missingEntry(kind, inside, oldPath): string;
/// ```
fn missing_entry(kind: std::fs::FileType, inside: &Path, old: &Path) -> String {
    if kind.is_socket() || kind.is_block_device() || kind.is_char_device() {
        return format!(
            "{} is a socket or a device node, which this copy never recreates, and the installed worktree does not hold it; delete it from {} and run the command again",
            inside.display(),
            old.display()
        );
    }
    return format!("{} is missing from the installed worktree", inside.display());
}

/// What:
///  The first way the old tree at `old` is not held by the installed tree at `installed`,
/// or nothing when it is held whole.
/// Why:
///   See the module description.
/// A file is the same when its type,
/// size,
/// modification time,
/// permission bits,
/// owner,
/// group and carried
///  extended attributes are,
/// a link when its target and those attributes are;
/// the installed tree may hold more
///  than the old one,
/// because it has been in use since the exchange.
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
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Some(missing_entry(before.file_type(), &inside, &old.join(&inside)))),
                Err(error) => return Err(error),
            };
            if before.file_type() != after.file_type() {
                return Ok(Some(format!("{} is not the same kind of entry in the installed worktree", inside.display())));
            }
            if before.file_type().is_dir() {
                if let Some(why) = attribute_difference(&before, &after, &old.join(&inside), &installed.join(&inside)) {
                    return Ok(Some(format!("{} differs from the installed worktree ({why})", inside.display())));
                }
                pending.push(inside);
            } else if before.file_type().is_symlink() {
                if std::fs::read_link(old.join(&inside))? != std::fs::read_link(installed.join(&inside))? {
                    return Ok(Some(format!("the link {} points elsewhere in the installed worktree", inside.display())));
                }
                if let Some(why) = attribute_difference(&before, &after, &old.join(&inside), &installed.join(&inside)) {
                    return Ok(Some(format!("{} differs from the installed worktree ({why})", inside.display())));
                }
            } else if before.file_type().is_file() {
                if before.size() != after.size() || before.mtime() != after.mtime() || before.mtime_nsec() != after.mtime_nsec() {
                    return Ok(Some(format!("{} differs from the installed worktree (size or modification time)", inside.display())));
                }
                if let Some(why) = attribute_difference(&before, &after, &old.join(&inside), &installed.join(&inside)) {
                    return Ok(Some(format!("{} differs from the installed worktree ({why})", inside.display())));
                }
            }
        }
    }
    return Ok(None);
}

/// Comparison controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_compare_tests.rs"]
mod tests;
