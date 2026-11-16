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
//! Another command's stage container is left out of every pass,
//! and so is a folder that held nothing but one:
//! the container holds another worktree's ignored state while its own copy fills it,
//! and nothing names it in the tree this pass builds.
//! A container is a directory,
//! so a file or a link whose name merely begins the stage prefix is the source's own entry
//!  and is copied like any other,
//! and a deletion pass leaves a container in the destination where it stands,
//! because it belongs to the command filling it and not to this pass.
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
//!  Every entry is copied under a temporary name in its directory and then created at its own name,
//! so a kill
//!  never leaves a half-written file at a name a later pass would take for a newer one.
//! The pass never renames a finished copy over a name it did not create:
//! a name the destination already holds is freed by parking what stands there under a name of its own,
//! and the entry that was parked is judged at once and removed or put back,
//! so no write of this pass replaces an entry it did not examine.
//! The destination is reached
//!  through open directory descriptors (`subvolume_anchor.rs`),
//! so a directory replaced by a link during the pass
//!  redirects nothing.
//! An update-only pass never removes a destination entry that may hold newer work:
//! a directory,
//! or
//!  an entry of another type that is not older than the source,
//! is kept and listed.
//! An update-only pass leaves out a source entry the destination lacks and that is older than its cut-off,
//!  because that entry was deleted from the destination on purpose;
//! this program's own bookkeeping there is the one exception,
//! and
//!  `subvolume_wrapper_bookkeeping.rs` holds the rule for it.
//!
//! Every place in this pass where something is looked at and acted on later,
//! and what each does when the tree changed in between.
//! The rule is one: a look and an act must be of the same thing,
//! and no write of this pass replaces an entry it did not examine.
//! No place here can end in the deletion of the old tree while it reports nothing:
//! the two that name an entry are counted where the deletion looks
//! (`kept_at_the_destination` and `changed_while_passing`), they are written beside the checkout before the pass goes
//!  on, and the deletion reads that record rather than only its own memory.
//!
//! - The source folder's listing, then each child's metadata, then the type the child is handled by:
//! the metadata from the listing is what the pass decides by,
//! and an entry that is another kind when the copy opens it is counted as `changed_while_passing` rather than as a
//! file that vanished.
//! This one could delete a person's file, when a listed file became a directory: it is closed.
//! - The source entry, read before the copy and again after the copy has taken the destination name:
//! the device, the inode, the size, the modification time and the change time are compared,
//! and a source that moved at any point is copied again from what stands there now, a bounded number of times;
//! a name that will not stand still is counted as `changed_while_passing`, which keeps the old tree.
//! The change time is in this reading because a writer cannot set it:
//! `utimensat` sets the modification time and not that one.
//! No check before the publish can close the window, so the copy is judged against the source once it is in place.
//! This one could copy an entry the old tree no longer held and then let the replacement be deleted with it: it is
//!  closed.
//! - The destination name's absence when the pass decided to copy, then the placement of the finished copy:
//! the placement creates the name with `renameat2(RENAME_NOREPLACE)` and never replaces,
//! and a name that came to hold something is kept and counted as `kept_at_the_destination`.
//! This one could delete a person's file: it is closed.
//! - The destination entry the pass examined, then the name it takes:
//! the entry is parked under its own parked name with `renameat2(RENAME_NOREPLACE)`,
//! its reading is taken in the instruction after the parking,
//! the copy is created at the freed name, and the parked entry is then judged as the object it is:
//! the device and inode,
//!  the size and the modification time it had when the pass examined it,
//!  and the reading taken at the parking,
//!  which carries the change time that the parking itself moved and that a writer cannot set.
//! Only the object the pass examined, unchanged since that reading, is removed; anything else is put back,
//! over the copy this pass made there and nothing else,
//! and is counted as `kept_at_the_destination`.
//! A writer who keeps the size, puts the modification time back and lands inside the two instructions between the
//!  parking and that reading cannot be told from the object the pass examined by any field the kernel offers, and that is
//!  the one window this pass cannot close; every other write to the parked entry moves a field the reading compares.
//! A kill between the parking and the judgement leaves the parked entry at `<name>.cli-git-migrate-displaced`,
//! which no pass deletes: every pass names it, so the deletion waits for the person.
//! This one could discard a write made in place, and a file another command put at the name in the instant between
//!  the comparison and the rename: both are closed, because the judgement is of the object that was parked and the
//!  only name this pass writes over is the one it created.
//! - A destination file the pass examined where the source holds a folder, then the folder:
//! the same parking and judgement, and the folder is made at the freed name with `mkdir`,
//!  which fails rather than replaces when the name was taken again.
//! - The temporary name in the destination (`.cli-git-migrate-partial`):
//! the pass removes what stands there before it creates its own copy there.
//! The name belongs to the pass inside the tree it is building, so what is removed there is the pass's own leftover;
//! a person's file at that exact name in a tree this pass builds is removed.
//! No tree this pass builds is the checkout, and nothing of it can reach the old tree's deletion.
//! - The source file's metadata on the descriptor before and after the copy:
//! a file that changes while it is copied is copied again, three times at most, and then the run stops with an error
//! naming the file, so nothing is taken for a faithful copy.
//! - A file this user may not read:
//! left in the source and listed, and the deletion of the old tree waits for its own flag.
//! - The deletion pass, which lists the destination after the source:
//! it removes what the source does not hold, which is what a deletion pass is for,
//! it leaves another command's stage container where it stands,
//! and it leaves every parked entry where it stands.
//! It runs only into a tree this migration is building, so it cannot reach the old tree's deletion.
//! - A folder the source held nothing but another command's stage container in, then its removal:
//! `rmdir` refuses a folder that gained a file in between, so what another command wrote there stays.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stats = await syncTree({ platform, source, destination, delete: true, updateOnly: false, excluded: ['.git/index.lock'] });
//! ```

