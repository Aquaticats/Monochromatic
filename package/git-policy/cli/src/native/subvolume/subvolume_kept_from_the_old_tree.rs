//! What:
//!  The entries a deletion of the old tree, or of a rollback's subvolume, could not prove to stand in the tree that is kept,
//!  moved into a folder of their own beside the checkout,
//!  with a record that names each of them.
//! Why:
//!  A person's bytes are never deleted,
//!  and an entry an earlier command wrote into a tree that is about to be deleted is somebody's until the person says
//!  otherwise.
//!  Refusing and stopping would leave the person with a tree they cannot delete and no way on but an unverified `rm -rf`;
//!  deleting it secretly is the loss this program exists to prevent.
//!  So the entry is moved where nothing deletes it,
//!  the person is told where it is and which path of the tree it came from,
//!  and the command finishes.
//!  The folder is beside the checkout and not in it,
//!  so the person's `git status` does not grow by every file that was kept,
//!  and the folder is never in a tree this program deletes.
//!  The names in it are digests of the paths,
//!  so no name can be too long for the filesystem,
//!  and the record is what says which path each name came from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const kept = await KeptFromTheOldTree.open(layout);
//! await kept.keep(oldTree, relative); // rename into layout.keptPath, note it
//! await kept.finish(); // write the record, remove the folder when it is empty
//! ```

use crate::subvolume_error::MigrateError;
use crate::subvolume_file_write_steps::write_beside_then_rename;
use crate::subvolume_old_tree_record::OldTreeRecord;
use crate::subvolume_preflight::Layout;
use crate::subvolume_tree_proof::{is_this_programs_own, proved_to_stand};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// What:
///  What a deletion moved out of the tree it deleted,
///  by the name it stands under in the folder and the path it held in the tree.
/// Why:
///   The words the command says and the record it writes are both built from this,
///  so a name in the folder is never left unexplained.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeptEntries = Map<string, string>; // kept name -> path in the deleted tree
/// ```
pub(crate) struct KeptFromTheOldTree {
    /// The folder beside the checkout.
    folder: PathBuf,
    /// The file beside the checkout that names each entry of the folder.
    record: PathBuf,
    /// What stands in the folder,
    /// by its name there and the path it came from,
    /// an earlier run's entries included.
    entries: BTreeMap<OsString, PathBuf>,
    /// The handle the lines are appended to,
    /// opened when the first entry is kept.
    journal: Option<std::fs::File>,
    /// A failure of the appending write,
    /// so the deletion stops with it rather than keeping an entry nothing names.
    failed: Option<MigrateError>,
}

impl KeptFromTheOldTree {
    /// What:
    ///  Open the folder and the record an earlier command left,
    ///  without creating either.
    /// Why:
    ///   A run that keeps nothing creates nothing,
    ///  and a run that keeps something creates only what it fills.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kept = KeptFromTheOldTree.open(layout);
    /// ```
    pub(crate) fn open(layout: &Layout) -> KeptFromTheOldTree {
        return KeptFromTheOldTree {
            folder: layout.kept_path.clone(),
            record: layout.kept_record_path.clone(),
            entries: read_the_record(layout.kept_record_path.as_path()).unwrap_or_default(),
            journal: None,
            failed: None,
        };
    }

    /// What:
    ///  Move the entry at `relative` in `tree` into the folder,
    ///  and note the line that names it.
    /// Why:
    ///   A rename moves the entry itself,
    ///  so what is kept is exactly the bytes that stood there,
    ///  and a writer holding the file open keeps writing into what was kept rather than into what is deleted.
    ///  The line is appended before the command ends rather than flushed for each entry:
    ///  an unsynced write survives the death of this process,
    ///  and the record is rewritten once when the deletion finishes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await kept.keep(oldTree, relative);
    /// ```
    pub(crate) fn keep(&mut self, tree: &Path, relative: &Path) -> Result<(), MigrateError> {
        let name: OsString = name_in_the_folder(relative);
        MigrateError::from_io_result(std::fs::create_dir_all(self.folder.as_path()), "create the folder of what is kept", self.folder.as_path())?;
        let destination: PathBuf = self.folder.join(&name);
        MigrateError::from_io_result(std::fs::rename(tree.join(relative), destination.as_path()), "move what is kept", destination.as_path())?;
        self.entries.insert(name.clone(), relative.to_path_buf());
        let line: String = format!("{}\t{}\n", name.to_string_lossy(), relative.to_string_lossy());
        if let Err(error) = self.append(line.as_bytes()) {
            self.failed = Some(error);
        }
        return Ok(());
    }

