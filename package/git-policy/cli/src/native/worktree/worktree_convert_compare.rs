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
//!
//!  The old tree's own top directory is compared like any entry,
//! and so is a named pipe or a socket,
//! whose kind decides
//!  only that nothing else is compared;
//! and a regular file whose size and modification time agree is compared by
//!  contents when its change time is at or after the proving pass,
//! because an edit can keep the length and put
//!  the modification time back,
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
use crate::worktree_copy_compare::COMPARE_BUFFER_BYTES;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

/// Whether the entry at `before` changed at or after the instant `synced_ms`,
/// with the change time the comparison reads.
/// Why:
///   A file's contents can be rewritten to the same length and its modification time put back,
/// so no other compared field sees it;
/// the change time cannot be put back by an ordinary
///  user,
/// and the proving pass set the copy's own change time before the window opened.
/// A journal without
///  the instant is one an earlier release wrote,
/// so every file is compared by contents rather than none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function changedSince(before: Metadata, syncedMs: number | undefined): boolean;
/// ```
fn changed_since(before: &std::fs::Metadata, synced_ms: Option<i64>) -> bool {
    let Some(after) = synced_ms else { return true };
    let changed: i128 = i128::from(before.ctime()) * 1_000_000_000 + i128::from(before.ctime_nsec());
    return changed >= i128::from(after) * 1_000_000;
}

/// Whether two files hold different bytes.
/// Why:
///   See `changed_since`;
/// the bytes are compared in bounded chunks,
/// through the plain-file open on both sides,
/// so memory
///  does not grow with the file and a name swapped for a pipe or a link cannot make the read wait or
///  follow it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function contentsDiffer(old: string, installed: string): boolean; // throws when a file cannot be read
/// ```
fn contents_differ(old: &Path, installed: &Path) -> std::io::Result<bool> {
    // Both sides go through the plain-file open: a name swapped for a link or a named pipe after the walk read its metadata
    // must not make this read follow it or wait for a writer, while the conversion holds its claim.
    let mut left: std::fs::File = crate::plain_file::open_plain_file(old)?;
    let mut right: std::fs::File = crate::plain_file::open_plain_file(installed)?;
    #[cfg(test)]
    note_contents_opened(old);
    let mut left_buffer: Vec<u8> = vec![0; COMPARE_BUFFER_BYTES];
    let mut right_buffer: Vec<u8> = vec![0; COMPARE_BUFFER_BYTES];
    loop {
        let left_read: usize = fill_buffer(&mut left, &mut left_buffer)?;
        let right_read: usize = fill_buffer(&mut right, &mut right_buffer)?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(true);
        }
        if left_read == 0 {
            return Ok(false);
        }
    }
}

/// Fill `buffer` from `file` as far as the file has bytes,
/// and answer how many it gave;
/// nothing means both met their end.
fn fill_buffer(file: &mut std::fs::File, buffer: &mut [u8]) -> std::io::Result<usize> {
    let mut filled: usize = 0;
    while filled < buffer.len() {
        let read: usize = std::io::Read::read(file, &mut buffer[filled..])?;
        if read == 0 {
            break;
        }
        filled += read;
    }
    return Ok(filled);
}

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
    return first_difference_after(old, installed, None);
}

