//! What:
//!  The pieces every step of a tree sync shares:
//! the options and the statistics,
//! the work stack's entries,
//! the open destination
//!  directory,
//! the state of one pass,
//! and the small checks and removals that compare or clear one entry.
//! Why:
//!  `subvolume_sync.rs` holds the walk and `subvolume_sync_copy_one_entry.rs` holds the copying of one entry;
//! both need these
//!  types and checks,
//! and neither may depend on the other's file,
//! so they live in a file that depends on neither.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type SyncOptions = { delete: boolean; updateOnly: boolean; excluded: string[] };
//! ```

use crate::subvolume_anchor::Anchor;
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_platform::Platform;
use crate::subvolume_remove::remove_tree;
use crate::subvolume_sync_attrs::{AttributeFailure, AttributeLosses};
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs::Metadata;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::path::{Path, PathBuf};

/// The name an entry is written under before it is renamed to its own,
/// unless the source has an entry of that name.
pub(crate) const PARTIAL_NAME: &str = ".cli-git-migrate-partial";

/// How many times a file that keeps changing under the copy is copied before the pass gives up.
pub(crate) const COPY_ATTEMPTS: u32 = 3;

/// The start of the name an entry the pass replaces is parked under while its own copy takes the name.
/// The rest of the name is a digest of the entry's path,
/// so the name is always short enough to stand beside any file name a checkout can hold,
/// and a person's own file is never taken for one of these:
/// the record of parked copies beside the checkout is what tells them apart.
pub(crate) const PARKED_PREFIX: &str = ".cli-git-migrate-parked-";

/// What:
///  The name an entry is parked under while the pass replaces it:
///  a fixed short prefix and a digest of the entry's own path.
/// Why:
///   A name derived from the entry's own is too long for the longest file name a checkout can hold,
///  and a name's ending is no way to tell this program's parked copy from a person's file.
///  The record of parked copies beside the checkout names each of them with the file it is a copy of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parkedName = (relative) => `.cli-git-migrate-parked-${digest(relative)}`;
/// ```
pub(crate) fn parked_name(relative: &Path) -> OsString {
    let mut text: OsString = OsString::from(PARKED_PREFIX);
    text.push(crate::service_instance::digest_of_bytes(relative.as_os_str().as_encoded_bytes()).as_str());
    return text;
}

/// What:
///  What the pass tells about the entries it parks,
///  and what it knows of those an earlier run parked and no judgement settled.
/// Why:
///  The record is written before the entry is parked,
///  so a kill at any instruction after that leaves the knowledge that the name holds this program's parked copy,
///  and a copy a person removes costs its line at the next pass.
///  A pass that is given no record (the conversion of a worktree, a control) reports no parked copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Parking { aboutToPark(original, parked); judged(parked); known(): Map<string, string>; }
/// ```
pub trait Parking {
    /// Record that the entry at `original` is about to be parked at `parked`,
    /// before the parking happens.
    fn about_to_park(&mut self, original: &Path, parked: &Path);
    /// Drop the record of `parked`:
    /// the entry it holds was judged and is gone from that name.
    fn judged(&mut self, parked: &Path);
    /// Every parked entry the record names,
    /// by the name it stands under and the path it is a copy of.
    fn known(&self) -> &std::collections::BTreeMap<PathBuf, PathBuf>;
}

/// What:
///  The kind of finding a pass learns about an entry it did not carry,
///  which forbids the deletion of the old tree until a person answers for it.
/// Why:
///   The record beside the checkout is written as each one is learned,
///  so a kill at any instruction after that loses the run's memory and not the fact:
///  the kinds are kept apart because the refusal names each of them in its own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Uncarried = 'left-out' | 'conflict' | 'kept-at-the-destination' | 'changed-while-passing' | 'unreadable';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Uncarried {
    /// A source entry the destination lacks that is older than the pass's cut-off.
    LeftOut,
    /// A destination entry of another kind,
    /// or one that is not older than the source,
    /// which an update-only pass keeps.
    Conflict,
    /// An entry the pass did not place because the destination holds something it did not examine,
    /// or because a replacement it parked was not judged.
    KeptAtTheDestination,
    /// A source entry that was another kind of entry when the copy reached it,
    /// or a name that would not stand still.
    ChangedWhilePassing,
    /// A file this user may not read,
    /// which stays in the source.
    Unreadable,
}

