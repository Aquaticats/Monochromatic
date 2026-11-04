//! What:
//!  Make one directory tree match another, in process: copy what is missing or changed (cloning file data where the
//!  filesystem can), optionally delete what the source no longer has, or only add what is newer in the source.
//! Why:
//!  This is the work `cp --archive --reflink=always`, `rsync --archive --delete` and `rsync --archive --update` did in
//!  the migration script, without starting those programs. One idempotent function serves all three passes, so a
//!  pass interrupted at any point is repeated safely: an entry already equal (same type, size, modification time and
//!  permission bits) is left alone. The walk uses a work stack, not recursion, so a deep tree cannot overflow the stack;
//!  a directory's own attributes are applied after its children, because adding a child changes its modification time.
//!  Files this user may not read (for example root-owned mode 600 logs) are left where they are and listed; any other
//!  failure, and a directory this user cannot list, stops the run (the rule of the 2026-11-01 real run: copying around
//!  an unreadable directory with deletion on would erase its subtree from the copy).
//!  Every entry is written under a temporary name in its directory and renamed over its name when complete, so a kill
//!  never leaves a half-written file at a name a later pass would take for a newer one. The destination is reached
//!  through open directory descriptors (`subvolume_anchor.rs`), so a directory replaced by a link during the pass
//!  redirects nothing. An update-only pass never removes a destination entry that may hold newer work: a directory, or
//!  an entry of another type that is not older than the source, is kept and listed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stats = await syncTree({ platform, source, destination, delete: true, updateOnly: false, excluded: ['.git/index.lock'] });
//! ```

use super::fs_primitive_error::PrimitiveError;
use super::subvolume_anchor::Anchor;
use super::subvolume_error::{Interrupt, MigrateError};
use super::subvolume_platform::Platform;
use super::subvolume_remove::remove_tree;
use super::subvolume_sync_attrs::{
    AttributeFailure, AttributeLosses, carried_attributes_equal, make_fifo, open_fifo, preserve_on_descriptor, preserve_on_link,
};
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs::{File, Metadata};
use std::io::Seek;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// The name an entry is written under before it is renamed to its own, unless the source has an entry of that name.
const PARTIAL_NAME: &str = ".cli-git-migrate-partial";

/// How many times a file that keeps changing under the copy is copied before the pass gives up.
const COPY_ATTEMPTS: u32 = 3;

/// What:
///  What one pass may do.
///  `&'a [PathBuf]` borrows the list of excluded paths for the pass.
/// Why:
///   The excluded paths (relative to the tree roots) are neither copied nor deleted: the lock files the migration
///  itself holds are the use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SyncOptions = { delete: boolean; updateOnly: boolean; excluded: string[] };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct SyncOptions<'a> {
    /// Remove destination entries the source does not have.
    pub delete: bool,
    /// Copy a file only when the destination lacks it or has an older one; never delete.
    pub update_only: bool,
    /// Paths relative to the roots that the pass leaves alone.
    pub excluded: &'a [PathBuf],
    /// Update-only pass: a source entry the destination lacks is created only when it was modified at or after this
    ///  second (since the epoch); an older one was deleted from the destination on purpose. `None` creates them all.
    pub created_since: Option<i64>,
}

