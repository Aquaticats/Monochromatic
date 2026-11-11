//! What:
//!  Make one directory tree match another,
//! in process:
//! copy what is missing or changed (cloning file data where the
//!  filesystem can),
//! optionally delete what the source no longer has,
//! or only add what is newer in the source.
//! Why:
//!  This is the work `cp --archive --reflink=always`,
//! `rsync --archive --delete` and `rsync --archive --update` did in
//!  the migration script,
//! without starting those programs.
//! One idempotent function serves all three passes,
//! so a
//!  pass interrupted at any point is repeated safely:
//! an entry already equal (same type,
//! size,
//! modification time and
//!  permission bits) is left alone.
//! The walk uses a work stack,
//! not recursion,
//! so a deep tree cannot overflow the stack;
//!  a directory's own attributes are applied after its children,
//! because adding a child changes its modification time.
//!  Files this user may not read (for example root-owned mode 600 logs) are left where they are and listed;
//! any other
//!  failure,
//! and a directory this user cannot list,
//! stops the run (the rule of the 2026-11-01 real run:
//! copying around
//!  an unreadable directory with deletion on would erase its subtree from the copy).
//!  Every entry is written under a temporary name in its directory and renamed over its name when complete,
//! so a kill
//!  never leaves a half-written file at a name a later pass would take for a newer one.
//! The destination is reached
//!  through open directory descriptors (`subvolume_anchor.rs`),
//! so a directory replaced by a link during the pass
//!  redirects nothing.
//! An update-only pass never removes a destination entry that may hold newer work:
//! a directory,
//! or
//!  an entry of another type that is not older than the source,
//! is kept and listed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stats = await syncTree({ platform, source, destination, delete: true, updateOnly: false, excluded: ['.git/index.lock'] });
//! ```

use crate::subvolume_anchor::Anchor;
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_platform::Platform;
use crate::subvolume_sync_attrs::preserve_on_descriptor;
/// Another command's stage container is no state of the source, so a pass never enters one and never removes one.
use crate::worktree_copy_prune::names_a_private_stage;
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs::Metadata;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
pub use crate::subvolume_sync_pass_state_and_checks::{SyncOptions, SyncStats};
use crate::subvolume_sync_pass_state_and_checks::*;