    /// Append one line to the record,
    /// opening it the first time.
    fn append(&mut self, bytes: &[u8]) -> Result<(), MigrateError> {
        if self.journal.is_none() {
            // A name the record already stands under is replaced by a regular file: the record is this program's own, and a
            //  link at its name is not something to write through.
            if self.record.exists() && !self.record.is_file() {
                std::fs::remove_file(self.record.as_path()).map_err(|error| {
                    return MigrateError::io("replace the name of the record of what is kept", self.record.as_path(), &error);
                })?;
            }
            let opened: std::fs::File = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .mode(0o600)
                .open(self.record.as_path())
                .map_err(|error| return MigrateError::io("open the record of what is kept", self.record.as_path(), &error))?;
            self.journal = Some(opened);
        }
        let Some(file) = self.journal.as_mut() else { return Ok(()) };
        return std::io::Write::write_all(file, bytes).map_err(|error| return MigrateError::io("write the record of what is kept", self.record.as_path(), &error));
    }

    /// What:
    ///  Write the record of everything the folder holds,
    ///  remove the folder when it holds nothing,
    ///  and answer every entry with the path it came from.
    /// Why:
    ///   One rewrite replaces the appended lines with the record as it stands,
    ///  and one durability sync is what it costs:
    ///  what the folder holds is beside the checkout, not in a tree this command is building.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const kept = await kept.finish(); // [name, path][]
    /// ```
    pub(crate) fn finish(&mut self) -> Result<Vec<(PathBuf, PathBuf)>, MigrateError> {
        if let Some(failure) = self.failed.take() {
            return Err(failure);
        }
        self.journal = None;
        let mut text: String = String::new();
        for (name, source) in &self.entries {
            text.push_str(name.to_string_lossy().as_ref());
            text.push('\t');
            text.push_str(source.to_string_lossy().as_ref());
            text.push('\n');
        }
        if !self.entries.is_empty() {
            MigrateError::from_io_result(write_beside_then_rename(self.record.as_path(), text.as_bytes()), "write the record of what is kept", self.record.as_path())?;
        }
        if std::fs::read_dir(self.folder.as_path()).map(|mut listing| return listing.next().is_none()).unwrap_or(false) {
            // Nothing was kept, so the folder an earlier run made is empty and goes with the record that named it.
            MigrateError::from_io_result(std::fs::remove_dir(self.folder.as_path()), "remove the folder of what is kept", self.folder.as_path())?;
            if self.entries.is_empty() {
                crate::subvolume_rollback_subvolume_removal::remove_if_present(self.record.as_path())?;
            }
        }
        return Ok(self.entries.iter().map(|(name, source)| return (self.folder.join(name), source.clone())).collect());
    }
}

/// The name an entry stands under in the folder:
/// a digest of the path it held, so a long path is no long name.
fn name_in_the_folder(relative: &Path) -> OsString {
    return OsString::from(crate::service_instance::digest_of_bytes(relative.as_os_str().as_bytes()));
}