/// What:
///  What one pass may do.
///  `&'a [PathBuf]` borrows the list of excluded paths for the pass.
/// Why:
///   The excluded paths (relative to the tree roots) are neither copied nor deleted:
/// the lock files the migration
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
    /// Copy a file only when the destination lacks it or has an older one;
    /// never delete.
    pub update_only: bool,
    /// Paths relative to the roots that the pass leaves alone.
    pub excluded: &'a [PathBuf],
    /// Never replace:
    /// a source entry the destination holds is left as it is unless the two are equal,
    ///  and one that differs is counted as an entry this pass did not place.
    /// The carry back of a rollback is such a pass:
    ///  the tree it writes into is the one the person keeps,
    ///  so a modification time is no reason to delete a byte of it,
    ///  and the proof against the subvolume the rollback deletes names what differs.
    pub never_replaces: bool,
    /// Update-only pass:
    /// a source entry the destination lacks is created only when it was modified at or after this
    ///  second (since the epoch);
    /// an older one was deleted from the destination on purpose.
    /// `None` creates them all.
    pub created_since: Option<i64>,
}

/// What:
///  What one pass did.
/// Why:
///   The report prints the counts,
/// and the unreadable and skipped lists are the loud part of it.
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
    /// Of those,
    /// files whose data blocks are shared with the original.
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
    /// Files this user may not read,
    /// left in the source tree;
    /// relative paths.
    pub unreadable: Vec<PathBuf>,
    /// Sockets and device nodes,
    /// which are not recreated;
    /// relative paths.
    pub special_skipped: Vec<PathBuf>,
    /// Attributes that could not be set.
    pub losses: AttributeLosses,
    /// Update-only pass:
    /// destination entries kept because they may hold newer work and the source has an entry of
    ///  another type there,
    /// or the destination directory was replaced;
    /// relative paths.
    pub conflicts: Vec<PathBuf>,
    /// Files that changed while they were copied and had to be copied again.
    pub recopied: u64,
    /// Source entries the destination lacks that were left out because they are older than `created_since`;
    /// relative paths.
    pub left_out: Vec<PathBuf>,
    /// The source records of the processes this program started that were left out because the destination lacks
    ///  them and they are older than `created_since`.
    /// They are kept apart from `left_out` because they hold nothing a person wrote:
    /// relative paths.
    pub holder_records_left_out: Vec<PathBuf>,
    /// The source notices this program shows a person that were carried into the destination although it lacked them
    ///  and they are older than `created_since`,
    /// so what they say is not lost with the old tree;
    /// relative paths.
    pub notices_carried: Vec<PathBuf>,
    /// Source entries this pass did not place because the destination came to hold an entry it did not see when it
    ///  looked.
    /// That entry stays and this one is reported, so the deletion of the old tree waits for the person:
    /// relative paths.
    pub kept_at_the_destination: Vec<PathBuf>,
    /// The parked copies a directory held whose replacement no pass finished,
    /// each by the name it stands under and the path it is a copy of,
    /// in relative paths.
    pub parked_copies: Vec<(PathBuf, PathBuf)>,
    /// Source entries that were another kind of entry when the copy reached them,
    /// so nothing was written at their name.
    /// They are reported for the same reason as `kept_at_the_destination`;
    /// relative paths.
    pub changed_while_passing: Vec<PathBuf>,
    /// The first few files whose content was copied,
    /// relative paths,
    /// for the report.
    pub copied_sample: Vec<PathBuf>,
}

/// How many copied paths a pass keeps for its report.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub(crate) const SAMPLE_SIZE: usize = 20;

/// One unit of the work stack.
pub(crate) enum Work {
    /// List a directory and handle its children.
    Visit(PathBuf),
    /// Apply a directory's own attributes after its children,
    /// and forget a folder the source kept nothing but another command's stage container in.
    Finish {
        relative: PathBuf,
        metadata: Metadata,
        /// Whether this pass made the destination directory,
        /// so that a folder the source held nothing of is not left behind.
        created: bool,
    },
}

/// The destination directory being filled and the temporary name used inside it.
pub(crate) struct Directory {
    /// The destination directory,
    /// held open.
    pub(crate) anchor: Anchor,
    /// The temporary name for the entry being written.
    pub(crate) temporary: OsString,
}

impl Directory {
    /// The path of a name in the directory.
    pub(crate) fn child(&self, name: &OsStr) -> PathBuf {
        return self.anchor.child(name);
    }

    /// The path of the temporary entry.
    pub(crate) fn partial(&self) -> PathBuf {
        return self.anchor.child(self.temporary.as_os_str());
    }
}

