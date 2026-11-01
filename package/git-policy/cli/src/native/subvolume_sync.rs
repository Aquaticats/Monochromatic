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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stats = await syncTree({ platform, source, destination, delete: true, updateOnly: false, excluded: ['.git/index.lock'] });
//! ```

use super::fs_primitive_error::PrimitiveError;
use super::subvolume_error::{Interrupt, MigrateError};
use super::subvolume_platform::Platform;
use super::subvolume_sync_attrs::{AttributeLosses, make_fifo, preserve_attributes};
use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::fs::{File, Metadata};
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// What:
///  What one pass may do.
///  `&'a [PathBuf]` borrows the list of excluded paths for the pass.
/// Why:
///   The excluded paths (relative to the tree roots) are neither copied nor deleted: the `index.lock` the migration
///  itself holds is the one use.
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
}

/// One unit of the work stack.
enum Work {
    /// List a directory and handle its children.
    Visit(PathBuf),
    /// Apply a directory's own attributes after its children.
    Finish(PathBuf, Metadata),
}

/// The state of one pass.
struct Pass<'a> {
    platform: &'a dyn Platform,
    source: &'a Path,
    destination: &'a Path,
    options: SyncOptions<'a>,
    interrupt: &'a Interrupt,
    stats: SyncStats,
    /// First copy made of each source file that has several names, by (device, inode).
    links: HashMap<(u64, u64), PathBuf>,
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

/// Remove a destination entry of any kind.
fn remove_entry(path: &Path, metadata: &Metadata) -> Result<(), MigrateError> {
    let result: std::io::Result<()> = if metadata.is_dir() { std::fs::remove_dir_all(path) } else { std::fs::remove_file(path) };
    return result.map_err(|error: std::io::Error| { return MigrateError::io("remove", path, &error); });
}