/// What:
///  What one pass did.
/// Why:
///   The report prints the counts, and the unreadable and skipped lists are the loud part of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SyncStats = { entries: number; filesCopied: number; ... unreadable: string[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SyncStats {
    /// Entries looked at.
    pub entries: u64,
    /// Directories created or kept.
    pub directories: u64,
    /// Files whose content was copied or cloned.
    pub files_copied: u64,
    /// Of those, files whose data blocks are shared with the original.
    pub cloned: u64,
    /// Files created as hard links of an earlier copy in the same pass.
    pub hard_linked: u64,
    /// Symbolic links created.
    pub symlinks: u64,
    /// Named pipes created.
    pub fifos: u64,
    /// Entries already equal.
    pub unchanged: u64,
    /// Destination entries deleted.
    pub deleted: u64,
    /// Entries that disappeared from the source during the pass.
    pub vanished: u64,
    /// Files this user may not read, left in the source tree; relative paths.
    pub unreadable: Vec<PathBuf>,
    /// Sockets and device nodes, which are not recreated; relative paths.
    pub special_skipped: Vec<PathBuf>,
    /// Attributes that could not be set.
    pub losses: AttributeLosses,
    /// Update-only pass: destination entries kept because they may hold newer work and the source has an entry of
    ///  another type there, or the destination directory was replaced; relative paths.
    pub conflicts: Vec<PathBuf>,
    /// Files that changed while they were copied and had to be copied again.
    pub recopied: u64,
    /// Source entries the destination lacks that were left out because they are older than `created_since`.
    pub left_out: u64,
    /// The first few files whose content was copied, relative paths, for the report.
    pub copied_sample: Vec<PathBuf>,
}

/// How many copied paths a pass keeps for its report.
///  Exempt from a citation under `LQM` (class: display cap): it limits how much text a message, log field or record
///  names, and no command result or duration depends on it. Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation, open to the owner's veto.
const SAMPLE_SIZE: usize = 20;

/// One unit of the work stack.
enum Work {
    /// List a directory and handle its children.
    Visit(PathBuf),
    /// Apply a directory's own attributes after its children.
    Finish(PathBuf, Metadata),
}

/// The destination directory being filled and the temporary name used inside it.
struct Directory {
    /// The destination directory, held open.
    anchor: Anchor,
    /// The temporary name for the entry being written.
    temporary: OsString,
}

impl Directory {
    /// The path of a name in the directory.
    fn child(&self, name: &OsStr) -> PathBuf {
        return self.anchor.child(name);
    }

    /// The path of the temporary entry.
    fn partial(&self) -> PathBuf {
        return self.anchor.child(self.temporary.as_os_str());
    }
}

/// The state of one pass.
struct Pass<'a> {
    platform: &'a dyn Platform,
    source: &'a Path,
    destination: &'a Path,
    options: SyncOptions<'a>,
    interrupt: &'a Interrupt,
    stats: SyncStats,
    /// Relative paths of the copies made of each source file that has several names, by (device, inode). Several copies
    ///  exist when the destination cannot link across a boundary (two subvolumes).
    links: HashMap<(u64, u64), Vec<PathBuf>>,
}

/// Whether the destination entry already equals the source file: a regular file with the same size, modification
///  time and permission bits (owner is not compared, because an unprivileged copy cannot set a foreign owner).
fn file_is_equal(copy: &Metadata, original: &Metadata) -> bool {
    return copy.is_file()
        && copy.len() == original.len()
        && copy.mtime() == original.mtime()
        && copy.mtime_nsec() == original.mtime_nsec()
        && copy.mode() & 0o7777 == original.mode() & 0o7777;
}

/// Whether the first entry's modification time is later than the second's.
fn is_newer(left: &Metadata, right: &Metadata) -> bool {
    return (left.mtime(), left.mtime_nsec()) > (right.mtime(), right.mtime_nsec());
}

/// Whether a file looks the same before and after it was copied: size, modification time and change time.
fn is_settled(before: &Metadata, after: &Metadata) -> bool {
    return before.len() == after.len()
        && (before.mtime(), before.mtime_nsec()) == (after.mtime(), after.mtime_nsec())
        && (before.ctime(), before.ctime_nsec()) == (after.ctime(), after.ctime_nsec());
}

/// Remove a destination entry of any kind.
fn remove_entry(path: &Path, shown: &Path) -> Result<(), MigrateError> {
    return remove_tree(path).map_err(|error: std::io::Error| { return MigrateError::io("remove", shown, &error); });
}

/// Remove a path that may not exist.
fn remove_if_present(path: &Path, shown: &Path) -> Result<(), MigrateError> {
    return match std::fs::symlink_metadata(path) {
        Ok(_) => remove_entry(path, shown),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(MigrateError::io("examine", shown, &error)),
    };
}