/// The state of one pass.
pub(crate) struct Pass<'a> {
    /// The record of the entries the pass parks and of those an earlier run parked,
    /// when the caller keeps one.
    pub(crate) parking: Option<&'a mut dyn Parking>,
    /// Told every entry the pass does not carry, as it learns it,
    /// so the record is durable before the pass goes on to the next entry;
    /// nothing when the caller keeps no record.
    pub(crate) note_uncarried: Option<&'a mut dyn FnMut(&Path, Uncarried)>,
    pub(crate) platform: &'a dyn Platform,
    pub(crate) source: &'a Path,
    pub(crate) destination: &'a Path,
    pub(crate) options: SyncOptions<'a>,
    pub(crate) interrupt: &'a Interrupt,
    pub(crate) stats: SyncStats,
    /// Relative paths of the copies made of each source file that has several names,
    /// by (device,
    /// inode).
    /// Several copies
    ///  exist when the destination cannot link across a boundary (two subvolumes).
    pub(crate) links: HashMap<(u64, u64), Vec<PathBuf>>,
    /// The source folders that held another command's stage container,
    /// so a folder that holds nothing else can be
    ///  recognised when its own turn comes and is not left in the destination.
    /// Empty on nearly every pass,
    /// which is what
    ///  keeps the question free for a tree with no container in it.
    pub(crate) stage_holders: HashSet<PathBuf>,
}

/// Whether the destination entry already equals the source file:
/// a regular file with the same size,
/// modification
///  time and permission bits (owner is not compared,
/// because an unprivileged copy cannot set a foreign owner).
pub(crate) fn file_is_equal(copy: &Metadata, original: &Metadata) -> bool {
    return copy.is_file()
        && copy.len() == original.len()
        && copy.mtime() == original.mtime()
        && copy.mtime_nsec() == original.mtime_nsec()
        && copy.mode() & 0o7777 == original.mode() & 0o7777;
}

/// Whether the first entry's modification time is later than the second's.
pub(crate) fn is_newer(left: &Metadata, right: &Metadata) -> bool {
    return (left.mtime(), left.mtime_nsec()) > (right.mtime(), right.mtime_nsec());
}

/// Whether a file looks the same before and after it was copied:
/// size,
/// modification time and change time.
pub(crate) fn is_settled(before: &Metadata, after: &Metadata) -> bool {
    return before.len() == after.len()
        && (before.mtime(), before.mtime_nsec()) == (after.mtime(), after.mtime_nsec())
        && (before.ctime(), before.ctime_nsec()) == (after.ctime(), after.ctime_nsec());
}

/// Remove a destination entry of any kind.
pub(crate) fn remove_entry(path: &Path, shown: &Path) -> Result<(), MigrateError> {
    return MigrateError::from_io_result(remove_tree(path), "remove", shown);
}

/// Remove a path that may not exist.
pub(crate) fn remove_if_present(path: &Path, shown: &Path) -> Result<(), MigrateError> {
    return match std::fs::symlink_metadata(path) {
        Ok(_) => remove_entry(path, shown),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(MigrateError::io("examine", shown, &error)),
    };
}

/// The success of an attribute step as it is,
/// its failure as the migration's error for `shown`.
pub(crate) fn attribute_result<T>(result: Result<T, AttributeFailure>, shown: &Path) -> Result<T, MigrateError> {
    return match result {
        Ok(value) => Ok(value),
        Err(failure) => Err(attribute_error(failure, shown)),
    };
}

/// An attribute failure as the migration's error for `shown`.
pub(crate) fn attribute_error(failure: AttributeFailure, shown: &Path) -> MigrateError {
    return MigrateError::io(failure.action, shown, &failure.error);
}

/// Whether opening a destination directory failed because it is no longer a directory of the tree.
pub(crate) fn is_replaced(error: &std::io::Error) -> bool {
    return matches!(error.raw_os_error(), Some(libc::ELOOP | libc::ENOTDIR | libc::ENOENT));
}

/// What:
///  Settle a read of the source that may find the entry gone:
/// a vanished entry is counted and skipped,
/// any other failure
///  is returned.
/// Why:
///   A source that is in use changes while it is copied.
/// An entry removed between the listing and the read is not an
///  error of the copy,
/// but every other failure (a permission,
/// a read error) is,
/// and the report counts the first kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await read(); } catch (error) { if (error.code === 'ENOENT') { vanished += 1; return undefined; } throw error; }
/// ```
pub(crate) fn vanished_is_counted<T>(outcome: std::io::Result<T>, vanished: &mut u64) -> std::io::Result<Option<T>> {
    return match outcome {
        Ok(found) => Ok(Some(found)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            *vanished += 1;
            Ok(None)
        }
        Err(error) => Err(error),
    };
}