/// What:
///  The first way the old tree at `old` is not held by the installed tree at `installed`,
/// for a conversion whose
///  proving pass finished at `synced_ms`.
/// Why:
///   See the module description.
/// A regular file whose size and modification time agree is compared by
///  contents when it changed at or after the proof,
/// which is when a person's edit can hide behind them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// firstDifferenceAfter(old, installed, syncedMs);
/// ```
pub fn first_difference_after(old: &Path, installed: &Path, synced_ms: Option<i64>) -> std::io::Result<Option<String>> {
    // The root may be named through a descriptor's own path, a link that is followed to the directory it holds.
    let before_root: std::fs::Metadata = std::fs::metadata(old)?;
    let after_root: std::fs::Metadata = std::fs::metadata(installed)?;
    if before_root.file_type() != after_root.file_type() {
        return Ok(Some(String::from("the old tree itself is not the same kind of entry as the installed worktree")));
    }
    if let Some(why) = attribute_difference(&before_root, &after_root, old, installed) {
        return Ok(Some(format!("the old tree itself differs from the installed worktree ({why})")));
    }
    let device: u64 = before_root.dev();
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
            // Every kind of entry is compared by its attributes, a named pipe and a socket included; the kind decides only
            // what else is compared.
            if let Some(why) = attribute_difference(&before, &after, &old.join(&inside), &installed.join(&inside)) {
                return Ok(Some(format!("{} differs from the installed worktree ({why})", inside.display())));
            }
            if before.file_type().is_dir() {
                pending.push(inside);
            } else if before.file_type().is_symlink() {
                if std::fs::read_link(old.join(&inside))? != std::fs::read_link(installed.join(&inside))? {
                    return Ok(Some(format!("the link {} points elsewhere in the installed worktree", inside.display())));
                }
            } else if before.file_type().is_file() {
                if before.size() != after.size() || before.mtime() != after.mtime() || before.mtime_nsec() != after.mtime_nsec() {
                    return Ok(Some(format!("{} differs from the installed worktree (size or modification time)", inside.display())));
                }
                // Either side can hold bytes the other does not: the old tree a late edit, the installed worktree one made
                // after the exchange, and neither side's change time sees the other's.
                if changed_since(&before, synced_ms) || changed_since(&after, synced_ms) {
                    #[cfg(test)]
                    note_contents_open(old.join(&inside).as_path());
                    if contents_differ(&old.join(&inside), &installed.join(&inside))? {
                        return Ok(Some(format!("{} differs from the installed worktree (contents)", inside.display())));
                    }
                }
            }
        }
    }
    return Ok(None);
}

#[cfg(test)]
thread_local! {
    // The callback a control installs to replace an entry just before its contents are read; nothing in a release build
    // reads or sets it.
    static BEFORE_CONTENTS: std::cell::RefCell<Option<Box<dyn FnMut(&Path)>>> = const { std::cell::RefCell::new(None) };
    // The callback a control installs to act on the two files once they are open and their lengths are known; nothing in a
    // release build reads or sets it.
    static AFTER_OPEN: std::cell::RefCell<Option<Box<dyn FnMut(&Path)>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Install `watch` to run for the old entry whose contents are about to be read,
///  or clear it with `None`.
/// Why:
///   A control of what the contents read can meet must replace the entry in the moment between the metadata the walk read
/// and the open the comparison makes;
/// no other seam reaches that moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchContentsOpen((path) => { ... });
/// ```
#[cfg(test)]
pub(crate) fn watch_contents_open(watch: Option<Box<dyn FnMut(&Path)>>) {
    BEFORE_CONTENTS.with(|slot| {
        *slot.borrow_mut() = watch;
    });
}

/// Run the installed callback for the old entry at `path` just before its contents are read.
#[cfg(test)]
fn note_contents_open(path: &Path) {
    BEFORE_CONTENTS.with(|slot| {
        let mut installed = slot.borrow_mut();
        if let Some(watch) = installed.as_mut() {
            watch(path);
        }
    });
}

/// What:
///  Install `watch` to run for the old entry whose contents are about to be read once both sides are open,
///  or clear it with `None`.
/// Why:
///   A control of how far the read is bounded must extend the two files after they were opened and before the first byte
/// is read;
/// no other seam reaches that moment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchContentsOpened((path) => { ... });
/// ```
#[cfg(test)]
pub(crate) fn watch_contents_opened(watch: Option<Box<dyn FnMut(&Path)>>) {
    AFTER_OPEN.with(|slot| {
        *slot.borrow_mut() = watch;
    });
}

/// Run the installed callback for the old entry at `path` once both files are open and their lengths are known.
#[cfg(test)]
fn note_contents_opened(path: &Path) {
    AFTER_OPEN.with(|slot| {
        let mut installed = slot.borrow_mut();
        if let Some(watch) = installed.as_mut() {
            watch(path);
        }
    });
}

/// Comparison controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_compare_tests.rs"]
mod tests;
