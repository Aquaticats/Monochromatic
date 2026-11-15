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
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The name an entry is written under before it is renamed to its own,
/// unless the source has an entry of that name.
pub(crate) const PARTIAL_NAME: &str = ".cli-git-migrate-partial";

/// How many times a file that keeps changing under the copy is copied before the pass gives up.
pub(crate) const COPY_ATTEMPTS: u32 = 3;

/// The end of the name an entry the pass replaces is parked under while its own copy takes the name.
/// A name ending in it is never deleted by a later pass:
/// what it holds is somebody's until the pass that parked it judged it.
pub(crate) const PARKED_SUFFIX: &str = ".cli-git-migrate-displaced";

/// What:
///  The name an entry is parked under while the pass replaces it:
///  its own name with the parked suffix,
///  so a person who meets one reads which path it stood at.
/// Why:
///   The name is derived, not chosen,
///  because a later pass must be able to recognise a parked entry without a journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parkedName = (name) => `${name}.cli-git-migrate-displaced`;
/// ```
pub(crate) fn parked_name(name: &OsStr) -> OsString {
    let mut text: OsString = name.to_os_string();
    text.push(PARKED_SUFFIX);
    return text;
}

/// Whether a destination name is one a pass parked an entry under.
pub(crate) fn is_a_parked_name(name: &OsStr) -> bool {
    return name.as_encoded_bytes().ends_with(PARKED_SUFFIX.as_bytes());
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

impl Pass<'_> {
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
            self.stats.conflicts.push(relative.to_path_buf());
            return false;
        }
        return true;
    }

    /// What:
    ///  Put the finished temporary entry at the destination name,
    ///  and never replace an entry this pass did not see.
    ///  `existing` is what the pass found at that name when it decided to copy.
    /// Why:
    ///   The look and the write are two steps,
    ///  and between them another command can put something at that name.
    ///  A name the pass found free is therefore created and never replaced,
    ///  and a name that held an entry is replaced only while it still holds that very entry,
    ///  so what came to stand there is somebody's and stays.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const published = pass.publish(directory, relative, existing); // 'placed' | 'kept-the-destination-entry'
    /// ```
    pub(crate) fn publish(&mut self, directory: &Directory, relative: &Path, existing: Option<&Metadata>) -> Result<Published, MigrateError> {
        let name: &OsStr = relative.file_name().unwrap_or_default();
        let destination: PathBuf = directory.child(name);
        let partial: PathBuf = directory.partial();
        let shown: PathBuf = self.shown(relative);
        let Some(found) = existing else {
            return place_where_nothing_was(partial.as_path(), destination.as_path(), shown.as_path());
        };
        // The pass saw this entry and decided the source wins. Only that very entry may be replaced.
        let now: Option<Metadata> = std::fs::symlink_metadata(&destination).ok();
        if !same_object(now.as_ref(), found) {
            remove_if_present(partial.as_path(), shown.as_path())?;
            return Ok(Published::KeptTheDestinationEntry);
        }
        if found.is_dir() {
            // A directory is in the way of a rename, so it goes first, and only the one the pass saw.
            remove_entry(destination.as_path(), shown.as_path())?;
            return place_where_nothing_was(partial.as_path(), destination.as_path(), shown.as_path());
        }
        return MigrateError::from_io_result(std::fs::rename(partial.as_path(), &destination), "put the finished copy in place of", shown.as_path()).map(|()| return Published::Placed);
    }
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

/// Whether the entry at a name is still the object the pass looked at:
/// the same device and inode.
/// Why:
///   A name says nothing about which object it holds,
///  and an object that a name held a moment ago may have been replaced by another one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = (now, looked) => now?.dev === looked.dev && now?.ino === looked.ino;
/// ```
pub(crate) fn same_object(held_now: Option<&Metadata>, looked: &Metadata) -> bool {
    let Some(held) = held_now else { return false };
    return held.dev() == looked.dev() && held.ino() == looked.ino();
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