//region A test-only seam: one callback at the instant a replaced name stands free
#[cfg(test)]
thread_local! {
    // The callback a control installs to act while a replaced name stands free, between the parking of the entry the
    // pass examined and the creation of its copy; nothing in a release build reads or sets it.
    static FREED_NAME_WATCH: std::cell::RefCell<Option<Box<dyn FnMut(&Path)>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Install `watch` to run for the destination name a pass has just freed by parking what stood there,
///  or clear the callback with `None`.
/// Why:
///   The window between parking an entry and creating its copy is the one a control of the replacement has to stand in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchTheFreedName((path) => { ... });
/// ```
#[cfg(test)]
pub(crate) fn watch_the_freed_name(watch: Option<Box<dyn FnMut(&Path)>>) {
    FREED_NAME_WATCH.with(|slot| {
        *slot.borrow_mut() = watch;
    });
}

/// Run the installed callback for the freed name at `path`,
/// if a control installed one.
#[cfg(test)]
fn note_the_freed_name(path: &Path) {
    FREED_NAME_WATCH.with(|slot| {
        // The borrow ends before the callback runs nothing of this module's, so it cannot re-enter the slot.
        let mut installed = slot.borrow_mut();
        if let Some(watch) = installed.as_mut() {
            watch(path);
        }
    });
}
//endregion

//region A test-only seam: one callback just before the entry a pass parked is judged
#[cfg(test)]
thread_local! {
    // The callback a control installs to act on the parked entry after the pass created its copy there and before it
    // judges what it parked; nothing in a release build reads or sets it.
    static BEFORE_THE_JUDGEMENT_WATCH: std::cell::RefCell<Option<Box<dyn FnMut(&Path)>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Install `watch` to run with the parked path after a pass has created its copy at the freed name and before it judges
///  what it parked,
///  or clear the callback with `None`.
/// Why:
///   The window between the parking and the judgement is where a writer can still change the entry the pass is about to
///  remove, and a control of that window has to stand in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchBeforeTheJudgement((parked) => { ... });
/// ```
#[cfg(test)]
pub(crate) fn watch_before_the_judgement(watch: Option<Box<dyn FnMut(&Path)>>) {
    BEFORE_THE_JUDGEMENT_WATCH.with(|slot| {
        *slot.borrow_mut() = watch;
    });
}

/// Run the installed callback for the parked path,
/// if a control installed one.
#[cfg(test)]
fn note_before_the_judgement(path: &Path) {
    BEFORE_THE_JUDGEMENT_WATCH.with(|slot| {
        // The borrow ends before the callback runs nothing of this module's, so it cannot re-enter the slot.
        let mut installed = slot.borrow_mut();
        if let Some(watch) = installed.as_mut() {
            watch(path);
        }
    });
}
//endregion

//region A test-only seam: one callback after the pass removed its own copy and before it puts the parked entry back
#[cfg(test)]
thread_local! {
    // The callback a control installs to act on the freed name between the removal of the pass's own copy and the
    // put-back of the parked entry; nothing in a release build reads or sets it.
    static BEFORE_THE_PUT_BACK_WATCH: std::cell::RefCell<Option<Box<dyn FnMut(&Path)>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Install `watch` to run with the destination name after a pass removed the copy it made there and before it puts the
///  entry it parked back,
///  or clear the callback with `None`.
/// Why:
///   That instant is where another command's file can appear at the name, and a control of the put-back has to stand in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// watchBeforeThePutBack((freed) => { ... });
/// ```
#[cfg(test)]
pub(crate) fn watch_before_the_put_back(watch: Option<Box<dyn FnMut(&Path)>>) {
    BEFORE_THE_PUT_BACK_WATCH.with(|slot| {
        *slot.borrow_mut() = watch;
    });
}

/// Run the installed callback for the freed destination name,
/// if a control installed one.
#[cfg(test)]
fn note_before_the_put_back(path: &Path) {
    BEFORE_THE_PUT_BACK_WATCH.with(|slot| {
        // The borrow ends before the callback runs nothing of this module's, so it cannot re-enter the slot.
        let mut installed = slot.borrow_mut();
        if let Some(watch) = installed.as_mut() {
            watch(path);
        }
    });
}
//endregion

impl Pass<'_> {
    /// Note one entry the pass does not carry,
    /// and count it where the report and the deletion read it.
    pub(crate) fn uncarried(&mut self, relative: &Path, kind: Uncarried) {
        // One entry is counted once per kind: a copy that is retried learns the same change again, and the counts the
        //  report reads would otherwise say two.
        let counted: bool = match kind {
            Uncarried::LeftOut => self.stats.left_out.iter().any(|path| return path == relative),
            Uncarried::Conflict => self.stats.conflicts.iter().any(|path| return path == relative),
            Uncarried::KeptAtTheDestination => self.stats.kept_at_the_destination.iter().any(|path| return path == relative),
            Uncarried::ChangedWhilePassing => self.stats.changed_while_passing.iter().any(|path| return path == relative),
            Uncarried::Unreadable => self.stats.unreadable.iter().any(|path| return path == relative),
        };
        if counted {
            return;
        }
        // A file this user may not read has its own record and its own flag, written after every pass and read with it,
        //  so it is counted here and not written into the record of the other kinds.
        if kind != Uncarried::Unreadable
            && let Some(note) = self.note_uncarried.as_mut()
        {
            note(relative, kind);
        }
        match kind {
            Uncarried::LeftOut => self.stats.left_out.push(relative.to_path_buf()),
            Uncarried::Conflict => self.stats.conflicts.push(relative.to_path_buf()),
            Uncarried::KeptAtTheDestination => self.stats.kept_at_the_destination.push(relative.to_path_buf()),
            Uncarried::ChangedWhilePassing => self.stats.changed_while_passing.push(relative.to_path_buf()),
            Uncarried::Unreadable => self.stats.unreadable.push(relative.to_path_buf()),
        };
    }

    /// The destination path of a relative path,
    /// for messages.
    pub(crate) fn shown(&self, relative: &Path) -> PathBuf {
        return self.destination.join(relative);
    }

    /// Whether a destination entry that is not a match for the source entry may be replaced.
    /// An update-only pass keeps a
    ///  directory and any entry not older than the source,
    /// and lists it.
    pub(crate) fn replace_allowed(&mut self, relative: &Path, found: &Metadata, original: &Metadata) -> bool {
        if !self.options.update_only {
            return true;
        }
        if found.is_dir() || !is_newer(original, found) {
            self.uncarried(relative, Uncarried::Conflict);
            return false;
        }
        return true;
    }

    /// What:
    ///  Put the finished temporary entry at the destination name,
    ///  replacing only the entry the pass examined and nothing else.
    ///  `examined` is what the pass found at that name when it decided to copy.
    /// Why:
    ///   A rename onto a name replaces whatever stands there, in every form the kernel offers,
    ///  so the pass frees the name by parking what stands there and creates its copy where it holds the name.
    ///  The parked entry is then judged against what the pass examined:
    ///  the object the pass saw is superseded and removed,
    ///  and anything else goes back to its name,
    ///  over the copy this pass made there and nothing else.
    ///  A write in place, and a replacement made while the copy ran, are therefore never discarded,
    ///  and a kill leaves the parked entry at a name no pass deletes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const published = pass.publish(directory, relative, examined); // 'placed' | 'kept-the-destination-entry'
    /// ```
    pub(crate) fn publish(&mut self, directory: &Directory, relative: &Path, examined: Option<&Metadata>) -> Result<Published, MigrateError> {
        let name: &OsStr = relative.file_name().unwrap_or_default();
        let destination: PathBuf = directory.child(name);
        let partial: PathBuf = directory.partial();
        let shown: PathBuf = self.shown(relative);
        let Some(examined_here) = examined else {
            return place_where_nothing_was(partial.as_path(), destination.as_path(), shown.as_path());
        };
        return self.replace_the_examined(directory, relative, examined_here, Creating::FromTheTemporaryName);
    }

    /// What:
    ///  Put a new directory where the pass examined an entry that is not one,
    ///  by the same parking and judging as `publish`.
    /// Why:
    ///   A directory cannot be moved over a name either,
    ///  and the file the pass examined there is somebody's until the judgement says it is the object it examined.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const placed = pass.publishDirectory(directory, relative, examined);
    /// ```
    pub(crate) fn publish_directory(&mut self, directory: &Directory, relative: &Path, examined: &Metadata) -> Result<Published, MigrateError> {
        return self.replace_the_examined(directory, relative, examined, Creating::Directory);
    }

    /// What:
    ///  Replace the entry the pass examined at `relative` with what `creating` makes,
    ///  parking what stands there and judging it afterwards.
    /// Why:
    ///   See `publish`.
    ///  Nothing here removes or renames an entry the pass did not examine:
    ///  the only name it writes over is the one it just created.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pass.replaceTheExamined(directory, relative, examined, creating);
    /// ```
    fn replace_the_examined(&mut self, directory: &Directory, relative: &Path, examined: &Metadata, creating: Creating) -> Result<Published, MigrateError> {
        let name: &OsStr = relative.file_name().unwrap_or_default();
        let destination: PathBuf = directory.child(name);
        let shown: PathBuf = self.shown(relative);
        let parked: PathBuf = directory.child(parked_name(relative).as_os_str());
        let parked_relative: PathBuf = relative.with_file_name(parked_name(relative).as_os_str());
        // The record is written before the parking, so a kill at any instruction after it leaves the knowledge that
        //  this name holds the pass's own parked copy and not a person's file.
        if let Some(parking) = self.parking.as_mut() {
            parking.about_to_park(relative, parked_relative.as_path());
        }
        // The name is freed by parking what stands there, and never by removing it.
        match crate::rename_that_never_replaces::rename_noreplace(destination.as_path(), parked.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // A parked entry already stands there: nothing of it is this pass's to judge, so nothing is freed and
                //  the source's copy is not placed. Both are named.
                self.uncarried(parked_relative.as_path(), Uncarried::KeptAtTheDestination);
                remove_if_present(directory.partial().as_path(), shown.as_path())?;
                return Ok(Published::KeptTheDestinationEntry);
            }
            Err(error) => return Err(MigrateError::io("park the entry the destination holds", shown.as_path(), &error)),
        }
        // The pass may be stopped here by a control, with the entry parked and nothing yet judged.
        self.interrupt.check("parked")?;
        // The baseline of the parked entry is read here, in the instruction after the parking:
        //  the parking itself moves the change time,
        //  so this reading is what "unchanged since the pass parked it" means,
        //  and a writer who changes the parked entry after it moves a field the pass can see.
        let parked_first: Option<Metadata> = std::fs::symlink_metadata(parked.as_path()).ok();
        #[cfg(test)]
        note_the_freed_name(destination.as_path());
        // The name is free here, and only here, so the copy is created and never renamed over anything.
        let created: std::io::Result<()> = match creating {
            Creating::FromTheTemporaryName => crate::rename_that_never_replaces::rename_noreplace(directory.partial().as_path(), destination.as_path()),
            Creating::Directory => std::fs::DirBuilder::new().mode(0o700).create(&destination),
        };
        match created {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // Somebody put a file at the freed name while it stood free. It stays, and the entry the pass parked is
                //  named for the person rather than removed or renamed over that file.
                self.uncarried(parked_relative.as_path(), Uncarried::KeptAtTheDestination);
                remove_if_present(directory.partial().as_path(), shown.as_path())?;
                return Ok(Published::KeptTheDestinationEntry);
            }
            Err(error) => {
                // The pass is stopping. The parked entry stays where it is, and the next command names it.
                return Err(MigrateError::io("create the copy at the freed name", shown.as_path(), &error));
            }
        }
        // The reading of the copy this pass made at the name, which is what tells it from a file another command put there
        //  before the put-back.
        let ours: Metadata = MigrateError::from_io_result(std::fs::symlink_metadata(destination.as_path()), "examine", shown.as_path())?;
        #[cfg(test)]
        note_before_the_judgement(parked.as_path());
        // The parked entry is judged as the object it is: the object the pass examined, and unchanged since the reading
        //  the parking was followed by. A write in place that keeps the size and puts the modification time back moves
        //  the change time, which no writer can set, so it is caught here; a write nobody can tell from the parked
        //  object keeps the file below.
        if parked_first.is_none() {
            // What the pass parked is gone: whoever held it removed it, so there is nothing of theirs left to keep.
            return Ok(Published::Placed);
        }
        let parked_now: Option<Metadata> = std::fs::symlink_metadata(parked.as_path()).ok();
        let superseded: bool = match (parked_first.as_ref(), parked_now.as_ref()) {
            (Some(first), Some(now)) => unchanged_since(first, examined) && same_reading(first, now),
            _ => false,
        };
        if superseded {
            remove_entry(parked.as_path(), parked_relative.as_path())?;
            if let Some(parking) = self.parking.as_mut() {
                parking.judged(parked_relative.as_path());
            }
            return Ok(Published::Placed);
        }
        // Not the object the pass examined: it goes back to its name, and only the copy this pass made there goes.
        //  The name is read first: a file another command put there is that command's and stays, and the parked entry
        //  keeps its own name for the person to look at.
        let our_copy_is_there: bool = matches!(
            std::fs::symlink_metadata(destination.as_path()).ok().as_ref(),
            Some(found) if same_reading(found, &ours)
        );
        if !our_copy_is_there {
            return Ok(Published::KeptTheDestinationEntry);
        }
        remove_if_present(destination.as_path(), shown.as_path())?;
        #[cfg(test)]
        note_before_the_put_back(destination.as_path());
        // The put-back creates the name and never replaces what came to stand there since the removal above.
        match crate::rename_that_never_replaces::rename_noreplace(parked.as_path(), destination.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(Published::KeptTheDestinationEntry),
            Err(error) => return Err(MigrateError::io("put the entry the pass did not examine back", shown.as_path(), &error)),
        }
        if let Some(parking) = self.parking.as_mut() {
            parking.judged(parked_relative.as_path());
        }
        return Ok(Published::KeptTheDestinationEntry);
    }

    /// What:
    ///  Whether the entry at `relative` in the source is still the object the pass examined,
    ///  read by everything a writer cannot keep while changing it:
    ///  the device,
    ///  the inode,
    ///  the size,
    ///  the modification time and the change time.
    /// Why:
    ///   The source is only read by this pass,
    ///  so no act of the pass moves any of those fields,
    ///  and a write in place or a replacement of the name moves at least one of them.
    ///  The change time is the field a writer cannot set:
    ///  `utimensat` sets the modification time and not that one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const unchanged = (examined) => sameReading(statSync(source), examined); // dev, ino, size, mtimeNs, ctimeNs
    /// ```
    pub(crate) fn source_unchanged_since(&self, examined: &Metadata, relative: &Path) -> bool {
        return match std::fs::symlink_metadata(self.source.join(relative)) {
            Ok(now) => same_reading(&now, examined),
            Err(_gone) => false,
        };
    }

    /// Whether the source path still names the object the pass examined:
    /// a path another command replaced names another object,
    /// and what was copied from the old name is then not what stands there.
    pub(crate) fn source_path_still_names(&self, examined: &Metadata, relative: &Path) -> bool {
        return match std::fs::symlink_metadata(self.source.join(relative)) {
            Ok(found) => found.dev() == examined.dev() && found.ino() == examined.ino(),
            Err(_gone) => false,
        };
    }

    /// What:
    ///  The parked copies a directory holds,
    ///  whose replacements the pass that parked them did not finish,
    ///  named by the record of parked copies and never by a name's ending.
    /// Why:
    ///   A parked copy is somebody's until a pass judges it,
    ///  so every later pass names it and none deletes it;
    ///  and a person's own file whose name happens to begin like one is that person's and is not named.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const parked = parkedCopiesTheRecordNames(directory, relative); // [parked, original][]
    /// ```
    pub(crate) fn parked_entries_in(&mut self, relative: &Path) {
        let Some(parking) = self.parking.as_ref() else { return };
        let mut found: Vec<(PathBuf, PathBuf)> = Vec::new();
        for (parked, original) in parking.known() {
            if parked.parent() != Some(relative) {
                continue;
            }
            if std::fs::symlink_metadata(self.destination.join(parked)).is_ok() {
                found.push((parked.clone(), original.clone()));
            }
        }
        for one in found {
            if !self.stats.parked_copies.contains(&one) {
                self.stats.parked_copies.push(one);
            }
        }
    }

    /// Whether a destination name is a parked copy the record holds,
    /// which no pass deletes.
    pub(crate) fn is_a_recorded_parked_copy(&self, path: &Path) -> bool {
        return self.parking.as_ref().map(|parking| return parking.known().contains_key(path)).unwrap_or(false);
    }
}