/// An attribute failure as the migration's error for `shown`.
fn attribute_error(failure: AttributeFailure, shown: &Path) -> MigrateError {
    return MigrateError::io(failure.action, shown, &failure.error);
}

/// Whether opening a destination directory failed because it is no longer a directory of the tree.
fn is_replaced(error: &std::io::Error) -> bool {
    return matches!(error.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR | libc::ENOENT));
}

/// What:
///  Settle a read of the source that may find the entry gone: a vanished entry is counted and skipped, any other failure
///  is returned.
/// Why:
///   A source that is in use changes while it is copied. An entry removed between the listing and the read is not an
///  error of the copy, but every other failure (a permission, a read error) is, and the report counts the first kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await read(); } catch (error) { if (error.code === 'ENOENT') { vanished += 1; return undefined; } throw error; }
/// ```
fn vanished_is_counted<T>(outcome: std::io::Result<T>, vanished: &mut u64) -> std::io::Result<Option<T>> {
    return match outcome {
        Ok(found) => Ok(Some(found)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            *vanished += 1;
            Ok(None)
        }
        Err(error) => Err(error),
    };
}

impl Pass<'_> {
    /// The destination path of a relative path, for messages.
    fn shown(&self, relative: &Path) -> PathBuf {
        return self.destination.join(relative);
    }

    /// Open the destination directory of a relative path.
    ///  `Ok(None)` means it was replaced by something else during an update-only pass, which keeps it and lists it.
    fn open_destination(&mut self, relative: &Path) -> Result<Option<Anchor>, MigrateError> {
        return match Anchor::open_below(self.destination, relative) {
            Ok(anchor) => Ok(Some(anchor)),
            Err(error) if is_replaced(&error) && self.options.update_only && !relative.as_os_str().is_empty() => {
                self.stats.conflicts.push(relative.to_path_buf());
                Ok(None)
            }
            Err(error) => Err(MigrateError::io("open the directory", self.shown(relative).as_path(), &error)),
        };
    }

    /// Whether a destination entry that is not a match for the source entry may be replaced. An update-only pass keeps a
    ///  directory and any entry not older than the source, and lists it.
    fn replace_allowed(&mut self, relative: &Path, found: &Metadata, original: &Metadata) -> bool {
        if !self.options.update_only {
            return true;
        }
        if found.is_dir() || !is_newer(original, found) {
            self.stats.conflicts.push(relative.to_path_buf());
            return false;
        }
        return true;
    }

    /// Handle one directory: delete extras, then handle each source child.
    fn visit(&mut self, relative: &Path, stack: &mut Vec<Work>) -> Result<(), MigrateError> {
        let source_directory: PathBuf = self.source.join(relative);
        let listing: std::fs::ReadDir = match vanished_is_counted(std::fs::read_dir(&source_directory), &mut self.stats.vanished) {
            Ok(Some(found)) => found,
            Ok(None) => return Ok(()),
            Err(error) => return Err(MigrateError::UnreadableDirectory { path: source_directory, reason: error.to_string() }),
        };
        let Some(anchor) = self.open_destination(relative)? else { return Ok(()) };
        make_writable(&anchor, self.shown(relative).as_path())?;
        let mut names: HashSet<OsString> = HashSet::new();
        let mut children: Vec<(OsString, Metadata)> = Vec::new();
        for item in listing {
            let entry: std::fs::DirEntry = item.map_err(|error: std::io::Error| {
                return MigrateError::UnreadableDirectory { path: source_directory.clone(), reason: error.to_string() };
            })?;
            names.insert(entry.file_name());
            if self.options.excluded.iter().any(|path: &PathBuf| { return *path == relative.join(entry.file_name()); }) {
                continue;
            }
            match vanished_is_counted(entry.metadata(), &mut self.stats.vanished) {
                Ok(Some(metadata)) => children.push((entry.file_name(), metadata)),
                Ok(None) => {}
                Err(error) => return Err(MigrateError::UnreadableDirectory { path: source_directory, reason: error.to_string() }),
            }
        }
        let mut temporary: OsString = OsString::from(PARTIAL_NAME);
        while names.contains(&temporary) {
            temporary.push("~");
        }
        let directory: Directory = Directory { anchor, temporary };
        // A kill leaves at most this one entry behind; it is incomplete by construction, so it never stays.
        remove_if_present(directory.partial().as_path(), self.shown(relative).join(&directory.temporary).as_path())?;
        if self.options.delete && !self.options.update_only {
            self.delete_extras(&directory, relative, &names)?;
        }
        for (name, metadata) in children {
            self.stats.entries += 1;
            self.handle_child(&directory, relative.join(&name), metadata, stack)?;
        }
        return Ok(());
    }

    /// Delete destination entries that the source (including its excluded names) no longer has.
    fn delete_extras(&mut self, directory: &Directory, relative: &Path, names: &HashSet<OsString>) -> Result<(), MigrateError> {
        let Ok(listing) = std::fs::read_dir(directory.anchor.path()) else { return Ok(()) };
        for item in listing.flatten() {
            if names.contains(&item.file_name()) {
                continue;
            }
            if self.options.excluded.iter().any(|path: &PathBuf| { return *path == relative.join(item.file_name()); }) {
                continue;
            }
            remove_entry(directory.child(item.file_name().as_os_str()).as_path(), self.shown(relative).join(item.file_name()).as_path())?;
            self.stats.deleted += 1;
        }
        return Ok(());
    }

    /// Handle one source child by its type.
    fn handle_child(&mut self, directory: &Directory, relative: PathBuf, metadata: Metadata, stack: &mut Vec<Work>) -> Result<(), MigrateError> {
        let name: OsString = relative.file_name().map(OsStr::to_os_string).unwrap_or_default();
        let destination: PathBuf = directory.child(name.as_os_str());
        let existing: Option<Metadata> = std::fs::symlink_metadata(&destination).ok();
        if existing.is_none() && self.options.update_only && self.options.created_since.is_some_and(|since: i64| { return metadata.mtime() < since; }) {
            self.stats.left_out += 1;
            return Ok(());
        }
        let file_type: std::fs::FileType = metadata.file_type();
        if file_type.is_dir() {
            if let Some(found) = &existing && !found.is_dir() {
                if !self.replace_allowed(relative.as_path(), found, &metadata) {
                    return Ok(());
                }
                remove_entry(destination.as_path(), self.shown(relative.as_path()).as_path())?;
            }
            if existing.as_ref().is_none_or(|found: &Metadata| { return !found.is_dir(); }) {
                std::fs::DirBuilder::new().mode(0o700).create(&destination).map_err(|error: std::io::Error| {
                    return MigrateError::io("create the directory", self.shown(relative.as_path()).as_path(), &error);
                })?;
            }
            self.stats.directories += 1;
            stack.push(Work::Finish(relative.clone(), metadata));
            stack.push(Work::Visit(relative));
            return Ok(());
        }
        if file_type.is_symlink() {
            return self.copy_symlink(directory, &relative, &metadata, existing);
        }
        if file_type.is_fifo() {
            return self.copy_fifo(directory, &relative, &metadata, existing);
        }
        if file_type.is_file() {
            return self.copy_file(directory, &relative, &metadata, existing);
        }
        self.stats.special_skipped.push(relative);
        return Ok(());
    }

    /// Put the finished temporary entry in place of the destination name.
    fn publish(&self, directory: &Directory, relative: &Path, existing: Option<&Metadata>) -> Result<(), MigrateError> {
        let name: &OsStr = relative.file_name().unwrap_or_default();
        let destination: PathBuf = directory.child(name);
        if let Some(found) = existing && found.is_dir() {
            remove_entry(destination.as_path(), self.shown(relative).as_path())?;
        }
        return std::fs::rename(directory.partial(), &destination).map_err(|error: std::io::Error| {
            return MigrateError::io("put the finished copy in place of", self.shown(relative).as_path(), &error);
        });
    }

    /// Copy a symbolic link.
    fn copy_symlink(&mut self, directory: &Directory, relative: &Path, metadata: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = directory.child(relative.file_name().unwrap_or_default());
        let target: PathBuf = std::fs::read_link(&source).map_err(|error: std::io::Error| { return MigrateError::io("read the link", source.as_path(), &error); })?;
        if let Some(found) = &existing {
            if found.file_type().is_symlink() && std::fs::read_link(&destination).ok().as_ref() == Some(&target) {
                if (found.mtime(), found.mtime_nsec()) != (metadata.mtime(), metadata.mtime_nsec()) && !self.options.update_only {
                    preserve_on_link(source.as_path(), destination.as_path(), metadata, &mut self.stats.losses)
                        .map_err(|failure: AttributeFailure| { return attribute_error(failure, self.shown(relative).as_path()); })?;
                }
                self.stats.unchanged += 1;
                return Ok(());
            }
            if !self.replace_allowed(relative, found, metadata) {
                return Ok(());
            }
        }
        let partial: PathBuf = directory.partial();
        std::os::unix::fs::symlink(&target, &partial).map_err(|error: std::io::Error| { return MigrateError::io("create the link", self.shown(relative).as_path(), &error); })?;
        preserve_on_link(source.as_path(), partial.as_path(), metadata, &mut self.stats.losses)
            .map_err(|failure: AttributeFailure| { return attribute_error(failure, self.shown(relative).as_path()); })?;
        self.publish(directory, relative, existing.as_ref())?;
        self.stats.symlinks += 1;
        return Ok(());
    }

    /// Copy a named pipe.
    fn copy_fifo(&mut self, directory: &Directory, relative: &Path, metadata: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = directory.child(relative.file_name().unwrap_or_default());
        if let Some(found) = &existing {
            if found.file_type().is_fifo() {
                let same: bool = found.mode() & 0o7777 == metadata.mode() & 0o7777 && (found.mtime(), found.mtime_nsec()) == (metadata.mtime(), metadata.mtime_nsec());
                if !same && !self.options.update_only {
                    let handle: File = open_fifo(destination.as_path()).map_err(|error: std::io::Error| { return MigrateError::io("open the named pipe", self.shown(relative).as_path(), &error); })?;
                    preserve_on_descriptor(source.as_path(), &handle, metadata, true, &mut self.stats.losses)
                        .map_err(|failure: AttributeFailure| { return attribute_error(failure, self.shown(relative).as_path()); })?;
                }
                self.stats.unchanged += 1;
                return Ok(());
            }
            if !self.replace_allowed(relative, found, metadata) {
                return Ok(());
            }
        }
        let partial: PathBuf = directory.partial();
        make_fifo(partial.as_path(), metadata.mode()).map_err(|error: std::io::Error| { return MigrateError::io("create the named pipe", self.shown(relative).as_path(), &error); })?;
        let handle: File = open_fifo(partial.as_path()).map_err(|error: std::io::Error| { return MigrateError::io("open the named pipe", self.shown(relative).as_path(), &error); })?;
        preserve_on_descriptor(source.as_path(), &handle, metadata, true, &mut self.stats.losses)
            .map_err(|failure: AttributeFailure| { return attribute_error(failure, self.shown(relative).as_path()); })?;
        self.publish(directory, relative, existing.as_ref())?;
        self.stats.fifos += 1;
        return Ok(());
    }

    /// Whether the destination file is one of the recorded copies of this source file, or none is recorded yet.
    fn is_linked_as_recorded(&self, key: (u64, u64), found: &Metadata) -> bool {
        let Some(copies) = self.links.get(&key) else { return true };
        return copies.iter().any(|copy: &PathBuf| {
            return std::fs::symlink_metadata(self.destination.join(copy)).is_ok_and(|copy_meta: Metadata| { return copy_meta.ino() == found.ino(); });
        });
    }

    /// Record a destination file as a copy of a source file with several names.
    fn record_link(&mut self, key: (u64, u64), relative: &Path) {
        let copies: &mut Vec<PathBuf> = self.links.entry(key).or_default();
        if !copies.iter().any(|copy: &PathBuf| { return copy == relative; }) {
            copies.push(relative.to_path_buf());
        }
    }

    /// Try to make the destination another name of an earlier copy. True when it was done.
    fn link_to_earlier_copy(&mut self, directory: &Directory, relative: &Path, key: (u64, u64), existing: Option<&Metadata>) -> Result<bool, MigrateError> {
        for copy in self.links.get(&key).cloned().unwrap_or_default() {
            let Ok(earlier) = Anchor::open_below(self.destination, copy.parent().unwrap_or(Path::new(""))) else { continue };
            let first: PathBuf = earlier.child(copy.file_name().unwrap_or_default());
            remove_if_present(directory.partial().as_path(), self.shown(relative).as_path())?;
            if self.platform.hard_link(first.as_path(), directory.partial().as_path()).is_err() {
                continue;
            }
            self.publish(directory, relative, existing)?;
            self.stats.hard_linked += 1;
            return Ok(true);
        }
        return Ok(false);
    }

    /// Copy a regular file.
    fn copy_file(&mut self, directory: &Directory, relative: &Path, listing: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = directory.child(relative.file_name().unwrap_or_default());
        let shown: PathBuf = self.shown(relative);
        let listed_key: (u64, u64) = (listing.dev(), listing.ino());
        if let Some(found) = &existing {
            if !found.is_file() {
                if !self.replace_allowed(relative, found, listing) {
                    return Ok(());
                }
            } else if self.options.update_only && is_newer(found, listing) {
                // The destination holds newer work. It is not a faithful copy, so no other name may be linked to it.
                self.stats.unchanged += 1;
                return Ok(());
            } else if file_is_equal(found, listing) && (listing.nlink() <= 1 || self.is_linked_as_recorded(listed_key, found)) {
                if !self.options.update_only && !carried_attributes_equal(source.as_path(), destination.as_path()) {
                    self.refresh_attributes(&source, &destination, listing, shown.as_path())?;
                }
                self.stats.unchanged += 1;
                if listing.nlink() > 1 {
                    self.record_link(listed_key, relative);
                }
                return Ok(());
            }
        }
        if listing.nlink() > 1 && self.link_to_earlier_copy(directory, relative, listed_key, existing.as_ref())? {
            return Ok(());
        }
        let mut input: File = match File::open(&source) {
            Ok(opened) => opened,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                self.stats.unreadable.push(relative.to_path_buf());
                return Ok(());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.stats.vanished += 1;
                return Ok(());
            }
            Err(error) => return Err(MigrateError::io("open", source.as_path(), &error)),
        };
        let mut attempt: u32 = 0;
        loop {
            attempt += 1;
            let before: Metadata = input.metadata().map_err(|error: std::io::Error| { return MigrateError::io("examine", source.as_path(), &error); })?;
            if !before.is_file() {
                self.stats.vanished += 1;
                return Ok(());
            }
            remove_if_present(directory.partial().as_path(), shown.as_path())?;
            let mut output: File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(directory.partial())
                .map_err(|error: std::io::Error| { return MigrateError::io("create", shown.as_path(), &error); })?;
            self.interrupt.check("copy-partial")?;
            input.rewind().map_err(|error: std::io::Error| { return MigrateError::io("rewind", source.as_path(), &error); })?;
            let cloned: Result<(), PrimitiveError> = self.platform.clone_file(&input, &output, directory.partial().as_path());
            if cloned.is_err() {
                std::io::copy(&mut input, &mut output).map_err(|error: std::io::Error| { return MigrateError::io("copy to", shown.as_path(), &error); })?;
            }
            let after: Metadata = input.metadata().map_err(|error: std::io::Error| { return MigrateError::io("examine", source.as_path(), &error); })?;
            if !is_settled(&before, &after) {
                drop(output);
                remove_if_present(directory.partial().as_path(), shown.as_path())?;
                if attempt >= COPY_ATTEMPTS {
                    return Err(MigrateError::Io {
                        action: String::from("copy"),
                        path: source,
                        reason: format!("it changed while it was being copied, {COPY_ATTEMPTS} times in a row; stop the programs writing to it and run the command again"),
                    });
                }
                self.stats.recopied += 1;
                continue;
            }
            preserve_on_descriptor(source.as_path(), &output, &before, !self.options.update_only, &mut self.stats.losses)
                .map_err(|failure: AttributeFailure| { return attribute_error(failure, shown.as_path()); })?;
            drop(output);
            self.publish(directory, relative, existing.as_ref())?;
            if cloned.is_ok() {
                self.stats.cloned += 1;
            }
            self.stats.files_copied += 1;
            if self.stats.copied_sample.len() < SAMPLE_SIZE {
                self.stats.copied_sample.push(relative.to_path_buf());
            }
            if before.nlink() > 1 {
                self.record_link((before.dev(), before.ino()), relative);
            }
            return self.interrupt.check("copy-entry");
        }
    }

    /// Apply the original's attributes to a destination file that already has its content.
    fn refresh_attributes(&mut self, source: &Path, destination: &Path, metadata: &Metadata, shown: &Path) -> Result<(), MigrateError> {
        let handle: File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(destination)
            .map_err(|error: std::io::Error| { return MigrateError::io("open", shown, &error); })?;
        return preserve_on_descriptor(source, &handle, metadata, true, &mut self.stats.losses)
            .map_err(|failure: AttributeFailure| { return attribute_error(failure, shown); });
    }
}

