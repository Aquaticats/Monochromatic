//! What:
//!  The copying of one source entry (a file,
//! a symbolic link or a named pipe) into the destination,
//! as methods of `Pass`.
//! Why:
//!  The walk in `subvolume_sync.rs` decides what to copy;
//! this file does the copying of one entry,
//! with the recopy loop for
//!  a file that changes under the copy and the link bookkeeping for files with several names.
//! It depends on the shared pieces
//!  in `subvolume_sync_pass_state_and_checks.rs` and nothing from the walk's file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await pass.copyFile(directory, relative, listing, existing);
//! ```

use crate::fs_primitive_error::PrimitiveError;
use crate::subvolume_anchor::Anchor;
use crate::subvolume_error::MigrateError;
use crate::subvolume_sync_attrs::{carried_attributes_equal, make_fifo, open_fifo, preserve_on_descriptor, preserve_on_link};
use std::fs::{File, Metadata};
use std::io::Seek;
use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use crate::subvolume_sync_pass_state_and_checks::*;

impl Pass<'_> {
    /// Copy a symbolic link.
    pub(crate) fn copy_symlink(&mut self, directory: &Directory, relative: &Path, listed: &Metadata, existing: Option<Metadata>) -> Result<Copied, MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = directory.child(relative.file_name().unwrap_or_default());
        let shown: PathBuf = self.shown(relative);
        let target: PathBuf = match std::fs::read_link(&source) {
            Ok(read) => read,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.stats.vanished += 1;
                return Ok(Copied::Vanished);
            }
            // The name holds another kind of entry now: nothing is written here and the walk reports it.
            Err(error) if error.raw_os_error() == Some(libc::EINVAL) => {
                self.uncarried(relative, Uncarried::ChangedWhilePassing);
                return Ok(Copied::ChangedKind);
            }
            Err(error) => return Err(MigrateError::io("read the link", source.as_path(), &error)),
        };
        if let Some(found) = &existing {
            if found.file_type().is_symlink() && std::fs::read_link(&destination).ok().as_ref() == Some(&target) {
                if (found.mtime(), found.mtime_nsec()) != (listed.mtime(), listed.mtime_nsec()) && !self.options.update_only {
                    attribute_result(preserve_on_link(source.as_path(), destination.as_path(), listed, &mut self.stats.losses), shown.as_path())?;
                }
                self.stats.unchanged += 1;
                return Ok(Copied::LeftAsItIs);
            }
            // A pass that replaces nothing keeps the link standing here, and the proof against the tree it deletes names
            //  this path rather than deleting a person's link on a modification time.
            if self.options.never_replaces {
                self.uncarried(relative, Uncarried::KeptAtTheDestination);
                return Ok(Copied::KeptTheDestinationEntry);
            }
            if !self.replace_allowed(relative, found, listed) {
                return Ok(Copied::LeftAsItIs);
            }
        }
        // The link the pass read is the one that has to stand at the source path when the copy takes the destination
        //  name: another command can put a new link there while the copy runs, and that link is the one to copy.
        let mut names: u32 = 0;
        let mut wanted: PathBuf = target;
        loop {
            names += 1;
            let partial: PathBuf = directory.partial();
            MigrateError::from_io_result(std::os::unix::fs::symlink(&wanted, &partial), "create the link", shown.as_path())?;
            attribute_result(preserve_on_link(source.as_path(), partial.as_path(), listed, &mut self.stats.losses), shown.as_path())?;
            if self.source_path_still_names(listed, relative) && std::fs::read_link(&source).ok().as_ref() == Some(&wanted) {
                break;
            }
            remove_if_present(partial.as_path(), shown.as_path())?;
            if names >= COPY_ATTEMPTS {
                self.uncarried(relative, Uncarried::ChangedWhilePassing);
                return Ok(Copied::ChangedKind);
            }
            self.stats.recopied += 1;
            match std::fs::read_link(&source) {
                Ok(now) => wanted = now,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    self.stats.vanished += 1;
                    return Ok(Copied::Vanished);
                }
                Err(_no_longer_a_link) => {
                    self.uncarried(relative, Uncarried::ChangedWhilePassing);
                    return Ok(Copied::ChangedKind);
                }
            }
        }
        if self.publish(directory, relative, existing.as_ref())? == Published::KeptTheDestinationEntry {
            self.uncarried(relative, Uncarried::KeptAtTheDestination);
            self.interrupt.check("copy-kept")?;
            return Ok(Copied::KeptTheDestinationEntry);
        }
        if !self.source_unchanged_since(listed, relative) || std::fs::read_link(&source).ok().as_ref() != Some(&wanted) {
            self.uncarried(relative, Uncarried::ChangedWhilePassing);
            return Ok(Copied::ChangedKind);
        }
        self.stats.symlinks += 1;
        return Ok(Copied::Placed);
    }

    /// Copy a named pipe.
    pub(crate) fn copy_fifo(&mut self, directory: &Directory, relative: &Path, _listed: &Metadata, existing: Option<Metadata>) -> Result<Copied, MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = directory.child(relative.file_name().unwrap_or_default());
        let shown: PathBuf = self.shown(relative);
        // A named pipe is made from the listing's kind, so the name has to hold that kind still: another kind is no
        //  entry this pass can make here, and the walk reports it rather than writing its own entry.
        let mut examined: Metadata = match std::fs::symlink_metadata(&source) {
            Ok(now) if now.file_type().is_fifo() => now,
            Ok(_another_kind) => {
                self.uncarried(relative, Uncarried::ChangedWhilePassing);
                return Ok(Copied::ChangedKind);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.stats.vanished += 1;
                return Ok(Copied::Vanished);
            }
            Err(error) => return Err(MigrateError::io("examine", source.as_path(), &error)),
        };
        if let Some(found) = &existing {
            if found.file_type().is_fifo() {
                let same: bool = found.mode() & 0o7777 == examined.mode() & 0o7777 && (found.mtime(), found.mtime_nsec()) == (examined.mtime(), examined.mtime_nsec());
                if !same && !self.options.update_only {
                    let handle: File = MigrateError::from_io_result(open_fifo(destination.as_path()), "open the named pipe", shown.as_path())?;
                    attribute_result(preserve_on_descriptor(source.as_path(), &handle, &examined, true, &mut self.stats.losses), shown.as_path())?;
                }
                self.stats.unchanged += 1;
                return Ok(Copied::LeftAsItIs);
            }
            if self.options.never_replaces {
                self.uncarried(relative, Uncarried::KeptAtTheDestination);
                return Ok(Copied::KeptTheDestinationEntry);
            }
            if !self.replace_allowed(relative, found, &examined) {
                return Ok(Copied::LeftAsItIs);
            }
        }
        let mut names: u32 = 0;
        loop {
            names += 1;
            let partial: PathBuf = directory.partial();
            MigrateError::from_io_result(make_fifo(partial.as_path(), examined.mode()), "create the named pipe", shown.as_path())?;
            let handle: File = MigrateError::from_io_result(open_fifo(partial.as_path()), "open the named pipe", shown.as_path())?;
            attribute_result(preserve_on_descriptor(source.as_path(), &handle, &examined, true, &mut self.stats.losses), shown.as_path())?;
            if self.source_path_still_names(&examined, relative) {
                break;
            }
            remove_if_present(partial.as_path(), shown.as_path())?;
            if names >= COPY_ATTEMPTS {
                self.uncarried(relative, Uncarried::ChangedWhilePassing);
                return Ok(Copied::ChangedKind);
            }
            self.stats.recopied += 1;
            match std::fs::symlink_metadata(&source) {
                Ok(now) if now.file_type().is_fifo() => examined = now,
                Ok(_another_kind) => {
                    self.uncarried(relative, Uncarried::ChangedWhilePassing);
                    return Ok(Copied::ChangedKind);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    self.stats.vanished += 1;
                    return Ok(Copied::Vanished);
                }
                Err(error) => return Err(MigrateError::io("examine", source.as_path(), &error)),
            }
        }
        if self.publish(directory, relative, existing.as_ref())? == Published::KeptTheDestinationEntry {
            self.uncarried(relative, Uncarried::KeptAtTheDestination);
            self.interrupt.check("copy-kept")?;
            return Ok(Copied::KeptTheDestinationEntry);
        }
        if !self.source_unchanged_since(&examined, relative) {
            self.uncarried(relative, Uncarried::ChangedWhilePassing);
            return Ok(Copied::ChangedKind);
        }
        self.stats.fifos += 1;
        return Ok(Copied::Placed);
    }

    /// Whether the destination file is one of the recorded copies of this source file,
    /// or none is recorded yet.
    pub(crate) fn is_linked_as_recorded(&self, key: (u64, u64), found: &Metadata) -> bool {
        let Some(copies) = self.links.get(&key) else { return true };
        for copy in copies {
            if let Ok(copy_meta) = std::fs::symlink_metadata(self.destination.join(copy))
                && copy_meta.ino() == found.ino()
            {
                return true;
            }
        }
        return false;
    }

    /// Record a destination file as a copy of a source file with several names.
    pub(crate) fn record_link(&mut self, key: (u64, u64), relative: &Path) {
        let copies: &mut Vec<PathBuf> = self.links.entry(key).or_default();
        let mut recorded: bool = false;
        for copy in copies.iter() {
            if copy == relative {
                recorded = true;
                break;
            }
        }
        if !recorded {
            copies.push(relative.to_path_buf());
        }
    }

    /// Try to make the destination another name of an earlier copy.
    ///  `Ok(None)` means no earlier copy could be linked,
    /// and `Ok(Some(_))` is what placing the link came to.
    pub(crate) fn link_to_earlier_copy(
        &mut self,
        directory: &Directory,
        relative: &Path,
        key: (u64, u64),
        listing: &Metadata,
        existing: Option<&Metadata>,
    ) -> Result<Option<Copied>, MigrateError> {
        for copy in self.links.get(&key).cloned().unwrap_or_default() {
            let Ok(earlier) = Anchor::open_below(self.destination, copy.parent().unwrap_or(Path::new(""))) else { continue };
            let first: PathBuf = earlier.child(copy.file_name().unwrap_or_default());
            remove_if_present(directory.partial().as_path(), self.shown(relative).as_path())?;
            if self.platform.hard_link(first.as_path(), directory.partial().as_path()).is_err() {
                continue;
            }
            if self.publish(directory, relative, existing)? == Published::KeptTheDestinationEntry {
                return Ok(Some(Copied::KeptTheDestinationEntry));
            }
            // The link was placed with no copy of its own, so the source is read after the placement exactly as the copy
            //  loop reads it: a write made in the meantime means this name does not hold the newest bytes, and the copy
            //  loop below makes one.
            if !self.source_unchanged_since(listing, relative) {
                remove_if_present(directory.partial().as_path(), self.shown(relative).as_path())?;
                return Ok(None);
            }
            self.stats.hard_linked += 1;
            return Ok(Some(Copied::Placed));
        }
        return Ok(None);
    }

    /// Copy a regular file.
    pub(crate) fn copy_file(&mut self, directory: &Directory, relative: &Path, listing: &Metadata, existing: Option<Metadata>) -> Result<Copied, MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = directory.child(relative.file_name().unwrap_or_default());
        let shown: PathBuf = self.shown(relative);
        // The folder's listing is what the walk saw when it listed the folder;
        //  a write made since then is not in it, so the source is read again before anything is decided,
        //  one `lstat` for one file.
        let fresh: Option<Metadata> = match std::fs::symlink_metadata(&source) {
            Ok(found) => Some(found),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.stats.vanished += 1;
                return Ok(Copied::Vanished);
            }
            // Any other failure is answered by the open below, which names it.
            Err(_unexaminable) => None,
        };
        let now: &Metadata = fresh.as_ref().unwrap_or(listing);
        let listed_key: (u64, u64) = (now.dev(), now.ino());
        // What stands at the destination name, which is what the pass examined there; a retry after the publish examines
        //  the copy this pass made itself.
        let mut at_the_name: Option<Metadata> = existing.clone();
        if let Some(found) = &existing {
            if !found.is_file() {
                if self.options.never_replaces {
                    self.uncarried(relative, Uncarried::KeptAtTheDestination);
                    return Ok(Copied::KeptTheDestinationEntry);
                }
                if !self.replace_allowed(relative, found, now) {
                    return Ok(Copied::LeftAsItIs);
                }
            } else if file_is_equal(found, now) && (now.nlink() <= 1 || self.is_linked_as_recorded(listed_key, found)) {
                if !self.options.update_only && !carried_attributes_equal(source.as_path(), destination.as_path()) {
                    self.refresh_attributes(&source, &destination, now, shown.as_path())?;
                }
                self.stats.unchanged += 1;
                if now.nlink() > 1 {
                    self.record_link(listed_key, relative);
                }
                return Ok(Copied::LeftAsItIs);
            } else if self.options.never_replaces {
                // The bytes standing here are the person's, and the proof against the tree this pass deletes names
                //  this path rather than deleting them on a modification time.
                self.uncarried(relative, Uncarried::KeptAtTheDestination);
                return Ok(Copied::KeptTheDestinationEntry);
            } else if self.options.update_only && is_newer(found, now) {
                // The destination holds newer work. It is not a faithful copy, so no other name may be linked to it.
                self.stats.unchanged += 1;
                return Ok(Copied::LeftAsItIs);
            }
        }
        if now.nlink() > 1
            && self.source_path_still_names(now, relative)
            && let Some(outcome) = self.link_to_earlier_copy(directory, relative, listed_key, now, at_the_name.as_ref())?
        {
            return Ok(outcome);
        }
        // The file a pass copies is the one the source path names when the copy starts, and the path is looked at again
        //  before the finished copy takes the destination name: another command can rename a new file onto that path
        //  while the copy runs, and the file that stands there then is the one that has to reach the destination.
        let mut names: u32 = 0;
        loop {
            names += 1;
            // The open does not follow a link and never blocks: a name turned into a named pipe between the listing and
            //  this open is refused here instead of waited on for ever.
            let mut input: File = match crate::plain_file::open_plain_file(source.as_path()) {
                Ok(opened) => opened,
                Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                    self.uncarried(relative, Uncarried::Unreadable);
                    return Ok(Copied::Unreadable);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    self.stats.vanished += 1;
                    return Ok(Copied::Vanished);
                }
                // A name that holds another kind of entry now, a directory or a named pipe, is that kind and is
                //  reported; anything else is a failure of the open.
                Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => {
                    self.uncarried(relative, Uncarried::ChangedWhilePassing);
                    return Ok(Copied::ChangedKind);
                }
                Err(error) => return Err(MigrateError::io("open", source.as_path(), &error)),
            };
            let mut attempt: u32 = 0;
            loop {
                attempt += 1;
                let before: Metadata = MigrateError::from_io_result(input.metadata(), "examine", source.as_path())?;
                // The name holds another kind of entry now: nothing is written here, and the walk reports it, so the old
                //  tree is not deleted with an entry this pass never carried.
                if !before.is_file() {
                    self.uncarried(relative, Uncarried::ChangedWhilePassing);
                    return Ok(Copied::ChangedKind);
                }
                remove_if_present(directory.partial().as_path(), shown.as_path())?;
                let mut output: File = MigrateError::from_io_result(std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(directory.partial())
                    , "create", shown.as_path())?;
                self.interrupt.check("copy-partial")?;
                MigrateError::from_io_result(input.rewind(), "rewind", source.as_path())?;
                let cloned: Result<(), PrimitiveError> = self.platform.clone_file(&input, &output, directory.partial().as_path());
                if cloned.is_err() {
                    MigrateError::from_io_result(std::io::copy(&mut input, &mut output), "copy to", shown.as_path())?;
                }
                let after: Metadata = MigrateError::from_io_result(input.metadata(), "examine", source.as_path())?;
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
                attribute_result(preserve_on_descriptor(source.as_path(), &output, &before, !self.options.update_only, &mut self.stats.losses)
                    , shown.as_path())?;
                drop(output);
                if !self.source_path_still_names(&before, relative) {
                    // The name leads to another file now: the copy is dropped and that file is copied, a bounded number
                    //  of times. A name that will not stand still is an entry that changed while passing.
                    remove_if_present(directory.partial().as_path(), shown.as_path())?;
                    if names >= COPY_ATTEMPTS {
                        self.uncarried(relative, Uncarried::ChangedWhilePassing);
                        return Ok(Copied::ChangedKind);
                    }
                    self.stats.recopied += 1;
                    break;
                }
                if self.publish(directory, relative, at_the_name.as_ref())? == Published::KeptTheDestinationEntry {
                    self.uncarried(relative, Uncarried::KeptAtTheDestination);
                    self.interrupt.check("copy-kept")?;
                    return Ok(Copied::KeptTheDestinationEntry);
                }
                // What was published is checked against the source once it is in place: no check before the publish can
                //  close the window, so the copy is compared with the source as it stands now. A source that moved is
                //  copied again, a bounded number of times, and a name that will not stand still keeps its newer bytes in
                //  the old tree and its path in the record.
                if !self.source_unchanged_since(&before, relative) {
                    if names >= COPY_ATTEMPTS {
                        self.uncarried(relative, Uncarried::ChangedWhilePassing);
                        self.interrupt.check("copy-changed-after-the-publish")?;
                        return Ok(Copied::ChangedKind);
                    }
                    self.stats.recopied += 1;
                    at_the_name = std::fs::symlink_metadata(&destination).ok();
                    break;
                }
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
                self.interrupt.check("copy-entry")?;
                return Ok(Copied::Placed);
            }
        }
    }

    /// Apply the original's attributes to a destination file that already has its content.
    pub(crate) fn refresh_attributes(&mut self, source: &Path, destination: &Path, metadata: &Metadata, shown: &Path) -> Result<(), MigrateError> {
        let handle: File = MigrateError::from_io_result(std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(destination)
            , "open", shown)?;
        return attribute_result(preserve_on_descriptor(source, &handle, metadata, true, &mut self.stats.losses)
            , shown);
    }
}