/// What:
///  What the pass makes at a name it freed by parking what stood there.
/// Why:
///   The two are made differently,
///  and neither is renamed over anything:
///  a copy is moved into the free name,
///  a directory is made there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Creating = 'from-the-temporary-name' | 'directory';
/// ```
#[derive(Clone, Copy)]
enum Creating {
    /// A finished copy of a file, a link or a named pipe.
    FromTheTemporaryName,
    /// The directory the pass examined an entry that is not one for.
    Directory,
}

/// What:
///  What turning the finished temporary entry into the destination's name did.
/// Why:
///   The caller counts and reports what happened,
///  and only a placed entry is a copy the pass can say it carried.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Published = 'placed' | 'kept-the-destination-entry';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Published {
    /// The destination name now holds the copy.
    Placed,
    /// The destination came to hold an entry the pass did not see:
    /// it stays and the copy was dropped.
    KeptTheDestinationEntry,
}

/// What:
///  What the copying of one source entry came to,
///  which the walk reports and counts.
/// Why:
///   An entry the pass could not place,
///  or that was another kind of entry when the copy reached it,
///  is a fact about the old tree the person must hear before it is deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Copied = 'placed' | 'left-as-it-is' | 'vanished' | 'unreadable' | 'changed-kind' | 'kept-the-destination-entry';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Copied {
    /// The destination name holds the copy.
    Placed,
    /// The destination already held what it should,
    /// so the entry was left as it is.
    LeftAsItIs,
    /// The source entry is gone.
    Vanished,
    /// This user may not read the source entry.
    Unreadable,
    /// The source entry was another kind of entry when the copy reached it,
    /// so nothing was written at its name.
    ChangedKind,
    /// The destination came to hold an entry this pass did not see:
    /// it stays and the copy was dropped.
    KeptTheDestinationEntry,
}