/// Make an existing destination directory writable for the pass; its final attributes come with `Finish`.
fn make_writable(anchor: &Anchor, shown: &Path) -> Result<(), MigrateError> {
    let metadata: Metadata = anchor.file().metadata().map_err(|error: std::io::Error| { return MigrateError::io("examine", shown, &error); })?;
    if metadata.mode() & 0o700 != 0o700 {
        anchor.file().set_permissions(std::os::unix::fs::PermissionsExt::from_mode(metadata.mode() | 0o700))
            .map_err(|error: std::io::Error| { return MigrateError::io("make writable", shown, &error); })?;
    }
    return Ok(());
}

/// What:
///  Run one pass from `source` to `destination` (both existing directories).
/// Why:
///   See the module description; the work stack replaces recursion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stats = await syncTree({ platform, source, destination, options, interrupt });
/// ```
pub fn sync_tree(
    platform: &dyn Platform,
    source: &Path,
    destination: &Path,
    options: SyncOptions<'_>,
    interrupt: &Interrupt,
) -> Result<SyncStats, MigrateError> {
    let mut pass: Pass<'_> = Pass { platform, source, destination, options, interrupt, stats: SyncStats::default(), links: HashMap::new() };
    let root_metadata: Metadata = std::fs::symlink_metadata(source).map_err(|error: std::io::Error| { return MigrateError::io("examine", source, &error); })?;
    let mut stack: Vec<Work> = vec![Work::Finish(PathBuf::new(), root_metadata), Work::Visit(PathBuf::new())];
    while let Some(work) = stack.pop() {
        match work {
            Work::Visit(relative) => pass.visit(relative.as_path(), &mut stack)?,
            Work::Finish(relative, metadata) => {
                let Some(anchor) = pass.open_destination(relative.as_path())? else { continue };
                let shown: PathBuf = pass.shown(relative.as_path());
                preserve_on_descriptor(source.join(&relative).as_path(), anchor.file(), &metadata, !options.update_only, &mut pass.stats.losses)
                    .map_err(|failure: AttributeFailure| { return attribute_error(failure, shown.as_path()); })?;
            }
        }
    }
    pass.stats.unreadable.sort();
    pass.stats.conflicts.sort();
    pass.stats.conflicts.dedup();
    return Ok(pass.stats);
}

/// Tree sync controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_sync_tests.rs"]
mod tests;