/// What:
///  The entries the record at `path` names,
///  by their name in the folder and the path they came from.
/// Why:
///   An entry an earlier command kept is still somebody's,
///  and a later command names it again rather than leaving it unexplained.
///  A line the record cannot parse names nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = readTheRecord(path); // Map<name, path>
/// ```
pub(crate) fn read_the_record(path: &Path) -> std::io::Result<BTreeMap<OsString, PathBuf>> {
    let file: std::fs::File = match crate::plain_file::open_plain_file(path) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error),
    };
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(file);
    let mut entries: BTreeMap<OsString, PathBuf> = BTreeMap::new();
    let mut line: Vec<u8> = Vec::new();
    loop {
        line.clear();
        let read: usize = std::io::BufRead::read_until(&mut reader, b'\n', &mut line)?;
        if read == 0 {
            break;
        }
        if line.last() == Some(&b'\n') {
            line.pop();
        }
        let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(line.as_slice());
        let Some((name, source)) = text.split_once('\t') else { continue };
        if name.is_empty() || source.is_empty() {
            continue;
        }
        entries.insert(OsString::from(name), PathBuf::from(source));
    }
    return Ok(entries);
}

/// What:
///  Walk a tree that is about to be deleted and move every entry it cannot prove to stand in the tree that is kept into
///  the folder,
///  where nothing deletes it,
///  and answer the paths that could not be moved.
/// Why:
///   The proof answers one question per entry:
///   is this the entry the migration carried, unchanged since it read it,
///   or does the kept tree hold these bytes now?
///  An entry that answers neither was written into the tree that is about to be deleted,
///  and it is somebody's.
///  Moving it out is what lets the command finish without deleting a byte nobody proved:
///  refusing would leave the person with a tree they cannot delete,
///  and deleting would be the loss this program exists to prevent.
///  A directory is judged by its children,
///  so it is walked and never moved on its own account;
///  a directory that cannot be listed is moved whole, since nothing below it can be judged.
///  What a person accepted with a flag is deleted rather than moved,
///  and the index,
///  the locks and the bookkeeping this program writes are nobody's bytes to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unproved = await keepWhatTheKeptTreeDoesNotHold(deleted, kept, record, accepted, folder);
/// ```
pub(crate) fn keep_what_the_kept_tree_does_not_hold(
    deleted: &Path,
    kept_tree: &Path,
    record: Option<&OldTreeRecord>,
    accepted: &BTreeSet<PathBuf>,
    kept: &mut KeptFromTheOldTree,
) -> Result<Vec<PathBuf>, MigrateError> {
    let mut unproved: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let listing: std::fs::ReadDir = match std::fs::read_dir(deleted.join(&relative)) {
            Ok(found) => found,
            // Nothing below a directory this user cannot list can be judged, so the directory itself is moved where nothing
            //  deletes it; what it holds is kept whole rather than named entry by entry.
            Err(_unreadable) => {
                if relative.as_os_str().is_empty() {
                    // The tree itself cannot be listed: nothing can be kept from it and nothing can be deleted from it.
                    unproved.push(relative);
                    continue;
                }
                if kept.keep(deleted, relative.as_path()).is_err() {
                    unproved.push(relative);
                }
                continue;
            }
        };
        for item in listing.flatten() {
            let child: PathBuf = relative.join(item.file_name());
            if accepted.contains(&child) {
                continue;
            }
            let Ok(kind) = item.file_type() else {
                unproved.push(child);
                continue;
            };
            if is_this_programs_own(deleted, child.as_path(), kind) {
                continue;
            }
            if kind.is_dir() {
                pending.push(child);
                continue;
            }
            let Ok(now) = item.metadata() else {
                unproved.push(child);
                continue;
            };
            if let Some(readings) = record
                && readings.unchanged(child.as_path(), &now)
            {
                continue;
            }
            if proved_to_stand(deleted, kept_tree, child.as_path(), kind) {
                continue;
            }
            if kept.keep(deleted, child.as_path()).is_err() {
                unproved.push(child);
            }
        }
    }
    unproved.sort();
    return Ok(unproved);
}

/// Controls of what a deletion keeps stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_kept_from_the_old_tree_tests.rs"]
mod tests;