/// What:
///  Put a finished copy at a name the pass found free,
///  and leave whatever came to stand there since.
/// Why:
///   Creating a name and never replacing one is the only write that cannot lose what another command put there;
///  a filesystem that cannot do it refuses the migration rather than replacing unseen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placed = placeWhereNothingWas(partial, destination); // renameat2(RENAME_NOREPLACE)
/// ```
pub(crate) fn place_where_nothing_was(partial: &Path, destination: &Path, shown: &Path) -> Result<Published, MigrateError> {
    let outcome: std::io::Result<()> = crate::rename_that_never_replaces::rename_noreplace(partial, destination);
    return match outcome {
        Ok(()) => Ok(Published::Placed),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            remove_if_present(partial, shown)?;
            Ok(Published::KeptTheDestinationEntry)
        }
        Err(error) if crate::rename_that_never_replaces::rename_unsupported(&error) => Err(MigrateError::Io {
            action: String::from("put a copy at a free name without replacing what stands there, which this filesystem cannot do"),
            path: shown.to_path_buf(),
            reason: error.to_string(),
        }),
        Err(error) => Err(MigrateError::io("put the finished copy in place", shown, &error)),
    };
}

/// What:
///  Whether the entry the pass judged is still the object it examined:
///  the same device and inode,
///  the same size,
///  and the same modification time.
/// Why:
///   A write in place keeps a file's device and inode and moves its size and its modification time,
///  so an identity of two numbers alone would call a written file the one it examined.
///  The change time is not compared:
///  parking an entry moves that time itself,
///  so comparing it would call every entry another object,
///  and the two fields above are the ones a write moves.
///  The entry judged here is the one the pass parked out of the name,
///  so the judgement is of the object that stood there at the instant the name was freed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unchanged = (found, examined) => found.dev === examined.dev && found.ino === examined.ino
///   && found.size === examined.size && found.mtimeNs === examined.mtimeNs;
/// ```
/// What:
///  Whether two readings describe the same object unchanged:
///  the device and inode,
///  the size,
///  the modification time and the change time.
/// Why:
///   This is the whole identity a writer cannot keep while changing a file,
///  and it is what two readings of one object have to agree on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = (a, b) => a.dev === b.dev && a.ino === b.ino && a.size === b.size && a.mtimeNs === b.mtimeNs && a.ctimeNs === b.ctimeNs;
/// ```
pub(crate) fn same_reading(left: &Metadata, right: &Metadata) -> bool {
    return left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && (left.mtime(), left.mtime_nsec()) == (right.mtime(), right.mtime_nsec())
        && (left.ctime(), left.ctime_nsec()) == (right.ctime(), right.ctime_nsec());
}

pub(crate) fn unchanged_since(found: &Metadata, examined: &Metadata) -> bool {
    return found.dev() == examined.dev()
        && found.ino() == examined.ino()
        && found.len() == examined.len()
        && (found.mtime(), found.mtime_nsec()) == (examined.mtime(), examined.mtime_nsec());
}

/// Make an existing destination directory writable for the pass;
/// its final attributes come with `Finish`.
pub(crate) fn make_writable(anchor: &Anchor, shown: &Path) -> Result<(), MigrateError> {
    let metadata: Metadata = MigrateError::from_io_result(anchor.file().metadata(), "examine", shown)?;
    if metadata.mode() & 0o700 != 0o700 {
        MigrateError::from_io_result(anchor.file().set_permissions(std::os::unix::fs::PermissionsExt::from_mode(metadata.mode() | 0o700))
            , "make writable", shown)?;
    }
    return Ok(());
}
