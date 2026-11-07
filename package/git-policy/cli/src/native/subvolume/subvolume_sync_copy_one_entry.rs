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
    pub(crate) fn copy_symlink(&mut self, directory: &Directory, relative: &Path, metadata: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = directory.child(relative.file_name().unwrap_or_default());
        let target: PathBuf = MigrateError::from_io_result(std::fs::read_link(&source), "read the link", source.as_path())?;
        if let Some(found) = &existing {
            if found.file_type().is_symlink() && std::fs::read_link(&destination).ok().as_ref() == Some(&target) {
                if (found.mtime(), found.mtime_nsec()) != (metadata.mtime(), metadata.mtime_nsec()) && !self.options.update_only {
                    attribute_result(preserve_on_link(source.as_path(), destination.as_path(), metadata, &mut self.stats.losses)
                        , self.shown(relative).as_path())?;
                }
                self.stats.unchanged += 1;
                return Ok(());
            }
            if !self.replace_allowed(relative, found, metadata) {
                return Ok(());
            }
        }
        let partial: PathBuf = directory.partial();
        MigrateError::from_io_result(std::os::unix::fs::symlink(&target, &partial), "create the link", self.shown(relative).as_path())?;
        attribute_result(preserve_on_link(source.as_path(), partial.as_path(), metadata, &mut self.stats.losses)
            , self.shown(relative).as_path())?;
        self.publish(directory, relative, existing.as_ref())?;
        self.stats.symlinks += 1;
        return Ok(());
    }

    /// Copy a named pipe.
    pub(crate) fn copy_fifo(&mut self, directory: &Directory, relative: &Path, metadata: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
        let source: PathBuf = self.source.join(relative);
        let destination: PathBuf = directory.child(relative.file_name().unwrap_or_default());
        if let Some(found) = &existing {
            if found.file_type().is_fifo() {
                let same: bool = found.mode() & 0o7777 == metadata.mode() & 0o7777 && (found.mtime(), found.mtime_nsec()) == (metadata.mtime(), metadata.mtime_nsec());
                if !same && !self.options.update_only {
                    let handle: File = MigrateError::from_io_result(open_fifo(destination.as_path()), "open the named pipe", self.shown(relative).as_path())?;
                    attribute_result(preserve_on_descriptor(source.as_path(), &handle, metadata, true, &mut self.stats.losses)
                        , self.shown(relative).as_path())?;
                }
                self.stats.unchanged += 1;
                return Ok(());
            }
            if !self.replace_allowed(relative, found, metadata) {
                return Ok(());
            }
        }
        let partial: PathBuf = directory.partial();
        MigrateError::from_io_result(make_fifo(partial.as_path(), metadata.mode()), "create the named pipe", self.shown(relative).as_path())?;
        let handle: File = MigrateError::from_io_result(open_fifo(partial.as_path()), "open the named pipe", self.shown(relative).as_path())?;
        attribute_result(preserve_on_descriptor(source.as_path(), &handle, metadata, true, &mut self.stats.losses)
            , self.shown(relative).as_path())?;
        self.publish(directory, relative, existing.as_ref())?;
        self.stats.fifos += 1;
        return Ok(());
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
    /// True when it was done.
    pub(crate) fn link_to_earlier_copy(&mut self, directory: &Directory, relative: &Path, key: (u64, u64), existing: Option<&Metadata>) -> Result<bool, MigrateError> {
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
    pub(crate) fn copy_file(&mut self, directory: &Directory, relative: &Path, listing: &Metadata, existing: Option<Metadata>) -> Result<(), MigrateError> {
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
            let before: Metadata = MigrateError::from_io_result(input.metadata(), "examine", source.as_path())?;
            if !before.is_file() {
                self.stats.vanished += 1;
                return Ok(());
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
    pub(crate) fn refresh_attributes(&mut self, source: &Path, destination: &Path, metadata: &Metadata, shown: &Path) -> Result<(), MigrateError> {
        let handle: File = MigrateError::from_io_result(std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC).open(destination)
            , "open", shown)?;
        return attribute_result(preserve_on_descriptor(source, &handle, metadata, true, &mut self.stats.losses)
            , shown);
    }
}