//region A test-only seam: one callback for every entry a pass examines
#[cfg(test)]
thread_local! {
    // The callback a control installs to change one entry *during* a pass, after the pass reached it; nothing in a release
    // build reads or sets it.
    static ENTRY_WATCH: std::cell::RefCell<Option<Box<dyn FnMut(&Path)>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Install `watch` to run for every entry a pass examines on this thread,
///  or clear the callback with `None`.
/// Why:
///   A control of what a pass cannot see must change a file at a point no other seam reaches:
/// the pass
///  has compared that entry and will not look at it again.
/// The callback runs on this thread only,
/// so two controls in
///  one process cannot see each other's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchEntries((path) => { ... });
/// ```
#[cfg(test)]
pub(crate) fn watch_entries(watch: Option<Box<dyn FnMut(&Path)>>) {
    ENTRY_WATCH.with(|slot| {
        *slot.borrow_mut() = watch;
    });
}

/// Run the installed callback for the entry at `path`,
/// if a control installed one.
#[cfg(test)]
fn note_entry(path: &Path) {
    ENTRY_WATCH.with(|slot| {
        // The borrow ends before the callback runs nothing of this module's, so it cannot re-enter the slot.
        let mut installed = slot.borrow_mut();
        if let Some(watch) = installed.as_mut() {
            watch(path);
        }
    });
}
//endregion

impl Pass<'_> {

    /// Open the destination directory of a relative path.
    ///  `Ok(None)` means it was replaced by something else during an update-only pass,
    /// which keeps it and lists it.
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

    /// Handle one directory:
    /// delete extras,
    /// then handle each source child.
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
            let entry: std::fs::DirEntry = match item {
                Ok(found) => found,
                Err(error) => return Err(MigrateError::UnreadableDirectory { path: source_directory.clone(), reason: error.to_string() }),
            };
            names.insert(entry.file_name());
            if self.options.excluded.contains(&relative.join(entry.file_name())) {
                continue;
            }
            // Another command's stage container holds the ignored state a worktree's copy is filling, credentials included,
            //  and nothing names it in the destination: a pass that carried it would leave that state in a worktree on another
            //  branch for good, and one that deleted it would delete a copy in progress.
            if names_a_private_stage(relative.join(entry.file_name()).as_os_str().as_encoded_bytes()) {
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
            #[cfg(test)]
            note_entry(self.source.join(relative).join(&name).as_path());
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
            if self.options.excluded.contains(&relative.join(item.file_name())) {
                continue;
            }
            // The same rule as the source side: a stage container belongs to the command filling it, so a deletion pass leaves
            //  it where it is.
            if names_a_private_stage(relative.join(item.file_name()).as_os_str().as_encoded_bytes()) {
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
        let older_than_since: bool = match self.options.created_since {
            Some(since) => metadata.mtime() < since,
            None => false,
        };
        if existing.is_none() && self.options.update_only && older_than_since {
            self.stats.left_out.push(relative);
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
            let is_directory_now: bool = match existing.as_ref() {
                Some(found) => found.is_dir(),
                None => false,
            };
            if !is_directory_now {
                MigrateError::from_io_result(std::fs::DirBuilder::new().mode(0o700).create(&destination), "create the directory", self.shown(relative.as_path()).as_path())?;
                self.made_here.insert(relative.clone());
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

    /// What:
    ///  Remove a directory this pass made when every entry of the source's directory was left out.
    /// Why:
    ///   A stage container is left out wherever it stands,
    /// and a worktree's ignored state may hold nothing else,
    /// so a copy
    ///  would otherwise leave `.claude/worktrees` and the folders above it behind as empty directories nothing names,
    ///  in a tree whose source has no state there;
    /// a person comparing the two trees then sees entries that exist on one side
    ///  only.
    /// A directory of the source that holds no entry at all is a directory in its own right,
    /// and a copy makes it and
    ///  keeps it.
    fn remove_a_directory_that_holds_nothing(&mut self, relative: &Path) -> Result<(), MigrateError> {
        if relative.as_os_str().is_empty() || !self.made_here.contains(relative) {
            return Ok(());
        }
        // Only a source directory whose entries were all left out answers this; one with no entry at all is copied.
        let source_holds_an_entry: bool = std::fs::read_dir(self.source.join(relative)).is_ok_and(|mut entries: std::fs::ReadDir| { return entries.next().is_some(); });
        if !source_holds_an_entry {
            return Ok(());
        }
        let made_holds_nothing: bool = std::fs::read_dir(self.destination.join(relative)).is_ok_and(|mut entries: std::fs::ReadDir| { return entries.next().is_none(); });
        if !made_holds_nothing {
            return Ok(());
        }
        remove_entry(self.destination.join(relative).as_path(), self.shown(relative).as_path())?;
        self.made_here.remove(relative);
        self.stats.directories = self.stats.directories.saturating_sub(1);
        return Ok(());
    }
}

/// What:
///  Run one pass from `source` to `destination` (both existing directories).
/// Why:
///   See the module description;
/// the work stack replaces recursion.
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
    let mut pass: Pass<'_> = Pass { platform, source, destination, options, interrupt, stats: SyncStats::default(), links: HashMap::new(), made_here: HashSet::new() };
    let root_metadata: Metadata = MigrateError::from_io_result(std::fs::symlink_metadata(source), "examine", source)?;
    let mut stack: Vec<Work> = vec![Work::Finish(PathBuf::new(), root_metadata), Work::Visit(PathBuf::new())];
    while let Some(work) = stack.pop() {
        match work {
            Work::Visit(relative) => pass.visit(relative.as_path(), &mut stack)?,
            Work::Finish(relative, metadata) => {
                let Some(anchor) = pass.open_destination(relative.as_path())? else { continue };
                let shown: PathBuf = pass.shown(relative.as_path());
                attribute_result(preserve_on_descriptor(source.join(&relative).as_path(), anchor.file(), &metadata, !options.update_only, &mut pass.stats.losses)
                    , shown.as_path())?;
                // A directory this pass made whose whole content was left out is not a copy of anything, so it goes once
                //  its entries have been handled and none was copied.
                pass.remove_a_directory_that_holds_nothing(relative.as_path())?;
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