impl Pass<'_> {
    /// Handle one directory: delete extras, then handle each source child.
    fn visit(&mut self, relative: &Path, stack: &mut Vec<Work>) -> Result<(), MigrateError> {
        let source_directory: PathBuf = self.source.join(relative);
        let destination_directory: PathBuf = self.destination.join(relative);
        let listing: std::fs::ReadDir = match std::fs::read_dir(&source_directory) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.stats.vanished += 1;
                return Ok(());
            }
            Err(error) => return Err(MigrateError::UnreadableDirectory { path: source_directory, reason: error.to_string() }),
        };
        self.make_writable(destination_directory.as_path())?;
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
            match entry.metadata() {
                Ok(metadata) => children.push((entry.file_name(), metadata)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.stats.vanished += 1,
                Err(error) => return Err(MigrateError::UnreadableDirectory { path: source_directory, reason: error.to_string() }),
            }
        }
        if self.options.delete && !self.options.update_only {
            self.delete_extras(destination_directory.as_path(), relative, &names)?;
        }
        for (name, metadata) in children {
            self.stats.entries += 1;
            self.handle_child(relative.join(&name), metadata, stack)?;
        }
        return Ok(());
    }

    /// Make an existing destination directory writable for the pass; its final attributes come with `Finish`.
    fn make_writable(&self, directory: &Path) -> Result<(), MigrateError> {
        let Ok(metadata) = std::fs::symlink_metadata(directory) else { return Ok(()) };
        if metadata.is_dir() && metadata.mode() & 0o700 != 0o700 {
            std::fs::set_permissions(directory, std::fs::Permissions::from_mode(metadata.mode() | 0o700))
                .map_err(|error: std::io::Error| { return MigrateError::io("make writable", directory, &error); })?;
        }
        return Ok(());
    }

    /// Delete destination entries that the source (including its excluded names) no longer has.
    fn delete_extras(&mut self, directory: &Path, relative: &Path, names: &HashSet<OsString>) -> Result<(), MigrateError> {
        let Ok(listing) = std::fs::read_dir(directory) else { return Ok(()) };
        for item in listing.flatten() {
            if names.contains(&item.file_name()) {
                continue;
            }
            if self.options.excluded.iter().any(|path: &PathBuf| { return *path == relative.join(item.file_name()); }) {
                continue;
            }
            let metadata: Metadata = item.metadata().map_err(|error: std::io::Error| { return MigrateError::io("examine", item.path().as_path(), &error); })?;
            remove_entry(item.path().as_path(), &metadata)?;
            self.stats.deleted += 1;
        }
        return Ok(());
    }

    /// Handle one source child by its type.
    fn handle_child(&mut self, relative: PathBuf, metadata: Metadata, stack: &mut Vec<Work>) -> Result<(), MigrateError> {
        let destination: PathBuf = self.destination.join(&relative);
        let existing: Option<Metadata> = std::fs::symlink_metadata(&destination).ok();
        let file_type: std::fs::FileType = metadata.file_type();
        if file_type.is_dir() {
            if let Some(found) = &existing && !found.is_dir() {
                remove_entry(destination.as_path(), found)?;
            }
            if existing.as_ref().is_none_or(|found: &Metadata| { return !found.is_dir(); }) {
                std::fs::DirBuilder::new().mode(0o700).create(&destination).map_err(|error: std::io::Error| {
                    return MigrateError::io("create the directory", destination.as_path(), &error);
                })?;
            }
            self.stats.directories += 1;
            stack.push(Work::Finish(relative.clone(), metadata));
            stack.push(Work::Visit(relative));
            return Ok(());
        }
        if file_type.is_symlink() {
            return self.copy_symlink(&relative, &metadata, existing);
        }
        if file_type.is_fifo() {
            return self.copy_fifo(&relative, &metadata, existing);
        }
        if file_type.is_file() {
            return self.copy_file(&relative, &metadata, existing);
        }
        self.stats.special_skipped.push(relative);
        return Ok(());
    }

    /// Copy a symbolic link.
    fn copy_symlink(&mut self, relative: &Path, metadata: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = self.destination.join(relative);
        let target: PathBuf = std::fs::read_link(&source).map_err(|error: std::io::Error| { return MigrateError::io("read the link", source.as_path(), &error); })?;
        if let Some(found) = &existing {
            if found.file_type().is_symlink() && std::fs::read_link(&destination).ok().as_ref() == Some(&target) {
                self.stats.unchanged += 1;
                return Ok(());
            }
            remove_entry(destination.as_path(), found)?;
        }
        std::os::unix::fs::symlink(&target, &destination).map_err(|error: std::io::Error| { return MigrateError::io("create the link", destination.as_path(), &error); })?;
        preserve_attributes(source.as_path(), destination.as_path(), metadata, &mut self.stats.losses);
        self.stats.symlinks += 1;
        return Ok(());
    }

    /// Copy a named pipe.
    fn copy_fifo(&mut self, relative: &Path, metadata: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
        let destination: PathBuf = self.destination.join(relative);
        if let Some(found) = &existing {
            if found.file_type().is_fifo() {
                self.stats.unchanged += 1;
                return Ok(());
            }
            remove_entry(destination.as_path(), found)?;
        }
        make_fifo(destination.as_path(), metadata.mode()).map_err(|error: std::io::Error| { return MigrateError::io("create the named pipe", destination.as_path(), &error); })?;
        preserve_attributes(self.source.join(relative).as_path(), destination.as_path(), metadata, &mut self.stats.losses);
        self.stats.fifos += 1;
        return Ok(());
    }

    /// Copy a regular file.
    fn copy_file(&mut self, relative: &Path, metadata: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = self.destination.join(relative);
        let key: (u64, u64) = (metadata.dev(), metadata.ino());
        if let Some(found) = &existing {
            let newer_there: bool = self.options.update_only && (found.mtime(), found.mtime_nsec()) > (metadata.mtime(), metadata.mtime_nsec());
            let linked_already: bool = metadata.nlink() > 1 && self.links.get(&key).is_none_or(|first: &PathBuf| {
                return std::fs::symlink_metadata(first).is_ok_and(|first_meta: Metadata| { return first_meta.ino() == found.ino(); });
            });
            if (file_is_equal(found, metadata) && (metadata.nlink() <= 1 || linked_already)) || (newer_there && found.is_file()) {
                self.stats.unchanged += 1;
                if metadata.nlink() > 1 {
                    self.links.entry(key).or_insert(destination);
                }
                return Ok(());
            }
        }
        if metadata.nlink() > 1 && let Some(first) = self.links.get(&key).cloned() {
            if let Some(found) = &existing {
                remove_entry(destination.as_path(), found)?;
            }
            std::fs::hard_link(&first, &destination).map_err(|error: std::io::Error| { return MigrateError::io("link", destination.as_path(), &error); })?;
            self.stats.hard_linked += 1;
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
        if let Some(found) = &existing {
            remove_entry(destination.as_path(), found)?;
        }
        let mut output: File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&destination)
            .map_err(|error: std::io::Error| { return MigrateError::io("create", destination.as_path(), &error); })?;
        let cloned: Result<(), PrimitiveError> = self.platform.clone_file(&input, &output, destination.as_path());
        if cloned.is_ok() {
            self.stats.cloned += 1;
        } else {
            std::io::copy(&mut input, &mut output).map_err(|error: std::io::Error| { return MigrateError::io("copy to", destination.as_path(), &error); })?;
        }
        drop(output);
        preserve_attributes(source.as_path(), destination.as_path(), metadata, &mut self.stats.losses);
        self.stats.files_copied += 1;
        if metadata.nlink() > 1 {
            self.links.insert(key, destination);
        }
        return self.interrupt.check("copy-entry");
    }
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
                preserve_attributes(source.join(&relative).as_path(), destination.join(&relative).as_path(), &metadata, &mut pass.stats.losses);
            }
        }
    }
    pass.stats.unreadable.sort();
    return Ok(pass.stats);
}

/// Tree sync controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_sync_tests.rs"]
mod tests;