use crate::path_bytes::path_bytes_of;
use crate::subvolume_anchor::Anchor;
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_platform::Platform;
use crate::subvolume_sync_attrs::preserve_on_descriptor;
use crate::subvolume_wrapper_bookkeeping::{OwnBookkeeping, own_bookkeeping_of};
use crate::worktree_copy_prune::{holds_only_private_stages, names_a_private_stage};
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
                self.uncarried(relative, Uncarried::Conflict);
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
            let child: PathBuf = relative.join(entry.file_name());
            // Another command's stage container holds the whole ignored state of a worktree its own copy is filling, so
            //  nothing of it is copied. Its name stays in `names`, so the delete pass leaves the container that is there
            //  alone: it is that command's to remove, not this one's. A container is a directory, so a file or a link whose
            //  name merely begins the prefix is the source's own entry and is copied like any other.
            let is_a_container: bool = match entry.file_type() {
                Ok(kind) => kind.is_dir() && names_a_private_stage(path_bytes_of(child.as_path())),
                Err(_unknown) => false,
            };
            if is_a_container {
                self.stage_holders.insert(relative.to_path_buf());
                continue;
            }
            if self.options.excluded.contains(&child) {
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
        // A parked entry is what a replacement this pass parks and has not judged yet: a kill leaves it, and nothing
        //  here deletes or renames it, so the person is told and the old tree is kept.
        self.parked_entries_in(relative);
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
            //  it where it is. The name a source container has is in `names` already; this is the container the source does not
            //  hold, which is a copy another command is filling into this destination right now.
            if names_a_private_stage(relative.join(item.file_name()).as_os_str().as_encoded_bytes()) {
                continue;
            }
            // A parked copy is somebody's until a pass judges it, and a kill can leave one whose name the source no
            //  longer has: it stays, and the walk has already named it. Only the record says which names are ours.
            if self.is_a_recorded_parked_copy(relative.join(item.file_name()).as_path()) {
                continue;
            }
            remove_entry(directory.child(item.file_name().as_os_str()).as_path(), self.shown(relative).join(item.file_name()).as_path())?;
            self.stats.deleted += 1;
        }
        return Ok(());
    }

    /// What:
    ///  Forget the destination folder this pass made for a source folder that held nothing but another command's stage
    ///  container.
    /// Why:
    ///   No copy carries a container,
    /// so a folder that held nothing else is no state of the source either;
    /// a conversion that
    ///  kept it would hand the new worktree an empty folder that nothing names.
    /// Only a folder this pass made is ever
    ///  forgotten,
    /// so nothing that was in the destination before the pass leaves,
    /// and a folder another process wrote
    ///  into meanwhile stays with its content,
    /// because that is somebody's work.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// forgetsAFolderOfStageContainersOnly('.claude/worktrees'); // true when the source held only a container there
    /// ```
    fn forgets_a_folder_of_stage_containers_only(&mut self, relative: &Path) -> Result<bool, MigrateError> {
        // A tree with no stage container in it is nearly every tree, and the question costs nothing there: the source
        //  folders are only looked at when a container was seen in one of them.
        let mut seen_a_stage_holder: bool = false;
        for holder in &self.stage_holders {
            if holder.starts_with(relative) {
                seen_a_stage_holder = true;
                break;
            }
        }
        if !seen_a_stage_holder {
            return Ok(false);
        }
        let source_directory: PathBuf = self.source.join(relative);
        if !holds_only_private_stages(source_directory.as_path()) {
            return Ok(false);
        }
        let destination: PathBuf = self.destination.join(relative);
        return match std::fs::remove_dir(&destination) {
            Ok(()) => Ok(true),
            // The folder holds something another process wrote in the moment since it was made, or was replaced or taken:
            //  none of it is this pass's to remove, so it stays.
            Err(error)
                if error.kind() == std::io::ErrorKind::DirectoryNotEmpty
                    || error.kind() == std::io::ErrorKind::NotFound
                    || error.kind() == std::io::ErrorKind::NotADirectory => Ok(false),
            Err(error) => Err(MigrateError::io("remove the folder the source held nothing of", self.shown(relative).as_path(), &error)),
        };
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
        // Whether this entry is one of this program's notices that the cut-off would otherwise leave out. It is
        //  counted as carried only once the copy placed it, so a notice the destination turns out to hold something else
        //  at is reported as what it is.
        let mut carried_as_notice: bool = false;
        if existing.is_none() && self.options.update_only && older_than_since {
            // What a person or Git put there is what this cut-off holds the pass to; this program's own bookkeeping is
            //  named once, by its writers, in `subvolume_wrapper_bookkeeping.rs`, and judged by the kind this walk
            //  listed rather than by a second look at the name.
            match own_bookkeeping_of(self.source, relative.as_path(), &metadata) {
                // A record of one of this program's own processes:
                // it names that process and nothing a person wrote, and the checkout sweeps such records itself, so it
                //  is left out and counted apart from a person's entries.
                Some(OwnBookkeeping::LeftOut) => {
                    self.stats.holder_records_left_out.push(relative.clone());
                    return Ok(());
                }
                // A notice this program wrote for the person: carried over although it is older than the cut-off, so
                //  what it says reaches the checkout instead of going with the old tree. A folder is carried silently,
                //  because it holds no words of its own; every file it holds is judged here on its own.
                Some(OwnBookkeeping::CarriedOver) => carried_as_notice = !metadata.file_type().is_dir(),
                // A person's entry, or one this program cannot prove it wrote: left out, as it always was, so the
                //  deletion of the old tree does not go without a word about it.
                None => {
                    self.uncarried(relative.as_path(), Uncarried::LeftOut);
                    return Ok(());
                }
            }
        }
        let file_type: std::fs::FileType = metadata.file_type();
        if file_type.is_dir() {
            let mut made_by_this_hand: bool = false;
            if let Some(found) = &existing && !found.is_dir() {
                if !self.replace_allowed(relative.as_path(), found, &metadata) {
                    return Ok(());
                }
                // The file the pass examined is parked and judged, and the new directory is made at the freed name.
                if self.publish_directory(directory, relative.as_path(), found)? == Published::KeptTheDestinationEntry {
                    self.record_outcome(relative.clone(), Copied::KeptTheDestinationEntry, carried_as_notice);
                    return Ok(());
                }
                made_by_this_hand = true;
            }
            let is_directory_now: bool = made_by_this_hand
                || match existing.as_ref() {
                    Some(found) => found.is_dir(),
                    None => false,
                };
            if !is_directory_now {
                MigrateError::from_io_result(std::fs::DirBuilder::new().mode(0o700).create(&destination), "create the directory", self.shown(relative.as_path()).as_path())?;
            }
            self.stats.directories += 1;
            stack.push(Work::Finish { relative: relative.clone(), metadata, created: !is_directory_now });
            stack.push(Work::Visit(relative));
            return Ok(());
        }
        let outcome: Copied = if file_type.is_symlink() {
            self.copy_symlink(directory, &relative, &metadata, existing)?
        } else if file_type.is_fifo() {
            self.copy_fifo(directory, &relative, &metadata, existing)?
        } else if file_type.is_file() {
            self.copy_file(directory, &relative, &metadata, existing)?
        } else {
            self.stats.special_skipped.push(relative);
            return Ok(());
        };
        self.record_outcome(relative, outcome, carried_as_notice);
        return Ok(());
    }

    /// What:
    ///  Record and report what the copying of one source entry came to.
    /// Why:
    ///   Only an entry the destination now holds is a notice the pass carried;
    ///  an entry the destination came to hold something else at,
    ///  or one that was another kind of entry when the copy reached it,
    ///  is a fact the person must hear before the old tree is deleted,
    ///  so it is counted where the deletion looks.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// recordOutcome(relative, outcome, carriedAsNotice);
    /// ```
    fn record_outcome(&mut self, relative: PathBuf, outcome: Copied, carried_as_notice: bool) {
        match outcome {
            Copied::Placed => {
                if carried_as_notice {
                    self.stats.notices_carried.push(relative);
                }
            }
            // An entry the pass kept at the destination or found changed is recorded where it was learned, in the copy
            //  method that learned it, so nothing is counted twice and the record is durable before the pass goes on.
            Copied::KeptTheDestinationEntry | Copied::ChangedKind | Copied::LeftAsItIs | Copied::Vanished | Copied::Unreadable => {}
        }
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
pub fn sync_tree<'a>(
    platform: &'a dyn Platform,
    source: &'a Path,
    destination: &'a Path,
    options: SyncOptions<'a>,
    interrupt: &'a Interrupt,
    note_uncarried: Option<&'a mut dyn FnMut(&Path, Uncarried)>,
    parking: Option<&'a mut dyn Parking>,
) -> Result<SyncStats, MigrateError> {
    let mut pass: Pass<'a> = Pass { parking, note_uncarried, platform, source, destination, options, interrupt, stats: SyncStats::default(), links: HashMap::new(), stage_holders: HashSet::new() };
    let root_metadata: Metadata = MigrateError::from_io_result(std::fs::symlink_metadata(source), "examine", source)?;
    let mut stack: Vec<Work> = vec![Work::Finish { relative: PathBuf::new(), metadata: root_metadata, created: false }, Work::Visit(PathBuf::new())];
    while let Some(work) = stack.pop() {
        match work {
            Work::Visit(relative) => pass.visit(relative.as_path(), &mut stack)?,
            Work::Finish { relative, metadata, created } => {
                if created && pass.forgets_a_folder_of_stage_containers_only(relative.as_path())? {
                    continue;
                }
                let Some(anchor) = pass.open_destination(relative.as_path())? else { continue };
                let shown: PathBuf = pass.shown(relative.as_path());
                attribute_result(preserve_on_descriptor(source.join(&relative).as_path(), anchor.file(), &metadata, !options.update_only, &mut pass.stats.losses)
                    , shown.as_path())?;
            }
        }
    }
    pass.stats.unreadable.sort();
    pass.stats.conflicts.sort();
    pass.stats.conflicts.dedup();
    pass.stats.holder_records_left_out.sort();
    pass.stats.notices_carried.sort();
    pass.stats.kept_at_the_destination.sort();
    pass.stats.parked_copies.sort();
    pass.stats.changed_while_passing.sort();
    return Ok(pass.stats);
}

/// Tree sync controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_sync_tests.rs"]
mod tests;
